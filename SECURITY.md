# Security Policy

Wincy stores everything you copy, which can include passwords, tokens and personal data. We treat any bug that could expose that data as a security issue.

## Reporting a vulnerability

**Do not open a public issue.** Report privately through GitHub Security Advisories:
[github.com/devnielo/wincy/security/advisories/new](https://github.com/devnielo/wincy/security/advisories/new)

Include the Wincy and Windows versions, steps to reproduce and the impact you expect. Never include real clipboard contents or secrets; use dummy data.

We aim to acknowledge reports within 7 days and to ship a fix for confirmed issues as fast as their severity requires. We will credit you in the advisory unless you prefer otherwise.

## Supported versions

Only the latest release receives security fixes. Installed copies update themselves through the signed auto-updater.

## In scope

- Clipboard history leaving the machine, or being readable by other users of the same PC.
- Content that should be skipped being captured anyway: data marked with `ExcludeClipboardContentFromMonitorProcessing`, `Clipboard Viewer Ignore` or `CanIncludeInClipboardHistory = 0`, or copied from an app in the ignored-apps list.
- Clipboard data written to logs, crash reports or anywhere outside the app data folder.
- Script injection through clipboard contents (for example HTML or file names rendered in the palette).
- Bypassing the updater signature check, or tampering with release artifacts.

## Out of scope

- Attacks that require an attacker already running code as your Windows user. Such code can read the clipboard and your files directly, with or without Wincy.
- Data you copied before adding an app to the ignored list. Delete those items or clear the history.
