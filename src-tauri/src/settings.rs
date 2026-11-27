//! User settings persisted as `<configDir>/settings.json` (see `paths`).

use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter, Manager};
use tauri_plugin_autostart::ManagerExt as _;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, Shortcut};

use crate::{AppState, commands, db, ocr, paths, shortcuts, status, tray, window};

pub const MAX_ITEMS_RANGE: std::ops::RangeInclusive<u32> = 50..=10_000;
pub const MAX_EXPIRE_DAYS: u32 = 365;

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
    pub accent: Accent,
    pub ignore_secrets: bool,
    /// 0 = never.
    pub expire_days: u32,
    pub ocr_images: bool,
}

#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Theme {
    System,
    Light,
    Dark,
}

/// Stored for the frontend only; `Windows` follows the system accent color.
#[derive(Clone, Copy, Debug, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Accent {
    Windows,
    Blue,
    Violet,
    Teal,
    Green,
    Orange,
    Rose,
    Graphite,
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
            accent: Accent::Windows,
            ignore_secrets: true,
            expire_days: 0,
            ocr_images: true,
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
        if self.expire_days > MAX_EXPIRE_DAYS {
            return Err(format!(
                "expireDays must be between 0 and {MAX_EXPIRE_DAYS}"
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
    Ok(paths::config_dir(app)
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
            s.expire_days = s.expire_days.min(MAX_EXPIRE_DAYS);
            if s.shortcut.trim().parse::<Shortcut>().is_err() {
                log::warn!(
                    "settings: invalid shortcut {:?}; using the default",
                    s.shortcut
                );
                s.shortcut = Settings::default().shortcut;
            }
            s.validated().unwrap_or_default()
        }
        Err(e) => {
            log::warn!(
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
    gs.register(new)
        .inspect(|_| log::info!("shortcut {new} registered"))
        .map_err(|e| {
            if let Some(old) = old {
                let _ = gs.register(old);
            }
            let msg = format!("Could not register the shortcut \"{new}\": {e}");
            log::warn!("{msg}");
            msg
        })
}

pub fn sync_autostart(app: &AppHandle, enabled: bool) {
    if paths::is_store() {
        // MSIX virtualizes HKCU\...\Run; a packaged app starts at login through its StartupTask.
        // Startup only (`save` calls `sync_startup_task` itself): blocks on WinRT, so off the main thread.
        let app = app.clone();
        std::thread::spawn(move || {
            let actual = sync_startup_task(enabled);
            if actual != enabled {
                write_back_launch_at_startup(&app, actual);
            }
        });
        return;
    }
    let al = app.autolaunch();
    let result = match (enabled, al.is_enabled().unwrap_or(!enabled)) {
        (true, false) => al.enable(),
        (false, true) => al.disable(),
        _ => Ok(()),
    };
    if let Err(e) = result {
        log::warn!("autostart: {e}");
    }
}

/// Store build: enables/disables the `WincyStartup` task from `packaging/msix/AppxManifest.xml`
/// and returns whether Wincy will really start with Windows. Blocks on WinRT async calls.
fn sync_startup_task(enabled: bool) -> bool {
    use windows::ApplicationModel::{StartupTask, StartupTaskState as S};
    let state = (|| {
        let task = StartupTask::GetAsync(&"WincyStartup".into())?.join()?;
        match (enabled, task.State()?) {
            (true, S::Disabled) => task.RequestEnableAsync()?.join(),
            (false, S::Enabled) => task.Disable().and_then(|_| task.State()),
            (_, state) => Ok(state),
        }
    })();
    match state {
        Ok(S::Enabled | S::EnabledByPolicy) => true,
        Ok(S::DisabledByUser) if enabled => {
            log::info!(
                "autostart: Wincy can't turn itself on: it is turned off in Windows startup apps"
            );
            false
        }
        Ok(S::DisabledByPolicy) if enabled => {
            log::info!("autostart: Wincy can't turn itself on: it is disabled by policy");
            false
        }
        Ok(_) => false,
        Err(e) => {
            log::warn!("autostart: {e}");
            enabled
        }
    }
}

/// Reflects the real StartupTask state so the toggle doesn't lie about starting with Windows.
fn write_back_launch_at_startup(app: &AppHandle, enabled: bool) {
    let state = app.state::<AppState>();
    let settings = {
        let mut s = state.settings.lock().unwrap();
        s.launch_at_startup = enabled;
        s.clone()
    };
    match path(app).and_then(|p| write(&p, &settings)) {
        Ok(()) => {
            let _ = app.emit("settings-changed", &settings);
        }
        Err(e) => log::warn!("settings: {e}"),
    }
}

/// Validates, applies and persists new settings. Returns the stored settings.
pub fn save(app: &AppHandle, new: Settings) -> Result<Settings, String> {
    let mut new = new.validated()?;
    let state = app.state::<AppState>();
    let old = state.settings.lock().unwrap().clone();
    if new.shortcut != old.shortcut
        && let Ok(shortcut) = new.shortcut.parse::<Shortcut>()
    {
        let conn = state.db.lock().unwrap();
        if shortcuts::taken(&conn, "", &shortcut, None).map_err(|e| e.to_string())? {
            return Err(format!(
                "{} is already used by a pinned clip or a snippet",
                new.shortcut
            ));
        }
    }

    let shortcut_error = match swap_shortcut(app, Some(&old.shortcut), &new.shortcut) {
        Ok(()) => None,
        // Still dead and unchanged: keep the banner, but let the other settings save.
        Err(e) if old.shortcut == new.shortcut => Some(e),
        Err(e) => return Err(e),
    };
    // Store: synchronous (async command, so a worker thread) so the real state is what gets saved.
    if paths::is_store() && old.launch_at_startup != new.launch_at_startup {
        new.launch_at_startup = sync_startup_task(new.launch_at_startup);
    }
    if let Err(e) = write(&path(app)?, &new) {
        let _ = swap_shortcut(app, Some(&new.shortcut), &old.shortcut);
        return Err(format!("Could not save settings: {e}"));
    }
    *state.settings.lock().unwrap() = new.clone();
    status::update(app, |s| s.shortcut_error = shortcut_error);

    // Portable: the registry entry would point at a removable or moved exe. Store: synced above.
    if old.launch_at_startup != new.launch_at_startup && !paths::is_portable() && !paths::is_store()
    {
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
            Err(e) => log::warn!("trim: {e}"),
        }
    }
    commands::expire(app);
    if old.theme != new.theme {
        window::apply_theme(app, new.theme);
    }
    if new.ocr_images && !old.ocr_images {
        ocr::backfill(app);
    }
    // A pin/snippet shortcut may have clashed with the old palette shortcut.
    shortcuts::sync(app);
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
        assert_eq!(s.accent, Accent::Windows);
        assert!(s.ignore_secrets && s.ocr_images);
        assert_eq!(s.expire_days, 0);
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
        for days in [0, 1, 365] {
            let s = Settings {
                expire_days: days,
                ..Default::default()
            };
            assert!(s.validated().is_ok());
        }

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
            Settings {
                expire_days: 366,
                ..Default::default()
            },
        ] {
            assert!(bad.validated().is_err());
        }
    }
}
