//! Runtime status for the frontend (`get_status`, `status-changed`).

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager};
use windows::UI::ViewManagement::{UIColorType, UISettings};

use crate::AppState;

#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Status {
    pub shortcut_error: Option<String>,
    pub system_accent: Option<String>,
    pub log_dir: String,
    /// Pin/snippet shortcuts that could not be registered, e.g. `"Alt+Shift+M (pin)"`.
    pub shortcut_conflicts: Vec<String>,
    /// Data lives next to the exe: no autostart, no in-place updates.
    pub portable: bool,
    /// Microsoft Store (MSIX) package: no self-update, autostart through its `StartupTask`.
    pub store: bool,
}

/// Applies `change` and emits `status-changed` when it changed anything.
pub fn update(app: &AppHandle, change: impl FnOnce(&mut Status)) {
    let state = app.state::<AppState>();
    let changed = {
        let mut status = state.status.lock().unwrap();
        let before = status.clone();
        change(&mut status);
        (*status != before).then(|| status.clone())
    };
    if let Some(status) = changed {
        let _ = app.emit("status-changed", status);
    }
}

/// The Windows accent color as `#rrggbb`. Re-read whenever the palette opens, which is
/// the only time it is visible, so no `ColorValuesChanged` subscription is needed.
pub fn system_accent() -> Option<String> {
    let c = UISettings::new()
        .and_then(|s| s.GetColorValue(UIColorType::Accent))
        .ok()?;
    Some(format!("#{:02x}{:02x}{:02x}", c.R, c.G, c.B))
}

pub fn refresh_accent(app: &AppHandle) {
    let accent = system_accent();
    update(app, |s| s.system_accent = accent);
}

#[cfg(test)]
mod tests {
    #[test]
    fn accent_is_a_hex_color() {
        let accent = super::system_accent();
        assert!(accent.is_none_or(|c| c.len() == 7 && c.starts_with('#')));
    }
}
