mod clipboard;
mod commands;
mod db;
mod detect;
mod paste;
mod settings;
mod tray;
mod window;

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;

use rusqlite::Connection;
use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;

use settings::Settings;

pub struct AppState {
    pub db: Mutex<Connection>,
    pub settings: Mutex<Settings>,
    pub paused: AtomicBool,
    pub images_dir: PathBuf,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be registered first so a second launch exits before doing anything else.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show(app, "palette");
        }))
        .plugin(
            tauri_plugin_global_shortcut::Builder::new()
                .with_handler(|app, _shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        window::toggle(app);
                    }
                })
                .build(),
        )
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--autostart"])
                .build(),
        )
        .plugin(tauri_plugin_updater::Builder::new().build())
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .setup(|app| {
            let handle = app.handle();
            let data_dir = app.path().app_data_dir()?;
            let images_dir = data_dir.join("images");
            std::fs::create_dir_all(&images_dir)?;
            let conn = db::open_or_recover(&data_dir.join("wincy.db"))?;
            let settings = settings::load(handle);

            if let Err(e) = settings::swap_shortcut(handle, None, &settings.shortcut) {
                eprintln!("{e}");
            }
            // Dev builds would register target/debug/wincy.exe to run at login.
            if !cfg!(debug_assertions) {
                settings::sync_autostart(handle, settings.launch_at_startup);
            }
            if let Err(e) = db::trim(&conn, settings.max_items)
                .map(|(_, images)| db::remove_images(&images_dir, &images))
            {
                eprintln!("trim: {e}");
            }

            app.manage(AppState {
                db: Mutex::new(conn),
                settings: Mutex::new(settings),
                paused: AtomicBool::new(false),
                images_dir,
            });
            window::setup_preview(handle);
            window::apply_vibrancy(handle);
            tray::setup(handle)?;
            clipboard::start(handle.clone())?;
            Ok(())
        })
        .on_window_event(|win, event| {
            let app = win.app_handle();
            let palette = win.label() == "main";
            match event {
                WindowEvent::Focused(false)
                    if palette && !window::own_window_is_foreground(app) =>
                {
                    window::hide(app);
                }
                // The preview must never keep focus; hand it back to the palette.
                WindowEvent::Focused(true) if !palette => {
                    if let Some(main) = app.get_webview_window("main") {
                        let _ = main.set_focus();
                    }
                }
                WindowEvent::Moved(_) if palette => window::palette_moved(app),
                // Both windows are reused for the app's lifetime; Alt+F4 only hides them.
                WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    window::hide(app);
                }
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::get_clips,
            commands::get_clip,
            commands::paste_clip,
            commands::toggle_pin,
            commands::delete_clip,
            commands::clear_history,
            commands::get_settings,
            commands::save_settings,
            commands::hide_window,
            commands::set_preview,
            commands::get_paused,
            commands::set_paused,
            commands::open_data_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Wincy");
}
