import tauri from "../../src-tauri/tauri.conf.json";

export const REPO_URL = "https://github.com/devnielo/wincy";
export const SUPPORT_URL = "https://buymeacoffee.com/devnielo";
export const DOWNLOAD_URL = `${REPO_URL}/releases/latest`;
// Read at build time from the app itself, so the site and the installer name the same release.
export const VERSION = tauri.version;
export const INSTALLER = `${tauri.productName}_${tauri.version}_x64-setup.exe`;
