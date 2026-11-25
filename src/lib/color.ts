export interface Rgba {
  r: number; // 0..255
  g: number;
  b: number;
  a: number; // 0..1
}

const clamp = (n: number, min: number, max: number) => Math.min(max, Math.max(min, n));

/** Number with optional trailing %, or null. `scale` is what 100% maps to. */
function num(token: string | undefined, scale: number, allowBare = true): number | null {
  if (!token) return null;
  const pct = token.endsWith("%");
  if (!pct && !allowBare) return null;
  const body = pct ? token.slice(0, -1) : token;
  if (!/^[+-]?(\d+\.?\d*|\.\d+)$/.test(body)) return null;
  return pct ? (Number(body) / 100) * scale : Number(body);
}

function hue(token: string): number | null {
  const m = /^([+-]?(?:\d+\.?\d*|\.\d+))(deg|turn|rad)?$/.exec(token);
  if (!m) return null;
  const n = Number(m[1]);
  const deg = m[2] === "turn" ? n * 360 : m[2] === "rad" ? (n * 180) / Math.PI : n;
  return ((deg % 360) + 360) % 360;
}

function hslToRgb(h: number, s: number, l: number): [number, number, number] {
  const a = s * Math.min(l, 1 - l);
  const f = (n: number) => {
    const k = (n + h / 30) % 12;
    return (l - a * Math.max(-1, Math.min(k - 3, 9 - k, 1))) * 255;
  };
  return [f(0), f(8), f(4)];
}

/** Splits "a, b, c[, d]" or "a b c[ / d]" into its components. */
function args(body: string): string[] | null {
  const [main, slashAlpha, extra] = body.split("/");
  if (extra !== undefined) return null;
  const parts = (main.includes(",") ? main.split(",") : main.trim().split(/\s+/)).map((p) => p.trim());
  if (slashAlpha !== undefined) {
    if (parts.length !== 3) return null;
    parts.push(slashAlpha.trim());
  }
  return parts.length === 3 || parts.length === 4 ? parts : null;
}

/**
 * Parses the color syntaxes Wincy detects (hex, rgb[a](), hsl[a]()).
 * Returns numbers only, so callers never inject raw clipboard text into CSS.
 */
export function parseColor(input: string): Rgba | null {
  const text = input.trim().toLowerCase();

  const hex = /^#([0-9a-f]{3,4}|[0-9a-f]{6}|[0-9a-f]{8})$/.exec(text);
  if (hex) {
    const h = hex[1].length <= 4 ? [...hex[1]].map((c) => c + c).join("") : hex[1];
    const byte = (i: number) => parseInt(h.slice(i, i + 2), 16);
    return { r: byte(0), g: byte(2), b: byte(4), a: h.length === 8 ? byte(6) / 255 : 1 };
  }

  const fn = /^(rgba?|hsla?)\(([^()]*)\)$/.exec(text);
  const parts = fn && args(fn[2]);
  if (!fn || !parts) return null;
  const a = parts[3] === undefined ? 1 : num(parts[3], 1);
  if (a === null) return null;

  if (fn[1].startsWith("rgb")) {
    const [r, g, b] = parts.slice(0, 3).map((p) => num(p, 255));
    if (r === null || g === null || b === null) return null;
    return { r: clamp(r, 0, 255), g: clamp(g, 0, 255), b: clamp(b, 0, 255), a: clamp(a, 0, 1) };
  }
  const h = hue(parts[0]);
  const s = num(parts[1], 1, false);
  const l = num(parts[2], 1, false);
  if (h === null || s === null || l === null) return null;
  const [r, g, b] = hslToRgb(h, clamp(s, 0, 1), clamp(l, 0, 1));
  return { r, g, b, a: clamp(a, 0, 1) };
}

const hex2 = (n: number) => Math.round(n).toString(16).padStart(2, "0");

export function toHex({ r, g, b, a }: Rgba): string {
  return `#${hex2(r)}${hex2(g)}${hex2(b)}${a < 1 ? hex2(a * 255) : ""}`.toUpperCase();
}

export function toHslString({ r, g, b, a }: Rgba): string {
  const [R, G, B] = [r, g, b].map((n) => n / 255);
  const max = Math.max(R, G, B);
  const min = Math.min(R, G, B);
  const d = max - min;
  const l = (max + min) / 2;
  const s = d === 0 ? 0 : d / (1 - Math.abs(2 * l - 1));
  const h = d === 0 ? 0 : max === R ? ((G - B) / d + 6) % 6 : max === G ? (B - R) / d + 2 : (R - G) / d + 4;
  const hsl = `${Math.round(h * 60) % 360}, ${Math.round(s * 100)}%, ${Math.round(l * 100)}%`;
  return a < 1 ? `hsla(${hsl}, ${+a.toFixed(3)})` : `hsl(${hsl})`;
}

export function toRgbString({ r, g, b, a }: Rgba): string {
  const rgb = [r, g, b].map((n) => Math.round(n)).join(", ");
  return a < 1 ? `rgba(${rgb}, ${+a.toFixed(3)})` : `rgb(${rgb})`;
}
