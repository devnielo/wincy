//! Text recognition in image clips with the built-in Windows OCR (`Windows.Media.Ocr`).
//!
//! One low-priority worker thread owns the engine and is fed image hashes through a channel,
//! so recognition never runs on the clipboard listener thread.

use std::path::Path;
use std::sync::OnceLock;
use std::sync::mpsc::{Sender, channel};

use image::ImageFormat;
use image::imageops::FilterType;
use tauri::{AppHandle, Emitter, Manager};
use windows::Graphics::Imaging::{BitmapPixelFormat, SoftwareBitmap};
use windows::Media::Ocr::OcrEngine;
use windows::Storage::Streams::DataWriter;
use windows::Win32::System::Threading::{
    GetCurrentThread, SetThreadPriority, THREAD_PRIORITY_BELOW_NORMAL,
};
use windows::Win32::System::WinRT::{RO_INIT_MULTITHREADED, RoInitialize};

use crate::{AppState, db};

const BACKFILL_LIMIT: u32 = 200;

static QUEUE: OnceLock<Sender<String>> = OnceLock::new();
/// Unset until the worker has tried to create the engine.
static AVAILABLE: OnceLock<bool> = OnceLock::new();

/// Spawns the worker. Without an installed OCR language it logs once and exits, and
/// `queue` becomes a no-op.
pub fn start(app: AppHandle) -> std::io::Result<()> {
    let (tx, rx) = channel::<String>();
    let _ = QUEUE.set(tx);
    std::thread::Builder::new()
        .name("ocr".into())
        .spawn(move || {
            unsafe {
                let _ = SetThreadPriority(GetCurrentThread(), THREAD_PRIORITY_BELOW_NORMAL);
                // MTA: RecognizeAsync(..).join() blocks, which is only allowed off an STA.
                if let Err(e) = RoInitialize(RO_INIT_MULTITHREADED) {
                    log::warn!("ocr: RoInitialize failed: {e}");
                }
            }
            let engine = match OcrEngine::TryCreateFromUserProfileLanguages() {
                Ok(engine) => engine,
                Err(e) => {
                    let _ = AVAILABLE.set(false);
                    log::info!("OCR unavailable (no OCR language installed?): {e}");
                    return;
                }
            };
            let _ = AVAILABLE.set(true);
            log::info!("OCR available");
            backfill(&app);
            for hash in rx {
                process(&app, &engine, &hash);
            }
        })?;
    Ok(())
}

/// Queues an image clip for recognition.
pub fn queue(hash: String) {
    if let Some(tx) = QUEUE.get() {
        let _ = tx.send(hash);
    }
}

/// Queues the most recent images that were never recognized (startup, OCR switched on).
pub fn backfill(app: &AppHandle) {
    let state = app.state::<AppState>();
    if !state.settings.lock().unwrap().ocr_images {
        return;
    }
    let pending = {
        let conn = state.db.lock().unwrap();
        db::ocr_pending(&conn, BACKFILL_LIMIT)
    };
    match pending {
        Ok(hashes) => hashes.into_iter().for_each(queue),
        Err(e) => log::warn!("ocr backfill: {e}"),
    }
}

pub fn availability() -> &'static str {
    match AVAILABLE.get() {
        Some(true) => "available",
        Some(false) => "unavailable",
        None => "not checked yet",
    }
}

fn process(app: &AppHandle, engine: &OcrEngine, hash: &str) {
    let state = app.state::<AppState>();
    // Settings may have changed, or the clip been deleted or recognized, since it was queued.
    if !state.settings.lock().unwrap().ocr_images {
        return;
    }
    let pending = {
        let conn = state.db.lock().unwrap();
        db::needs_ocr(&conn, hash)
    };
    if !matches!(pending, Ok(true)) {
        return;
    }
    let text = match recognize(engine, &db::image_path(&state.images_dir, hash)) {
        Ok(text) => text,
        Err(e) => {
            log::warn!("ocr failed: {e}");
            return;
        }
    };
    let stored = {
        let conn = state.db.lock().unwrap();
        db::set_ocr_text(&conn, hash, &text)
    };
    match stored {
        Ok(true) => {
            let _ = app.emit("clips-changed", ());
        }
        Ok(false) => {}
        Err(e) => log::warn!("storing ocr text failed: {e}"),
    }
}

/// Recognized lines joined with `\n`, trimmed. Blocks; worker thread only.
fn recognize(engine: &OcrEngine, png_path: &Path) -> Result<String, Box<dyn std::error::Error>> {
    let png = std::fs::read(png_path)?;
    let mut img = image::load_from_memory_with_format(&png, ImageFormat::Png)?;
    let max = OcrEngine::MaxImageDimension()?;
    if img.width() > max || img.height() > max {
        img = img.resize(max, max, FilterType::Triangle);
    }
    let rgba = img.to_rgba8();
    // Flattened onto white so dark text on a transparent background stays readable.
    let bgra: Vec<u8> = rgba
        .pixels()
        .flat_map(|p| {
            let [r, g, b, a] = p.0.map(u32::from);
            let mix = |c: u32| (c * a / 255 + 255 - a) as u8;
            [mix(b), mix(g), mix(r), 255]
        })
        .collect();
    let writer = DataWriter::new()?;
    writer.WriteBytes(&bgra)?;
    let bitmap = SoftwareBitmap::CreateCopyFromBuffer(
        &writer.DetachBuffer()?,
        BitmapPixelFormat::Bgra8,
        rgba.width() as i32,
        rgba.height() as i32,
    )?;
    let result = engine.RecognizeAsync(&bitmap)?.join()?;
    let lines = result
        .Lines()?
        .into_iter()
        .map(|line| line.Text().map(|t| t.to_string()))
        .collect::<windows::core::Result<Vec<_>>>()?;
    Ok(lines.join("\n").trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercises the WinRT pipeline (decode, downscale, SoftwareBitmap, RecognizeAsync)
    /// where an OCR language is installed.
    #[test]
    fn blank_oversized_image_has_no_text() {
        let Ok(engine) = OcrEngine::TryCreateFromUserProfileLanguages() else {
            return;
        };
        let path = std::env::temp_dir().join(format!("wincy-ocr-{}.png", std::process::id()));
        image::RgbaImage::from_pixel(6000, 120, image::Rgba([0, 0, 0, 0]))
            .save(&path)
            .unwrap();
        let text = recognize(&engine, &path);
        let _ = std::fs::remove_file(&path);
        assert_eq!(text.unwrap(), "");
    }
}
