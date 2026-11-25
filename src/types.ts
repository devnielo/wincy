// Mirrors docs/ARCHITECTURE.md "Data types" — keep in sync with src-tauri (serde camelCase).

export type ClipKind = "text" | "url" | "color" | "code" | "image" | "files";

export interface ClipSummary {
  id: number;
  kind: ClipKind;
  preview: string; // first 500 chars of text (files: newline-joined paths; image: "")
  hasHtml: boolean;
  thumbPath: string | null; // absolute path of the thumbnail (image only) — use convertFileSrc()
  imagePath: string | null; // absolute path of the full PNG (image only)
  imageSize: [number, number] | null; // width, height
  files: string[] | null;
  sourceApp: string | null; // e.g. "chrome.exe"
  pinned: boolean;
  createdAt: number; // unix ms, first copy
  updatedAt: number; // unix ms, last copy/paste — list order key
  useCount: number; // times pasted from Wincy
}

export interface Clip extends ClipSummary {
  text: string | null; // full text
  html: string | null;
}

export type PalettePosition = "cursor"; // reserved for future "center"
export type Theme = "system" | "light" | "dark";
export type Language = "system" | "en" | "es";

export interface Settings {
  shortcut: string;
  maxItems: number;
  launchAtStartup: boolean;
  pasteDirectly: boolean;
  captureImages: boolean;
  ignoredApps: string[];
  theme: Theme;
  language: Language;
  checkUpdates: boolean;
}

export type View = "palette" | "settings";

export interface PaletteShownPayload {
  view: View;
}
