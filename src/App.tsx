import { Toast } from "@heroui/react";
import { useEffect } from "react";
import { api, checkForUpdate, errorMessage, onEvent } from "./api";
import { ClipList } from "./components/ClipList";
import { Footer } from "./components/Footer";
import { QuickActions } from "./components/QuickActions";
import { SearchBar } from "./components/SearchBar";
import { SettingsView } from "./components/SettingsView";
import { useAppearance } from "./hooks/useAppearance";
import { useKeyboardNav } from "./hooks/useKeyboardNav";
import { useI18n } from "./i18n";
import { selectSelected, selectSelectedSnippet, useClipboardStore } from "./store/useClipboardStore";

let updateChecked = false;

/** Backend events + initial load. */
function useBackendSync() {
  const checkUpdates = useClipboardStore((s) => s.settings?.checkUpdates);
  // Undefined until the status loads; the Store build never updates itself.
  const store = useClipboardStore((s) => s.status?.store);

  useEffect(() => {
    const s = useClipboardStore.getState();
    void s.loadSettings();
    // The backend opens the settings view when the shortcut failed at startup, but that
    // palette-shown event can fire before this listener exists.
    void s.loadStatus().then(() => {
      const st = useClipboardStore.getState();
      if (st.status?.shortcutError) st.setView("settings");
    });
    void s.refresh();
    void s.loadSnippets();
    void s.refreshPaused();
    const subscriptions = [
      onEvent("clips-changed", () => void useClipboardStore.getState().refresh()),
      onEvent("palette-shown", (p) => useClipboardStore.getState().onShown(p.view)),
      onEvent("settings-changed", (settings) => {
        const st = useClipboardStore.getState();
        st.setSettings(settings);
        void st.refreshPaused(); // also emitted when capture is paused/resumed
      }),
      onEvent("status-changed", (status) => useClipboardStore.getState().setStatus(status)),
      onEvent("snippets-changed", () => void useClipboardStore.getState().loadSnippets()),
    ];
    return () => subscriptions.forEach((p) => void p.then((unlisten) => unlisten()));
  }, []);

  useEffect(() => {
    if (!checkUpdates || store !== false || updateChecked) return;
    updateChecked = true;
    // Silent (only logged): a failed background check (offline, rate limit) is not worth bothering the user.
    checkForUpdate().then(useClipboardStore.getState().setUpdate, () => undefined);
  }, [checkUpdates, store]);
}

/** Drives the `preview` window: the selected clip while the preview is on, else hidden. */
function usePreviewSync() {
  // Snippets go as negative ids: set_preview only forwards the id, and the preview window loads a snippet for it.
  const id = useClipboardStore((s) => {
    if (!s.showPreview) return null;
    const snippet = selectSelectedSnippet(s);
    return snippet ? -snippet.id : (selectSelected(s)?.id ?? null);
  });
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
  const actionsOpen = useClipboardStore((s) => s.actionsOpen);
  return (
    <>
      <SearchBar />
      <ClipList />
      <Footer />
      {actionsOpen && <QuickActions />}
    </>
  );
}

export default function App() {
  const view = useClipboardStore((s) => s.view);
  const { locale } = useI18n();
  useAppearance();
  useBackendSync();

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  return (
    <main className="wincy-surface relative flex h-full flex-col overflow-hidden text-foreground">
      {view === "palette" ? <Palette /> : <SettingsView />}
      {/* No hotkey: the default Alt+T would pull focus out of the search input. */}
      <Toast.Provider placement="bottom" width={380} hotkey={[]} />
    </main>
  );
}
