import { getVersion } from "@tauri-apps/api/app";
import { convertFileSrc, invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { error as logToFile } from "@tauri-apps/plugin-log";
import { open as openDialog, save as saveDialog } from "@tauri-apps/plugin-dialog";
import { openUrl, revealItemInDir } from "@tauri-apps/plugin-opener";
import { check, type Update } from "@tauri-apps/plugin-updater";
import type {
  Clip,
  ClipPage,
  ClipSummary,
  ExportResult,
  ImportResult,
  PaletteShownPayload,
  Settings,
  Snippet,
  SnippetInput,
  Status,
} from "./types";

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
  getClips: () => call<ClipPage>("get_clips"),
  searchClips: (query: string, limit: number) => call<ClipSummary[]>("search_clips", { query, limit }),
  pasteText: (text: string) => call<void>("paste_text", { text }),
  setClipShortcut: (id: number, shortcut: string | null) => call<void>("set_clip_shortcut", { id, shortcut }),
  getSnippets: () => call<Snippet[]>("get_snippets"),
  saveSnippet: (snippet: SnippetInput) => call<Snippet>("save_snippet", { snippet }),
  deleteSnippet: (id: number) => call<void>("delete_snippet", { id }),
  pasteSnippet: (id: number) => call<void>("paste_snippet", { id }),
  exportHistory: (path: string, includeImages: boolean) => call<ExportResult>("export_history", { path, includeImages }),
  importHistory: (path: string) => call<ImportResult>("import_history", { path }),
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
  "snippets-changed": null;
  /** A pin/snippet shortcut was pressed while Wincy was in front: it was ignored. */
  "shortcut-in-use": null;
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

export async function revealFile(path: string): Promise<void> {
  if (isTauri) await revealItemInDir(path);
  else console.info("[mock] reveal in Explorer");
}

const EXPORT_FILTERS = [{ name: "Wincy", extensions: ["wincy.json"] }];

/** Save dialog for an export; null when cancelled. The browser mock pretends a file was picked. */
export async function chooseExportPath(): Promise<string | null> {
  const name = `wincy-${new Date().toISOString().slice(0, 10)}.wincy.json`;
  if (!isTauri) return `C:\\Users\\me\\Documents\\${name}`;
  const path = await saveDialog({ defaultPath: name, filters: EXPORT_FILTERS });
  // The dialog doesn't always append the filter's extension to a typed name.
  return path && !/\.json$/i.test(path) ? `${path}.wincy.json` : path;
}

/** Open dialog for an import; null when cancelled. */
export async function chooseImportPath(): Promise<string | null> {
  if (!isTauri) return "C:\\Users\\me\\Documents\\backup.wincy.json";
  const path = await openDialog({ multiple: false, directory: false, filters: EXPORT_FILTERS });
  return typeof path === "string" ? path : null;
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
