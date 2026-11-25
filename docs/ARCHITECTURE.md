# Wincy — Architecture & contract

Wincy is an open-source, keyboard-first clipboard manager for Windows 10/11 (Tauri v2 + Rust + React 19).
It lives in the system tray; **Alt+V** opens a command palette **next to the mouse cursor**; type to fuzzy-filter,
↑/↓ to select, **Enter** pastes into the previously focused app, **Shift+Enter** pastes as plain text.

This file is the **single source of truth** shared by the backend (Rust), the frontend (React) and the docs/website.
If you change a name here you must change it on both sides.

Repository: `https://github.com/devnielo/wincy` (website: `https://devnielo.github.io/wincy/`). A fork must change these URLs
(updater endpoint in `tauri.conf.json`, READMEs, `src/components/SettingsView.tsx`, `website/astro.config.mjs`, `website/src/consts.ts`).

## Layout

```
/                       app (Vite + React + TS) — package.json, index.html, vite.config.ts
├── src/                frontend
├── src-tauri/          Rust backend (Tauri v2)
├── website/            Astro site for GitHub Pages (independent package.json)
├── assets/logo.svg     master logo (icons generated with `pnpm tauri icon assets/logo.svg`)
├── docs/               ARCHITECTURE.md (this), RELEASING.md, SIGNING.md
├── packaging/winget/   winget manifest templates + render script
└── .github/            workflows (ci, release, pages), issue templates
```

## Backend (src-tauri/src)

| file | responsibility |
|---|---|
| `main.rs` | calls `wincy_lib::run()` (windows_subsystem = "windows") |
| `lib.rs` | builder, plugins (single-instance, log, global-shortcut, autostart, updater, process, opener), setup, command registration |
| `db.rs` | SQLite via `rusqlite` (bundled). Schema, queries, dedupe, trimming. `secure_delete=ON`, and a WAL checkpoint after `delete_clip`/`clear_history`, so deleted clips are overwritten on disk. A database that fails to open is renamed `wincy.db.corrupt-<unix secs>` (with its `-wal`/`-shm`) and a fresh one is created |
| `clipboard.rs` | Win32 listener (`AddClipboardFormatListener` on a hidden message-only window, own thread — **no polling**), reading formats, privacy filters, writing back to the clipboard |
| `detect.rs` | content-kind detection (pure fn + unit tests) |
| `paste.rs` | restore previous foreground window and simulate Ctrl+V with `enigo` |
| `window.rs` | show/hide palette next to cursor (clamped to the monitor work area), remember previous foreground HWND, vibrancy (`window-vibrancy`: Mica on Win11, Acrylic on Win10), hide on blur; the non-activating `preview` window (placement beside the palette, show/hide, follows palette drags) |
| `tray.rs` | tray icon + menu |
| `settings.rs` | `settings.json` in app config dir; load/save; apply (shortcut, autostart, trimming) |
| `ocr.rs` | Windows OCR worker thread and startup backfill (see v0.2.0) |
| `status.rs` | `Status` (`get_status`, `status-changed`), Windows accent color |
| `commands.rs` | all `#[tauri::command]`s listed below |

### Capture rules
- Triggered by `WM_CLIPBOARDUPDATE`. The privacy checks below and the read run on one snapshot: the clipboard is held open from the first check to the end of the read. Read (in priority order): file list (`CF_HDROP`) → image (`PNG` registered format or `CF_DIBV5`/`CF_DIB`) → text (`CF_UNICODETEXT`) plus optional `HTML Format`.
  Exception: when `Rich Text Format` is present with non-blank text (Office, LibreOffice), text wins over the image — the bitmap is just a rendering of the copied cells/text.
- **Privacy — skip the capture** when any of:
  - the clipboard contains the registered format `ExcludeClipboardContentFromMonitorProcessing` or `Clipboard Viewer Ignore`, or `CanIncludeInClipboardHistory` whose DWORD value is 0 (emitted by Bitwarden, 1Password, KeePassXC, browsers in private mode…);
  - the source process exe name (clipboard owner window → PID → exe file name; fallback: foreground window) matches `settings.ignoredApps` (case-insensitive, compare file names like `bitwarden.exe`; the settings UI appends `.exe` to a name typed without an extension);
  - capture is paused (tray toggle).
- Dedupe by `hash` (SHA-256 of kind + canonical content: text, sorted file list, or PNG bytes). On duplicate, bump `updatedAt` (and replace html/source) — the item moves to the top.
- Empty / whitespace-only text is ignored. Text larger than 1 MB is truncated to 1 MB.
- Images: saved as `<appDataDir>/images/<hash>.png` plus a ≤ 320 px thumbnail `<hash>.thumb.png`. Deleted files when the clip is deleted/trimmed.
- After each insert: trim to `settings.maxItems` (pinned items are **never** trimmed and do not count), then emit `clips-changed`.

### Kind detection (`detect.rs`, applies to text)
`color` — whole trimmed text is `#rgb`, `#rgba`, `#rrggbb`, `#rrggbbaa`, `rgb()/rgba()`, `hsl()/hsla()`.
`url` — single token starting with `http://`, `https://`, `ftp://`, `mailto:` or `www.`.
`code` — multi-line (≥ 2 lines) text with code signals (braces/semicolon line endings, indentation, keywords like `function`, `const`, `fn`, `def`, `class`, `import`, `return`, `<div`, `SELECT`…); heuristic, unit-tested.
`text` — everything else. `image` and `files` come from the format, not detection.

### Paste flow (`paste_clip`)
1. Write the clip to the clipboard (text+html for rich paste; text only when `plain = true` or the clip has no html; image as `CF_DIB` + `PNG`; files as `CF_HDROP`). Mark this write as own so the listener only bumps the existing item (same hash). The write fails with "The clipboard is busy, try again" while a capture is reading the clipboard.
2. Hide the palette, `SetForegroundWindow(previousHwnd)`.
3. Wait until all modifier keys (Alt/Shift/Ctrl/Win) are physically released (poll `GetAsyncKeyState`, max ~600 ms) — required for Alt+1..9 — then a ~40 ms settle delay.
   If a modifier is still held after ~600 ms, Ctrl+V is skipped (the clip stays on the clipboard).
4. If `settings.pasteDirectly` → `enigo` Ctrl+V. Otherwise only copy.

## Data types (serde `rename_all = "camelCase"` ⇄ TS in `src/types.ts`)

```ts
export type ClipKind = "text" | "url" | "color" | "code" | "image" | "files";

export interface ClipSummary {
  id: number;
  kind: ClipKind;
  preview: string;          // first 500 chars of text (files: newline-joined paths; image: "" )
  hasHtml: boolean;
  thumbPath: string | null; // absolute path of the thumbnail (image only) — use convertFileSrc()
  imagePath: string | null; // absolute path of the full PNG (image only)
  imageSize: [number, number] | null; // width, height
  files: string[] | null;
  sourceApp: string | null; // e.g. "chrome.exe"
  pinned: boolean;
  createdAt: number;        // unix ms, first copy
  updatedAt: number;        // unix ms, last copy/paste — list order key
  useCount: number;         // times pasted from Wincy
}

export interface Clip extends ClipSummary {
  text: string | null;      // full text
  html: string | null;
}

export type PalettePosition = "cursor"; // reserved for future "center"
export type Theme = "system" | "light" | "dark";
export type Language = "system" | "en" | "es";

export interface Settings {
  shortcut: string;          // default "Alt+V" (tauri global-shortcut accelerator syntax; the recorder refuses Ctrl(+Shift)+C/V/X/Z/Y/A and Alt+1..9)
  maxItems: number;          // default 1000, range 50..10000 (lowering it in Settings asks for confirmation: it trims at once)
  launchAtStartup: boolean;  // default true
  pasteDirectly: boolean;    // default true (false = only copy to clipboard)
  captureImages: boolean;    // default true
  ignoredApps: string[];     // default ["bitwarden.exe","1password.exe","keepass.exe","keepassxc.exe","enpass.exe","dashlane.exe","lastpass.exe","nordpass.exe","proton pass.exe"]
  theme: Theme;              // default "system"
  language: Language;        // default "system"
  checkUpdates: boolean;     // default true
}
```

## Commands (frontend → Rust, `invoke(name, args)`; args camelCase)

| command | args | returns |
|---|---|---|
| `get_clips` | — | `ClipSummary[]` ordered: pinned first (by `updatedAt` desc), then the rest by `updatedAt` desc |
| `get_clip` | `{ id }` | `Clip` |
| `paste_clip` | `{ id, plain }` | `void` (bumps `updatedAt`, `useCount`) |
| `toggle_pin` | `{ id }` | `boolean` new pinned state |
| `delete_clip` | `{ id }` | `void` |
| `clear_history` | — | `void` (keeps pinned) |
| `get_settings` | — | `Settings` |
| `save_settings` | `{ settings }` | `Settings` (applies; error string if shortcut cannot be registered) |
| `hide_window` | — | `void` (restores previous foreground window, no paste) |
| `get_paused` / `set_paused` | — / `{ paused }` | `boolean` |
| `open_data_dir` | — | `void` (opens app data folder in Explorer) |
| `set_preview` | `{ id: number \| null }` | `void` — `id`: position + show the preview panel without activating it and emit `preview-clip` to it; `null`: hide it |

Errors are returned as `string`.

## Events (Rust → frontend, `listen(name)`)
| event | payload | when |
|---|---|---|
| `clips-changed` | none | after insert/bump/delete/pin/clear/trim |
| `palette-shown` | `{ view: "palette" \| "settings" }` | every time the window is shown (shortcut, tray, second instance) — frontend resets query, selects index 0, focuses search |
| `settings-changed` | `Settings` | after settings were saved or paused toggled (`Settings` has no paused field: call `get_paused` on receipt) |
| `preview-clip` | `{ id: number }` | sent only to the `preview` window by `set_preview` |

## Window
Label `main`, created hidden at startup and reused (show/hide only — instant open). `decorations: false`, `transparent: true`,
`alwaysOnTop: true`, `skipTaskbar: true`, `resizable: false`, `shadow: true`, size 600×460 logical px (list only; the content preview lives in the `preview` window). Hidden on focus loss and on Esc.
Shown next to the cursor (top-left at cursor + 8 px, flipped/clamped to stay inside the monitor work area).
The paste target is the window that was in the foreground when the palette opened; taskbar/tray windows and Wincy's own
windows (e.g. its hidden tray-menu window) are never used (opened from the tray, Windows activates the app below the palette when it hides).
Re-opening while the palette itself is focused keeps the previous target.
Body background must be transparent so vibrancy shows; UI draws a semi-transparent surface on top.

The palette is **draggable** by its header (search bar row, outside the input and buttons) via `data-tauri-drag-region`
(`core:window:allow-start-dragging`). The dragged position is not remembered: the next open is next to the cursor again.

### Preview panel (second window)
Label `preview`, same frontend bundle (the frontend renders `PreviewWindow` when the current window label is `preview`).
Created hidden at startup, `decorations: false`, `transparent: true`, `alwaysOnTop: true`, `skipTaskbar: true`, `resizable: false`,
`shadow: true`, `focus: false`, size 560×560 logical px, same vibrancy as the palette. It must **never take focus**: extended style
`WS_EX_NOACTIVATE` set at setup and shown with `ShowWindow(SW_SHOWNOACTIVATE)` (not Tauri `show()`), so the palette keeps focus and
its hide-on-blur is not triggered. Mouse wheel / scrollbar / text selection work without activation.
Placement (`set_preview`): beside the palette with an 8 px gap — right side if it fits in the palette's monitor work area, else left,
else the side with more room; top aligned with the palette, clamped vertically to the work area. It is re-placed when the palette moves
(drag) and hidden whenever the palette hides, the view switches to settings, the preview is toggled off, or no clip is selected.
The palette calls `set_preview` (debounced ~80 ms) whenever the selected clip changes by keyboard or mouse hover.
The panel shows the full content large with scrolling (vertical and, for code, horizontal): text/code/url, a big swatch + HEX/RGB(A)/HSL for colors,
the image at full available size, the file list, and the metadata (source app, dates, uses, chars/lines, image size, pinned/rich chips).

### Mouse-first actions (every shortcut has a button)
Palette header: kind filter buttons + **Settings** (gear) button. Each row on hover/selection shows icon buttons **Pin/Unpin** and **Delete**.
Footer: buttons **Paste**, **Paste as plain text**, **Pin/Unpin**, **Delete** (act on the selected clip), and on the right **Pause/Resume capture**,
**Preview on/off**, **Settings**. Buttons use icon + short label (icon-only where space is tight, with an accessible label and a tooltip that
mentions the keyboard shortcut). Buttons must not steal focus from the search input (`preventFocusOnPress` / mousedown preventDefault).
Settings view: a back button (already present) returns to the palette.

## Frontend keyboard map
| keys | action |
|---|---|
| type | fuzzy filter (Fuse.js over `preview`, `sourceApp`, `kind`; threshold ~0.35; empty query = original order) |
| ↑ / ↓, PageUp/PageDown, Home/End (Ctrl+Home/End) | move selection |
| Enter | paste |
| Shift+Enter | paste as plain text |
| Alt+1 … Alt+9 | paste the Nth visible item (pins are first, so Alt+1 = first pin) |
| Ctrl+P | pin / unpin |
| Ctrl+Delete | delete item |
| Tab / Shift+Tab | cycle kind filter: all → text → url → color → code → image → files |
| Ctrl+Space | toggle the preview panel |
| Ctrl+, | open settings |
| Esc | clear query if any, otherwise hide window (settings view: back to palette) |

## Updater
`tauri-plugin-updater`, endpoint `https://github.com/devnielo/wincy/releases/latest/download/latest.json`,
pubkey (minisign, also in `tauri.conf.json`):
`dW50cnVzdGVkIGNvbW1lbnQ6IG1pbmlzaWduIHB1YmxpYyBrZXk6IDYzNkRBMzY4QzU3QzRBQUYKUldTdlNuekZhS050WTBBazNwM0N2WWZxajkzN2cwODk3Zmk4a0FiUHp6Q1hvOTN1dzdCaTMwMFcK`
Private key lives outside the repo (`%USERPROFILE%\.tauri\wincy.key`, no password) and goes into the GitHub secret
`TAURI_SIGNING_PRIVATE_KEY`. Bundle target: **NSIS only**, `installMode: "currentUser"`, `createUpdaterArtifacts: true`.

## i18n
Frontend: tiny dictionary in `src/i18n.ts` (`en`, `es`), `language: "system"` uses `navigator.language`.
Website: Astro i18n routing, `en` default at `/`, Spanish at `/es/`.

## v0.2.0 additions (binding)

### Settings (new fields, serde defaults keep old files valid)
```ts
export type Accent = "windows" | "blue" | "violet" | "teal" | "green" | "orange" | "rose" | "graphite";
interface Settings {
  // …existing fields…
  accent: Accent;          // default "windows" = the Windows accent color (falls back to "blue" if unavailable)
  ignoreSecrets: boolean;  // default true — don't save clips that look like secrets (see below)
  expireDays: number;      // default 0 = never; 1..365 — delete UNPINNED clips whose updatedAt is older than N days
  ocrImages: boolean;      // default true — recognize text in images with the built-in Windows OCR
}
```

### ClipSummary (new field)
`ocrText: string | null` — first 500 chars of the text recognized in an image (image clips only; `""` when OCR found nothing,
`null` when not processed yet or OCR disabled/unavailable). Included in fuzzy search. `Clip` (from `get_clip`) carries the full OCR text in the same field.
DB: schema `user_version` 2 adds `ocr_text TEXT` (nullable).

### OCR (backend)
`Windows.Media.Ocr` via the `windows` crate (`OcrEngine::TryCreateFromUserProfileLanguages`; if `None` — no OCR language installed — log once and skip).
Runs on a dedicated low-priority worker thread fed by a channel (never on the clipboard listener thread): after an image is captured
(when `ocrImages`), and at startup as a backfill of up to 200 most recent images with `ocr_text IS NULL`. Images larger than
`OcrEngine::MaxImageDimension` are downscaled first. Writes `ocr_text` (trimmed, lines joined with `\n`), then emits `clips-changed`.

### Secret detection (`detect.rs`, pure + unit tests)
`looks_secret(text) -> Option<&'static str>` (the matched pattern name), high precision only, evaluated on the trimmed text (skip if > 8 KB):
PEM private key block (`-----BEGIN … PRIVATE KEY-----`); JWT (3 base64url segments whose header decodes to JSON containing `"alg"`);
AWS access key id (`AKIA|ASIA` + 16 `[0-9A-Z]`); GitHub tokens (`ghp_|gho_|ghu_|ghs_|ghr_` + 36 alnum, `github_pat_…`); Slack (`xox[abprs]-…`);
Stripe live keys (`sk_live_|rk_live_`); Google API key (`AIza` + 35 `[0-9A-Za-z_-]`); OpenAI/Anthropic (`sk-proj-…`, `sk-ant-…`);
a payment card number (whole text is 13–19 digits with optional spaces/dashes, first digit 2–6, and passes Luhn).
When `ignoreSecrets` and it matches, the capture is skipped (logged as "skipped: secret pattern <name>", never the content).
Must NOT match: prose, UUIDs, git SHAs, phone numbers, IBAN-less numbers failing Luhn, base64 images, URLs with tokens are out of scope.

### Expiry
When `expireDays > 0`: delete unpinned clips older than N days (and their image files) at startup, every hour, and right after `save_settings`; emit `clips-changed` if anything was deleted.

### Status, shortcut failure, logging, diagnostics
| command | args | returns |
|---|---|---|
| `get_status` | — | `Status` |
| `copy_diagnostics` | — | `string` (also written to the clipboard as an own write — not captured) |
| `open_log_dir` | — | `void` |

```ts
interface Status {
  shortcutError: string | null; // set when the configured shortcut could not be registered (at startup or later); null when OK
  systemAccent: string | null;  // Windows accent color "#rrggbb" (UISettings Accent), null if unavailable
  logDir: string;
}
```
Event `status-changed` (payload `Status`) whenever any field changes (shortcut registered/failed, Windows accent changed — listen to `UISettings.ColorValuesChanged` or re-read on `palette-shown`).
If the shortcut fails at startup: log it, set the tray tooltip to "Wincy — shortcut unavailable" (localized), and show the palette window in the `settings` view so the user sees the banner and can record another shortcut.
Logging: `tauri-plugin-log` to a file in the app log dir (`%LOCALAPPDATA%\dev.wincy.app\logs`), level Info (Debug in debug builds), size-rotated (~1 MB, keep a few). Replace every `eprintln!`.
**Privacy: never log clip contents, OCR text or ignored-app lists' matches beyond the exe name.** Diagnostics = app version, Windows version/build, settings
(ignoredApps as a count), clip count, OCR availability, and the last ~200 log lines.
Frontend logs caught invoke/updater errors through `@tauri-apps/plugin-log` (`log:default` permission on both windows).

### Theme (Firefox-like picker) and native backdrop
Settings → Appearance shows Firefox-style visual cards: **mode** cards System / Light / Dark (each with a mini palette preview) and a row of round **accent** swatches
(Windows, Blue, Violet, Teal, Green, Orange, Rose, Graphite; the Windows swatch shows the current system accent). The accent drives the CSS accent tokens (selection bar, active filter,
focus rings, switches, primary buttons) in both windows, with a subtle accent tint on the surface; text on accent must keep WCAG AA contrast (pick light/dark foreground per accent).
The native Mica/Acrylic backdrop follows the **app** theme: on settings change call `set_theme(Some(Light|Dark))` / `None` for system on both windows.

### Search: filter by source app
Query tokens `app:<term>` (case-insensitive, may be repeated = OR) keep only clips whose `sourceApp` (without `.exe`) contains `<term>`; the rest of the query goes to Fuse.
Parser is a pure function with tests. Example: `app:code debounce`, `app:chrome app:edge`.
