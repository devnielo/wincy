# Changelog

All notable changes to Wincy are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/devnielo/wincy/compare/v0.2.1...HEAD
[0.2.1]: https://github.com/devnielo/wincy/compare/v0.2.0...v0.2.1
[0.2.0]: https://github.com/devnielo/wincy/compare/v0.1.0...v0.2.0
[0.1.0]: https://github.com/devnielo/wincy/releases/tag/v0.1.0
