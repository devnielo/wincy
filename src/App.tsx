import { useEffect } from "react";
import { api, checkForUpdate, errorMessage, onEvent } from "./api";
import { ClipList } from "./components/ClipList";
import { Footer } from "./components/Footer";
import { SearchBar } from "./components/SearchBar";
import { SettingsView } from "./components/SettingsView";
import { useKeyboardNav } from "./hooks/useKeyboardNav";
import { useI18n } from "./i18n";
import { selectSelected, useClipboardStore } from "./store/useClipboardStore";
import type { Theme } from "./types";

export function useTheme(theme: Theme) {
  useEffect(() => {
    const media = window.matchMedia("(prefers-color-scheme: dark)");
    const apply = () => {
      const dark = theme === "dark" || (theme === "system" && media.matches);
      const root = document.documentElement;
      root.classList.toggle("dark", dark);
      root.classList.toggle("light", !dark);
      root.dataset.theme = dark ? "dark" : "light";
    };
    apply();
    media.addEventListener("change", apply);
    return () => media.removeEventListener("change", apply);
  }, [theme]);
}

let updateChecked = false;

/** Backend events + initial load. */
function useBackendSync() {
  const checkUpdates = useClipboardStore((s) => s.settings?.checkUpdates);

  useEffect(() => {
    const s = useClipboardStore.getState();
    void s.loadSettings();
    void s.refresh();
    void s.refreshPaused();
    const subscriptions = [
      onEvent("clips-changed", () => void useClipboardStore.getState().refresh()),
      onEvent("palette-shown", (p) => useClipboardStore.getState().onShown(p.view)),
      onEvent("settings-changed", (settings) => {
        const st = useClipboardStore.getState();
        st.setSettings(settings);
        void st.refreshPaused(); // also emitted when capture is paused/resumed
      }),
    ];
    return () => subscriptions.forEach((p) => void p.then((unlisten) => unlisten()));
  }, []);

  useEffect(() => {
    if (!checkUpdates || updateChecked) return;
    updateChecked = true;
    // Silent: a failed background check (offline, rate limit) is not worth bothering the user.
    checkForUpdate().then(useClipboardStore.getState().setUpdate, () => undefined);
  }, [checkUpdates]);
}

/** Drives the `preview` window: the selected clip while the preview is on, else hidden. */
function usePreviewSync() {
  const id = useClipboardStore((s) => (s.showPreview ? (selectSelected(s)?.id ?? null) : null));
  // Re-send on every open: the backend hid the panel with the palette, even if the id is unchanged.
  const shownCount = useClipboardStore((s) => s.shownCount);

  useEffect(() => {
    const timer = setTimeout(() => {
      api.setPreview(id).catch((e: unknown) => useClipboardStore.getState().setError(errorMessage(e)));
    }, 80);
    return () => clearTimeout(timer);
  }, [id, shownCount]);

  // Leaving the palette (settings view) hides it.
  useEffect(() => () => void api.setPreview(null).catch(() => undefined), []);
}

function Palette() {
  useKeyboardNav();
  usePreviewSync();
  return (
    <>
      <SearchBar />
      <ClipList />
      <Footer />
    </>
  );
}

export default function App() {
  const view = useClipboardStore((s) => s.view);
  const theme = useClipboardStore((s) => s.settings?.theme ?? "system");
  const { locale } = useI18n();
  useTheme(theme);
  useBackendSync();

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  return (
    <main className="wincy-surface flex h-full flex-col overflow-hidden text-foreground">
      {view === "palette" ? <Palette /> : <SettingsView />}
    </main>
  );
}
