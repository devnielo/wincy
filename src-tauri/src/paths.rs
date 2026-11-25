//! Where Wincy keeps its files: the usual AppData folders, or `<exe dir>\data\` in portable
//! mode (a file named `portable` next to `wincy.exe`). Every path to data goes through here.

use std::path::{Path, PathBuf};
use std::sync::OnceLock;

use tauri::{AppHandle, Manager};

/// `<exe dir>\data` when the portable marker exists next to the exe.
fn portable_root() -> Option<&'static Path> {
    static ROOT: OnceLock<Option<PathBuf>> = OnceLock::new();
    ROOT.get_or_init(|| portable_root_in(std::env::current_exe().ok()?.parent()?))
        .as_deref()
}

fn portable_root_in(exe_dir: &Path) -> Option<PathBuf> {
    exe_dir
        .join("portable")
        .is_file()
        .then(|| exe_dir.join("data"))
}

pub fn is_portable() -> bool {
    portable_root().is_some()
}

/// DB, images.
pub fn data_dir(app: &AppHandle) -> tauri::Result<PathBuf> {
    portable_root().map_or_else(|| app.path().app_data_dir(), |r| Ok(r.to_path_buf()))
}

/// `settings.json`.
pub fn config_dir(app: &AppHandle) -> tauri::Result<PathBuf> {
    portable_root().map_or_else(|| app.path().app_config_dir(), |r| Ok(r.to_path_buf()))
}

pub fn log_dir(app: &AppHandle) -> tauri::Result<PathBuf> {
    portable_log_dir().map_or_else(|| app.path().app_log_dir(), Ok)
}

/// The WebView2 profile folder; `None` keeps Tauri's default (AppData).
pub fn webview_dir() -> Option<PathBuf> {
    portable_root().map(|r| r.join("webview"))
}

/// Folder for the log plugin, which is built before the app (and its path resolver) exists.
/// `None` = the default app log dir.
pub fn portable_log_dir() -> Option<PathBuf> {
    portable_root().map(|r| r.join("logs"))
}

#[cfg(test)]
mod tests {
    use super::portable_root_in;

    #[test]
    fn marker_file_enables_portable_mode() {
        let dir = std::env::temp_dir().join(format!("wincy-paths-{}", std::process::id()));
        std::fs::create_dir_all(&dir).unwrap();
        assert_eq!(portable_root_in(&dir), None);
        std::fs::write(dir.join("portable"), "").unwrap();
        assert_eq!(portable_root_in(&dir), Some(dir.join("data")));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
