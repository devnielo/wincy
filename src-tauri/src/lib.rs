mod clipboard;
mod commands;
mod db;
mod detect;
mod ocr;
mod paste;
mod settings;
mod status;
mod tray;
mod window;

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use log::LevelFilter;
use rusqlite::Connection;
use tauri::{Manager, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

use settings::Settings;
use status::Status;

/// `<appLogDir>/wincy.log`, rotated at ~1 MB.
pub const LOG_FILE: &str = "wincy";

pub struct AppState {
    pub db: Mutex<Connection>,
    pub settings: Mutex<Settings>,
    pub paused: AtomicBool,
    pub images_dir: PathBuf,
    pub status: Mutex<Status>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        // Must be registered first so a second launch exits before doing anything else.
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            window::show(app, "palette");
        }))
        .plugin(
            tauri_plugin_log::Builder::new()
                .clear_targets()
                .target(Target::new(TargetKind::LogDir {
                    file_name: Some(LOG_FILE.into()),
                }))
                .targets(cfg!(debug_assertions).then(|| Target::new(TargetKind::Stdout)))
                .level(if cfg!(debug_assertions) {
                    LevelFilter::Debug
                } else {
                    LevelFilter::Info
                })
                .max_file_size(1_000_000)
                .rotation_strategy(RotationStrategy::KeepSome(3))
                .build(),
        )
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
            log::info!("Wincy {} starting", app.package_info().version);
            let data_dir = app.path().app_data_dir()?;
            let images_dir = data_dir.join("images");
            std::fs::create_dir_all(&images_dir)?;
            let conn = db::open_or_recover(&data_dir.join("wincy.db"))?;
            let settings = settings::load(handle);

            let shortcut_error = settings::swap_shortcut(handle, None, &settings.shortcut).err();
            // Dev builds would register target/debug/wincy.exe to run at login.
            if !cfg!(debug_assertions) {
                settings::sync_autostart(handle, settings.launch_at_startup);
            }
            if let Err(e) = db::trim(&conn, settings.max_items)
                .map(|(_, images)| db::remove_images(&images_dir, &images))
            {
                log::warn!("trim: {e}");
            }

            let theme = settings.theme;
            app.manage(AppState {
                db: Mutex::new(conn),
                settings: Mutex::new(settings),
                paused: AtomicBool::new(false),
                images_dir,
                status: Mutex::new(Status {
                    shortcut_error,
                    system_accent: status::system_accent(),
                    log_dir: app.path().app_log_dir()?.to_string_lossy().into_owned(),
                }),
            });
            commands::expire(handle);
            let h = handle.clone();
            std::thread::Builder::new()
                .name("expiry".into())
                .spawn(move || {
                    loop {
                        std::thread::sleep(Duration::from_secs(3600));
                        commands::expire(&h);
                    }
                })?;
            window::setup_preview(handle);
            window::apply_theme(handle, theme);
            tray::setup(handle)?;
            clipboard::start(handle.clone())?;
            ocr::start(handle.clone())?;
            // The settings view shows the shortcut error banner and lets the user record another one.
            if app
                .state::<AppState>()
                .status
                .lock()
                .unwrap()
                .shortcut_error
                .is_some()
            {
                window::show(handle, "settings");
            }
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
            commands::open_log_dir,
            commands::get_status,
            commands::copy_diagnostics,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Wincy");
}
