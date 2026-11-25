//! `#[tauri::command]`s invoked by the frontend. Errors are returned as strings.
//!
//! They are `async` so they run off the main thread (Tauri runs sync commands on it).

use std::sync::atomic::Ordering;

use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_opener::OpenerExt;

use crate::db::{self, Clip, ClipSummary};
use crate::settings::{self, Settings};
use crate::{AppState, paste, tray, window};

type Res<T> = Result<T, String>;

fn err(e: impl std::fmt::Display) -> String {
    e.to_string()
}

fn changed(app: &AppHandle) {
    let _ = app.emit("clips-changed", ());
}

#[tauri::command]
pub async fn get_clips(app: AppHandle) -> Res<Vec<ClipSummary>> {
    let state = app.state::<AppState>();
    let conn = state.db.lock().unwrap();
    db::list(&conn, &state.images_dir).map_err(err)
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
pub async fn toggle_pin(app: AppHandle, id: i64) -> Res<bool> {
    let state = app.state::<AppState>();
    let pinned = {
        let conn = state.db.lock().unwrap();
        db::toggle_pin(&conn, id).map_err(err)?
    };
    changed(&app);
    Ok(pinned)
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
    changed(&app);
    Ok(())
}

/// Deletes all unpinned clips (also used by the tray menu).
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
    let dir = app.path().app_data_dir().map_err(err)?;
    app.opener()
        .open_path(dir.to_string_lossy(), None::<&str>)
        .map_err(err)
}
