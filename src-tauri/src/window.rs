//! The palette window (label `main`): show next to the cursor, hide, focus restoration;
//! and the preview panel (label `preview`) beside it, which never takes focus.

use std::sync::atomic::{AtomicIsize, Ordering};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, PhysicalPosition, PhysicalSize, WebviewWindow};
use windows::Win32::Foundation::HWND;
use windows::Win32::System::Threading::GetCurrentProcessId;
use windows::Win32::UI::WindowsAndMessaging::{
    GWL_EXSTYLE, GetClassNameW, GetForegroundWindow, GetWindowLongPtrW, GetWindowThreadProcessId,
    HWND_TOPMOST, IsWindow, IsWindowVisible, SW_HIDE, SW_SHOWNOACTIVATE, SWP_NOACTIVATE,
    SetForegroundWindow, SetWindowLongPtrW, SetWindowPos, ShowWindow, WS_EX_NOACTIVATE,
};

use crate::settings::Theme;
use crate::status;

const LABEL: &str = "main";
const PREVIEW: &str = "preview";
const WIDTH: f64 = 600.0;
const HEIGHT: f64 = 460.0;
const PREVIEW_SIZE: f64 = 560.0;
const CURSOR_GAP: f64 = 8.0;

/// Window that had focus before the palette opened (paste target).
static PREVIOUS: AtomicIsize = AtomicIsize::new(0);

#[derive(Clone, Serialize)]
struct Shown<'a> {
    view: &'a str,
}

#[derive(Clone, Serialize)]
struct PreviewClip {
    id: i64,
}

/// Mica on Windows 11; Acrylic when Mica is unavailable (Windows 10). Failures are cosmetic.
fn apply_vibrancy(app: &AppHandle) {
    for label in [LABEL, PREVIEW] {
        if let Some(win) = app.get_webview_window(label)
            && window_vibrancy::apply_mica(&win, None).is_err()
        {
            // Acrylic's tint is fixed, unlike Mica: pick it from the window's resolved theme.
            let tint = match win.theme() {
                Ok(tauri::Theme::Light) => (243, 243, 243, 160),
                _ => (18, 18, 18, 160),
            };
            let _ = window_vibrancy::apply_acrylic(&win, Some(tint));
        }
    }
}

/// Mica/Acrylic follow the window theme, so it tracks the app theme, not only the system one.
pub fn apply_theme(app: &AppHandle, theme: Theme) {
    let theme = match theme {
        Theme::System => None,
        Theme::Light => Some(tauri::Theme::Light),
        Theme::Dark => Some(tauri::Theme::Dark),
    };
    for label in [LABEL, PREVIEW] {
        if let Some(win) = app.get_webview_window(label) {
            let _ = win.set_theme(theme);
        }
    }
    apply_vibrancy(app);
}

fn preview_hwnd(app: &AppHandle) -> Option<HWND> {
    app.get_webview_window(PREVIEW)?.hwnd().ok()
}

/// The preview must never be activated, so the palette keeps focus and its hide-on-blur.
/// (`focusable: false` in tauri.conf.json sets the same style; this keeps it explicit.)
pub fn setup_preview(app: &AppHandle) {
    if let Some(hwnd) = preview_hwnd(app) {
        unsafe {
            let style = GetWindowLongPtrW(hwnd, GWL_EXSTYLE);
            SetWindowLongPtrW(hwnd, GWL_EXSTYLE, style | WS_EX_NOACTIVATE.0 as isize);
        }
    }
}

/// True when the foreground window is the palette or the preview. A WebView2 click can still
/// focus the preview, and the bounce back to the palette can arrive after a stale palette blur:
/// focus moving between our own two windows must not hide the palette.
pub fn own_window_is_foreground(app: &AppHandle) -> bool {
    let fg = unsafe { GetForegroundWindow() };
    [LABEL, PREVIEW]
        .iter()
        .filter_map(|l| app.get_webview_window(l)?.hwnd().ok())
        .any(|h| h == fg)
}

// The preview is shown and hidden with raw ShowWindow, never Tauri show()/hide(): Tauri's show
// activates the window, and its hide is a no-op while its own visible flag was never set.
fn preview_visible(app: &AppHandle) -> bool {
    preview_hwnd(app).is_some_and(|h| unsafe { IsWindowVisible(h) }.as_bool())
}

pub fn hide_preview(app: &AppHandle) {
    if let Some(hwnd) = preview_hwnd(app) {
        let _ = unsafe { ShowWindow(hwnd, SW_HIDE) };
    }
}

/// `set_preview`: `Some(id)` places and shows the panel beside the palette and sends it the
/// clip; `None` (or a hidden palette, e.g. a late debounced call) hides it. Main thread only.
pub fn set_preview(app: &AppHandle, id: Option<i64>) {
    let palette_visible = app
        .get_webview_window(LABEL)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    let (Some(id), true) = (id, palette_visible) else {
        hide_preview(app);
        return;
    };
    let _ = app.emit_to(PREVIEW, "preview-clip", PreviewClip { id });
    if let Err(e) = place_preview(app) {
        log::warn!("positioning preview failed: {e}");
    }
    if let Some(hwnd) = preview_hwnd(app) {
        let _ = unsafe { ShowWindow(hwnd, SW_SHOWNOACTIVATE) };
    }
}

/// Keeps the preview beside the palette while the palette is dragged.
pub fn palette_moved(app: &AppHandle) {
    if preview_visible(app)
        && let Err(e) = place_preview(app)
    {
        log::warn!("positioning preview failed: {e}");
    }
}

/// Physical pixels throughout: the palette's outer rect, its monitor's work area and DPI.
fn place_preview(app: &AppHandle) -> tauri::Result<()> {
    let (Some(palette), Some(hwnd)) = (app.get_webview_window(LABEL), preview_hwnd(app)) else {
        return Ok(());
    };
    let pos = palette.outer_position()?;
    let palette_size = palette.outer_size()?;
    let monitor = match palette.current_monitor()? {
        Some(m) => m,
        None => match app.primary_monitor()? {
            Some(m) => m,
            None => return Ok(()),
        },
    };
    let scale = monitor.scale_factor();
    let size = (PREVIEW_SIZE * scale).round() as i32;
    let gap = (CURSOR_GAP * scale).round() as i32;
    let area = monitor.work_area();
    let x = beside(
        pos.x,
        palette_size.width as i32,
        size,
        gap,
        area.position.x,
        area.size.width as i32,
    );
    let y = clamp_into(pos.y, size, area.position.y, area.size.height as i32);
    // Twice: moving to a monitor with another DPI makes Windows rescale the window.
    for _ in 0..2 {
        // HWND_TOPMOST raises it like show() raises the palette; SW_SHOWNOACTIVATE never does.
        unsafe { SetWindowPos(hwnd, Some(HWND_TOPMOST), x, y, size, size, SWP_NOACTIVATE) }
            .map_err(|e| tauri::Error::Anyhow(e.into()))?;
    }
    Ok(())
}

pub fn show(app: &AppHandle, view: &str) {
    let Some(win) = app.get_webview_window(LABEL) else {
        return;
    };
    let own = win.hwnd().map_or(0, |h| h.0 as isize);
    let foreground = unsafe { GetForegroundWindow() };
    let mut pid = 0;
    unsafe { GetWindowThreadProcessId(foreground, Some(&mut pid)) };
    if pid != unsafe { GetCurrentProcessId() } {
        // Opened from the tray: the taskbar is no paste target. With none, Windows activates
        // the app below the palette when it hides, which is where the user was working.
        let target = if is_taskbar(foreground) {
            0
        } else {
            foreground.0 as isize
        };
        PREVIOUS.store(target, Ordering::SeqCst);
    } else if foreground.0 as isize != own {
        // Our own hidden tray window (the tray menu makes it foreground) is no target either.
        // The palette itself being foreground (re-open) keeps the old target.
        PREVIOUS.store(0, Ordering::SeqCst);
    }
    if let Err(e) = place_near_cursor(app, &win) {
        log::warn!("positioning palette failed: {e}");
    }
    let _ = win.show();
    let _ = win.set_focus();
    status::refresh_accent(app);
    let _ = app.emit_to(LABEL, "palette-shown", Shown { view });
}

fn is_taskbar(hwnd: HWND) -> bool {
    let mut buf = [0u16; 64];
    let len = unsafe { GetClassNameW(hwnd, &mut buf) } as usize;
    matches!(
        String::from_utf16_lossy(&buf[..len]).as_str(),
        "Shell_TrayWnd"
            | "Shell_SecondaryTrayWnd"
            | "NotifyIconOverflowWindow"
            | "TopLevelWindowForOverflowXamlIsland"
    )
}

pub fn hide(app: &AppHandle) {
    hide_preview(app);
    if let Some(win) = app.get_webview_window(LABEL) {
        let _ = win.hide();
    }
    // Off the main thread the palette hide is only queued; a set_preview task queued before it
    // still sees the palette visible and re-shows the preview. Hide it again behind both.
    let h = app.clone();
    let _ = app.run_on_main_thread(move || hide_preview(&h));
}

/// Hides the palette and gives focus back to the window that had it before.
pub fn hide_and_restore(app: &AppHandle) {
    hide(app);
    let prev = HWND(PREVIOUS.load(Ordering::SeqCst) as *mut _);
    if !prev.is_invalid() && unsafe { IsWindow(Some(prev)) }.as_bool() {
        let _ = unsafe { SetForegroundWindow(prev) };
    }
}

pub fn toggle(app: &AppHandle) {
    let visible = app
        .get_webview_window(LABEL)
        .and_then(|w| w.is_visible().ok())
        .unwrap_or(false);
    if visible {
        hide_and_restore(app);
    } else {
        show(app, "palette");
    }
}

/// Everything here is in physical pixels: the cursor, the monitor work area and the
/// window size scaled by that monitor's DPI.
fn place_near_cursor(app: &AppHandle, win: &WebviewWindow) -> tauri::Result<()> {
    let cursor = app.cursor_position()?;
    let monitor = match app.monitor_from_point(cursor.x, cursor.y)? {
        Some(m) => m,
        None => match app.primary_monitor()? {
            Some(m) => m,
            None => return Ok(()),
        },
    };
    let scale = monitor.scale_factor();
    let size = PhysicalSize::new(
        (WIDTH * scale).round() as i32,
        (HEIGHT * scale).round() as i32,
    );
    let gap = (CURSOR_GAP * scale).round() as i32;
    let area = monitor.work_area();
    let x = place(
        cursor.x as i32,
        size.width,
        gap,
        area.position.x,
        area.size.width as i32,
    );
    let y = place(
        cursor.y as i32,
        size.height,
        gap,
        area.position.y,
        area.size.height as i32,
    );
    let position = PhysicalPosition::new(x, y);
    let size = PhysicalSize::new(size.width as u32, size.height as u32);
    // Moving to a monitor with another DPI makes Windows rescale the window; re-apply both.
    win.set_position(position)?;
    win.set_size(size)?;
    win.set_position(position)
}

/// One axis: after the cursor + gap, flipped before it if it overflows, clamped to the area.
fn place(cursor: i32, size: i32, gap: i32, start: i32, len: i32) -> i32 {
    let mut p = cursor + gap;
    if p + size > start + len {
        p = cursor - gap - size;
    }
    clamp_into(p, size, start, len)
}

/// Keeps `[p, p + size)` inside `[start, start + len)`; the start wins when it is too big.
fn clamp_into(p: i32, size: i32, start: i32, len: i32) -> i32 {
    p.clamp(start, (start + len - size).max(start))
}

/// Preview x: right of the palette (+ gap) if it fits, else left, else the side with more room,
/// clamped to the area (it may then overlap the palette).
fn beside(palette: i32, palette_len: i32, size: i32, gap: i32, start: i32, len: i32) -> i32 {
    let end = start + len;
    let right = palette + palette_len + gap;
    let left = palette - gap - size;
    let right_room = end - right;
    let left_room = palette - gap - start;
    let p = if right_room >= size || (left_room < size && right_room >= left_room) {
        right
    } else {
        left
    };
    clamp_into(p, size, start, len)
}

#[cfg(test)]
mod tests {
    use super::{beside, clamp_into, place};

    #[test]
    fn preview_placement() {
        // Palette 600 wide, preview 560, gap 8.
        assert_eq!(beside(100, 600, 560, 8, 0, 1920), 708); // right fits
        assert_eq!(beside(1200, 600, 560, 8, 0, 1920), 632); // right overflows: left
        // Area 0..1200, neither side fits. x=100: right room 492 > left 92 → right, clamped.
        assert_eq!(beside(100, 600, 560, 8, 0, 1200), 640);
        // x=500: right room 92 < left room 492 → left (-68), clamped to 0.
        assert_eq!(beside(500, 600, 560, 8, 0, 1200), 0);
        assert_eq!(beside(-1700, 600, 560, 8, -1920, 1920), -1092); // monitor left of primary
        // Vertical: top aligned with the palette, clamped to the work area.
        assert_eq!(clamp_into(200, 560, 0, 1040), 200);
        assert_eq!(clamp_into(700, 560, 0, 1040), 480);
        assert_eq!(clamp_into(-50, 560, 0, 1040), 0);
        assert_eq!(clamp_into(0, 560, 0, 400), 0); // taller than the area
    }

    #[test]
    fn placement() {
        assert_eq!(place(100, 50, 8, 0, 1000), 108); // fits after the cursor
        assert_eq!(place(980, 50, 8, 0, 1000), 922); // flipped before the cursor
        assert_eq!(place(30, 50, 8, 0, 70), 0); // too big either way: clamped
        assert_eq!(place(-1900, 50, 8, -1920, 1920), -1892); // monitor left of primary
        assert_eq!(place(10, 500, 8, 0, 300), 0); // larger than the area
    }
}
