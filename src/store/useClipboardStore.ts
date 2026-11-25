import Fuse from "fuse.js";
import type { Update } from "@tauri-apps/plugin-updater";
import { create } from "zustand";
import { api, errorMessage } from "../api";
import { matchesApp, parseQuery } from "../lib/query";
import type { ClipKind, ClipSummary, Settings, Status, View } from "../types";

export type KindFilter = "all" | ClipKind;
export const KIND_FILTERS: KindFilter[] = ["all", "text", "url", "color", "code", "image", "files"];

interface State {
  clips: ClipSummary[];
  query: string;
  kindFilter: KindFilter;
  selectedIndex: number;
  showPreview: boolean;
  view: View;
  settings: Settings | null;
  status: Status | null;
  paused: boolean;
  /** Set by the startup update check; SettingsView installs it. */
  update: Update | null;
  /** An update download/install is running. Lives here so leaving Settings can't start a second one. */
  installing: boolean;
  /** Last failed action, shown in the current view until it changes. */
  error: string | null;
  /** Bumped on every palette-shown so the search input re-focuses. */
  shownCount: number;

  refresh: () => Promise<void>;
  onShown: (view: View) => void;
  setQuery: (query: string) => void;
  cycleKind: (step: 1 | -1) => void;
  setKind: (kind: KindFilter) => void;
  select: (index: number) => void;
  setView: (view: View) => void;
  togglePreview: () => void;
  paste: (index: number, plain: boolean) => Promise<void>;
  togglePin: () => Promise<void>;
  deleteSelected: () => Promise<void>;
  loadSettings: () => Promise<void>;
  setSettings: (settings: Settings) => void;
  loadStatus: () => Promise<void>;
  setStatus: (status: Status) => void;
  /** Saves immediately; resolves to the backend error message, or null on success. */
  saveSettings: (patch: Partial<Settings>) => Promise<string | null>;
  setPaused: (paused: boolean) => Promise<void>;
  refreshPaused: () => Promise<void>;
  setUpdate: (update: Update | null) => void;
  setInstalling: (installing: boolean) => void;
  setError: (error: string | null) => void;
}

let fuseCache: { clips: ClipSummary[]; fuse: Fuse<ClipSummary> } | null = null;
let filterCache: { clips: ClipSummary[]; query: string; kind: KindFilter; result: ClipSummary[] } | null = null;

function fuseFor(clips: ClipSummary[]): Fuse<ClipSummary> {
  if (fuseCache?.clips !== clips) {
    fuseCache = {
      clips,
      fuse: new Fuse(clips, {
        keys: ["preview", "ocrText", "sourceApp", "kind"],
        threshold: 0.35,
        // Matches can sit anywhere in the 500-char preview, not only near the start.
        ignoreLocation: true,
      }),
    };
  }
  return fuseCache.fuse;
}

/** Visible list. Returns a stable reference while its inputs are unchanged (safe as a selector). */
export function selectFiltered(s: Pick<State, "clips" | "query" | "kindFilter">): ClipSummary[] {
  const query = s.query.trim();
  const c = filterCache;
  if (c && c.clips === s.clips && c.query === query && c.kind === s.kindFilter) return c.result;
  const { apps, text } = parseQuery(query);
  const base = text ? fuseFor(s.clips).search(text).map((r) => r.item) : s.clips;
  const result = base.filter((clip) => (s.kindFilter === "all" || clip.kind === s.kindFilter) && matchesApp(clip.sourceApp, apps));
  filterCache = { clips: s.clips, query, kind: s.kindFilter, result };
  return result;
}

export const selectSelected = (s: State): ClipSummary | undefined => selectFiltered(s)[s.selectedIndex];

const clampIndex = (index: number, length: number) => Math.max(0, Math.min(index, length - 1));

let refreshSeq = 0;
/** palette-shown asks for the top row; other refreshes keep the selected clip selected. */
let selectTopOnRefresh = false;

export const useClipboardStore = create<State>()((set, get) => {
  const fail = (e: unknown) => set({ error: errorMessage(e) });

  return {
    clips: [],
    query: "",
    kindFilter: "all",
    selectedIndex: 0,
    showPreview: true,
    view: "palette",
    settings: null,
    status: null,
    paused: false,
    update: null,
    installing: false,
    error: null,
    shownCount: 0,

    refresh: async () => {
      const seq = ++refreshSeq;
      try {
        const clips = await api.getClips();
        if (seq !== refreshSeq) return; // a newer refresh is in flight
        set((s) => {
          // Pins and new copies reorder the list: follow the selected clip, not its row number.
          const prevId = selectTopOnRefresh ? undefined : selectSelected(s)?.id;
          selectTopOnRefresh = false;
          const list = selectFiltered({ ...s, clips });
          const i = prevId === undefined ? -1 : list.findIndex((c) => c.id === prevId);
          return { clips, selectedIndex: i >= 0 ? i : clampIndex(s.selectedIndex, list.length) };
        });
      } catch (e) {
        fail(e);
      }
    },

    onShown: (view) => {
      set((s) => ({ query: "", kindFilter: "all", selectedIndex: 0, view, error: null, shownCount: s.shownCount + 1 }));
      selectTopOnRefresh = true;
      void get().refresh();
      void get().refreshPaused();
    },

    setQuery: (query) => set({ query, selectedIndex: 0 }),

    cycleKind: (step) =>
      set((s) => {
        const i = KIND_FILTERS.indexOf(s.kindFilter);
        return { kindFilter: KIND_FILTERS[(i + step + KIND_FILTERS.length) % KIND_FILTERS.length], selectedIndex: 0 };
      }),

    setKind: (kindFilter) => set({ kindFilter, selectedIndex: 0 }),

    select: (index) => set((s) => ({ selectedIndex: clampIndex(index, selectFiltered(s).length) })),

    setView: (view) => set({ view, error: null }),

    togglePreview: () => set((s) => ({ showPreview: !s.showPreview })),

    paste: async (index, plain) => {
      const clip = selectFiltered(get())[index];
      if (!clip) return;
      try {
        await api.pasteClip(clip.id, plain);
      } catch (e) {
        fail(e);
      }
    },

    togglePin: async () => {
      const clip = selectSelected(get());
      if (!clip) return;
      try {
        await api.togglePin(clip.id);
      } catch (e) {
        fail(e);
      }
    },

    deleteSelected: async () => {
      const clip = selectSelected(get());
      if (!clip) return;
      try {
        await api.deleteClip(clip.id);
      } catch (e) {
        fail(e);
      }
    },

    loadSettings: async () => {
      try {
        set({ settings: await api.getSettings() });
      } catch (e) {
        fail(e);
      }
    },

    setSettings: (settings) => set({ settings }),

    loadStatus: async () => {
      try {
        set({ status: await api.getStatus() });
      } catch (e) {
        fail(e);
      }
    },

    setStatus: (status) => set({ status }),

    saveSettings: async (patch) => {
      const current = get().settings;
      if (!current) return null;
      set({ settings: { ...current, ...patch } }); // optimistic, so controls respond instantly
      try {
        set({ settings: await api.saveSettings({ ...current, ...patch }) });
        return null;
      } catch (e) {
        // Re-read instead of rolling back, so a concurrent successful save isn't undone.
        await get().loadSettings();
        return errorMessage(e);
      }
    },

    setPaused: async (paused) => {
      try {
        set({ paused: await api.setPaused(paused) });
      } catch (e) {
        fail(e);
      }
    },

    refreshPaused: async () => {
      try {
        set({ paused: await api.getPaused() });
      } catch (e) {
        fail(e);
      }
    },

    setUpdate: (update) => set({ update }),

    setInstalling: (installing) => set({ installing }),

    setError: (error) => set({ error }),
  };
});
