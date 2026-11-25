import { getVersion } from "@tauri-apps/api/app";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { error as logToFile } from "@tauri-apps/plugin-log";
import { openUrl } from "@tauri-apps/plugin-opener";
import { check, type Update } from "@tauri-apps/plugin-updater";
import type { Clip, ClipSummary, PaletteShownPayload, Settings, Status } from "./types";

/** False in a plain browser (`pnpm dev`): every call is then served by ./mock. */
export const isTauri = "__TAURI_INTERNALS__" in window;

// Loaded lazily so the sample data never ships in the desktop bundle's main chunk.
const mock = isTauri ? null : import("./mock");

/** Writes to the app log file (console in the browser). Callers must never pass clip contents. */
export function logError(message: string): void {
  if (isTauri) void logToFile(message).catch(() => undefined);
  else console.error(message);
}

async function call<T>(cmd: string, args?: Record<string, unknown>): Promise<T> {
  try {
    return await (mock ? (await mock).mockInvoke<T>(cmd, args) : invoke<T>(cmd, args));
  } catch (e) {
    // Backend errors are short status strings (never clip contents), so they are safe to log.
    logError(`invoke ${cmd} failed: ${errorMessage(e)}`);
    throw e;
  }
}

export const api = {
  getClips: () => call<ClipSummary[]>("get_clips"),
  getClip: (id: number) => call<Clip>("get_clip", { id }),
  pasteClip: (id: number, plain: boolean) => call<void>("paste_clip", { id, plain }),
  togglePin: (id: number) => call<boolean>("toggle_pin", { id }),
  deleteClip: (id: number) => call<void>("delete_clip", { id }),
  clearHistory: () => call<void>("clear_history"),
  getSettings: () => call<Settings>("get_settings"),
  saveSettings: (settings: Settings) => call<Settings>("save_settings", { settings }),
  hideWindow: () => call<void>("hide_window"),
  getPaused: () => call<boolean>("get_paused"),
  setPaused: (paused: boolean) => call<boolean>("set_paused", { paused }),
  openDataDir: () => call<void>("open_data_dir"),
  setPreview: (id: number | null) => call<void>("set_preview", { id }),
  getStatus: () => call<Status>("get_status"),
  copyDiagnostics: () => call<string>("copy_diagnostics"),
  openLogDir: () => call<void>("open_log_dir"),
};

export interface EventMap {
  "clips-changed": null;
  "palette-shown": PaletteShownPayload;
  "settings-changed": Settings;
  // The backend only sends ids; null comes from the browser mock, which has no window to hide.
  "preview-clip": { id: number | null };
  "status-changed": Status;
}

/** Subscribes to a backend event; resolves to the unsubscribe function. */
export async function onEvent<K extends keyof EventMap>(
  name: K,
  handler: (payload: EventMap[K]) => void,
): Promise<() => void> {
  if (mock) return (await mock).mockListen(name, handler);
  return listen<EventMap[K]>(name, (e) => handler(e.payload));
}

/** URL for an absolute file path on disk (thumbnails, images). */
export function fileSrc(path: string): string {
  return isTauri ? convertFileSrc(path) : path;
}

export async function openExternal(url: string): Promise<void> {
  if (!isTauri) {
    window.open(url, "_blank", "noopener");
    return;
  }
  await openUrl(url);
}

export const appVersion = (): Promise<string> => (isTauri ? getVersion() : Promise.resolve("dev"));

/** Null when up to date, and always in the browser mock (there is no updater there). */
export async function checkForUpdate(): Promise<Update | null> {
  if (!isTauri) return null;
  try {
    return await check();
  } catch (e) {
    logError(`update check failed: ${errorMessage(e)}`);
    throw e;
  }
}

/** Commands reject with a plain string (see contract); anything else is unexpected. */
export function errorMessage(e: unknown): string {
  return typeof e === "string" ? e : e instanceof Error ? e.message : String(e);
}
