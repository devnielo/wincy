# winget package

Templates for publishing Wincy to the [Windows Package Manager](https://learn.microsoft.com/windows/package-manager/) as **`devnielo.Wincy`**, so users can run `winget install devnielo.Wincy`.

| file | what |
|---|---|
| `templates/devnielo.Wincy.yaml` | version manifest |
| `templates/devnielo.Wincy.installer.yaml` | installer: NSIS (`nullsoft`), per-user (`Scope: user`), x64 |
| `templates/devnielo.Wincy.locale.en-US.yaml` | default locale (English) |
| `templates/devnielo.Wincy.locale.es-ES.yaml` | Spanish locale |
| `render.mjs` | fills `{{VERSION}}`, `{{INSTALLER_URL}}` and `{{SHA256}}` for one release |

The manifests use schema **1.28.0** (`ManifestVersion`). If `winget validate` or the winget-pkgs bot asks for a newer one, change the `ManifestVersion` lines and the `$schema` comments.

## Submitting a release

Only submit a **published** GitHub release: winget downloads the installer from its release URL and checks the hash. If SignPath signing is set up ([docs/SIGNING.md](../../docs/SIGNING.md)), submit the signed installer.

1. Download the installer from the release, then render the manifests (Node 22+, no dependencies):

   ```powershell
   gh release download v0.2.0 -R devnielo/wincy -p "*_x64-setup.exe" -D $env:TEMP\wincy
   node packaging/winget/render.mjs 0.2.0 $env:TEMP\wincy\Wincy_0.2.0_x64-setup.exe
   ```

   This writes `packaging/winget/manifests/d/devnielo/Wincy/0.2.0/` (git-ignored). Always hash the downloaded file, never a local build: a rebuilt installer has a different SHA256.

2. Validate:

   ```powershell
   winget validate --manifest packaging/winget/manifests/d/devnielo/Wincy/0.2.0
   ```

3. Test the install in **Windows Sandbox** or a VM, not on a machine where Wincy is already running (the installed copy owns Alt+V):

   ```powershell
   winget settings --enable LocalManifestFiles
   winget install --manifest <folder>
   ```

4. Fork [microsoft/winget-pkgs](https://github.com/microsoft/winget-pkgs), copy the folder to the same path (`manifests/d/devnielo/Wincy/0.2.0/`) and open a pull request titled `New package: devnielo.Wincy version 0.2.0` (later releases: `New version: devnielo.Wincy version X.Y.Z`). Fill in the PR template checklist. The validation bot installs the package; a moderator merges it, which can take a few days.

For later versions you can also let [wingetcreate](https://github.com/microsoft/winget-create) do steps 1–4: `wingetcreate update devnielo.Wincy --version X.Y.Z --urls <installer URL> --submit`.

## Notes

- `ProductCode: Wincy` matches the uninstall registry key Tauri's NSIS installer writes (`HKCU\Software\Microsoft\Windows\CurrentVersion\Uninstall\Wincy`), so winget recognizes copies installed from GitHub too.
- `UpgradeBehavior: install` runs the new installer over the old one, the same as the in-app updater. Both can coexist; winget simply sees the newer version once the app has updated itself.
- Once the package is merged, update the "Not on winget yet" note on the website (`website/src/i18n/en.ts` and `es.ts`, `install.note`), the "Not available" list in `website/PRODUCT.md`, the Install section of both READMEs and the roadmap item.
