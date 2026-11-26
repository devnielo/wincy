<p align="center">
  <img src="assets/logo.svg" width="96" height="96" alt="Wincy logo">
</p>

<h1 align="center">Wincy</h1>

<p align="center">
  <b>The keyboard-first clipboard manager for Windows.</b><br>
  Press <kbd>Alt</kbd>+<kbd>V</kbd>, type, hit <kbd>Enter</kbd>. Fast, private, open source.
</p>

<p align="center">
  <a href="https://github.com/devnielo/wincy/releases/latest"><img src="https://img.shields.io/github/v/release/devnielo/wincy?sort=semver" alt="Latest release"></a>
  <a href="https://github.com/devnielo/wincy/actions/workflows/ci.yml"><img src="https://github.com/devnielo/wincy/actions/workflows/ci.yml/badge.svg" alt="CI"></a>
  <a href="LICENSE"><img src="https://img.shields.io/github/license/devnielo/wincy" alt="MIT license"></a>
  <a href="https://buymeacoffee.com/devnielo"><img src="https://img.shields.io/badge/Buy%20me%20a%20coffee-FFDD00?logo=buymeacoffee&logoColor=000" alt="Buy me a coffee"></a>
</p>

<p align="center">
  English · <a href="README.es.md">Español</a>
</p>

<p align="center"><img src="docs/screenshots/palette-code.png" width="720" alt="Wincy palette with the code preview panel"></p>

## Features

- **Opens where you are:** a command palette appears next to the mouse cursor, not in a corner of the screen.
- **Everything you copy:** text, rich text (HTML), images, file lists, URLs, colors and code, each detected and labeled automatically.
- **Fuzzy search** across contents and source apps, plus quick kind filters (<kbd>Tab</kbd>) and an `app:` filter (`app:chrome`). In very large histories, a full-text index finds older clips too.
- **Search inside images:** text in screenshots and other copied images is read with the OCR built into Windows, on your PC, so searching finds them too.
- **Paste anywhere:** <kbd>Enter</kbd> pastes straight into the app you were using; <kbd>Shift</kbd>+<kbd>Enter</kbd> pastes as plain text.
- **Quick paste:** <kbd>Alt</kbd>+<kbd>1</kbd>…<kbd>9</kbd> pastes the Nth visible item.
- **Quick actions:** <kbd>Ctrl</kbd>+<kbd>K</kbd> on a clip to change its case, trim it, format JSON, encode or decode Base64 and URLs, open a link, show a file in Explorer or paste a color as HEX, RGB or HSL.
- **Snippets** for text you type again and again, with variables such as `{date}` and `{clipboard}` filled in when you paste.
- **Pins** keep clips forever; they are never trimmed from history.
- **Global shortcuts** for pinned clips and snippets: paste them into any app without opening the palette.
- **Export / import** your history and snippets to a file, to back them up or move them to another PC.
- **Floating preview panel** beside the palette (right or left, wherever it fits) that shows the hovered or selected clip in full: long text, code, images, colors and file lists, with scrolling.
- **Mouse-friendly:** drag the palette by its search bar, and every shortcut has a button: paste, plain text, pin, delete, pause capture, preview and settings.
- **Private by design:** local only, skips password managers, content marked as sensitive and text that looks like a secret (private keys, API tokens, card numbers); pause capture from the tray, and optionally let old clips expire after a number of days.
- **Light on resources:** event-driven capture (no polling), native Windows vibrancy (Mica / Acrylic), small Tauri binary.
- **Your look:** system, light or dark theme and an accent color (the Windows accent by default, or one of seven others).
- **Automatic, signed updates** and an English / Spanish interface, or a **portable version** that keeps everything in its own folder.

## Install

1. Download the latest installer (`*_x64-setup.exe`) from [Releases](https://github.com/devnielo/wincy/releases/latest).
2. Run it. It installs for your user only; no administrator rights needed.
3. Wincy starts in the system tray. Press <kbd>Alt</kbd>+<kbd>V</kbd>.

> **"Windows protected your PC"?** The installer is not signed with a paid code-signing certificate yet, so SmartScreen may warn about it. Click **More info → Run anyway**. If you prefer, [build it yourself](#build-from-source).

Wincy checks for updates on startup (you can turn this off in Settings). Updates are signed and verified before they are installed.

Requires Windows 10 or 11 (x64). WebView2 is already part of Windows 10/11.

### Portable version

Prefer not to install anything? Download `Wincy_<version>_x64-portable.zip` from the same release, extract the whole zip into a folder you can write to (a USB drive works) and run `wincy.exe`. Don't run it from inside the zip.

The empty file named `portable` next to `wincy.exe` is what makes it portable: your history, images, settings and logs are kept in a `data` folder beside the exe instead of your user profile. The portable version doesn't start with Windows (the option is hidden) and doesn't update itself: when a new version is out it tells you and opens the download page; replace `wincy.exe` and keep the `data` folder. Like the installer, the exe is not code-signed yet, so SmartScreen may warn the first time.

## Usage

Copy things as usual. When you need something again, press <kbd>Alt</kbd>+<kbd>V</kbd> (configurable), type a few letters and press <kbd>Enter</kbd>.

### Keyboard shortcuts

| Keys | Action |
|---|---|
| <kbd>Alt</kbd>+<kbd>V</kbd> | Open the palette (global, configurable) |
| *type* | Fuzzy filter by content (including text found in images), source app or kind; `app:name` keeps only clips from that app |
| <kbd>↑</kbd> / <kbd>↓</kbd>, <kbd>PgUp</kbd> / <kbd>PgDn</kbd>, <kbd>Home</kbd> / <kbd>End</kbd> (<kbd>Ctrl</kbd>+<kbd>Home</kbd> / <kbd>End</kbd>) | Move the selection |
| <kbd>Enter</kbd> | Paste |
| <kbd>Shift</kbd>+<kbd>Enter</kbd> | Paste as plain text |
| <kbd>Alt</kbd>+<kbd>1</kbd> … <kbd>Alt</kbd>+<kbd>9</kbd> | Paste the Nth visible item (pins come first, so <kbd>Alt</kbd>+<kbd>1</kbd> is the first pin) |
| <kbd>Ctrl</kbd>+<kbd>K</kbd> | Quick actions for the selected clip (type to filter, <kbd>Enter</kbd> runs, <kbd>Esc</kbd> closes) |
| <kbd>Ctrl</kbd>+<kbd>P</kbd> | Pin / unpin |
| <kbd>Ctrl</kbd>+<kbd>Delete</kbd> | Delete the item |
| <kbd>Tab</kbd> / <kbd>Shift</kbd>+<kbd>Tab</kbd> | Cycle the kind filter: all → text → url → color → code → image → files → snippets |
| <kbd>Ctrl</kbd>+<kbd>Space</kbd> | Show / hide the preview panel |
| <kbd>Ctrl</kbd>+<kbd>,</kbd> | Open settings |
| <kbd>Esc</kbd> | Clear the search; if it is empty, close the palette (in settings: back to the palette) |
| *your own* | Paste a pinned clip or snippet that has its own shortcut, from any app (global) |

The tray icon menu lets you open the palette or settings, pause capture, clear history (pinned items are kept), or quit.

If **Paste into the active app** is turned off in Settings, Wincy only copies the item to the clipboard and you paste it yourself.

If another program already uses your shortcut, Wincy can't register it: it opens Settings with a warning when it starts (the tray tooltip also says the shortcut is unavailable), so you can record a different one.

### Search

Type a few letters of anything in the clip: its text, the text Wincy found in an image, the source app or the kind. Add `app:<name>` to keep only clips copied from apps whose name contains `<name>` (case-insensitive, without `.exe`); repeat it to allow several apps.

| Query | Finds |
|---|---|
| `invoice` | clips containing something like "invoice", including screenshots of one |
| `app:code debounce` | "debounce" in clips copied from VS Code (`Code.exe`) |
| `app:chrome app:edge` | everything copied from Chrome or Edge |

Text in images is recognized in the background with the Windows OCR engine, in one of your Windows display languages; nothing is uploaded. If no OCR language is installed, images are still saved, just not searchable by their text. To turn it off, use Settings → **Recognize text in images**.

With a large history, the palette keeps all pins and the newest 2,000 clips at hand for instant fuzzy search. If you have more, search also looks through the older ones with a full-text index (matches word beginnings, ignores accents) and lists what it finds after the other results.

### Quick actions

Press <kbd>Ctrl</kbd>+<kbd>K</kbd> (or click **Actions** in the footer) to see what you can do with the selected clip. Type to filter the list, <kbd>Enter</kbd> runs an action, <kbd>Esc</kbd> closes it. Only the actions that make sense for the clip are shown:

| Clip | Actions |
|---|---|
| Text | UPPERCASE, lowercase, Title Case, Trim, Collapse whitespace, Remove line breaks; Format / Minify JSON (when it is valid JSON); Base64 and URL encode / decode (decode only when the result is readable text); **Save as snippet** |
| Link | **Open in browser** (http, https and mailto links) |
| Files | **Show in Explorer** (the first file) |
| Color | Paste as HEX, RGB or HSL |
| Pinned clip | **Set shortcut…** |

Transformed text is pasted as plain text. The clip in your history stays as it was, and the transformed text is not added to it.

### Snippets

Snippets are texts you paste often: a signature, an address, a reply template. Add, edit and delete them in Settings → **Snippets**, or turn a clip into one with **Save as snippet** in its quick actions. In the palette they have their own filter (**Snippets**, the last one in the <kbd>Tab</kbd> cycle) and search covers their name and text; <kbd>Enter</kbd> pastes a snippet as plain text.

These variables are filled in when you paste:

| Variable | Becomes |
|---|---|
| `{date}` | today's date in your Windows short date format |
| `{time}` | the current time (`HH:MM`) |
| `{datetime}` | date and time |
| `{clipboard}` | the text currently on the clipboard |
| `{{` | a literal `{` |

Any other `{word}` is pasted as is.

### Shortcuts for pins and snippets

Give a pinned clip its own shortcut with **Set shortcut…** in its quick actions, or a snippet in the snippet editor. Pressing that shortcut in any app pastes the clip or snippet straight away, without opening the palette. A row with a shortcut shows it as a small keycap next to the pin.

Unpinning or deleting a clip removes its shortcut. Wincy refuses shortcuts it already uses and the same combinations as for the main shortcut (such as <kbd>Ctrl</kbd>+<kbd>C</kbd> or <kbd>Alt</kbd>+<kbd>1</kbd>). If another program already owns one of your shortcuts, Settings lists it so you can pick another.

## Privacy

Your clipboard history never leaves your computer: no accounts, no cloud, no telemetry. The only network request Wincy makes is the update check against GitHub Releases, which you can disable.

Wincy does **not** capture:

- content that the source app marks as private with the standard Windows clipboard formats `ExcludeClipboardContentFromMonitorProcessing`, `Clipboard Viewer Ignore` or `CanIncludeInClipboardHistory = 0` (Bitwarden, 1Password, KeePassXC and browsers in private mode do this);
- anything copied from an app in the **ignored apps** list. By default: Bitwarden, 1Password, KeePass, KeePassXC, Enpass, Dashlane, LastPass, NordPass and Proton Pass. Add your own in Settings by exe name (e.g. `myvault.exe`);
- text that **looks like a secret** (on by default; Settings → **Ignore secrets**): a PEM private key, a JSON Web Token, AWS access key IDs, GitHub, Slack, Stripe live, Google API, OpenAI and Anthropic keys, or a payment card number (checked with the Luhn algorithm). The check only matches these exact formats, so ordinary text is never thrown away by accident; it is not a guarantee that every secret is caught;
- anything at all while capture is **paused** from the tray menu.

Pinned items are kept until you delete them. Everything else is trimmed to the history size you choose (1000 items by default) and, if you choose a period in Settings → **Delete old clips**, deleted once it hasn't been copied or pasted for that many days (1 to 365; off by default).

### Where your data lives

| What | Where |
|---|---|
| History database | `%APPDATA%\dev.wincy.app\wincy.db` (SQLite) |
| Copied images and thumbnails | `%APPDATA%\dev.wincy.app\images\` |
| Settings | `%APPDATA%\dev.wincy.app\settings.json` (the app config folder) |
| Logs | `%LOCALAPPDATA%\dev.wincy.app\logs\` (rotated at about 1 MB; never contain clip contents) |

**Settings → Open data folder** opens it in Explorer. **Clear history** deletes everything except pinned items. Text recognized in images and your snippets are stored in the same database.

In the [portable version](#portable-version), all of the above lives in the `data` folder next to `wincy.exe` instead.

### Export and import

Settings → **Data** → **Export…** saves your history and snippets to a `.wincy.json` file; with **Include images** (on by default) copied images go in too, which makes the file larger. Settings are not exported, as they belong to each PC.

**Import…** reads such a file and adds it to your current history. Clips you already have are not duplicated, secrets are skipped when **Ignore secrets** is on, and the history is trimmed to its size afterwards (pinned clips are kept). Files that aren't a valid Wincy export are rejected.

The export file holds your clipboard history as plain text. Keep it somewhere safe.

## Reporting a bug

Wincy writes a log file (see the table above; Settings → **Open log folder** opens it). Logs record what happened, such as a skipped capture or a failed shortcut, but never what you copied: no clip contents, no recognized text.

When you [open a bug report](https://github.com/devnielo/wincy/issues/new?template=bug_report.yml), click **Settings → Copy diagnostics** and paste the result into it. It contains the Wincy version, Windows version and build, your settings (the ignored apps only as a count), the number of clips, whether OCR is available and the last ~200 log lines. Read it before you post it.

## Uninstall

Windows Settings → Apps → Installed apps → **Wincy** → Uninstall. The uninstaller offers to delete your app data too; to remove it by hand, delete both `%APPDATA%\dev.wincy.app` and `%LOCALAPPDATA%\dev.wincy.app`.

Portable version: quit Wincy from the tray and delete its folder.

## Build from source

Prerequisites:

- [Rust](https://rustup.rs/) stable with the MSVC toolchain (`x86_64-pc-windows-msvc`; needs the Visual Studio Build Tools "Desktop development with C++" workload)
- [Node.js](https://nodejs.org/) 22.23.4 or newer (current LTS)
- [pnpm](https://pnpm.io/installation)
- WebView2 (preinstalled on Windows 10/11)

```powershell
git clone https://github.com/devnielo/wincy.git
cd wincy
pnpm install
pnpm tauri dev      # run with hot reload
pnpm tauri build    # installer in src-tauri/target/release/bundle/nsis/
```

`pnpm tauri build` signs the updater artifacts, so it needs `TAURI_SIGNING_PRIVATE_KEY`. For a local build, create your own key with `pnpm tauri signer generate -w "$env:USERPROFILE\.tauri\local.key"` and set the variable to the contents of that file. A build signed with your own key will not accept official updates.

## Project structure

```
├── src/                 React + TypeScript frontend (palette, settings)
├── src-tauri/           Rust backend (Tauri v2)
│   └── src/             clipboard listener, SQLite storage, paste, window, tray, settings
├── website/             Astro website (GitHub Pages)
├── assets/logo.svg      master logo (app icons are generated from it)
├── docs/
│   ├── ARCHITECTURE.md  contract between backend and frontend: commands, events, types
│   ├── RELEASING.md     how releases and auto-updates are produced
│   └── SIGNING.md       Authenticode signing through SignPath Foundation
├── packaging/           winget manifest templates, portable zip README
└── .github/             CI, release and Pages workflows, issue templates
```

## Tech stack

- [Tauri v2](https://tauri.app/) and Rust: Win32 clipboard listener (`AddClipboardFormatListener`), [rusqlite](https://github.com/rusqlite/rusqlite) (bundled SQLite), [enigo](https://github.com/enigo-rs/enigo) for paste, [window-vibrancy](https://github.com/tauri-apps/window-vibrancy)
- React 19 and TypeScript, [HeroUI](https://www.heroui.com/), [Tailwind CSS 4](https://tailwindcss.com/), [TanStack Virtual](https://tanstack.com/virtual), [Fuse.js](https://www.fusejs.io/), [Zustand](https://zustand.docs.pmnd.rs/), [Lucide](https://lucide.dev/) icons
- SQLite FTS5 full-text index for large histories
- NSIS installer with signed auto-updates via `tauri-plugin-updater`, plus a portable zip

## Roadmap ideas

Directions, not promises. Open an issue to discuss before starting work.

- A cursor-position placeholder in snippets
- Per-app paste rules and an "always plain text" list
- Optional encryption of the history database
- "Center of screen" palette position and multi-monitor options
- More languages
- winget and Scoop packages

## Code signing policy

Every release is built by GitHub Actions from this public repository. Once approved, Windows releases will be signed: free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org). Until then, releases are unsigned.

- Committers and reviewers: [@devnielo](https://github.com/devnielo)
- Approvers: [@devnielo](https://github.com/devnielo)

Multi-factor authentication is required for everyone with these roles.

**Privacy:** this program will not transfer any information to other networked systems unless specifically requested by the user or the person installing or operating it. The only network call is the update check against GitHub Releases, which you can turn off in Settings.

## Support the project

Wincy is free and open source, and always will be. If it saves you time every day, you can [buy me a coffee](https://buymeacoffee.com/devnielo) — it helps keep the project maintained. Starring the repo and reporting bugs helps too.

## Contributing

Contributions are welcome. Read [CONTRIBUTING.md](CONTRIBUTING.md) for the development setup and conventions, and the [Code of Conduct](CODE_OF_CONDUCT.md). Report security issues privately as described in [SECURITY.md](SECURITY.md). The [changelog](CHANGELOG.md) lists what changed in each release.

## License

[MIT](LICENSE) © 2026 Wincy contributors

## Credits

Inspired by [Maccy](https://maccy.app/) (simplicity), [Raycast](https://www.raycast.com/) (the keyboard-first palette) and [Microsoft PowerToys](https://github.com/microsoft/PowerToys) (Windows power-user tools). Built with [Tauri](https://tauri.app/).
