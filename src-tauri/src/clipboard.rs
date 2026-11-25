//! Win32 clipboard: change listener, reading/writing formats and privacy filters.
//!
//! The listener runs on its own thread with a message-only window registered through
//! `AddClipboardFormatListener`, so captures are event driven (no polling). Writes are
//! marshalled to that same thread, which makes "is this update our own write?" race free.

use std::io::Cursor;
use std::path::Path;
use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, AtomicIsize, AtomicU32, Ordering};
use std::time::Duration;

use image::{DynamicImage, ImageFormat, RgbaImage};
use sha2::{Digest, Sha256};
use tauri::{AppHandle, Emitter, Manager};
use windows::Win32::Foundation::{
    CloseHandle, GlobalFree, HANDLE, HGLOBAL, HINSTANCE, HWND, LPARAM, LRESULT, WPARAM,
};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, EmptyClipboard, GetClipboardData,
    GetClipboardOwner, GetClipboardSequenceNumber, IsClipboardFormatAvailable, OpenClipboard,
    RegisterClipboardFormatW, SetClipboardData,
};
use windows::Win32::System::LibraryLoader::GetModuleHandleW;
use windows::Win32::System::Memory::{
    GMEM_MOVEABLE, GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock,
};
use windows::Win32::System::Threading::{
    OpenProcess, PROCESS_NAME_WIN32, PROCESS_QUERY_LIMITED_INFORMATION, QueryFullProcessImageNameW,
};
use windows::Win32::UI::Shell::{DragQueryFileW, HDROP};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DefWindowProcW, DispatchMessageW, GetForegroundWindow, GetMessageW,
    GetWindowThreadProcessId, HWND_MESSAGE, MSG, RegisterClassW, SendMessageW, WM_APP,
    WM_CLIPBOARDUPDATE, WNDCLASSW,
};
use windows::core::{PCWSTR, PWSTR, w};

use crate::db::{self, Clip, NewClip};
use crate::{AppState, detect, ocr};

const CF_DIB: u32 = 8;
const CF_UNICODETEXT: u32 = 13;
const CF_HDROP: u32 = 15;
const CF_DIBV5: u32 = 17;
pub const MAX_TEXT_BYTES: usize = 1 << 20;
// Rebuilding a truncated CF_HTML fragment is not worth it; huge HTML is dropped, text is kept.
pub const MAX_HTML_BYTES: usize = 4 << 20;
const THUMB_SIZE: u32 = 320;
const WM_WRITE: u32 = WM_APP + 1;

static APP: OnceLock<AppHandle> = OnceLock::new();
static LISTENER_HWND: AtomicIsize = AtomicIsize::new(0);
// All three only touched on the listener thread.
static OWN_SEQ: AtomicU32 = AtomicU32::new(0);
static WRITING: AtomicBool = AtomicBool::new(false);
/// A capture holds the clipboard open. GetClipboardData can dispatch a nested WM_WRITE
/// while the owner renders a delayed format; writing then would corrupt that read.
static READING: AtomicBool = AtomicBool::new(false);

struct Formats {
    png: u32,
    html: u32,
    rtf: u32,
    exclude: u32,
    viewer_ignore: u32,
    can_include: u32,
}

fn formats() -> &'static Formats {
    static F: OnceLock<Formats> = OnceLock::new();
    F.get_or_init(|| unsafe {
        Formats {
            png: RegisterClipboardFormatW(w!("PNG")),
            html: RegisterClipboardFormatW(w!("HTML Format")),
            rtf: RegisterClipboardFormatW(w!("Rich Text Format")),
            exclude: RegisterClipboardFormatW(w!("ExcludeClipboardContentFromMonitorProcessing")),
            viewer_ignore: RegisterClipboardFormatW(w!("Clipboard Viewer Ignore")),
            can_include: RegisterClipboardFormatW(w!("CanIncludeInClipboardHistory")),
        }
    })
}

pub fn start(app: AppHandle) -> std::io::Result<()> {
    let _ = APP.set(app);
    std::thread::Builder::new()
        .name("clipboard-listener".into())
        .spawn(|| {
            if let Err(e) = unsafe { listen() } {
                log::error!("clipboard listener failed: {e}");
            }
        })?;
    Ok(())
}

unsafe fn listen() -> windows::core::Result<()> {
    unsafe {
        let instance: HINSTANCE = GetModuleHandleW(None)?.into();
        let class = w!("WincyClipboardListener");
        let wc = WNDCLASSW {
            lpfnWndProc: Some(wndproc),
            hInstance: instance,
            lpszClassName: class,
            ..Default::default()
        };
        RegisterClassW(&wc);
        let hwnd = CreateWindowExW(
            Default::default(),
            class,
            PCWSTR::null(),
            Default::default(),
            0,
            0,
            0,
            0,
            Some(HWND_MESSAGE),
            None,
            Some(instance),
            None,
        )?;
        LISTENER_HWND.store(hwnd.0 as isize, Ordering::SeqCst);
        AddClipboardFormatListener(hwnd)?;
        let mut msg = MSG::default();
        while GetMessageW(&mut msg, None, 0, 0).as_bool() {
            DispatchMessageW(&msg);
        }
    }
    Ok(())
}

extern "system" fn wndproc(hwnd: HWND, msg: u32, wparam: WPARAM, lparam: LPARAM) -> LRESULT {
    match msg {
        WM_CLIPBOARDUPDATE => {
            if let Some(app) = APP.get() {
                on_update(app);
            }
            LRESULT(0)
        }
        WM_WRITE => {
            // SAFETY: `write` passes a pointer to a live WriteJob and blocks in SendMessageW.
            let job = unsafe { &mut *(lparam.0 as *mut WriteJob) };
            if READING.load(Ordering::SeqCst) {
                job.result = Err("The clipboard is busy, try again".into());
                return LRESULT(0);
            }
            WRITING.store(true, Ordering::SeqCst);
            job.result = write_now(&job.items);
            // A failed write never touched the clipboard: the sequence number is someone else's.
            if job.result.is_ok() {
                OWN_SEQ.store(unsafe { GetClipboardSequenceNumber() }, Ordering::SeqCst);
            }
            WRITING.store(false, Ordering::SeqCst);
            LRESULT(0)
        }
        _ => unsafe { DefWindowProcW(hwnd, msg, wparam, lparam) },
    }
}

fn on_update(app: &AppHandle) {
    // Our own write (paste_clip already bumped that clip).
    if WRITING.load(Ordering::SeqCst)
        || unsafe { GetClipboardSequenceNumber() } == OWN_SEQ.load(Ordering::SeqCst)
    {
        return;
    }
    let state = app.state::<AppState>();
    if state.paused.load(Ordering::SeqCst) {
        log::debug!("capture skipped: paused");
        return;
    }
    let Some((content, source)) = snapshot(&state) else {
        return;
    };
    let (max_items, ignore_secrets, ocr_images) = {
        let s = state.settings.lock().unwrap();
        (s.max_items, s.ignore_secrets, s.ocr_images)
    };
    if let (true, Content::Text(text, _)) = (ignore_secrets, &content)
        && let Some(pattern) = detect::looks_secret(text)
    {
        log::info!("capture skipped: secret pattern {pattern}");
        return;
    }
    let Some(clip) = to_new_clip(content, source, &state.images_dir) else {
        return;
    };
    let result = {
        let conn = state.db.lock().unwrap();
        db::upsert(&conn, &clip).and_then(|_| db::trim(&conn, max_items))
    };
    match result {
        Ok((_, images)) => {
            db::remove_images(&state.images_dir, &images);
            let _ = app.emit("clips-changed", ());
            if ocr_images && clip.kind == "image" {
                ocr::queue(clip.hash);
            }
        }
        Err(e) => log::error!("storing clip failed: {e}"),
    }
}

enum Content {
    Files(Vec<String>),
    Png(Vec<u8>),
    Rgba(u32, u32, Vec<u8>),
    Text(String, Option<String>),
}

/// Closes the clipboard on drop.
struct Opened;

impl Opened {
    /// Other apps hold the clipboard open briefly; retry a few times before giving up.
    fn new() -> Option<Self> {
        let owner = HWND(LISTENER_HWND.load(Ordering::SeqCst) as *mut _);
        for _ in 0..10 {
            if unsafe { OpenClipboard(Some(owner)) }.is_ok() {
                return Some(Opened);
            }
            std::thread::sleep(Duration::from_millis(15));
        }
        None
    }
}

impl Drop for Opened {
    fn drop(&mut self) {
        let _ = unsafe { CloseClipboard() };
    }
}

fn available(format: u32) -> bool {
    format != 0 && unsafe { IsClipboardFormatAvailable(format) }.is_ok()
}

/// Copies at most `max` bytes of a clipboard format. Clipboard must be open.
fn get_bytes(format: u32, max: usize) -> Option<Vec<u8>> {
    if !available(format) {
        return None;
    }
    unsafe {
        let mem = HGLOBAL(GetClipboardData(format).ok()?.0);
        let ptr = GlobalLock(mem) as *const u8;
        if ptr.is_null() {
            return None;
        }
        let bytes = std::slice::from_raw_parts(ptr, GlobalSize(mem).min(max)).to_vec();
        let _ = GlobalUnlock(mem);
        Some(bytes)
    }
}

/// Runs the privacy checks and the read on one clipboard snapshot: the clipboard stays
/// open throughout, so a newer (possibly excluded) payload cannot slip in between.
/// It is closed again before the slow PNG encoding and database work.
fn snapshot(state: &AppState) -> Option<(Content, Option<String>)> {
    let _open = Opened::new()?;
    READING.store(true, Ordering::SeqCst);
    let result = checked_read(state);
    READING.store(false, Ordering::SeqCst);
    result
}

/// Clipboard must be open.
fn checked_read(state: &AppState) -> Option<(Content, Option<String>)> {
    let f = formats();
    if available(f.exclude) || available(f.viewer_ignore) {
        log::info!("capture skipped: excluded by the source app");
        return None;
    }
    let source = source_app();
    let capture_images = {
        let s = state.settings.lock().unwrap();
        if let Some(exe) = source.as_deref().filter(|exe| s.is_ignored(exe)) {
            log::info!("capture skipped: ignored app {exe}");
            return None;
        }
        s.capture_images
    };
    Some((read(capture_images)?, source))
}

/// Clipboard must be open.
fn read(capture_images: bool) -> Option<Content> {
    let f = formats();
    if get_bytes(f.can_include, 4).is_some_and(|b| b == [0, 0, 0, 0]) {
        log::info!("capture skipped: excluded from clipboard history by the source app");
        return None;
    }
    if available(CF_HDROP) {
        let files = unsafe { GetClipboardData(CF_HDROP) }
            .map(|h| read_hdrop(HDROP(h.0)))
            .unwrap_or_default();
        if !files.is_empty() {
            return Some(Content::Files(files));
        }
    }
    // Office/LibreOffice put a bitmap rendering next to copied text; RTF marks text as the real content.
    if available(f.rtf)
        && let Some(text) =
            read_text().filter(|c| matches!(c, Content::Text(t, _) if !t.trim().is_empty()))
    {
        return Some(text);
    }
    if capture_images {
        if let Some(png) = get_bytes(f.png, usize::MAX) {
            return Some(Content::Png(png));
        }
        let rgba = [CF_DIBV5, CF_DIB]
            .into_iter()
            .find_map(|fmt| get_bytes(fmt, usize::MAX).and_then(|d| dib_to_rgba(&d)));
        if let Some((w, h, px)) = rgba {
            return Some(Content::Rgba(w, h, px));
        }
    }
    read_text()
}

/// `CF_UNICODETEXT` plus optional `HTML Format`. Clipboard must be open.
fn read_text() -> Option<Content> {
    let f = formats();
    // Enough UTF-16 units to fill MAX_TEXT_BYTES of UTF-8; the rest is truncated anyway.
    let raw = get_bytes(CF_UNICODETEXT, MAX_TEXT_BYTES * 2)?;
    let units: Vec<u16> = raw
        .as_chunks::<2>()
        .0
        .iter()
        .map(|&c| u16::from_le_bytes(c))
        .take_while(|&u| u != 0)
        .collect();
    let mut text = String::from_utf16_lossy(&units);
    truncate_utf8(&mut text, MAX_TEXT_BYTES);
    let html = get_bytes(f.html, MAX_HTML_BYTES + 1)
        .filter(|b| b.len() <= MAX_HTML_BYTES)
        .and_then(|mut b| {
            b.truncate(b.iter().position(|&c| c == 0).unwrap_or(b.len()));
            String::from_utf8(b).ok()
        })
        .filter(|h| h.starts_with("Version:"));
    Some(Content::Text(text, html))
}

/// The clipboard's current text (snippet `{clipboard}`), from any thread.
pub fn current_text() -> Option<String> {
    let _open = Opened::new()?;
    match read_text()? {
        Content::Text(text, _) => Some(text),
        _ => None,
    }
}

fn truncate_utf8(s: &mut String, max: usize) {
    if s.len() > max {
        let mut end = max;
        while !s.is_char_boundary(end) {
            end -= 1;
        }
        s.truncate(end);
    }
}

fn read_hdrop(drop: HDROP) -> Vec<String> {
    unsafe {
        let count = DragQueryFileW(drop, u32::MAX, None);
        (0..count)
            .filter_map(|i| {
                let len = DragQueryFileW(drop, i, None) as usize;
                let mut buf = vec![0u16; len + 1];
                let got = DragQueryFileW(drop, i, Some(&mut buf)) as usize;
                (got > 0).then(|| String::from_utf16_lossy(&buf[..got]))
            })
            .collect()
    }
}

/// Exe file name of the clipboard owner, falling back to the foreground window.
fn source_app() -> Option<String> {
    let owner = unsafe { GetClipboardOwner() }.ok();
    owner
        .and_then(exe_name)
        .or_else(|| exe_name(unsafe { GetForegroundWindow() }))
}

fn exe_name(hwnd: HWND) -> Option<String> {
    if hwnd.is_invalid() {
        return None;
    }
    unsafe {
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, Some(&mut pid));
        if pid == 0 {
            return None;
        }
        let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, pid).ok()?;
        let mut buf = [0u16; 1024];
        let mut len = buf.len() as u32;
        let ok = QueryFullProcessImageNameW(
            process,
            PROCESS_NAME_WIN32,
            PWSTR(buf.as_mut_ptr()),
            &mut len,
        );
        let _ = CloseHandle(process);
        ok.ok()?;
        let path = String::from_utf16_lossy(&buf[..len as usize]);
        Path::new(&path)
            .file_name()
            .map(|n| n.to_string_lossy().to_lowercase())
    }
}

pub fn hash(kind: &str, content: &[u8]) -> String {
    let mut h = Sha256::new();
    h.update(kind.as_bytes());
    h.update([0]);
    h.update(content);
    h.finalize().iter().map(|b| format!("{b:02x}")).collect()
}

fn to_new_clip(content: Content, source_app: Option<String>, images_dir: &Path) -> Option<NewClip> {
    let mut clip = NewClip {
        hash: String::new(),
        kind: "text",
        text: None,
        html: None,
        files: None,
        image_size: None,
        source_app,
    };
    match content {
        Content::Files(files) => {
            let mut sorted = files.clone();
            sorted.sort();
            clip.hash = hash("files", sorted.join("\n").as_bytes());
            clip.kind = "files";
            clip.files = Some(files);
        }
        Content::Text(text, html) => {
            if text.trim().is_empty() {
                return None;
            }
            clip.kind = detect::detect(&text);
            clip.hash = hash(clip.kind, text.as_bytes());
            clip.text = Some(text);
            clip.html = html;
        }
        Content::Png(png) => {
            let img = image::load_from_memory_with_format(&png, ImageFormat::Png).ok()?;
            clip.hash = hash("image", &png);
            clip.image_size = Some((img.width(), img.height()));
            clip.kind = "image";
            if let Err(e) = save_image(images_dir, &clip.hash, &png, &img) {
                log::error!("saving image failed: {e}");
                return None;
            }
        }
        Content::Rgba(w, h, px) => {
            let img = DynamicImage::ImageRgba8(RgbaImage::from_raw(w, h, px)?);
            let png = encode_png(&img).ok()?;
            return to_new_clip(Content::Png(png), clip.source_app, images_dir);
        }
    }
    Some(clip)
}

fn encode_png(img: &DynamicImage) -> image::ImageResult<Vec<u8>> {
    let mut out = Vec::new();
    img.write_to(&mut Cursor::new(&mut out), ImageFormat::Png)?;
    Ok(out)
}

pub fn save_image(dir: &Path, hash: &str, png: &[u8], img: &DynamicImage) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let full = db::image_path(dir, hash);
    let thumb = db::thumb_path(dir, hash);
    if !full.exists() {
        write_atomic(&full, png)?;
    }
    if !thumb.exists() {
        let small = if img.width() > THUMB_SIZE || img.height() > THUMB_SIZE {
            encode_png(&img.thumbnail(THUMB_SIZE, THUMB_SIZE)).map_err(std::io::Error::other)?
        } else {
            png.to_vec()
        };
        write_atomic(&thumb, &small)?;
    }
    Ok(())
}

fn write_atomic(path: &Path, data: &[u8]) -> std::io::Result<()> {
    let tmp = path.with_extension("tmp");
    std::fs::write(&tmp, data)?;
    std::fs::rename(&tmp, path)
}

/// Decodes a packed DIB (CF_DIB / CF_DIBV5) with 24/32-bit BI_RGB or 32-bit BI_BITFIELDS pixels.
pub fn dib_to_rgba(dib: &[u8]) -> Option<(u32, u32, Vec<u8>)> {
    const BI_RGB: u32 = 0;
    const BI_BITFIELDS: u32 = 3;
    let u32_at = |o: usize| Some(u32::from_le_bytes(dib.get(o..o + 4)?.try_into().ok()?));
    let header = u32_at(0)? as usize;
    let width = u32_at(4)? as i32;
    let height = u32_at(8)? as i32;
    let bpp = u16::from_le_bytes(dib.get(14..16)?.try_into().ok()?);
    let compression = u32_at(16)?;
    let colors = u32_at(32)? as usize;
    if header < 40 || width <= 0 || height == 0 || height == i32::MIN {
        return None;
    }
    let (w, h) = (width as usize, height.unsigned_abs() as usize);
    if w.checked_mul(h)? > 1 << 28 {
        return None;
    }
    // (red, green, blue, alpha) masks for 32 bpp; None means packed 24-bit BGR.
    let (masks, extra) = match (compression, bpp) {
        (BI_RGB, 24) => (None, 0),
        (BI_RGB, 32) => (Some([0xff_0000, 0xff00, 0xff, 0xff00_0000]), 0),
        // A plain BITMAPINFOHEADER is followed by three DWORD masks; V4/V5 headers embed them.
        (BI_BITFIELDS, 32) if header == 40 => {
            (Some([u32_at(40)?, u32_at(44)?, u32_at(48)?, 0]), 12)
        }
        (BI_BITFIELDS, 32) => (
            Some([u32_at(40)?, u32_at(44)?, u32_at(48)?, u32_at(52)?]),
            0,
        ),
        _ => return None,
    };
    let stride = (w * bpp as usize).div_ceil(32) * 4;
    let mut offset = header + extra + colors.checked_mul(4)?;
    let pixels_len = stride.checked_mul(h)?;
    // Some producers append masks after a V5 header too; detect it from the exact size.
    if compression == BI_BITFIELDS && extra == 0 && dib.len() == offset + 12 + pixels_len {
        offset += 12;
    }
    let data = dib.get(offset..offset.checked_add(pixels_len)?)?;

    let channel = |px: u32, mask: u32| -> u8 {
        if mask == 0 {
            return 0;
        }
        let max = mask >> mask.trailing_zeros();
        (((px & mask) >> mask.trailing_zeros()) as u64 * 255 / max as u64) as u8
    };
    let mut out = Vec::with_capacity(w * h * 4);
    for y in 0..h {
        let row_index = if height > 0 { h - 1 - y } else { y };
        let row = &data[row_index * stride..][..stride];
        for x in 0..w {
            match masks {
                None => {
                    let p = &row[x * 3..x * 3 + 3];
                    out.extend_from_slice(&[p[2], p[1], p[0], 255]);
                }
                Some([r, g, b, a]) => {
                    let px = u32::from_le_bytes(row[x * 4..x * 4 + 4].try_into().ok()?);
                    out.extend_from_slice(&[
                        channel(px, r),
                        channel(px, g),
                        channel(px, b),
                        channel(px, a),
                    ]);
                }
            }
        }
    }
    // Most apps leave the alpha byte at 0 for opaque images.
    if masks.is_some() && out.as_chunks::<4>().0.iter().all(|p| p[3] == 0) {
        out.as_chunks_mut::<4>()
            .0
            .iter_mut()
            .for_each(|p| p[3] = 255);
    }
    Some((w as u32, h as u32, out))
}

/// Encodes RGBA pixels as a bottom-up 32-bit BI_RGB packed DIB (BITMAPINFOHEADER + pixels).
pub fn rgba_to_dib(width: u32, height: u32, rgba: &[u8]) -> Vec<u8> {
    let row_bytes = width as usize * 4;
    let mut dib = Vec::with_capacity(40 + rgba.len());
    for v in [40u32, width, height] {
        dib.extend_from_slice(&v.to_le_bytes());
    }
    dib.extend_from_slice(&1u16.to_le_bytes()); // planes
    dib.extend_from_slice(&32u16.to_le_bytes()); // bpp
    dib.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB
    dib.extend_from_slice(&(rgba.len() as u32).to_le_bytes());
    dib.extend_from_slice(&[0; 16]); // resolution, colors used/important
    for row in rgba.chunks_exact(row_bytes.max(1)).rev() {
        for p in row.as_chunks::<4>().0 {
            dib.extend_from_slice(&[p[2], p[1], p[0], p[3]]);
        }
    }
    dib
}

/// DROPFILES header followed by a double-NUL terminated list of wide paths.
fn dropfiles(files: &[String]) -> Vec<u8> {
    let mut v = Vec::new();
    v.extend_from_slice(&20u32.to_le_bytes()); // pFiles: offset of the list
    v.extend_from_slice(&[0; 12]); // pt, fNC
    v.extend_from_slice(&1u32.to_le_bytes()); // fWide
    for unit in files
        .iter()
        .flat_map(|f| f.encode_utf16().chain([0]))
        .chain([0])
    {
        v.extend_from_slice(&unit.to_le_bytes());
    }
    v
}

fn utf16z(s: &str) -> Vec<u8> {
    s.encode_utf16()
        .chain([0])
        .flat_map(u16::to_le_bytes)
        .collect()
}

struct WriteJob {
    items: Vec<(u32, Vec<u8>)>,
    result: Result<(), String>,
}

/// Puts a stored clip on the clipboard. The listener ignores this change.
pub fn write(clip: &Clip, plain: bool) -> Result<(), String> {
    let f = formats();
    let s = &clip.summary;
    let items: Vec<(u32, Vec<u8>)> = match s.kind.as_str() {
        "image" => {
            let path = s.image_path.as_deref().ok_or("Image path missing")?;
            let png = std::fs::read(path).map_err(|e| format!("Cannot read image: {e}"))?;
            let img = image::load_from_memory_with_format(&png, ImageFormat::Png)
                .map_err(|e| format!("Cannot decode image: {e}"))?
                .to_rgba8();
            let dib = rgba_to_dib(img.width(), img.height(), img.as_raw());
            vec![(f.png, png), (CF_DIB, dib)]
        }
        "files" => vec![(CF_HDROP, dropfiles(s.files.as_deref().unwrap_or_default()))],
        _ => {
            let mut items = vec![(
                CF_UNICODETEXT,
                utf16z(clip.text.as_deref().unwrap_or_default()),
            )];
            // The stored CF_HTML is byte-identical to what was captured, so its offsets stay valid.
            if let (false, Some(html)) = (plain, &clip.html) {
                let mut bytes = html.as_bytes().to_vec();
                bytes.push(0);
                items.push((f.html, bytes));
            }
            items
        }
    };

    send(items)
}

/// Puts plain text on the clipboard. The listener ignores this change.
pub fn write_text(text: &str) -> Result<(), String> {
    send(vec![(CF_UNICODETEXT, utf16z(text))])
}

fn send(items: Vec<(u32, Vec<u8>)>) -> Result<(), String> {
    let hwnd = LISTENER_HWND.load(Ordering::SeqCst);
    if hwnd == 0 {
        return Err("Clipboard listener is not running".into());
    }
    let mut job = WriteJob {
        items,
        result: Err("Clipboard write was not processed".into()),
    };
    unsafe {
        SendMessageW(
            HWND(hwnd as *mut _),
            WM_WRITE,
            None,
            Some(LPARAM(&mut job as *mut WriteJob as isize)),
        );
    }
    job.result
}

fn write_now(items: &[(u32, Vec<u8>)]) -> Result<(), String> {
    let _open = Opened::new().ok_or("The clipboard is busy, try again")?;
    unsafe { EmptyClipboard() }.map_err(|e| e.to_string())?;
    for (format, data) in items {
        set_data(*format, data).map_err(|e| e.to_string())?;
    }
    Ok(())
}

fn set_data(format: u32, data: &[u8]) -> windows::core::Result<()> {
    unsafe {
        let mem = GlobalAlloc(GMEM_MOVEABLE, data.len())?;
        let ptr = GlobalLock(mem) as *mut u8;
        if ptr.is_null() {
            let _ = GlobalFree(Some(mem));
            return Err(windows::core::Error::from_thread());
        }
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
        let _ = GlobalUnlock(mem);
        // On success the system owns the memory; on failure it is still ours to free.
        if let Err(e) = SetClipboardData(format, Some(HANDLE(mem.0))) {
            let _ = GlobalFree(Some(mem));
            return Err(e);
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn header(w: i32, h: i32, bpp: u16, compression: u32, size: u32) -> Vec<u8> {
        let mut v = Vec::new();
        v.extend_from_slice(&size.to_le_bytes());
        v.extend_from_slice(&w.to_le_bytes());
        v.extend_from_slice(&h.to_le_bytes());
        v.extend_from_slice(&1u16.to_le_bytes());
        v.extend_from_slice(&bpp.to_le_bytes());
        v.extend_from_slice(&compression.to_le_bytes());
        v.resize(size as usize, 0);
        v
    }

    #[test]
    fn dib_roundtrip_32bit_bottom_up() {
        let rgba = vec![
            255, 0, 0, 255, 0, 255, 0, 128, // top row: red, translucent green
            0, 0, 255, 255, 1, 2, 3, 4, // bottom row
        ];
        let dib = rgba_to_dib(2, 2, &rgba);
        assert_eq!(dib_to_rgba(&dib), Some((2, 2, rgba)));
    }

    #[test]
    fn dib_24bit_bottom_up_with_padding() {
        let mut dib = header(1, 2, 24, 0, 40);
        dib.extend_from_slice(&[1, 2, 3, 0]); // bottom row: BGR + 1 byte padding
        dib.extend_from_slice(&[4, 5, 6, 0]); // top row
        let (w, h, px) = dib_to_rgba(&dib).unwrap();
        assert_eq!((w, h), (1, 2));
        assert_eq!(px, [6, 5, 4, 255, 3, 2, 1, 255]);
    }

    #[test]
    fn dib_32bit_bitfields_top_down_zero_alpha_is_opaque() {
        let mut dib = header(1, -2, 32, 3, 40);
        for m in [0xff_0000u32, 0xff00, 0xff] {
            dib.extend_from_slice(&m.to_le_bytes());
        }
        dib.extend_from_slice(&[10, 20, 30, 0]); // top row (B, G, R, X)
        dib.extend_from_slice(&[40, 50, 60, 0]);
        let (_, _, px) = dib_to_rgba(&dib).unwrap();
        assert_eq!(px, [30, 20, 10, 255, 60, 50, 40, 255]);
    }

    #[test]
    fn dib_v5_bitfields_with_alpha() {
        let mut dib = header(1, 1, 32, 3, 124);
        for (i, m) in [0xff_0000u32, 0xff00, 0xff, 0xff00_0000].iter().enumerate() {
            dib[40 + i * 4..44 + i * 4].copy_from_slice(&m.to_le_bytes());
        }
        dib.extend_from_slice(&[1, 2, 3, 4]);
        assert_eq!(dib_to_rgba(&dib).unwrap().2, [3, 2, 1, 4]);
    }

    #[test]
    fn dib_rejects_bad_input() {
        assert!(dib_to_rgba(&[]).is_none());
        assert!(dib_to_rgba(&header(1, 1, 16, 0, 40)).is_none()); // unsupported bpp
        assert!(dib_to_rgba(&header(4, 4, 32, 0, 40)).is_none()); // truncated pixels
        assert!(dib_to_rgba(&header(0, 4, 32, 0, 40)).is_none());
    }

    #[test]
    fn dropfiles_roundtrip_through_shell() {
        let files = vec![r"C:\a b\ñ.txt".to_string(), r"D:\x".to_string()];
        let data = dropfiles(&files);
        unsafe {
            let mem = GlobalAlloc(GMEM_MOVEABLE, data.len()).unwrap();
            let ptr = GlobalLock(mem) as *mut u8;
            std::ptr::copy_nonoverlapping(data.as_ptr(), ptr, data.len());
            let _ = GlobalUnlock(mem);
            assert_eq!(read_hdrop(HDROP(mem.0)), files);
            let _ = GlobalFree(Some(mem));
        }
    }

    #[test]
    fn truncation_respects_char_boundaries() {
        let mut s = "aé".repeat(10);
        truncate_utf8(&mut s, 4);
        assert_eq!(s, "aéa");
    }
}
