# Product

<!-- impeccable:product-schema 1 -->

> Inferred from the repository (README.md, docs/ARCHITECTURE.md, the app source in `src/` and `src-tauri/`, and the website copy) plus the owner's stated brief. No live interview round was possible in the session that wrote this file; facts marked *(inferred)* should be confirmed by the owner.

## Platform

web

## Users

- **Developers** who copy commands, snippets, URLs, hashes and colors between a terminal, an editor and a browser dozens of times an hour.
- **Designers** who move hex/rgb values, links and screenshots between Figma, browsers and chat.
- **Windows power users** (the Raycast / PowerToys crowd) who live on the keyboard and find the built-in Win+V history too small and too forgetful.

Situation: mid-task, hands on the keyboard, needing something they copied minutes or days ago, right now, without leaving the app they are in. On the website they are evaluating whether a small, unsigned, open-source utility is worth installing and trusting with their clipboard. *(inferred)*

## Product Purpose

Wincy is a keyboard-first clipboard manager for Windows 10 and 11. It records what the user copies and brings it back in a compact palette that opens next to the mouse pointer on Alt+V: type to fuzzy-filter, Enter to paste into the previous app. Success is the user never losing a copied thing again and never reaching for the mouse to get it back.

The website's job: explain that moment fast, let the visitor feel it (interactive palette demo), earn trust (local-only, open source, honest about the SmartScreen warning) and send them to the GitHub release download.

## Positioning

A native-feeling Windows 11 utility in the Raycast / PowerToys lineage with Maccy's philosophy: one small palette, one shortcut, no account, no cloud. It is what Win+V would be if it kept 1,000+ items across restarts, had fuzzy search, pins with Alt+1..9, type filters and plain-text paste, while staying entirely on the user's disk.

## Operating Context

- Invoked with a global hotkey (Alt+V by default, configurable) from any app; palette is about 600x460, appears beside the cursor, is pre-loaded so it shows instantly, and pastes back into the previously focused app.
- A floating preview window sits beside the palette (left or right, wherever it fits) showing the hovered/selected clip in full.
- Palette is draggable by its search bar; every keyboard action also has a button (paste, plain text, pin, delete, pause capture, preview, settings).
- Lives in the system tray; starts with Windows (can be disabled); per-user install, no admin rights.
- Distributed as an NSIS installer from GitHub Releases. Not code-signed: Windows SmartScreen shows "Windows protected your PC" and the user must click More info, then Run anyway.

## Capabilities and Constraints

Confirmed functionality (ground truth: README.md, docs/ARCHITECTURE.md, `src/`):

- Captures text, rich text (pasteable as plain text with Shift+Enter), URLs, colors, code, images (stored as PNG) and file lists; kind detected automatically, colors shown as swatches, images as thumbnails.
- Fuzzy search across content, source app and kind; kind filter icons in the search bar and Tab / Shift+Tab to cycle them.
- Pins (Ctrl+P) stay on top and are never trimmed; Alt+1..9 pastes the Nth visible item.
- History size 1,000 items by default, configurable from 50 to 10,000; stored in a local SQLite database in the user profile.
- Privacy: skips content marked with `ExcludeClipboardContentFromMonitorProcessing`, `Clipboard Viewer Ignore` or `CanIncludeInClipboardHistory = 0`; default ignore list (Bitwarden, 1Password, KeePass, KeePassXC, Enpass, Dashlane, LastPass, NordPass, Proton Pass), user-extendable by exe name; pause capture from the tray.
- Settings: shortcut, history size, image capture on/off, Enter copies instead of pasting, autostart, update checks, ignored apps, open data folder.
- Signed, verified auto-updates from GitHub Releases (the only network request; can be turned off). No telemetry, no analytics, no account, no cloud sync.
- Built with Tauri v2 + Rust + React on the system WebView2; Mica on Windows 11, Acrylic on Windows 10; light and dark themes; English and Spanish UI.

Not available (must not be claimed): winget / Scoop packages, portable .zip, code-signing certificate, sync between machines, macOS/Linux builds.

Website: Astro static site, EN at `/` and ES at `/es/`, base `/wincy`, deployed to GitHub Pages. Every string exists in both `src/i18n/en.ts` and `src/i18n/es.ts`.

## Brand Commitments

- Name: **Wincy**. Logo `public/logo.svg`; brand gradient `#6D5BFF` to `#22B8CF` (violet to teal).
- Voice: fast, quiet, precise, trustworthy, keyboard-first. Plain, specific claims; honest about limitations (unsigned installer, missing winget). No hype words. *(inferred from existing copy and the owner's brief)*
- Must read as a native Windows 11 utility, not a generic SaaS product.

## Evidence on Hand

- Open-source repository (MIT): https://github.com/devnielo/wincy, with public release pipeline.
- v0.1.0 published on GitHub Releases.
- The real app UI (source in `src/`) as the basis for any depiction of the palette.
- OG image `public/og.png`.

Absent and must not be fabricated: user counts, download numbers, GitHub stars, testimonials, press quotes, benchmarks (memory/CPU figures), company logos, screenshots that don't match the real UI.

## Product Principles

1. **The moment is the product.** Every surface serves "I need what I copied earlier, now"; anything that does not shorten that moment is cut.
2. **Keyboard first, mouse welcome.** Shortcuts lead, but every action is reachable by pointer.
3. **Local and honest.** Nothing leaves the PC except the optional update check; limitations are stated plainly.
4. **Feel native.** Look and behave like a first-party Windows 11 tool, not a web app in a frame.
5. **Small and quiet.** Light on resources, light on UI, no nagging.

## Accessibility & Inclusion

Target WCAG 2.2 AA on the website: keyboard-operable demo, visible focus, reduced-motion alternative, sufficient contrast in light and dark, bilingual EN/ES parity. *(inferred)*
