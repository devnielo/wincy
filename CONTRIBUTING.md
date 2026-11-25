# Contributing to Wincy

Thanks for helping! Bug reports, ideas, translations, docs and code are all welcome.

## Before you start

- Search [existing issues](https://github.com/devnielo/wincy/issues) first. For larger changes, open an issue to agree on the approach before writing code.
- Security problems go through [SECURITY.md](SECURITY.md), never a public issue.
- By participating you agree to the [Code of Conduct](CODE_OF_CONDUCT.md).

## Development setup

Prerequisites (Windows 10/11 only; Wincy uses Win32 APIs directly):

- [Rust](https://rustup.rs/) stable with the MSVC toolchain (`x86_64-pc-windows-msvc`) and the "Desktop development with C++" workload from Visual Studio Build Tools.
- [Node.js](https://nodejs.org/) 22.23.4 or newer (current LTS) and [pnpm](https://pnpm.io/installation).
- WebView2 is preinstalled on Windows 10/11.

```powershell
pnpm install
pnpm tauri dev     # app with hot reload; Alt+V opens the palette
```

Useful checks (the same ones CI runs):

```powershell
pnpm build                                   # tsc (strict) + vite build
pnpm test                                    # frontend unit tests (node --test)
cd src-tauri
cargo fmt --all --check
cargo clippy --all-targets -- -D warnings
cargo test
```

Run a production build with `pnpm tauri build`. The NSIS installer lands in `src-tauri/target/release/bundle/nsis/`.

The website lives in `website/` with its own `package.json` (`cd website; pnpm install; pnpm dev`).

## The contract: docs/ARCHITECTURE.md

[`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) is the single source of truth for command names and arguments, event names and payloads, data types, settings and defaults, the keyboard map and the capture/paste rules. Rust (`src-tauri/src`) and TypeScript (`src/types.ts`) must both match it.

If your change touches any of these, update `ARCHITECTURE.md` in the same pull request and change both sides together.

## Coding standards

- **Rust:** `cargo fmt`; `cargo clippy -- -D warnings` must be clean. No `unwrap()`/`expect()` on data that comes from the clipboard, the database or the file system. Return errors as `String` from commands. Keep pure logic (such as `detect.rs`) unit-tested.
- **TypeScript:** `strict` mode, no `any`. Components are function components; shared state lives in the zustand store.
- **UI strings:** never hard-code; add them to `src/i18n.ts` in both `en` and `es`.
- **Privacy:** never log clipboard contents, never send them over the network, never write them outside the app data folder.
- Keep changes small and focused. Comment the *why*, not the *what*.

## Commits and pull requests

Commits and PR titles follow [Conventional Commits](https://www.conventionalcommits.org/):

```
feat: add "paste as plain text" to the context menu
fix(clipboard): skip empty CF_HDROP lists
docs: describe portable mode
```

Common types: `feat`, `fix`, `perf`, `refactor`, `docs`, `test`, `build`, `ci`, `chore`. Add `!` (e.g. `feat!:`) for breaking changes.

PR checklist:

1. CI is green (build, fmt, clippy, tests).
2. The change was tested by hand on Windows; the PR says which version, and UI changes include a screenshot.
3. `docs/ARCHITECTURE.md` is updated if the contract changed.
4. User-facing changes have an entry under **Unreleased** in [`CHANGELOG.md`](CHANGELOG.md), and both `README.md` and `README.es.md` are updated if needed.

Maintainers squash-merge, so the PR title becomes the commit message.

## Releasing

Maintainers: see [docs/RELEASING.md](docs/RELEASING.md).
