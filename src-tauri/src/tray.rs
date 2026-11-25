//! System tray icon and its menu (English / Spanish).

use std::sync::atomic::Ordering;

use tauri::menu::{CheckMenuItem, Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, Wry};
use windows::Win32::Globalization::GetUserDefaultUILanguage;

use crate::settings::Language;
use crate::{AppState, commands, window};

const TRAY_ID: &str = "main";

struct Labels {
    open: &'static str,
    pause: &'static str,
    settings: &'static str,
    clear: &'static str,
    quit: &'static str,
}

const EN: Labels = Labels {
    open: "Open Wincy",
    pause: "Pause capture",
    settings: "Settings…",
    clear: "Clear history",
    quit: "Quit",
};

const ES: Labels = Labels {
    open: "Abrir Wincy",
    pause: "Pausar captura",
    settings: "Configuración…",
    clear: "Borrar historial",
    quit: "Salir",
};

fn labels(language: Language) -> &'static Labels {
    const LANG_SPANISH: u16 = 0x0a;
    let spanish = match language {
        Language::Es => true,
        Language::En => false,
        Language::System => (unsafe { GetUserDefaultUILanguage() } & 0x3ff) == LANG_SPANISH,
    };
    if spanish { &ES } else { &EN }
}

fn menu(app: &AppHandle) -> tauri::Result<Menu<Wry>> {
    let state = app.state::<AppState>();
    let (shortcut, language) = {
        let s = state.settings.lock().unwrap();
        (s.shortcut.clone(), s.language)
    };
    let paused = state.paused.load(Ordering::SeqCst);
    let l = labels(language);
    let none = None::<&str>;
    Menu::with_items(
        app,
        &[
            // The tab right-aligns the shortcut like a native accelerator hint.
            &MenuItem::with_id(app, "open", format!("{}\t{shortcut}", l.open), true, none)?,
            &CheckMenuItem::with_id(app, "pause", l.pause, true, paused, none)?,
            &MenuItem::with_id(app, "settings", l.settings, true, none)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "clear", l.clear, true, none)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", l.quit, true, none)?,
        ],
    )
}

pub fn setup(app: &AppHandle) -> tauri::Result<()> {
    let mut builder = TrayIconBuilder::with_id(TRAY_ID)
        .tooltip("Wincy")
        .menu(&menu(app)?)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| match event.id().as_ref() {
            "open" => window::show(app, "palette"),
            "pause" => {
                let paused = app.state::<AppState>().paused.load(Ordering::SeqCst);
                commands::apply_paused(app, !paused);
            }
            "settings" => window::show(app, "settings"),
            "clear" => {
                if let Err(e) = commands::clear(app) {
                    eprintln!("clear history failed: {e}");
                }
            }
            "quit" => app.exit(0),
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click {
                button: MouseButton::Left,
                button_state: MouseButtonState::Up,
                ..
            } = event
            {
                window::show(tray.app_handle(), "palette");
            }
        });
    if let Some(icon) = app.default_window_icon() {
        builder = builder.icon(icon.clone());
    }
    builder.build(app)?;
    Ok(())
}

/// Rebuilds the menu so labels, shortcut and pause state reflect the current settings.
pub fn refresh(app: &AppHandle) {
    if let (Some(tray), Ok(menu)) = (app.tray_by_id(TRAY_ID), menu(app)) {
        let _ = tray.set_menu(Some(menu));
    }
}
