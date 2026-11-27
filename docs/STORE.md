# Microsoft Store

Wincy is published in the Microsoft Store as **Wincy Clipboard**. The Store package is an **unsigned MSIX**: Partner Center signs it when you upload it, so no certificate is needed here.

| Partner Center field | value |
|---|---|
| Package/Identity/Name | `devnielo.WincyClipboard` |
| Package/Identity/Publisher | `CN=217EA17B-43B9-4F4E-BDA0-F142229CF27F` |
| Package/Properties/PublisherDisplayName | `devnielo` |
| Reserved name (manifest `DisplayName`) | `Wincy Clipboard` |
| Store ID | `9N0HTLPQMKCT` |

These values are in [`packaging/msix/AppxManifest.xml`](../packaging/msix/AppxManifest.xml). If Partner Center ever shows different ones (Product management → Product identity), change the manifest to match.

## How the Store build differs

The same `wincy.exe` runs everywhere. When it starts inside the Store package (it has a package identity), it reports `Status.store = true` and:

- **never updates itself**: the Tauri updater is not loaded, there is no update check and Settings → Updates says "Updates come from the Microsoft Store." (Store policy). The Store build makes no network requests at all;
- **Launch at startup** uses the package's startup task `WincyStartup` instead of the `HKCU\...\Run` registry key, which MSIX virtualizes. If the user turned Wincy off in Windows Settings → Apps → Startup (or a policy did), Windows doesn't let the app turn itself back on: Wincy logs it and switches its own toggle off.

## Building the package

Every tagged release builds it. The **Release** workflow runs [`packaging/msix/build.ps1`](../packaging/msix/build.ps1) on the same `wincy.exe` as the installer (the SignPath-signed one when signing is on) and uploads `Wincy_X.Y.Z.0_x64.msix` as the workflow artifact **msix**. It is **not** attached to the GitHub release: an unsigned MSIX can't be installed by users.

To download it: Actions → the Release run for the tag → Artifacts → **msix**, or

```bash
gh run download <run-id> -n msix
```

To build it locally (Windows, PowerShell):

```powershell
pnpm tauri build                                  # or: pnpm build; cd src-tauri; cargo build --release
./packaging/msix/build.ps1 -Version X.Y.Z         # -> src-tauri/target/msix/Wincy_X.Y.Z.0_x64.msix
```

`build.ps1` takes `-Version X.Y.Z` (a leading `v` is fine), `-Exe` (default `src-tauri/target/release/wincy.exe`) and `-OutDir` (default `src-tauri/target/msix`). It stages `wincy.exe` and the logos from `src-tauri/icons`, writes the version as `X.Y.Z.0` (the Store requires the fourth part to be `0`), builds `resources.pri` (en-US default, es-ES) with `makepri` and packs with `makeappx`. It uses the newest Windows SDK in `C:\Program Files (x86)\Windows Kits\10\bin`, or, without one, downloads the `Microsoft.Windows.SDK.BuildTools` NuGet package into `%TEMP%\wincy-sdk-buildtools`.

### Trying the Store build before uploading

An unsigned package can't be installed, but with **Developer Mode** on (Settings → System → For developers) you can register the staged folder:

```powershell
# Quit the installed Wincy first: both would fight over Alt+V and the clipboard.
Add-AppxPackage -Register src-tauri/target/msix/stage/AppxManifest.xml
# ...test, then:
Get-AppxPackage devnielo.WincyClipboard | Remove-AppxPackage
```

Check: Settings → Updates shows the Store line, **Launch at startup** adds "Wincy" to Windows Settings → Apps → Startup, **Copy diagnostics** says `(Store)`, and **Open data folder** / **Open log folder** show your data (MSIX redirects AppData writes into the package's own folder).

## Submitting a version

The Store accepts each package version only once, and every update needs a higher one, so each submission is a new tagged release.

1. Cut the release as in [RELEASING.md](RELEASING.md) and download the **msix** artifact.
2. [Partner Center](https://partner.microsoft.com/dashboard) → Apps and games → **Wincy Clipboard** → **Start update** (the first time: **Start your submission**).
3. **Packages**: remove the previous package, upload `Wincy_X.Y.Z.0_x64.msix`, keep only **Windows 10/11 Desktop** as device family.
4. **Store listings**: check English (United States) and Spanish (Spain) below; add "What's new in this version" from `CHANGELOG.md`.
5. First submission only:
   - **Pricing and availability**: Free, all markets.
   - **Properties**: category, privacy policy URL, support contact (below).
   - **Age ratings**: the questionnaire below.
   - **Submission options → Notes for certification**: the `runFullTrust` justification below.
6. **Submit to the Store**. Certification usually takes a few days.

## Listing text

### English (en-us)

**Product name:** Wincy Clipboard

**Short description** (shown on top):

> Keyboard-first clipboard history for Windows. Press Alt+V, type, hit Enter. Fast, private, open source.

**Description:**

> Wincy keeps everything you copy and gets it back in a second. Press Alt+V and a command palette opens right next to your mouse cursor. Type a few letters to find any clip, press Enter and it is pasted straight into the app you were using.
>
> Wincy remembers text, rich text, images, file lists, links, colors and code, and labels each one automatically. It even reads the text in screenshots with the OCR built into Windows, so searching finds images too.
>
> Your history never leaves your PC: no account, no cloud, no telemetry. Wincy skips password managers, content that apps mark as private and text that looks like a secret, such as private keys, API tokens and card numbers.
>
> Wincy is free and open source (MIT): https://github.com/devnielo/wincy

**Features** (one per line):

```
Command palette next to your mouse cursor (Alt+V)
Text, rich text, images, files, links, colors and code
Fuzzy search, kind filters and app: filter
Finds text inside images with Windows OCR
Enter pastes into the previous app; Shift+Enter as plain text
Quick actions: change case, format JSON, Base64 and URL encode
Snippets with variables like {date} and {clipboard}
Pins and global shortcuts for pinned clips and snippets
Export and import your history
Private: local only, skips password managers and secrets
Light, dark or system theme with accent colors
English and Spanish
```

**Keywords** (up to 7):

```
clipboard, clipboard manager, clipboard history, copy paste, snippets, productivity, keyboard
```

### Spanish (es-es)

**Nombre del producto:** Wincy Clipboard

**Descripción breve:**

> Historial del portapapeles para Windows, pensado para el teclado. Pulsa Alt+V, escribe y pulsa Intro. Rápido, privado y de código abierto.

**Descripción:**

> Wincy guarda todo lo que copias y te lo devuelve en un segundo. Pulsa Alt+V y se abre una paleta de comandos junto al cursor del ratón. Escribe unas letras para encontrar cualquier elemento, pulsa Intro y se pega directamente en la aplicación que estabas usando.
>
> Wincy recuerda texto, texto enriquecido, imágenes, listas de archivos, enlaces, colores y código, y los etiqueta automáticamente. Además lee el texto de las capturas de pantalla con el OCR integrado en Windows, así que al buscar también encuentras imágenes.
>
> Tu historial nunca sale de tu PC: sin cuenta, sin nube, sin telemetría. Wincy ignora los gestores de contraseñas, el contenido que las aplicaciones marcan como privado y el texto que parece un secreto, como claves privadas, tokens de API y números de tarjeta.
>
> Wincy es gratuito y de código abierto (MIT): https://github.com/devnielo/wincy

**Características:**

```
Paleta de comandos junto al cursor del ratón (Alt+V)
Texto, texto enriquecido, imágenes, archivos, enlaces, colores y código
Búsqueda difusa, filtros por tipo y filtro app:
Encuentra texto dentro de imágenes con el OCR de Windows
Intro pega en la aplicación anterior; Mayús+Intro como texto sin formato
Acciones rápidas: mayúsculas, formatear JSON, Base64 y URL
Plantillas con variables como {date} y {clipboard}
Fijados y atajos globales para elementos fijados y plantillas
Exporta e importa tu historial
Privado: solo local, ignora gestores de contraseñas y secretos
Tema claro, oscuro o del sistema con colores de acento
Inglés y español
```

**Palabras clave:**

```
portapapeles, gestor de portapapeles, historial del portapapeles, copiar pegar, plantillas, productividad, teclado
```

### Screenshots

Desktop screenshots must be at least 1366 × 768. Use, in this order:

1. `docs/screenshots/palette-code.png` (1542 × 772)
2. `docs/screenshots/palette-ocr.png` (1542 × 772)
3. `docs/screenshots/palette-color.png` (1542 × 772)

`docs/screenshots/settings-appearance.png` (832 × 657) is too small: place it on a 1366 × 768 background or take a new one first. The same images work for both languages; Spanish screenshots are optional.

## Properties

| field | value |
|---|---|
| Category | **Productivity** (alternative: Utilities & tools) |
| Privacy policy URL | `https://github.com/devnielo/wincy#privacy` |
| Website | `https://devnielo.github.io/wincy/` |
| Support contact | `https://github.com/devnielo/wincy/issues` |
| System requirements | Keyboard recommended; mouse optional |

## Age rating (IARC questionnaire)

Category: **app** (not a game, not a browser). Answer **No** to every question:

- No violence, sexual content, crude humor, drugs, alcohol, tobacco or gambling.
- No user-generated content shared with other users, no chat or interaction between users.
- No sharing of the user's location or personal information with anyone.
- No digital purchases, ads or in-app purchases.
- No unrestricted internet access (links only open in the user's own browser when they choose to).

Expected rating: 3+ / Everyone.

## Restricted capability: `runFullTrust`

Paste into **Submission options → Notes for certification** (and the capability's justification field when asked):

> Wincy is a clipboard manager (desktop app packaged with the Desktop Bridge) and needs runFullTrust for Win32 APIs that have no AppContainer equivalent:
> 1. Clipboard listener: AddClipboardFormatListener / WM_CLIPBOARDUPDATE to record clipboard history in the background, including when the app has no focus.
> 2. Global hotkey: RegisterHotKey (Alt+V by default, plus user-defined shortcuts for pinned clips and snippets) to open the palette from any app.
> 3. Pasting into the previous window: it restores the window that was in the foreground and sends Ctrl+V with SendInput, so the chosen clip is pasted where the user was typing.
> 4. Notification area (tray) icon and menu to show the history, pause capture and quit, while running in the background.
> All data stays on the device (local SQLite database); the app sends nothing over the network.
>
> To test: run the app, copy some text in any app, press Alt+V, pick an entry and press Enter: it is pasted into the previous app.
