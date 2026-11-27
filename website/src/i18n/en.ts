export type ClipKind = "text" | "url" | "color" | "code" | "image" | "files";
export type KindFilter = "all" | ClipKind;

/** One demo clip. `ago` and `first` are minutes before page load; text mirrors what the app stores. */
export interface DemoClip {
  kind: ClipKind;
  text: string;
  app: string;
  ago: number;
  first: number;
  uses: number;
  pinned?: boolean;
  rich?: boolean;
  size?: [number, number];
  files?: string[];
  ocr?: string; // text "recognized" in the image (shown with demo-invoice.svg)
}

const en = {
  htmlLang: "en",
  ogLocale: "en_US",
  languageName: "English",
  meta: {
    title: "Wincy · Keyboard-first clipboard manager for Windows",
    description:
      "Wincy keeps what you copy and brings it back with Alt+V, right next to your pointer. Fuzzy search, pins and plain-text paste. Free and open source for Windows.",
    logoAlt: "Wincy logo",
    ogAlt: "An illustrated clipboard with a brass clip beside the Wincy wordmark and the words Clipboard history for Windows, one shortcut away.",
  },
  nav: {
    skip: "Skip to content",
    label: "Main",
    menu: "Menu",
    closeMenu: "Close menu",
    features: "Features",
    shortcuts: "Shortcuts",
    privacy: "Privacy",
    install: "Install",
    faq: "FAQ",
    language: "Language",
    download: "Download",
  },
  hero: {
    title: "Clipboard history for Windows, one shortcut away.",
    lead: "Wincy keeps a history of what you copy. Press Alt+V and it opens next to your mouse pointer: type a few letters, press Enter, and it's pasted where you were working.",
    download: "Download for Windows",
    github: "View on GitHub",
    facts: ["Windows 10 and 11", "Free and open source (MIT)", "No admin rights needed"],
    tryIt: "Try it: type to filter, Tab changes the type, Enter pastes.",
    tryItTouch: "Try it: tap a clip to paste it.",
    phoneNote: "Wincy is a Windows desktop app. Open this page on your PC to download it.",
  },
  demo: {
    label: "Interactive Wincy demo",
    search: "Search clipboard history…",
    searchLabel: "Search clipboard history",
    searchHint:
      "Type to filter. Up and down arrows move, Enter pastes, Tab changes the type filter, and Esc clears the search, then closes the demo.",
    listLabel: "Clipboard history",
    previewLabel: "Preview",
    filterBy: "Filter: {kind}",
    kinds: {
      all: "All",
      text: "Text",
      url: "Links",
      color: "Colors",
      code: "Code",
      image: "Images",
      files: "Files",
    },
    pinned: "Pinned",
    rich: "Rich text",
    image: "Image",
    moreFiles: "+{n} more",
    empty: "No results",
    emptyHistory: "No clips yet — copy something",
    pasted: "Pasted",
    pastedPlain: "Pasted as plain text",
    closed: "Wincy is closed, just as after Esc in the app.",
    reopen: "Open Wincy",
    settingsNote: "In the app, this opens Settings.",
    actions: {
      paste: "Paste",
      plain: "Plain text",
      plainLong: "Paste as plain text",
      pin: "Pin",
      unpin: "Unpin",
      delete: "Delete",
      pause: "Pause capture",
      resume: "Resume capture",
      paused: "Paused",
      previewShow: "Show preview",
      previewHide: "Hide preview",
      settings: "Settings",
    },
    preview: {
      source: "Source",
      created: "First copied",
      updated: "Last used",
      uses: "Pastes",
      chars: "Characters",
      lines: "Lines",
      size: "Size",
      files: "{n} files",
      recognized: "Recognized text",
    },
    items: [
      { kind: "text", text: "Thanks! I'll send the signed contract over by Friday.", app: "outlook.exe", ago: 3, first: 4320, uses: 6, pinned: true },
      { kind: "code", text: "git log --oneline --graph --decorate -20", app: "WindowsTerminal.exe", ago: 18, first: 20160, uses: 14, pinned: true },
      { kind: "color", text: "#E8590C", app: "figma.exe", ago: 1, first: 1, uses: 0 },
      { kind: "url", text: "https://learn.microsoft.com/windows/win32/dataxchg/clipboard", app: "msedge.exe", ago: 7, first: 7, uses: 1 },
      { kind: "image", text: "", app: "SnippingTool.exe", ago: 12, first: 12, uses: 0, size: [1280, 720] },
      { kind: "image", text: "", app: "ShareX.exe", ago: 95, first: 95, uses: 1, size: [640, 360], ocr: "Northwind Studio\nFactura / Invoice #2026-114\n20 Nov 2026 – 20 Dec 2026\nTotal\n1.240,00 €" },
      {
        kind: "code",
        text: "export function debounce(fn, ms) {\n  let t;\n  return (...args) => {\n    clearTimeout(t);\n    t = setTimeout(() => fn(...args), ms);\n  };\n}",
        app: "Code.exe",
        ago: 25,
        first: 2900,
        uses: 3,
      },
      {
        kind: "files",
        text: "",
        app: "explorer.exe",
        ago: 40,
        first: 40,
        uses: 1,
        files: ["C:\\Users\\me\\Documents\\Q3-report.xlsx", "C:\\Users\\me\\Documents\\invoice-0423.pdf"],
      },
      { kind: "text", text: "Meeting moved to 3:30 pm, room B2", app: "ms-teams.exe", ago: 55, first: 55, uses: 0 },
      { kind: "color", text: "rgb(46, 160, 67)", app: "chrome.exe", ago: 90, first: 90, uses: 2 },
      { kind: "url", text: "https://tauri.app/plugin/updater/", app: "chrome.exe", ago: 130, first: 1440, uses: 1 },
      { kind: "text", text: "Order #48213 has shipped", app: "chrome.exe", ago: 200, first: 200, uses: 0, rich: true },
    ] satisfies DemoClip[] as DemoClip[],
  },
  features: {
    title: "Small app, sharp tools",
    lead: "Wincy is built around one moment: you need something you copied earlier, and you need it now.",
    items: [
      {
        fragment: "detect",
        keys: ["Tab"],
        title: "Knows what you copied",
        body: "Links, colors, code, images and files are recognized and shown as they should look, and the text in your screenshots is searchable too. Tab filters by type.",
      },
      {
        fragment: "pins",
        keys: ["Alt+1–9"],
        title: "Keeps your pins on top",
        body: "Ctrl+P pins a clip. Pins are never trimmed, stay at the top of the list, and Alt+1 to Alt+9 pastes the first nine clips without looking. Give a pin its own shortcut and it pastes from any app, without opening Wincy.",
      },
      {
        fragment: "plain",
        keys: ["Shift+Enter"],
        title: "Pastes plain text on demand",
        body: "Text from a web page arrives without its fonts, colors and links.",
      },
      {
        fragment: "preview",
        keys: ["Ctrl+Space"],
        title: "Shows the whole clip",
        body: "The selected clip opens in full beside the palette: code with its line breaks, long text, images, and colors with their HEX, RGB and HSL values.",
      },
    ],
  },
  shortcuts: {
    title: "Keyboard shortcuts",
    lead: "Wincy is made to be driven from the keyboard. Every shortcut also has a button, and you can drag the palette by its top bar. Pinned clips and snippets can have shortcuts of their own that work in any app.",
    typeLabel: "Just type",
    rows: [
      { keys: ["Alt+V"], action: "Open Wincy from any app (you can change it)" },
      { keys: [], action: "Filter the list with fuzzy search; add app:chrome to see only what came from Chrome" },
      { keys: ["↑", "↓", "PageUp", "PageDown", "Home", "End"], action: "Move the selection" },
      { keys: ["Enter"], action: "Paste" },
      { keys: ["Shift+Enter"], action: "Paste as plain text" },
      { keys: ["Alt+1–9"], action: "Paste one of the first nine clips (pins come first)" },
      { keys: ["Ctrl+K"], action: "Quick actions: change case, format JSON, decode Base64 or a URL, open a link, save as a snippet" },
      { keys: ["Ctrl+P"], action: "Pin or unpin" },
      { keys: ["Ctrl+Delete"], action: "Delete the clip" },
      { keys: ["Tab", "Shift+Tab"], action: "Switch the type filter: all, text, links, colors, code, images, files, snippets" },
      { keys: ["Ctrl+Space"], action: "Show or hide the preview" },
      { keys: ["Ctrl+,"], action: "Open Settings" },
      { keys: ["Esc"], action: "Clear the search; if it's already empty, close Wincy" },
    ],
  },
  privacy: {
    title: "Your clipboard never leaves your PC.",
    body: "Everything you copy stays in a database on your own disk.",
    facts: [
      ["Telemetry", "None, and no analytics"],
      ["Account", "None, and no cloud sync"],
      ["Network", "Only the update check, which you can turn off"],
      ["Passwords and keys", "Skipped: password managers, anything marked sensitive, and text that looks like an API key or card number"],
      ["Capture", "Pause it with one click, and let old clips expire after a few days if you like"],
      ["Source code", "Open, under the MIT License"],
    ],
    settings: {
      title: "Settings",
      back: "Back",
      privacy: "Privacy",
      ignoredApps: "Ignored apps",
      ignoredAppsHint: "Copies from these programs are never saved.",
      appPlaceholder: "e.g. bitwarden.exe",
      add: "Add",
      remove: "Remove {app}",
      updates: "Updates",
      checkUpdates: "Check for updates automatically",
    },
  },
  compare: {
    title: "Why not just Win+V?",
    lead: "Windows' built-in clipboard history is fine for the last few things you copied. Wincy is for everything else. The two don't clash, so keep Win+V if you like it.",
    caption: "Wincy compared with Windows clipboard history",
    featureHeader: "Feature",
    windows: "Windows (Win+V)",
    rows: [
      ["History size", "1,000 clips by default, up to 10,000", "25 items"],
      ["After a restart", "Everything is still there", "Cleared, except pinned items"],
      ["Fuzzy search", "Yes, as you type", "No"],
      ["Text in images", "Searchable, with the OCR built into Windows", "No"],
      ["Quick paste", "Alt+1 … Alt+9", "No"],
      ["Type filters", "Text, links, colors, code, images, files", "No"],
      ["Snippets", "Yes, with date, time and clipboard variables", "No"],
      ["Quick actions", "Change case, format JSON, Base64 and URL encode or decode", "No"],
      ["Export and import", "To a file on your PC", "No"],
      ["Cloud sync", "No, everything stays on your PC", "Optional, with a Microsoft account"],
    ],
  },
  install: {
    title: "Up and running in a minute",
    steps: [
      {
        fragment: "download",
        title: "Download the installer",
        body: "Download the installer for the latest version. It's a single .exe file.",
      },
      {
        fragment: "smartscreen",
        title: "If SmartScreen appears",
        body: "Wincy doesn't have a paid code-signing certificate yet, so Windows may show “Windows protected your PC”. Click More info, then Run anyway.",
      },
      {
        fragment: "startup",
        title: "Install for your account",
        body: "No admin rights needed. Wincy installs for your user only and starts with Windows; you can change that in Settings.",
      },
      {
        fragment: "tray",
        title: "Press Alt+V",
        body: "Copy a few things, then press Alt+V. Wincy lives in the system tray: click its icon to open the palette, or right-click it for more options.",
      },
    ],
    link: "Download the installer",
    note: [
      "Not on winget yet. Rather not install? Download the ",
      "portable .zip",
      ", extract it and run wincy.exe. It keeps its data in its own folder and doesn't update itself.",
    ],
    allReleases: "All versions on GitHub",
    smartscreen: {
      title: "Windows protected your PC",
      moreInfo: "More info",
      app: "App:",
      publisher: "Publisher:",
      unknown: "Unknown publisher",
      runAnyway: "Run anyway",
      dontRun: "Don't run",
    },
    startup: "Launch at startup",
    tray: ["Open Wincy", "Pause capture", "Settings…", "Clear history", "Quit"],
  },
  faq: {
    title: "Questions and answers",
    items: [
      {
        q: "Is Wincy really free?",
        a: "Yes. It's open source under the MIT License. No paid tier, no ads, no tracking.",
      },
      {
        q: "Where is my history stored?",
        a: "In a SQLite database in your Windows user profile, inside Wincy's data folder. Copied images are saved there as PNG files. Settings has a button that opens the folder in File Explorer. The portable version keeps everything in a data folder next to wincy.exe instead.",
      },
      {
        q: "Can I back up my history or move it to another PC?",
        a: "Yes. Settings → Data → Export saves your history and snippets, with or without images, to a .wincy.json file; Import on the other PC adds it to what's already there without duplicates. Settings stay on each PC. The file holds what you copied as plain text, so keep it somewhere safe.",
      },
      {
        q: "Will it save my passwords?",
        a: "Not from password managers. Wincy skips anything an app marks as sensitive (standard Windows clipboard flags such as ExcludeClipboardContentFromMonitorProcessing) and anything copied from the apps on its ignore list: Bitwarden, 1Password, KeePass, KeePassXC, Enpass, Dashlane, LastPass, NordPass and Proton Pass by default. You can add other apps in Settings. It also skips text that looks like a secret, such as a private key, an API token or a card number.",
      },
      {
        q: "Can I change the Alt+V shortcut?",
        a: "Yes, in Settings. If another app already uses your shortcut, Wincy tells you and opens Settings so you can pick another. You can also choose how many clips to keep (50 to 10,000), delete old clips after a number of days, stop capturing images, or make Enter copy without pasting.",
      },
      {
        q: "Does it work on Windows 10?",
        a: "Yes. Wincy is built with Rust and Tauri on the WebView2 that ships with Windows, and it looks at home on both: Mica on Windows 11, Acrylic on Windows 10, with light and dark themes and the accent color of your choice.",
      },
      {
        q: "How do updates work?",
        a: "Wincy checks GitHub Releases when it starts and tells you when a new version is out. You install it with one click in Settings; updates are signed and verified before they install. You can turn the check off. The portable version can't replace itself: it opens the download page, and you swap in the new wincy.exe.",
      },
      {
        q: "Why does Windows warn me when I install it?",
        a: "The installer isn't signed with a code-signing certificate, and SmartScreen warns about unsigned apps until they build a reputation. The source code and the release pipeline are public on GitHub, so you can check exactly what you're running.",
      },
      {
        q: "How do I report a bug?",
        a: "Open an issue on GitHub and paste what Settings → Copy diagnostics gives you: versions, settings and recent log lines, never what you copied.",
      },
      {
        q: "How do I uninstall it?",
        a: "Like any other app: in Windows Settings, go to Apps, find Wincy and choose Uninstall.",
      },
    ],
  },
  cta: {
    title: "Give your clipboard a memory.",
    body: "Install it, copy a few things, then press Alt+V.",
    smartscreen: "If SmartScreen appears:",
  },
  footer: {
    license: "Free and open source under the MIT License.",
    source: "Source code",
    releases: "Releases",
    issues: "Report a bug",
    support: "Buy me a coffee",
    signing: "Code signing policy",
  },
  appearance: {
    label: "Appearance",
    mode: "Theme",
    system: "System",
    light: "Light",
    dark: "Dark",
    accent: "Accent color",
    accents: {
      windows: "Windows",
      blue: "Blue",
      violet: "Violet",
      teal: "Teal",
      green: "Green",
      orange: "Orange",
      rose: "Rose",
      graphite: "Graphite",
    },
  },
  // Text parts; a [label, url] pair is a link.
  signing: {
    title: "Code signing policy",
    rows: [
      { term: "Builds", parts: ["Every release is built by GitHub Actions from the ", ["public repository", "https://github.com/devnielo/wincy"], ", so you can check what you run against its source."] },
      {
        term: "Signing",
        parts: [
          "Once approved, Windows releases will be signed: free code signing provided by ",
          ["SignPath.io", "https://about.signpath.io"],
          ", certificate by ",
          ["SignPath Foundation", "https://signpath.org"],
          ". Until then, releases are unsigned.",
        ],
      },
      {
        term: "Roles",
        parts: ["Author, reviewer and approver: the maintainer, ", ["@devnielo", "https://github.com/devnielo"], " on GitHub. Multi-factor authentication is required."],
      },
      { term: "Privacy", parts: ["Wincy sends no data anywhere except the update check against GitHub Releases, which you can turn off."] },
    ] as { term: string; parts: (string | [string, string])[] }[],
  },
};

export default en;
export type Dict = typeof en;
