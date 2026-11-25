import type { Accent } from "../types";
import { contrastRatio, parseColor, readableOn, toHex, type Rgba } from "./color.ts";

export type FixedAccent = Exclude<Accent, "windows">;

/** [light theme, dark theme]: darker in light mode and lighter in dark mode, so accent text stays legible on the surface. */
export const ACCENTS: Record<FixedAccent, [string, string]> = {
  blue: ["#005FB8", "#4CC2FF"],
  violet: ["#6B4FD8", "#B4A0FF"],
  teal: ["#00777A", "#4FD1C5"],
  green: ["#107C41", "#6CCB5F"],
  orange: ["#B84A00", "#FFA44F"],
  rose: ["#C0185B", "#FF8FB1"],
  graphite: ["#4B5563", "#C8CCD2"],
};

export const ACCENT_NAMES: Accent[] = ["windows", ...(Object.keys(ACCENTS) as FixedAccent[])];

const SURFACE = { light: parseColor("#F3F3F3")!, dark: parseColor("#202020")! };

/** Like the fixed palette: darkens in light mode and lightens in dark mode until text on the surface reaches AA. */
function legible(c: Rgba, dark: boolean): Rgba {
  const bg = dark ? SURFACE.dark : SURFACE.light;
  const to = dark ? 255 : 0;
  // Ends: black on #F3F3F3 and white on #202020 are both far above 4.5.
  while (contrastRatio(c, bg) < 4.5) c = { r: c.r + (to - c.r) * 0.1, g: c.g + (to - c.g) * 0.1, b: c.b + (to - c.b) * 0.1, a: 1 };
  return c;
}

/** The accent and its text color for the current mode. "windows" falls back to blue when the system accent is unknown. */
export function resolveAccent(accent: Accent, systemAccent: string | null, dark: boolean): { color: string; foreground: string } {
  const system = accent === "windows" && systemAccent ? parseColor(systemAccent) : null;
  // Normalized through parseColor, so only a plain hex ever reaches CSS.
  const color = system ? toHex(legible({ ...system, a: 1 }, dark)) : (ACCENTS[accent as FixedAccent] ?? ACCENTS.blue)[dark ? 1 : 0];
  return { color, foreground: readableOn(parseColor(color)!) };
}
