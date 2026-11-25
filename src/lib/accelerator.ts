export interface KeyLike {
  code: string;
  ctrlKey: boolean;
  altKey: boolean;
  shiftKey: boolean;
  metaKey: boolean;
}

// KeyboardEvent.code values the global-hotkey crate accepts verbatim.
const NAMED = new Set([
  "Space", "Enter", "Tab", "Backspace", "Delete", "Insert", "Home", "End", "PageUp", "PageDown",
  "Backquote", "Minus", "Equal", "BracketLeft", "BracketRight", "Backslash", "Semicolon", "Quote",
  "Comma", "Period", "Slash", "PrintScreen", "Pause",
]);

function keyName(code: string): string | null {
  const m = /^(?:Key([A-Z])|Digit(\d)|Arrow(Up|Down|Left|Right))$/.exec(code);
  if (m) return m[1] ?? m[2] ?? m[3];
  return /^(F([1-9]|1\d|2[0-4])|Numpad\d)$/.test(code) || NAMED.has(code) ? code : null;
}

/**
 * Builds a tauri global-shortcut accelerator ("Ctrl+Shift+Space") from a keydown.
 * Returns null while only modifiers are held, for unsupported keys, for keys with no
 * modifier but Shift (except F-keys): those would hijack normal typing system-wide, and for
 * Ctrl(+Shift)+C/V/X/Z/Y/A and Alt+1..9: those would break copy/paste everywhere (including
 * Wincy's own simulated Ctrl+V) and the palette's own Alt+N keys.
 */
export function acceleratorFromEvent(e: KeyLike): string | null {
  const key = keyName(e.code);
  if (!key) return null;
  const mods = [e.ctrlKey && "Ctrl", e.altKey && "Alt", e.shiftKey && "Shift", e.metaKey && "Super"].filter(
    (m): m is string => !!m,
  );
  const isFKey = /^F\d+$/.test(key);
  if (!isFKey && !mods.some((m) => m !== "Shift")) return null;
  const only = (...want: string[]) => mods.filter((m) => m !== "Shift").join() === want.join();
  if (only("Ctrl") && /^[CVXZYA]$/.test(key)) return null;
  if (only("Alt") && !e.shiftKey && /^[1-9]$/.test(key)) return null;
  return [...mods, key].join("+");
}
