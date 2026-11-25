# Code signing (SignPath Foundation)

Wincy's installer is not Authenticode-signed yet, so Windows SmartScreen shows "Windows protected your PC". [SignPath Foundation](https://signpath.org/) signs open-source projects for free with a certificate issued to SignPath Foundation (the installer's publisher then reads "SignPath Foundation"). The release workflow already contains the signing path; it stays **off** until the repository variable `SIGNPATH_ORGANIZATION_ID` is set.

This is separate from the **updater signature** (`TAURI_SIGNING_PRIVATE_KEY`, see [RELEASING.md](RELEASING.md)): that one proves an update came from us, Authenticode proves it to Windows. Releases always need the updater key; Authenticode is extra.

## 1. Apply

Apply at <https://signpath.org/apply>. Acceptance is at SignPath Foundation's discretion. Their [terms](https://signpath.org/terms) require, among other things:

- an OSI-approved license with no proprietary components (Wincy: MIT, all dependencies open source);
- an actively maintained project with at least one public release already out (v0.1.0) and documented features (README, website);
- signing only software built from your own repository, on GitHub-hosted runners, verifiable from source;
- no data sent anywhere the user did not choose without a privacy policy and an opt-out (Wincy's only request is the update check, which can be turned off; see the Privacy section of the README);
- a working uninstaller (the NSIS uninstaller);
- **MFA** on GitHub and SignPath for every team member;
- defined roles: *authors* (commit without review), *reviewers* (review outside contributions) and *approvers* (approve every signing request by hand);
- a **"Code signing policy"** section or link on the home page and the download page (see step 4);
- product name and version metadata enforced in the artifact configuration (below).

## 2. Configure SignPath (after acceptance)

In the SignPath web app for the organization they create for you:

1. **Trusted build system:** add the predefined *GitHub.com* trusted build system and link it to the project. Optionally install the SignPath GitHub App on `devnielo/wincy`.
2. **Project** with slug `wincy` (or set `SIGNPATH_PROJECT_SLUG`), linked to this repository.
3. **Signing policy** with slug `release-signing` (or set `SIGNPATH_SIGNING_POLICY_SLUG`), with yourself as approver.
4. **Artifact configuration** (default for the project). The workflow uploads the installer as a GitHub artifact, which GitHub zips, so the root is `<zip-file>`:

   ```xml
   <?xml version="1.0" encoding="utf-8"?>
   <artifact-configuration xmlns="http://signpath.io/artifact-configuration/v1">
     <parameters>
       <parameter name="version" />
     </parameters>
     <zip-file>
       <pe-file path="Wincy_${version}_x64-setup.exe" product-name="Wincy" product-version="${version}">
         <authenticode-sign />
       </pe-file>
     </zip-file>
   </artifact-configuration>
   ```

   The workflow passes `version` (the tag without `v`). If SignPath rejects the product version, open the installer's Properties → Details and match what it shows.
5. **API token:** create a CI user with the *submitter* role on the project and generate an API token for it.

## 3. Configure GitHub

Repo → Settings → Secrets and variables → Actions:

| kind | name | value |
|---|---|---|
| secret | `SIGNPATH_API_TOKEN` | the CI user's API token |
| variable | `SIGNPATH_ORGANIZATION_ID` | the organization ID (a GUID). **Setting it turns signing on**; deleting it turns it off |
| variable | `SIGNPATH_PROJECT_SLUG` | optional, defaults to `wincy` |
| variable | `SIGNPATH_SIGNING_POLICY_SLUG` | optional, defaults to `release-signing` |

`TAURI_SIGNING_PRIVATE_KEY` and `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` stay as they are.

## 4. Publish the policy

Add a "Code signing policy" section to `README.md`, `README.es.md` and the website download area, for example:

> **Code signing policy.** Free code signing provided by [SignPath.io](https://about.signpath.io), certificate by [SignPath Foundation](https://signpath.org). Committers and reviewers: [members of this repository](https://github.com/devnielo/wincy/graphs/contributors). Approvers: @devnielo. Privacy: Wincy sends no data anywhere except the optional update check against GitHub Releases; see [Privacy](#privacy).

Once the first signed release is out, also update the SmartScreen notes (README Install section and the website `install` / FAQ copy): SmartScreen may still warn until the new certificate builds reputation.

## How the signed release runs

When `SIGNPATH_ORGANIZATION_ID` is set, `.github/workflows/release.yml` skips `tauri-action` and runs, in one job:

1. `pnpm tauri build`: the unsigned NSIS installer.
2. `actions/upload-artifact`: the installer becomes a workflow artifact (SignPath verifies it was built by this workflow run).
3. `signpath/github-action-submit-signing-request@v3`: submits it and waits up to one hour for your **manual approval** in SignPath, then downloads the signed installer to `signed/`. Approve it from the link in the job log or the SignPath web app.
4. Checks the Authenticode signature is valid.
5. `tauri signer sign` on the **signed** installer. Authenticode changes the file's bytes, so the `.sig` that `tauri build` made in step 1 no longer matches; the updater would reject the update with the old one.
6. Writes `latest.json` (same shape as `tauri-action`'s) pointing at the signed installer and its new signature.
7. Creates the draft release `Wincy vX.Y.Z` with the installer, `.sig` and `latest.json` (or replaces them on an existing draft). Then continue with [RELEASING.md](RELEASING.md) step 6.

If the approval times out, approve or reject the request in SignPath and re-run the workflow for the tag (it only re-uploads while the release is a draft).

## Limitations

- Only the installer is signed. `wincy.exe` inside it is not: SignPath cannot open NSIS installers, and signing the binary before bundling would need a second signing request in the middle of `tauri build`. SmartScreen judges the downloaded installer, which is what matters for the warning.
- Each release waits for a human approval, by design of the Foundation's program.
