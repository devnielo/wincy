import tauri from "../../src-tauri/tauri.conf.json";

export const REPO_URL = "https://github.com/devnielo/wincy";
export const SUPPORT_URL = "https://buymeacoffee.com/devnielo";
export const DOWNLOAD_URL = `${REPO_URL}/releases/latest`;
// Read at build time from the app itself, so the site and the installer name the same release.
export const VERSION = tauri.version;
export const INSTALLER = `${tauri.productName}_${tauri.version}_x64-setup.exe`;
// Direct links to this version's assets. The site is rebuilt after a release is published (docs/RELEASING.md).
const ASSETS = `${REPO_URL}/releases/download/v${tauri.version}`;
export const INSTALLER_URL = `${ASSETS}/${INSTALLER}`;
export const PORTABLE_URL = `${ASSETS}/${tauri.productName}_${tauri.version}_x64-portable.zip`;
