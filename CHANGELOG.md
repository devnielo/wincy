# Changelog

All notable changes to Wincy are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

### Added

- Microsoft Store package (**Wincy Clipboard**): releases also build an unsigned MSIX (`packaging/msix/build.ps1`, workflow artifact `msix`) for Partner Center. In the Store version, updates come from the Microsoft Store (no update check) and **Launch at startup** uses the Windows startup task; the installer and portable versions are unchanged. See `docs/STORE.md`.

## [0.3.0] - 2026-11-25

### Added

- Quick actions: **Ctrl+K** (or the **Actions** button) lists what you can do with the selected clip; type to filter, Enter runs. Text: UPPERCASE, lowercase, Title Case, trim, collapse whitespace, remove line breaks, format or minify JSON, Base64 and URL encode / decode (only the actions that apply are shown); the result is pasted as plain text and the clip itself is not changed. Links: **Open in browser**. Files: **Show in Explorer**. Colors: paste as HEX, RGB or HSL. Also **Save as snippet** and, for pinned clips, **Set shortcut…**.
- Snippets: reusable text with variables filled in when pasted (`{date}`, `{time}`, `{datetime}`, `{clipboard}`; `{{` for a literal `{`). Managed in Settings → **Snippets**, shown in the palette under the new **Snippets** filter and included in search.
- Global shortcuts for pinned clips and snippets: paste them into the app you are using without opening the palette. Shortcuts that can't be registered are listed in Settings.
- Export and import (Settings → **Data**): history and snippets to a `.wincy.json` file, with or without images. Import validates the file, merges with the current history without duplicates and skips secrets when **Ignore secrets** is on. Settings are not exported.
- Portable version: `Wincy_X.Y.Z_x64-portable.zip`. With the `portable` file next to `wincy.exe`, all data is kept in a `data` folder beside it; it doesn't start with Windows and, instead of updating itself, opens the download page when a new version is out.
- Full-text search index for large histories: beyond the newest 2,000 clips, search also finds older ones (word prefixes, accents ignored).

### Changed

- The **Tab** filter cycle ends with **Snippets**.
- With more than 2,000 clips, the palette loads all pins and the newest 2,000 clips; older clips appear when you search.
- The history database is upgraded automatically for snippets, shortcuts and the search index; the index is built once on the first start, which can take a moment with a large history.
- Releases also include the portable zip; the release notes mention it.

## [0.2.1] - 2026-11-25

### Changed

- New brand: an illustrated kraft clipboard with a brass ring clip replaces the placeholder logo in the app icon, tray icon, Settings footer, README and website (favicon, header, wallpapers and social card).

## [0.2.0] - 2026-11-25

### Added

- Search inside images: text in copied images is recognized with the built-in Windows OCR, in the background, and included in search. Recent images already in the history are processed too. Can be turned off in Settings (**Recognize text in images**).
- Secret detection (**Ignore secrets**), on by default: text that looks like a private key (PEM), a JSON Web Token, an AWS access key ID, a GitHub, Slack, Stripe live, Google API, OpenAI or Anthropic key, or a payment card number (Luhn-checked) is not saved.
- **Delete old clips**: optionally delete unpinned clips that have not been copied or pasted for 1 to 365 days (off by default).
- `app:` search filter: `app:code debounce` searches only clips copied from apps whose name contains "code"; repeat it to allow several apps.
- Appearance picker in Settings: System / Light / Dark cards and eight accent colors, including the current Windows accent (the default).
- Warning when the shortcut cannot be registered because another app uses it: Wincy opens Settings at startup with a banner and the tray tooltip says the shortcut is unavailable.
- Log files in `%LOCALAPPDATA%\dev.wincy.app\logs` (rotated, never containing clip contents), **Open log folder** and **Copy diagnostics** in Settings for bug reports.
- **Buy me a coffee** button in Settings.

### Changed

- The Mica / Acrylic backdrop follows the theme chosen in Wincy instead of the Windows theme.
- The history database is upgraded automatically to store recognized image text.
- Errors are written to the log file instead of the console.

## [0.1.0] - 2026-11-25

### Added

- Tray app with a global shortcut (**Alt+V**) that opens a command palette next to the mouse cursor.
- Event-driven clipboard capture (no polling) for text, rich text (HTML), images and file lists.
- Automatic kind detection: text, URL, color, code, image, files.
- Fuzzy search, kind filters, preview pane and full keyboard control.
- Paste into the previously focused app (**Enter**) or as plain text (**Shift+Enter**); **Alt+1…9** quick paste.
- Pinned items that are never trimmed; configurable history size.
- Privacy: honors `ExcludeClipboardContentFromMonitorProcessing` and related formats, ignored-apps list (password managers by default), pause capture from the tray.
- Settings: shortcut, history size, launch at startup, paste directly, capture images, theme, language (English, Spanish).
- Signed auto-updates from GitHub Releases.

[Unreleased]: https://github.com/devnielo/wincy/compare/v0.3.0...HEAD
[0.3.0]: https://github.com/devnielo/wincy/compare/v0.2.1...v0.3.0
[0.2.1]: https://github.com/devnielo/wincy/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/devnielo/wincy/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/devnielo/wincy/releases/tag/v0.1.0
