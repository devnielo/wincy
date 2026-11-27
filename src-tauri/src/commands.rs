//! `#[tauri::command]`s invoked by the frontend. Errors are returned as strings.
//!
//! They are `async` so they run off the main thread (Tauri runs sync commands on it).

use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

use serde::Serialize;

use crate::db::{self, Clip, ClipSummary, ShortcutTarget, Snippet};
use crate::settings::{self, Settings};
use crate::snippets::{self, SnippetInput};
use crate::status::Status;
use crate::transfer::{self, ExportResult, ImportResult};
use crate::{AppState, LOG_FILE, clipboard, ocr, paste, paths, shortcuts, tray, window};

/// Unpinned clips `get_clips` loads; larger histories are searched with `search_clips`.
const LOADED_UNPINNED: u32 = 2000;
const MAX_SEARCH_LIMIT: u32 = 1000;
const MAX_QUERY_BYTES: usize = 4096;

type Res<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn changed(app: &AppHandle) {
    let _ = app.emit("clips-changed", ());
}

#[derive(Serialize)]
pub struct ClipPage {
    clips: Vec<ClipSummary>,
    /// All clips in the database, pinned included.
    total: i64,
}

#[tauri::command]
pub async fn get_clips(app: AppHandle) -> Res<ClipPage> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    Ok(ClipPage {
        clips: db::list(&conn, &state.images_dir, LOADED_UNPINNED).map_err(err)?,
        total: db::count(&conn).map_err(err)?,
    })
}

#[tauri::command]
pub async fn search_clips(app: AppHandle, query: String, limit: u32) -> Res<Vec<ClipSummary>> {
    if query.len() > MAX_QUERY_BYTES {
        return Err("The search is too long".into());
    }
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    db::search(
        &conn,
        &state.images_dir,
        &query,
        limit.clamp(1, MAX_SEARCH_LIMIT),
        LOADED_UNPINNED,
    )
    .map_err(err)
}

#[tauri::command]
pub async fn get_clip(app: AppHandle, id: i64) -> Res<Clip> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    db::get(&conn, id, &state.images_dir)
        .map_err(err)?
        .ok_or_else(|| "Clip not found".into())
}

#[tauri::command]
pub async fn paste_clip(app: AppHandle, id: i64, plain: bool) -> Res<()> {
    // Blocks for up to ~650 ms waiting for modifier keys; keep it off the async workers.
    tauri::async_runtime::spawn_blocking(move || paste::paste(&app, id, plain))
        .await
        .map_err(err)?
}

#[tauri::command]
pub async fn paste_text(app: AppHandle, text: String) -> Res<()> {
    tauri::async_runtime::spawn_blocking(move || paste::paste_text(&app, &text))
        .await
        .map_err(err)?
}

#[tauri::command]
pub async fn toggle_pin(app: AppHandle, id: i64) -> Res<bool> {
    let state = app.state::<AppState>();
    let pinned = {
        let conn = state.db.lock().unwrap();
        db::toggle_pin(&conn, id).map_err(err)?
    };
    if !pinned {
        shortcuts::sync(&app);
    }
    changed(&app);
    Ok(pinned)
}

#[tauri::command]
pub async fn set_clip_shortcut(app: AppHandle, id: i64, shortcut: Option<String>) -> Res<()> {
    let state = app.state::<AppState>();
    let main = state.settings.lock().unwrap().shortcut.clone();
    {
        let conn = state.db.lock().unwrap();
        let shortcut = shortcut
            .as_deref()
            .filter(|s| !s.trim().is_empty())
            .map(|s| shortcuts::validate(&conn, &main, s, Some(ShortcutTarget::Clip(id))))
            .transpose()?;
        if !db::set_clip_shortcut(&conn, id, shortcut.as_deref()).map_err(err)? {
            return Err("Only pinned clips can have a shortcut".into());
        }
    }
    shortcuts::sync(&app);
    changed(&app);
    Ok(())
}

#[tauri::command]
pub async fn get_snippets(app: AppHandle) -> Res<Vec<Snippet>> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    db::snippets(&conn).map_err(err)
}

#[tauri::command]
pub async fn save_snippet(app: AppHandle, snippet: SnippetInput) -> Res<Snippet> {
    snippets::save(&app, snippet)
}

#[tauri::command]
pub async fn delete_snippet(app: AppHandle, id: i64) -> Res<()> {
    snippets::delete(&app, id)
}

#[tauri::command]
pub async fn paste_snippet(app: AppHandle, id: i64) -> Res<()> {
    tauri::async_runtime::spawn_blocking(move || snippets::paste(&app, id))
        .await
        .map_err(err)?
}

#[tauri::command]
pub async fn export_history(
    app: AppHandle,
    path: String,
    include_images: bool,
) -> Res<ExportResult> {
    tauri::async_runtime::spawn_blocking(move || transfer::export(&app, &path, include_images))
        .await
        .map_err(err)?
}

#[tauri::command]
pub async fn import_history(app: AppHandle, path: String) -> Res<ImportResult> {
    tauri::async_runtime::spawn_blocking(move || transfer::import(&app, &path))
        .await
        .map_err(err)?
}

#[tauri::command]
pub async fn delete_clip(app: AppHandle, id: i64) -> Res<()> {
    let state = app.state::<AppState>();
    let images = {
        let conn = state.db.lock().unwrap();
        let images = db::delete(&conn, id).map_err(err)?;
        db::checkpoint(&conn);
        images
    };
    db::remove_images(&state.images_dir, &images);
    shortcuts::sync(&app);
    changed(&app);
    Ok(())
}

/// Deletes all unpinned clips (also used by the tray menu). Shortcuts need no sync:
/// only pinned clips have one (and the tray calls this on the main thread, where `sync`
/// could deadlock waiting for the main thread).
pub fn clear(app: &AppHandle) -> Res<()> {
    let state = app.state::<AppState>();
    let images = {
        let conn = state.db.lock().unwrap();
        let images = db::clear(&conn).map_err(err)?;
        db::checkpoint(&conn);
        images
    };
    db::remove_images(&state.images_dir, &images);
    changed(app);
    Ok(())
}

/// Deletes unpinned clips older than `expireDays` (none when it is 0).
pub fn expire(app: &AppHandle) {
    let state = app.state::<AppState>();
    let days = state.settings.lock().unwrap().expire_days;
    if days == 0 {
        return;
    }
    let cutoff = db::now_ms() - i64::from(days) * 86_400_000;
    let expired = {
        let conn = state.db.lock().unwrap();
        let expired = db::expire(&conn, cutoff);
        if matches!(expired, Ok((n, _)) if n > 0) {
            db::checkpoint(&conn);
        }
        expired
    };
    match expired {
        Ok((0, _)) => {}
        Ok((count, images)) => {
            db::remove_images(&state.images_dir, &images);
            log::info!("expired {count} clips");
            changed(app);
        }
        Err(e) => log::warn!("expire: {e}"),
    }
}

#[tauri::command]
pub async fn clear_history(app: AppHandle) -> Res<()> {
    clear(&app)
}

#[tauri::command]
pub async fn get_settings(app: AppHandle) -> Res<Settings> {
    Ok(app.state::<AppState>().settings.lock().unwrap().clone())
}

#[tauri::command]
pub async fn save_settings(app: AppHandle, settings: Settings) -> Res<Settings> {
    settings::save(&app, settings)
}

#[tauri::command]
pub async fn hide_window(app: AppHandle) -> Res<()> {
    window::hide_and_restore(&app);
    Ok(())
}

#[tauri::command]
pub async fn set_preview(app: AppHandle, id: Option<i64>) -> Res<()> {
    // On the main thread: Win32 window calls; window::hide queues a preview hide behind it.
    let handle = app.clone();
    app.run_on_main_thread(move || window::set_preview(&handle, id))
        .map_err(err)
}

#[tauri::command]
pub async fn get_paused(app: AppHandle) -> Res<bool> {
    Ok(app.state::<AppState>().paused.load(Ordering::SeqCst))
}

/// Sets the capture pause flag and notifies the tray and the frontend.
pub fn apply_paused(app: &AppHandle, paused: bool) -> bool {
    let state = app.state::<AppState>();
    state.paused.store(paused, Ordering::SeqCst);
    tray::refresh(app);
    let settings = state.settings.lock().unwrap().clone();
    let _ = app.emit("settings-changed", settings);
    paused
}

#[tauri::command]
pub async fn set_paused(app: AppHandle, paused: bool) -> Res<bool> {
    Ok(apply_paused(&app, paused))
}

#[tauri::command]
pub async fn open_data_dir(app: AppHandle) -> Res<()> {
    let dir = paths::data_dir(&app).map_err(err)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(err)
}

#[tauri::command]
pub async fn open_log_dir(app: AppHandle) -> Res<()> {
    let dir = paths::log_dir(&app).map_err(err)?;
    std::fs::create_dir_all(&dir).map_err(err)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(err)
}

#[tauri::command]
pub async fn get_status(app: AppHandle) -> Res<Status> {
    Ok(app.state::<AppState>().status.lock().unwrap().clone())
}

/// A bug-report summary, also put on the clipboard. Never includes clip contents.
#[tauri::command]
pub async fn copy_diagnostics(app: AppHandle) -> Res<String> {
    let state = app.state::<AppState>();
    let settings = state.settings.lock().unwrap().clone();
    let mut settings_json = serde_json::to_value(&settings).map_err(err)?;
    settings_json["ignoredApps"] = settings.ignored_apps.len().into();
    let clips = {
        let conn = state.db.lock().unwrap();
        db::count(&conn).map_err(err)?
    };
    let status = state.status.lock().unwrap().clone();
    log::logger().flush();
    let log = std::fs::read(std::path::Path::new(&status.log_dir).join(format!("{LOG_FILE}.log")))
        .map(|b| String::from_utf8_lossy(&b).into_owned())
        .unwrap_or_default();
    let lines: Vec<&str> = log.lines().collect();
    let tail = lines[lines.len().saturating_sub(200)..].join("\n");
    let text = format!(
        "Wincy {}{}{}\nWindows {}\nOCR: {}\nClips: {clips}\nShortcut error: {}\n\
         Shortcut conflicts: {:?}\nSettings: {settings_json:#}\n\nLast log lines:\n{tail}\n",
        app.package_info().version,
        if status.portable { " (portable)" } else { "" },
        if status.store { " (Store)" } else { "" },
        windows_version(),
        ocr::availability(),
        status.shortcut_error.as_deref().unwrap_or("none"),
        status.shortcut_conflicts,
    );
    clipboard::write_text(&text)?;
    Ok(text)
}

fn windows_version() -> String {
    windows::System::Profile::AnalyticsInfo::VersionInfo()
        .and_then(|v| v.DeviceFamilyVersion())
        .ok()
        .and_then(|v| v.to_string().parse::<u64>().ok())
        .map_or_else(
            || "unknown".into(),
            |v| {
                format!(
                    "{}.{}.{}.{}",
                    v >> 48,
                    (v >> 32) & 0xffff,
                    (v >> 16) & 0xffff,
                    v & 0xffff
                )
            },
        )
}
