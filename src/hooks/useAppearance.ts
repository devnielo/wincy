import { useLayoutEffect, useSyncExternalStore } from "react";
import { resolveAccent } from "../lib/accent";
import { useClipboardStore } from "../store/useClipboardStore";

const media = window.matchMedia("(prefers-color-scheme: dark)");
const subscribe = (onChange: () => void) => {
  media.addEventListener("change", onChange);
  return () => media.removeEventListener("change", onChange);
};

/** Whether the app theme currently resolves to dark. */
export function useDark(): boolean {
  const theme = useClipboardStore((s) => s.settings?.theme ?? "system");
  const systemDark = useSyncExternalStore(subscribe, () => media.matches);
  return theme === "dark" || (theme === "system" && systemDark);
}

/** Applies the theme (light/dark classes) and the accent tokens to this window's document. */
export function useAppearance() {
  const dark = useDark();
  const accent = useClipboardStore((s) => s.settings?.accent ?? "windows");
  const systemAccent = useClipboardStore((s) => s.status?.systemAccent ?? null);

  useLayoutEffect(() => {
    const root = document.documentElement;
    root.classList.toggle("dark", dark);
    root.classList.toggle("light", !dark);
    root.dataset.theme = dark ? "dark" : "light";
    // Inline on :root, so it beats HeroUI's theme variables; --focus, --accent-soft… derive from these.
    const { color, foreground } = resolveAccent(accent, systemAccent, dark);
    root.style.setProperty("--accent", color);
    root.style.setProperty("--accent-foreground", foreground);
  }, [dark, accent, systemAccent]);
}
