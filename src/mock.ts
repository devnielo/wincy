// In-memory stand-in for the Rust backend so `pnpm dev` works in a normal browser.
import type { EventMap } from "./api";
import type { Clip, ClipKind, Settings, Status } from "./types";

const MIN = 60_000;
const now = Date.now();

/** A gradient; with `lines`, a fake screenshot of a note showing them (its OCR text). */
function svgImage(from: string, to: string, w: number, h: number, lines: string[] = []): string {
  const note = lines.length
    ? `<rect x="8%" y="12%" width="84%" height="76%" rx="${w / 60}" fill="#fff"/>` +
      lines.map((l, i) => `<text x="13%" y="${26 + i * 15}%" font-family="Segoe UI, sans-serif" font-size="${h / 14}" font-weight="${i ? 400 : 600}" fill="#1f2937">${l}</text>`).join("")
    : "";
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="${w}" height="${h}" viewBox="0 0 ${w} ${h}"><defs><linearGradient id="g" x2="1" y2="1"><stop stop-color="${from}"/><stop offset="1" stop-color="${to}"/></linearGradient></defs><rect width="100%" height="100%" fill="url(#g)"/>${note}</svg>`;
  return `data:image/svg+xml,${encodeURIComponent(svg)}`;
}

type ImageSeed = [from: string, to: string, w: number, h: number, ocrLines?: string[]];
type Seed = [kind: ClipKind, content: string | string[] | ImageSeed, app: string, minutesAgo: number];

const seeds: Seed[] = [
  ["text", "Meeting moved to Thursday 3pm — same room.", "teams.exe", 1],
  ["url", "https://github.com/devnielo/wincy", "chrome.exe", 2],
  ["color", "#1E90FF", "figma.exe", 4],
  ["code", "export function debounce<T extends (...a: never[]) => void>(fn: T, ms: number) {\n  let t: number | undefined;\n  return (...args: Parameters<T>) => {\n    clearTimeout(t);\n    t = window.setTimeout(() => fn(...args), ms);\n  };\n}", "code.exe", 6],
  ["image", ["#7c3aed", "#06b6d4", 1920, 1080, ["Sprint review: action items", "1. Ship OCR search in v0.2.0", "2. Check accent contrast (AA)", "3. Update the website copy"]], "snippingtool.exe", 9],
  ["files", ["C:\\Users\\me\\Documents\\Q3 report.xlsx", "C:\\Users\\me\\Documents\\notes.md", "C:\\Users\\me\\Pictures\\diagram.png"], "explorer.exe", 12],
  ["text", "Hola Marta, te paso el enlace del documento en cuanto lo tenga listo.", "outlook.exe", 15],
  ["color", "rgb(255, 99, 71)", "chrome.exe", 18],
  ["code", "SELECT id, email, created_at\nFROM users\nWHERE created_at > now() - interval '7 days'\nORDER BY created_at DESC;", "datagrip64.exe", 22],
  ["url", "https://learn.microsoft.com/windows/win32/dataxchg/clipboard", "msedge.exe", 30],
  ["text", "  tracking number:\n\n   1Z 999 AA1 01 2345 6784  ", "chrome.exe", 41],
  ["color", "hsl(160, 84%, 39%)", "code.exe", 55],
  ["code", "fn main() {\n    let v: Vec<u32> = (1..=10).filter(|n| n % 2 == 0).collect();\n    println!(\"{v:?}\");\n}", "code.exe", 70],
  ["image", ["#f97316", "#facc15", 800, 600], "chrome.exe", 95],
  ["text", "The quick brown fox jumps over the lazy dog.", "notepad.exe", 120],
  ["url", "mailto:hello@example.com", "thunderbird.exe", 160],
  ["files", ["D:\\Projects\\wincy\\release\\wincy_0.1.0_x64-setup.exe"], "explorer.exe", 200],
  ["color", "#0f172a80", "figma.exe", 260],
  ["text", "Address: 742 Evergreen Terrace, Springfield", "chrome.exe", 330],
  ["code", "def fib(n):\n    a, b = 0, 1\n    for _ in range(n):\n        a, b = b, a + b\n    return a", "pycharm64.exe", 400],
  ["url", "www.example.org/docs/getting-started", "firefox.exe", 520],
  ["text", "Reminder: renew the domain before Dec 1st.", "notion.exe", 700],
  ["image", ["#22c55e", "#3b82f6", 320, 320], "discord.exe", 900],
  ["code", "<div className=\"card\">\n  <h2>{title}</h2>\n  <p>{children}</p>\n</div>", "code.exe", 1200],
  ["color", "rgba(16, 185, 129, 0.6)", "chrome.exe", 1600],
  ["text", "git rebase -i HEAD~3", "windowsterminal.exe", 2100],
  ["files", ["C:\\Users\\me\\Downloads\\invoice-0042.pdf", "C:\\Users\\me\\Downloads\\invoice-0043.pdf"], "explorer.exe", 2900],
  ["url", "https://tauri.app/plugin/updater/", "chrome.exe", 4000],
  ["text", "Thanks! Talk soon.", "whatsapp.exe", 6000],
  ["code", "const total = items.reduce((sum, i) => sum + i.price * i.qty, 0);\nconsole.log(total.toFixed(2));", "code.exe", 9000],
];

let nextId = 1;
let clips: Clip[] = seeds.map(([kind, content, app, ago], i) => {
  const t = now - ago * MIN;
  const base = {
    id: nextId++,
    kind,
    hasHtml: kind === "text" && i % 3 === 0,
    thumbPath: null,
    imagePath: null,
    imageSize: null,
    files: null,
    sourceApp: app,
    pinned: i === 2 || i === 18,
    createdAt: t - 3 * ago * MIN,
    updatedAt: t,
    useCount: (i * 7) % 5,
    html: null,
    ocrText: null,
  } satisfies Partial<Clip>;
  if (kind === "image") {
    const [from, to, w, h, lines] = content as ImageSeed;
    return {
      ...base,
      preview: "",
      text: null,
      thumbPath: svgImage(from, to, 64, 64),
      imagePath: svgImage(from, to, w, h, lines),
      imageSize: [w, h],
      ocrText: lines ? lines.join("\n") : "",
    };
  }
  if (kind === "files") {
    const files = content as string[];
    return { ...base, preview: files.join("\n"), text: null, files };
  }
  const text = content as string;
  return { ...base, preview: text.slice(0, 500), text };
});

let settings: Settings = {
  shortcut: "Alt+V",
  maxItems: 1000,
  launchAtStartup: true,
  pasteDirectly: true,
  captureImages: true,
  ignoredApps: ["bitwarden.exe", "1password.exe", "keepass.exe", "keepassxc.exe", "enpass.exe", "dashlane.exe", "lastpass.exe", "nordpass.exe", "proton pass.exe"],
  theme: "system",
  language: "system",
  checkUpdates: true,
  accent: "windows",
  ignoreSecrets: true,
  expireDays: 0,
  ocrImages: true,
};
let paused = false;

// `?shortcut-error` in the dev URL starts as if Alt+V were taken by another app.
let status: Status = {
  shortcutError: new URLSearchParams(location.search).has("shortcut-error") ? "HotKey already registered" : null,
  systemAccent: "#C239B3",
  logDir: "C:\\Users\\me\\AppData\\Local\\dev.wincy.app\\logs",
};

function setStatus(patch: Partial<Status>) {
  status = { ...status, ...patch };
  emit("status-changed", { ...status });
}

const listeners = new Map<string, Set<(payload: never) => void>>();

function emit<K extends keyof EventMap>(name: K, payload: EventMap[K]) {
  listeners.get(name)?.forEach((h) => (h as (p: EventMap[K]) => void)(payload));
}

export function mockListen<K extends keyof EventMap>(name: K, handler: (payload: EventMap[K]) => void): () => void {
  const set = listeners.get(name) ?? new Set();
  listeners.set(name, set);
  set.add(handler);
  return () => set.delete(handler);
}

function find(id: unknown): Clip {
  const clip = clips.find((c) => c.id === id);
  if (!clip) throw `clip ${String(id)} not found`;
  return clip;
}

function ordered(): Clip[] {
  return [...clips].sort((a, b) => Number(b.pinned) - Number(a.pinned) || b.updatedAt - a.updatedAt);
}

function changed() {
  queueMicrotask(() => emit("clips-changed", null));
}

export async function mockInvoke<T>(cmd: string, args: Record<string, unknown> = {}): Promise<T> {
  const result = ((): unknown => {
    switch (cmd) {
      case "get_clips":
        return ordered().map(({ text: _t, html: _h, ...summary }) => summary);
      case "get_clip":
        return { ...find(args.id) };
      case "paste_clip": {
        const clip = find(args.id);
        clip.updatedAt = Date.now();
        clip.useCount++;
        console.info(`[mock] paste #${clip.id} (plain: ${String(args.plain)})`);
        changed();
        return null;
      }
      case "toggle_pin": {
        const clip = find(args.id);
        clip.pinned = !clip.pinned;
        changed();
        return clip.pinned;
      }
      case "delete_clip":
        clips = clips.filter((c) => c.id !== args.id);
        changed();
        return null;
      case "clear_history":
        clips = clips.filter((c) => c.pinned);
        changed();
        return null;
      case "get_settings":
        return { ...settings };
      case "save_settings": {
        const next = args.settings as Settings;
        if (/(^|\+)Super\+L$/.test(next.shortcut)) {
          setStatus({ shortcutError: "HotKey already registered" });
          throw `Shortcut "${next.shortcut}" is reserved by Windows`;
        }
        if (status.shortcutError) setStatus({ shortcutError: null });
        settings = { ...next, ignoredApps: [...next.ignoredApps] };
        emit("settings-changed", { ...settings });
        return { ...settings };
      }
      case "get_paused":
        return paused;
      case "set_paused":
        paused = Boolean(args.paused);
        emit("settings-changed", { ...settings });
        return paused;
      case "set_preview":
        emit("preview-clip", { id: (args.id as number | null) ?? null });
        return null;
      case "get_status":
        return { ...status };
      case "copy_diagnostics":
        return `Wincy dev (browser mock)\nclips: ${clips.length}`;
      case "hide_window":
      case "open_data_dir":
      case "open_log_dir":
        console.info(`[mock] ${cmd}`);
        return null;
      default:
        throw `unknown command ${cmd}`;
    }
  })();
  return result as T;
}
