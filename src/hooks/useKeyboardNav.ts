import { useEffect } from "react";
import { api, errorMessage } from "../api";
import { selectFiltered, useClipboardStore } from "../store/useClipboardStore";

// shortcut: fixed page size matching ~8 visible 44px rows in the 460px window; measure if the window becomes resizable.
const PAGE = 8;
const REPEATABLE = new Set(["ArrowDown", "ArrowUp", "PageDown", "PageUp"]);

/** Palette keyboard map from docs/ARCHITECTURE.md. Window-level so it works while the search input has focus. */
export function useKeyboardNav() {
  useEffect(() => {
    function onKeyDown(e: KeyboardEvent) {
      if (e.isComposing || e.keyCode === 229) return; // IME composition in progress
      const s = useClipboardStore.getState();
      const i = s.selectedIndex;
      const last = selectFiltered(s).length - 1;
      // AltGr arrives as Ctrl+Alt on Windows and types characters (e.g. "@" on es layouts): never treat it as Alt.
      const alt = e.altKey && !e.ctrlKey && !e.metaKey;
      const ctrl = e.ctrlKey && !e.altKey && !e.metaKey;

      const action = ((): (() => unknown) | null => {
        if (alt && /^Digit[1-9]$/.test(e.code)) return () => s.paste(Number(e.code.slice(5)) - 1, false);
        if (ctrl) {
          switch (e.key.toLowerCase()) {
            case "p": return s.togglePin;
            case "delete": return s.deleteSelected;
            case " ": return s.togglePreview;
            case ",": return () => s.setView("settings");
            case "home": return () => s.select(0);
            case "end": return () => s.select(last);
          }
          return null;
        }
        if (e.altKey || e.metaKey) return null;
        switch (e.key) {
          case "ArrowDown": return () => s.select(i + 1);
          case "ArrowUp": return () => s.select(i - 1);
          case "PageDown": return () => s.select(i + PAGE);
          case "PageUp": return () => s.select(i - PAGE);
          // Plain Home/End keep moving the caret while there is text to edit.
          case "Home": return s.query ? null : () => s.select(0);
          case "End": return s.query ? null : () => s.select(last);
          case "Enter": return () => s.paste(i, e.shiftKey);
          case "Tab": return () => s.cycleKind(e.shiftKey ? -1 : 1);
          case "Escape":
            return s.query
              ? () => s.setQuery("")
              : () => api.hideWindow().catch((err: unknown) => useClipboardStore.setState({ error: errorMessage(err) }));
        }
        return null;
      })();

      if (!action) return;
      e.preventDefault();
      // Holding a key may scroll, but must never paste/delete/toggle repeatedly.
      if (!e.repeat || REPEATABLE.has(e.key)) void action();
    }

    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, []);
}
