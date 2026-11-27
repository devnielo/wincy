mod clipboard;
mod commands;
mod db;
mod detect;
mod ocr;
mod paste;
mod paths;
mod settings;
mod shortcuts;
mod snippets;
mod status;
mod transfer;
mod tray;
mod window;

use std::path::PathBuf;
use std::sync::Mutex;
use std::sync::atomic::AtomicBool;
use std::time::Duration;

use log::LevelFilter;
use rusqlite::Connection;
use tauri::{Manager, WebviewWindowBuilder, WindowEvent};
use tauri_plugin_global_shortcut::ShortcutState;
use tauri_plugin_log::{RotationStrategy, Target, TargetKind};

use settings::Settings;
use status::Status;

/// `<logDir>/wincy.log` (see `paths`), rotated at ~1 MB.
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
                .target(Target::new(match paths::portable_log_dir() {
                    Some(path) => TargetKind::Folder {
                        path,
                        file_name: Some(LOG_FILE.into()),
                    },
                    None => TargetKind::LogDir {
                        file_name: Some(LOG_FILE.into()),
                    },
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
                .with_handler(|app, shortcut, event| {
                    if event.state == ShortcutState::Pressed {
                        shortcuts::pressed(app, shortcut);
                    }
                })
                .build(),
        )
        .plugin(
            tauri_plugin_autostart::Builder::new()
                .args(["--autostart"])
                .build(),
        )
        .plugin(tauri_plugin_process::init())
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            let handle = app.handle();
            let portable = paths::is_portable();
            let store = paths::is_store();
            log::info!(
                "Wincy {} starting{}{}",
                app.package_info().version,
                if portable { " (portable)" } else { "" },
                if store { " (Store)" } else { "" }
            );
            // Store policy: Microsoft Store packages never update themselves.
            if !store {
                handle.plugin(tauri_plugin_updater::Builder::new().build())?;
            }
            let data_dir = paths::data_dir(handle)?;
            let images_dir = data_dir.join("images");
            std::fs::create_dir_all(&images_dir)?;
            // The configured asset scope only covers AppData.
            app.asset_protocol_scope()
                .allow_directory(&images_dir, true)?;
            let conn = db::open_or_recover(&data_dir.join("wincy.db"))?;
            let settings = settings::load(handle);

            let shortcut_error = settings::swap_shortcut(handle, None, &settings.shortcut).err();
            if let Err(e) = db::trim(&conn, settings.max_items)
                .map(|(_, images)| db::remove_images(&images_dir, &images))
            {
                log::warn!("trim: {e}");
            }

            let theme = settings.theme;
            let launch_at_startup = settings.launch_at_startup;
            app.manage(AppState {
                db: Mutex::new(conn),
                settings: Mutex::new(settings),
                paused: AtomicBool::new(false),
                images_dir,
                status: Mutex::new(Status {
                    shortcut_error,
                    system_accent: status::system_accent(),
                    log_dir: paths::log_dir(handle)?.to_string_lossy().into_owned(),
                    shortcut_conflicts: Vec::new(),
                    portable,
                    store,
                }),
            });
            // After `manage`: the Store build syncs on a thread that may write the settings back.
            // Dev builds would register target/debug/wincy.exe to run at login; a portable
            // exe can be moved or live on a removable drive.
            if !cfg!(debug_assertions) && !portable {
                settings::sync_autostart(handle, launch_at_startup);
            }
            // Before the frontend exists: a command calling `sync` meanwhile could deadlock it.
            shortcuts::sync(handle);
            // Created here (`create: false` in tauri.conf.json) to set the WebView2 profile folder.
            for config in app.config().app.windows.clone() {
                let mut builder = WebviewWindowBuilder::from_config(handle, &config)?;
                if let Some(dir) = paths::webview_dir() {
                    builder = builder.data_directory(dir);
                }
                builder.build()?;
            }
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
            commands::search_clips,
            commands::get_clip,
            commands::paste_clip,
            commands::paste_text,
            commands::toggle_pin,
            commands::set_clip_shortcut,
            commands::get_snippets,
            commands::save_snippet,
            commands::delete_snippet,
            commands::paste_snippet,
            commands::export_history,
            commands::import_history,
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
