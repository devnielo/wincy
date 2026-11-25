# Changelog

All notable changes to Wincy are documented here.
The format follows [Keep a Changelog](https://keepachangelog.com/en/1.1.0/) and the project uses [Semantic Versioning](https://semver.org/).

## [Unreleased]

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

[Unreleased]: https://github.com/devnielo/wincy/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/devnielo/wincy/releases/tag/v0.1.0
