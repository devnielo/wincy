# Releasing Wincy

Releases are built by GitHub Actions ([`.github/workflows/release.yml`](../.github/workflows/release.yml)) when a `vX.Y.Z` tag is pushed. The workflow builds the signed NSIS installer and creates a **draft** GitHub Release with the installer, its `.sig` signature, `latest.json` for the auto-updater and the portable zip.

## One-time setup

1. **Add the signing key secrets** (repo → Settings → Secrets and variables → Actions → New repository secret):

   | name | value |
   |---|---|
   | `TAURI_SIGNING_PRIVATE_KEY` | the full contents of `%USERPROFILE%\.tauri\wincy.key` |
   | `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | empty (the key has no password) |

   The matching public key is already in `tauri.conf.json` (`plugins.updater.pubkey`). Back the private key up somewhere safe. **If it is lost, installed copies can never auto-update again**, and you would have to ship a new key in a release users install by hand.

   GitHub does not allow saving a secret with an empty value. If it refuses, leave `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` undefined: the workflow then passes an empty string, which is what an unencrypted key needs.

2. **Enable GitHub Pages:** Settings → Pages → Build and deployment → Source: **GitHub Actions** (used by `pages.yml` for the website).

3. **Leave Workflow permissions on Read** (Settings → Actions → General): the workflows declare the write scopes they need.

4. **Optional: Authenticode signing** through SignPath Foundation, so SmartScreen stops warning. It stays off until configured; see [SIGNING.md](SIGNING.md).

## Cutting a release

1. Make sure `main` is green in CI.
2. Bump the version (same `X.Y.Z`, no `v`) in all three files:
   - `package.json` → `"version"`
   - `src-tauri/Cargo.toml` → `[package] version` (then run `cargo check` in `src-tauri/` so `Cargo.lock` is updated too)
   - `src-tauri/tauri.conf.json` → `"version"`
3. In `CHANGELOG.md`, turn the release's heading into `## [X.Y.Z] - YYYY-MM-DD` (it is either `## [X.Y.Z] - Unreleased` or the `## [Unreleased]` section), keep an empty `## [Unreleased]` above it and update the compare links at the bottom.
4. Commit and tag:

   ```bash
   git commit -am "chore(release): vX.Y.Z"
   git tag vX.Y.Z
   git push origin main vX.Y.Z
   ```

5. Wait for the **Release** workflow to finish. It creates a draft release named `Wincy vX.Y.Z`. With SignPath signing on, it first waits for you to approve the signing request in SignPath (link in the job log).
6. Check the draft: the installer (`*_x64-setup.exe`), `*_x64-setup.exe.sig`, `latest.json` and `*_x64-portable.zip` must be attached. Optionally paste the changelog section into the release notes.
7. **Publish the draft.** Until it is published, the updater cannot see it: `releases/latest` only resolves to published, non-prerelease releases.
8. **Rebuild the website**: Actions → Pages → Run workflow (on `main`). Its download buttons link straight to this version's installer and zip, and they keep pointing at the previous version until the site is rebuilt. Don't push `website/` changes between the version bump and publishing the release, or the buttons would point at a draft and return 404.

To rebuild an existing tag, run the workflow manually (Actions → Release → Run workflow) and pick the **tag** in "Use workflow from". The job is skipped when run from a branch. This only works while the release is still a draft. Once it is published, delete the release (keep the tag) and re-run, or ship a new patch version.

## How auto-updates work

- `tauri.conf.json` sets `bundle.createUpdaterArtifacts: true`, so `tauri build` signs the installer with `TAURI_SIGNING_PRIVATE_KEY` and writes a `.sig` file next to it.
- `tauri-action` with `uploadUpdaterJson: true` builds `latest.json` from the version, release notes, installer URL and signature, and uploads it to the release.
- The app's updater checks `https://github.com/devnielo/wincy/releases/latest/download/latest.json`. GitHub redirects `latest/download/<file>` to the asset of the newest **published** release.
- The updater downloads the installer only if its signature verifies against the public key built into the app, then runs it in passive mode and restarts Wincy.

With SignPath signing on, the workflow builds with `pnpm tauri build` instead of `tauri-action`, has the installer Authenticode-signed, then signs the **signed** installer again with `tauri signer sign` (Authenticode changes the bytes, so the first `.sig` would not verify) and writes `latest.json` itself. See [SIGNING.md](SIGNING.md#how-the-signed-release-runs).

## Portable zip

After the installer, the workflow packs `Wincy_X.Y.Z_x64-portable.zip` with `src-tauri/target/release/wincy.exe` (the SignPath-signed copy when signing is on), an empty `portable` marker file and [`packaging/portable/README.txt`](../packaging/portable/README.txt) (EN/ES), and uploads it to the same draft. The marker makes Wincy keep its data in `data\` next to the exe. The portable build has no auto-update: `latest.json` only points at the installer, and a portable copy only says a new version is out and opens the download page.

winget stays **NSIS-only**: [`packaging/winget`](../packaging/winget/README.md) uses the `*_x64-setup.exe` installer, never the zip.

## Microsoft Store

The workflow also packs an unsigned `Wincy_X.Y.Z.0_x64.msix` with the same `wincy.exe` and uploads it as the workflow artifact **msix** (not to the release: Partner Center signs it). To submit it to the Store, see [STORE.md](STORE.md).

## If a release is broken

If a release turns out to be broken, publish a fixed higher version. Deleting a release makes `latest` fall back to the previous one, but users who already updated stay on the broken version until a newer one ships.
