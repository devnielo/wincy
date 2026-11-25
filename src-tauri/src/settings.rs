//! User settings persisted as `<appConfigDir>/settings.json`.

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::{AppState, db, tray};

pub const MAX_ITEMS_RANGE: std::ops::RangeInclusive<u32> = 50..=10_000;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct Settings {
    pub shortcut: String,
    pub max_items: u32,
    pub launch_at_startup: bool,
    pub paste_directly: bool,
    pub capture_images: bool,
    pub ignored_apps: Vec<String>,
    pub theme: Theme,
    pub language: Language,
    pub check_updates: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Language {
    System,
    En,
    Es,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            shortcut: "Alt+V".into(),
            max_items: 1000,
            launch_at_startup: true,
            paste_directly: true,
            capture_images: true,
            ignored_apps: [
                "bitwarden.exe",
                "1password.exe",
                "keepass.exe",
                "keepassxc.exe",
                "enpass.exe",
                "dashlane.exe",
                "lastpass.exe",
                "nordpass.exe",
                "proton pass.exe",
            ]
            .map(String::from)
            .into(),
            theme: Theme::System,
            language: Language::System,
            check_updates: true,
        }
    }
}

impl Settings {
    /// Normalizes user input and rejects values the app cannot work with.
    pub fn validated(mut self) -> Result<Self, String> {
        if !MAX_ITEMS_RANGE.contains(&self.max_items) {
            return Err(format!(
                "maxItems must be between {} and {}",
                MAX_ITEMS_RANGE.start(),
                MAX_ITEMS_RANGE.end()
            ));
        }
        self.shortcut = self.shortcut.trim().to_string();
        self.shortcut
            .parse::<Shortcut>()
            .map_err(|e| format!("Invalid shortcut \"{}\": {e}", self.shortcut))?;
        self.ignored_apps = self
            .ignored_apps
            .iter()
            .map(|a| a.trim().to_lowercase())
            .filter(|a| !a.is_empty())
            .collect();
        Ok(self)
    }

    pub fn is_ignored(&self, exe: &str) -> bool {
        self.ignored_apps
            .iter()
            .any(|a| a.eq_ignore_ascii_case(exe))
    }
}

fn path(app: &AppHandle) -> Result<PathBuf, String> {
    Ok(app
        .path()
        .app_config_dir()
        .map_err(|e| e.to_string())?
        .join("settings.json"))
}

/// Never fails: a missing file yields defaults; an unreadable one is kept as `.bak` so
/// the user's data is not silently overwritten on the next save.
pub fn load(app: &AppHandle) -> Settings {
    let Ok(path) = path(app) else {
        return Settings::default();
    };
    let Ok(raw) = std::fs::read_to_string(&path) else {
        return Settings::default();
    };
    match serde_json::from_str::<Settings>(&raw) {
        Ok(mut s) => {
            s.max_items = s
                .max_items
                .clamp(*MAX_ITEMS_RANGE.start(), *MAX_ITEMS_RANGE.end());
            if s.shortcut.trim().parse::<Shortcut>().is_err() {
                eprintln!(
                    "settings: invalid shortcut {:?}; using the default",
                    s.shortcut
                );
                s.shortcut = Settings::default().shortcut;
            }
            s.validated().unwrap_or_default()
        }
        Err(e) => {
            eprintln!(
                "settings: {} is invalid ({e}); backing it up",
                path.display()
            );
            let _ = std::fs::rename(&path, path.with_extension("json.bak"));
            Settings::default()
        }
    }
}

fn write(path: &Path, settings: &Settings) -> Result<(), String> {
    if let Some(dir) = path.parent() {
        std::fs::create_dir_all(dir).map_err(|e| e.to_string())?;
    }
    let tmp = path.with_extension("json.tmp");
    let json = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    std::fs::write(&tmp, json).map_err(|e| e.to_string())?;
    std::fs::rename(&tmp, path).map_err(|e| e.to_string())
}

/// Registers `new` and unregisters `old`. On failure the old shortcut stays active.
pub fn swap_shortcut(app: &AppHandle, old: Option<&str>, new: &str) -> Result<(), String> {
    let gs = app.global_shortcut();
    // Unchanged and live: nothing to do. Unchanged but dead (startup registration failed): retry.
    if old == Some(new) && gs.is_registered(new) {
        return Ok(());
    }
    if let Some(old) = old {
        let _ = gs.unregister(old);
    }
    gs.register(new).map_err(|e| {
        if let Some(old) = old {
            let _ = gs.register(old);
        }
        format!("Could not register the shortcut \"{new}\": {e}")
    })
}

pub fn sync_autostart(app: &AppHandle, enabled: bool) {
    let al = app.autolaunch();
    let result = match (enabled, al.is_enabled().unwrap_or(!enabled)) {
        (true, false) => al.enable(),
        (false, true) => al.disable(),
        _ => Ok(()),
    };
    if let Err(e) = result {
        eprintln!("autostart: {e}");
    }
}

/// Validates, applies and persists new settings. Returns the stored settings.
pub fn save(app: &AppHandle, new: Settings) -> Result<Settings, String> {
    let new = new.validated()?;
    let state = app.state::<AppState>();
    let old = state.settings.lock().unwrap().clone();

    swap_shortcut(app, Some(&old.shortcut), &new.shortcut)?;
    if let Err(e) = write(&path(app)?, &new) {
        let _ = swap_shortcut(app, Some(&new.shortcut), &old.shortcut);
        return Err(format!("Could not save settings: {e}"));
    }
    *state.settings.lock().unwrap() = new.clone();

    if old.launch_at_startup != new.launch_at_startup {
        sync_autostart(app, new.launch_at_startup);
    }
    if new.max_items < old.max_items {
        let trimmed = {
            let conn = state.db.lock().unwrap();
            db::trim(&conn, new.max_items)
        };
        match trimmed {
            Ok((count, images)) => {
                db::remove_images(&state.images_dir, &images);
                if count > 0 {
                    let _ = app.emit("clips-changed", ());
                }
            }
            Err(e) => eprintln!("trim: {e}"),
        }
    }
    tray::refresh(app);
    let _ = app.emit("settings-changed", &new);
    Ok(new)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn missing_fields_use_defaults() {
        let s: Settings = serde_json::from_str(r#"{"maxItems": 200, "unknown": 1}"#).unwrap();
        assert_eq!(s.max_items, 200);
        assert_eq!(s.shortcut, "Alt+V");
        assert!(s.paste_directly);
    }

    #[test]
    fn validation() {
        let ok = Settings {
            ignored_apps: vec![" KeePass.exe ".into(), String::new()],
            ..Default::default()
        }
        .validated()
        .unwrap();
        assert_eq!(ok.ignored_apps, ["keepass.exe"]);
        assert!(ok.is_ignored("KEEPASS.EXE"));

        for bad in [
            Settings {
                max_items: 10,
                ..Default::default()
            },
            Settings {
                max_items: 20_000,
                ..Default::default()
            },
            Settings {
                shortcut: "Alt+Nope".into(),
                ..Default::default()
            },
        ] {
            assert!(bad.validated().is_err());
        }
    }
}
