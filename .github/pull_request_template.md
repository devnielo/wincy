## What and why

<!-- What does this change, and why? Link the issue: "Closes #123". -->

## How was it tested

<!-- Windows version, steps you followed, screenshots for UI changes. -->

## Checklist

- [ ] Title follows [Conventional Commits](https://www.conventionalcommits.org/) (`feat:`, `fix:`, `docs:`…)
- [ ] `pnpm build` passes (TypeScript strict)
- [ ] `cargo fmt --check`, `cargo clippy --all-targets -- -D warnings` and `cargo test` pass in `src-tauri/`
- [ ] Command, event, setting or type names changed? `docs/ARCHITECTURE.md` and both sides (Rust + TS) are updated
- [ ] User-facing change? `CHANGELOG.md` (Unreleased) and both READMEs are updated; new UI strings exist in `en` and `es`
- [ ] No clipboard data is logged, sent over the network or written outside the app data folder
