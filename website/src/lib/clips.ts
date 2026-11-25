// Pure helpers shared by the server render and the browser script of the palette demo.
// Formats mirror the app (src/components/ClipItem.tsx, PreviewWindow.tsx, src/lib/color.ts).
import type { DemoClip, Dict } from "../i18n/en";

type Demo = Dict["demo"];

export const fill = (s: string, vars: Record<string, string | number>) =>
  s.replace(/\{(\w+)\}/g, (m, k: string) => String(vars[k] ?? m));

const fileName = (path: string) => path.split(/[\\/]/).filter(Boolean).pop() ?? path;

/** The row's one-line label, as ClipItem renders it. */
export function rowLabel(clip: DemoClip, d: Demo): string {
  if (clip.kind === "image" && clip.size) return `${d.image} ${clip.size[0]}×${clip.size[1]}`;
  if (clip.kind === "files" && clip.files?.length) {
    const more = clip.files.length - 1;
    return more > 0 ? `${fileName(clip.files[0])} ${fill(d.moreFiles, { n: more })}` : fileName(clip.files[0]);
  }
  return clip.text.replace(/\s+/g, " ").trim();
}

const UNITS: [Intl.RelativeTimeFormatUnit, number][] = [
  ["day", 1440],
  ["hour", 60],
  ["minute", 1],
];

/** "2m ago" / "hace 2 min", like the app's relativeTime (narrow, numeric auto). */
export function relTime(minutesAgo: number, lang: string): string {
  const [unit, size] = UNITS.find(([, s]) => minutesAgo >= s) ?? UNITS[UNITS.length - 1];
  return new Intl.RelativeTimeFormat(lang, { numeric: "auto", style: "narrow" }).format(-Math.round(minutesAgo / size), unit);
}

export const rowMeta = (clip: DemoClip, lang: string) => `${clip.app.replace(/\.exe$/i, "")} · ${relTime(clip.ago, lang)}`;

export const dateAgo = (minutesAgo: number, lang: string, now = Date.now()) =>
  new Intl.DateTimeFormat(lang, { dateStyle: "medium", timeStyle: "short" }).format(now - minutesAgo * 60_000);

// Colors: the demo only stores #RRGGBB and rgb(r, g, b), so that is all this parses.
export function parseColor(text: string): [number, number, number] | null {
  const hex = /^#([0-9a-f]{6})$/i.exec(text.trim());
  if (hex) return [0, 2, 4].map((i) => parseInt(hex[1].slice(i, i + 2), 16)) as [number, number, number];
  const rgb = /^rgb\(\s*(\d{1,3}),\s*(\d{1,3}),\s*(\d{1,3})\s*\)$/i.exec(text.trim());
  return rgb ? [Number(rgb[1]), Number(rgb[2]), Number(rgb[3])] : null;
}

export function colorValues([r, g, b]: [number, number, number]): [string, string][] {
  const hex = "#" + [r, g, b].map((n) => n.toString(16).padStart(2, "0")).join("").toUpperCase();
  const [R, G, B] = [r / 255, g / 255, b / 255];
  const max = Math.max(R, G, B);
  const min = Math.min(R, G, B);
  const d = max - min;
  const l = (max + min) / 2;
  const s = d === 0 ? 0 : d / (1 - Math.abs(2 * l - 1));
  const h = d === 0 ? 0 : max === R ? ((G - B) / d + 6) % 6 : max === G ? (B - R) / d + 2 : (R - G) / d + 4;
  return [
    ["HEX", hex],
    ["RGB", `rgb(${r}, ${g}, ${b})`],
    ["HSL", `hsl(${Math.round(h * 60) % 360}, ${Math.round(s * 100)}%, ${Math.round(l * 100)}%)`],
  ];
}

// Search: approximate substring match (Sellers), like Fuse's bitap with ignoreLocation.
export const fold = (s: string) => s.normalize("NFD").replace(/\p{Diacritic}/gu, "").toLowerCase();

/** Fewest edits that turn `p` into some substring of `t`, and where that substring starts. */
function nearest(p: string, t: string): { errors: number; start: number } {
  const m = p.length;
  let prev = Array.from({ length: m + 1 }, (_, i) => i);
  let starts = new Array<number>(m + 1).fill(0);
  let best = { errors: m, start: 0 };
  for (let j = 0; j < t.length; j++) {
    const cur = [0];
    const curStart = [j + 1];
    for (let i = 1; i <= m; i++) {
      const sub = prev[i - 1] + (p[i - 1] === t[j] ? 0 : 1);
      const del = prev[i] + 1;
      const ins = cur[i - 1] + 1;
      cur[i] = Math.min(sub, del, ins);
      curStart[i] = cur[i] === sub ? (i === 1 ? j : starts[i - 1]) : cur[i] === del ? starts[i] : curStart[i - 1];
    }
    if (cur[m] < best.errors) best = { errors: cur[m], start: curStart[m] };
    prev = cur;
    starts = curStart;
  }
  return best;
}

// shortcut: a fixed error budget per query length stands in for Fuse's 0.35 threshold; it is stricter on 1-3 letters so the demo stays precise.
const budget = (len: number) => (len <= 3 ? 0 : len <= 6 ? 1 : 2);

/** Lower is better; null when the clip does not match. Fields are ordered by weight: content, app, kind. */
export function score(query: string, fields: string[]): number | null {
  const words = fold(query).split(/\s+/).filter(Boolean);
  if (!words.length) return 0;
  let total = 0;
  for (const w of words) {
    let best: number | null = null;
    fields.forEach((f, fi) => {
      const { errors, start } = nearest(w, f);
      if (errors > budget(w.length)) return;
      const wordStart = start === 0 || !/[\p{L}\p{N}]/u.test(f[start - 1]);
      const s = errors * 10 + fi * 3 + (wordStart ? 0 : 1);
      if (best === null || s < best) best = s;
    });
    if (best === null) return null;
    total += best;
  }
  return total;
}

/** Splits `app:<term>` tokens (OR, without ".exe") from the rest of the query, like the app's src/lib/query.ts. */
export function splitQuery(query: string): { apps: string[]; rest: string } {
  const words = query.trim().split(/\s+/).filter(Boolean);
  const isApp = (w: string) => /^app:/i.test(w);
  const apps = words.filter(isApp).map((w) => fold(w.slice(4)).replace(/\.exe$/, "")).filter(Boolean);
  return { apps, rest: words.filter((w) => !isApp(w)).join(" ") };
}

export const matchesApp = (clip: DemoClip, apps: string[]) =>
  !apps.length || apps.some((a) => fold(clip.app).replace(/\.exe$/, "").includes(a));

/** What the search looks at for a clip: its content, its source app, its kind (enum and label). */
export const haystack = (clip: DemoClip, d: Demo) => [
  fold(clip.kind === "files" ? (clip.files ?? []).join(" ") : `${rowLabel(clip, d)} ${clip.ocr ?? ""}`),
  fold(clip.app),
  fold(`${clip.kind} ${d.kinds[clip.kind]}`),
];
