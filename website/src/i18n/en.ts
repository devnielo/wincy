export type ClipKind = "text" | "url" | "color" | "code" | "image" | "files";

const en = {
  htmlLang: "en",
  ogLocale: "en_US",
  languageName: "English",
  meta: {
    title: "Wincy · The keyboard-first clipboard manager for Windows",
    description:
      "Wincy keeps everything you copy and brings it back with Alt+V. Fuzzy search, pins, plain-text paste and privacy built in. Free and open source for Windows 10 and 11.",
    logoAlt: "Wincy logo",
  },
  nav: {
    skip: "Skip to content",
    label: "Main",
    features: "Features",
    shortcuts: "Shortcuts",
    privacy: "Privacy",
    install: "Install",
    faq: "FAQ",
    language: "Language",
  },
  hero: {
    title: "Everything you copy, one keystroke away.",
    lead: "Wincy remembers what you copy on Windows and brings it back in a palette that opens right at your cursor. Press Alt+V, type a few letters, hit Enter.",
    download: "Download for Windows",
    github: "View on GitHub",
    note: "Windows 10 & 11 · Free & open source (MIT)",
    tryIt: "Try it: type to filter, use ↑ ↓ to move and Enter to paste.",
  },
  demo: {
    label: "Interactive Wincy demo",
    search: "Search your clipboard…",
    listLabel: "Clipboard history",
    pinned: "Pinned",
    empty: "No clips match your search.",
    pasted: "Pasted",
    pastedPlain: "Pasted as plain text",
    closed: "Wincy is hidden, just like after Esc in the app.",
    reopen: "Open Wincy",
    kinds: {
      text: "Text",
      url: "Link",
      color: "Color",
      code: "Code",
      image: "Image",
      files: "Files",
    } satisfies Record<ClipKind, string>,
    hints: [
      { keys: ["↑", "↓"], label: "Navigate" },
      { keys: ["Enter"], label: "Paste" },
      { keys: ["Shift", "Enter"], label: "Plain text" },
      { keys: ["Alt", "1–9"], label: "Quick paste" },
      { keys: ["Esc"], label: "Close" },
    ],
    items: [
      { kind: "text", text: "Thanks! I'll send the signed contract over by Friday.", app: "outlook.exe", pinned: true },
      { kind: "code", text: "git log --oneline --graph --decorate -20", app: "WindowsTerminal.exe", pinned: true },
      { kind: "color", text: "#6D5BFF", app: "figma.exe" },
      { kind: "url", text: "https://learn.microsoft.com/windows/win32/dataxchg/clipboard", app: "msedge.exe" },
      { kind: "image", text: "Screenshot · 1280 × 720", app: "SnippingTool.exe" },
      { kind: "code", text: "export const debounce = (fn, ms) => { let t; … }", app: "Code.exe" },
      { kind: "files", text: "Q3-report.xlsx, invoice-0423.pdf", app: "explorer.exe" },
      { kind: "text", text: "Meeting moved to 3:30 pm, room B2", app: "ms-teams.exe" },
      { kind: "color", text: "rgb(34, 184, 207)", app: "chrome.exe" },
      { kind: "url", text: "https://www.figma.com/design/brand-kit", app: "chrome.exe" },
      { kind: "text", text: "Order #48213 has shipped", app: "chrome.exe" },
    ] satisfies { kind: ClipKind; text: string; app: string; pinned?: boolean }[],
  },
  features: {
    title: "Small app, sharp tools",
    lead: "Everything Wincy does is built around one moment: you need something you copied earlier, and you need it now.",
    items: [
      {
        icon: "palette",
        title: "A palette, not a window",
        body: "Alt+V opens a compact command palette next to your mouse pointer. It's already loaded in the background, so it appears instantly, and Enter pastes straight into the app you were using. Hover a clip to see it in full in a floating preview beside the palette, drag the palette by its search bar, and use the buttons if you prefer the mouse.",
      },
      {
        icon: "detect",
        title: "Knows what you copied",
        body: "Links, colors, code, images and files are detected automatically and shown the way they should look: swatches for colors, thumbnails for images. Tab filters by type.",
      },
      {
        icon: "privacy",
        title: "Private by default",
        body: "Copies from password managers like Bitwarden, 1Password and KeePassXC are never saved. Wincy honors the flag apps use to keep sensitive data out of clipboard history (ExcludeClipboardContentFromMonitorProcessing), and you can pause capture from the tray.",
      },
      {
        icon: "plain",
        title: "Plain text on demand",
        body: "Shift+Enter pastes without formatting, so text from a web page lands in your document without its fonts, colors and links tagging along.",
      },
      {
        icon: "pin",
        title: "Pins and Alt+1…9",
        body: "Pin the snippets you reuse (addresses, replies, commands) so they stay on top and are never trimmed. Alt+1 to Alt+9 pastes the first nine without even looking.",
      },
      {
        icon: "search",
        title: "Fuzzy search",
        body: "Typos welcome. Search across content, source app and type, and find that link from last Tuesday in a few keystrokes.",
      },
      {
        icon: "native",
        title: "Light and native",
        body: "Built with Rust and Tauri on the system WebView, not a bundled browser. Mica on Windows 11, Acrylic on Windows 10, light and dark themes.",
      },
      {
        icon: "local",
        title: "Stored on your PC",
        body: "Your history lives in a local SQLite database in your user profile. No account, no cloud, no sync. Clear it any time; pinned items stay.",
      },
      {
        icon: "update",
        title: "Keeps itself up to date",
        body: "Wincy checks GitHub Releases for new versions and updates itself. Every update is signed and verified before it's installed, and you can turn checks off in Settings.",
      },
    ],
  },
  shortcuts: {
    title: "Keyboard shortcuts",
    lead: "Wincy is made to be driven from the keyboard. Here is every shortcut.",
    keysHeader: "Keys",
    actionHeader: "Action",
    typeLabel: "Just type",
    rows: [
      { keys: ["Alt+V"], action: "Open Wincy from any app (configurable)" },
      { keys: [], action: "Fuzzy-filter the list" },
      { keys: ["↑", "↓", "PageUp", "PageDown", "Home", "End"], action: "Move the selection" },
      { keys: ["Enter"], action: "Paste" },
      { keys: ["Shift+Enter"], action: "Paste as plain text" },
      { keys: ["Alt+1–9"], action: "Paste the Nth visible item (pins come first)" },
      { keys: ["Ctrl+P"], action: "Pin or unpin" },
      { keys: ["Ctrl+Delete"], action: "Delete the item" },
      { keys: ["Tab", "Shift+Tab"], action: "Cycle the type filter: all, text, links, colors, code, images, files" },
      { keys: ["Ctrl+Space"], action: "Show or hide the preview panel" },
      { keys: ["Ctrl+,"], action: "Open settings" },
      { keys: ["Esc"], action: "Clear the search, or close Wincy if it's empty" },
    ],
  },
  privacy: {
    title: "Your clipboard never leaves your PC.",
    body: "Wincy has no telemetry, no analytics and no account. Everything you copy stays in a database on your own disk. The only network request it ever makes is the update check against GitHub Releases, and you can switch that off.",
    points: [
      "No telemetry or analytics",
      "No account, no cloud sync",
      "Password managers are ignored",
      "Pause capture from the tray at any time",
      "Open source: read every line",
    ],
  },
  compare: {
    title: "Why not just Win+V?",
    lead: "The clipboard history built into Windows is fine for the last thing or two you copied. Wincy is for everything else. They don't clash, so keep Win+V if you like it.",
    caption: "Wincy compared with Windows clipboard history",
    featureHeader: "Feature",
    rows: [
      ["History size", "1,000 items by default, up to 10,000", "25 items"],
      ["After a restart", "Everything is still there", "Cleared, except pinned items"],
      ["Search", "Fuzzy search as you type", "None"],
      ["Quick paste", "Alt+1 … Alt+9", "None"],
      ["Type filters", "Text, links, colors, code, images, files", "None"],
      ["Cloud sync", "Never: local only", "Optional, with a Microsoft account"],
    ],
  },
  install: {
    title: "Up and running in a minute",
    steps: [
      {
        title: "Download the installer",
        body: "Grab the file ending in -setup.exe from the latest release on GitHub.",
        link: "Open the latest release",
      },
      {
        title: "Get past SmartScreen",
        body: "Wincy isn't code-signed yet (certificates cost money an open-source project doesn't have), so Windows may say “Windows protected your PC”. Click:",
        moreInfo: "More info",
        runAnyway: "Run anyway",
      },
      {
        title: "Install for your account",
        body: "No admin rights needed. Wincy installs for your user only and starts with Windows. You can turn that off in Settings.",
      },
      {
        title: "Press Alt+V",
        body: "Copy a few things, then press Alt+V. Wincy lives in the system tray; right-click its icon for more options.",
      },
    ],
    note: "winget and a portable .zip aren't available yet.",
  },
  faq: {
    title: "Questions",
    items: [
      {
        q: "Is Wincy really free?",
        a: "Yes. It's open source under the MIT License, with no paid tier, no ads and no tracking.",
      },
      {
        q: "Where is my history stored?",
        a: "In a SQLite database inside your Windows user profile (the app's data folder). Settings has a button that opens it in File Explorer. Copied images are saved there as PNG files.",
      },
      {
        q: "Will it save my passwords?",
        a: "Not from password managers that mark what they copy as sensitive, and not from apps on the ignore list: Bitwarden, 1Password, KeePass, KeePassXC, Enpass, Dashlane, LastPass, NordPass and Proton Pass by default. You can add any other app in Settings.",
      },
      {
        q: "Can I change the Alt+V shortcut?",
        a: "Yes, in Settings. You can also choose how many items to keep, stop capturing images, or make Enter copy instead of paste.",
      },
      {
        q: "Does it work on Windows 10?",
        a: "Yes. Windows 11 gets the Mica material and Windows 10 gets Acrylic. Both get light and dark themes.",
      },
      {
        q: "Why does Windows warn me when I install it?",
        a: "Because the installer isn't code-signed, and SmartScreen flags unsigned apps until they build up a reputation. The source code and the release pipeline are public on GitHub, so you can check exactly what you're running.",
      },
      {
        q: "How do I uninstall it?",
        a: "Like any other app: open Settings → Apps, find Wincy and choose Uninstall.",
      },
    ],
  },
  footer: {
    license: "Free and open source under the MIT License.",
    source: "Source code",
    releases: "Releases",
    issues: "Report a bug",
  },
};

export default en;
export type Dict = typeof en;
