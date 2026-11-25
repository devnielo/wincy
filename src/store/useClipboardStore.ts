import Fuse from "fuse.js";
import type { Update } from "@tauri-apps/plugin-updater";
import { create } from "zustand";
import { api, errorMessage } from "../api";
import { matchesApp, parseQuery } from "../lib/query";
import type { ClipKind, ClipSummary, Settings, Snippet, Status, View } from "../types";

export type KindFilter = "all" | ClipKind | "snippets";
export const KIND_FILTERS: KindFilter[] = ["all", "text", "url", "color", "code", "image", "files", "snippets"];
const FTS_LIMIT = 200;

interface State {
  clips: ClipSummary[];
  /** All clips in the database; more than `clips.length` means search must also ask the full-text index. */
  total: number;
  /** search_clips results for `ftsQuery`, merged after the Fuse results. */
  fts: ClipSummary[];
  ftsQuery: string;
  snippets: Snippet[];
  /** The quick actions overlay (Ctrl+K) is open over the palette. */
  actionsOpen: boolean;
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
  loadSnippets: () => Promise<void>;
  setActionsOpen: (open: boolean) => void;
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
let filterCache: { clips: ClipSummary[]; fts: ClipSummary[]; query: string; kind: KindFilter; result: ClipSummary[] } | null = null;
let snippetCache: { snippets: Snippet[]; query: string; fuse: Fuse<Snippet>; result: Snippet[] } | null = null;

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

type FilterInput = Pick<State, "clips" | "fts" | "ftsQuery" | "query" | "kindFilter">;

/** Visible clips (none in the snippets filter). Returns a stable reference while its inputs are unchanged (safe as a selector). */
export function selectFiltered(s: FilterInput): ClipSummary[] {
  const query = s.query.trim();
  // Index results for an older query must never show.
  const fts = s.ftsQuery === query ? s.fts : NO_CLIPS;
  const c = filterCache;
  if (c && c.clips === s.clips && c.fts === fts && c.query === query && c.kind === s.kindFilter) return c.result;
  const { apps, text } = parseQuery(query);
  let base = text ? fuseFor(s.clips).search(text).map((r) => r.item) : s.clips;
  if (fts.length) {
    const seen = new Set(base.map((clip) => clip.id));
    base = [...base, ...fts.filter((clip) => !seen.has(clip.id))];
  }
  const result = base.filter((clip) => (s.kindFilter === "all" || clip.kind === s.kindFilter) && matchesApp(clip.sourceApp, apps));
  filterCache = { clips: s.clips, fts, query, kind: s.kindFilter, result };
  return result;
}
const NO_CLIPS: ClipSummary[] = [];

/** Visible snippets (only in the snippets filter); search covers name and body. */
export function selectSnippets(s: Pick<State, "snippets" | "query" | "kindFilter">): Snippet[] {
  if (s.kindFilter !== "snippets") return NO_SNIPPETS;
  const query = s.query.trim();
  const c = snippetCache;
  if (c?.snippets === s.snippets && c.query === query) return c.result;
  const fuse = c?.snippets === s.snippets ? c.fuse : new Fuse(s.snippets, { keys: ["name", "body"], threshold: 0.35, ignoreLocation: true });
  const { apps, text } = parseQuery(query);
  // Snippets have no source app, so an app: filter matches none.
  const result = apps.length ? NO_SNIPPETS : text ? fuse.search(text).map((r) => r.item) : s.snippets;
  snippetCache = { snippets: s.snippets, query, fuse, result };
  return result;
}
const NO_SNIPPETS: Snippet[] = [];

export const selectSelected = (s: State): ClipSummary | undefined => selectFiltered(s)[s.selectedIndex];
export const selectSelectedSnippet = (s: State): Snippet | undefined => selectSnippets(s)[s.selectedIndex];
/** Rows in the visible list, clips or snippets. */
export const selectCount = (s: State): number =>
  s.kindFilter === "snippets" ? selectSnippets(s).length : selectFiltered(s).length;

const clampIndex = (index: number, length: number) => Math.max(0, Math.min(index, length - 1));

let refreshSeq = 0;
let ftsSeq = 0;
let ftsTimer: ReturnType<typeof setTimeout> | undefined;
/** palette-shown asks for the top row; other refreshes keep the selected clip selected. */
let selectTopOnRefresh = false;

export const useClipboardStore = create<State>()((set, get) => {
  const fail = (e: unknown) => set({ error: errorMessage(e) });

  /** Large histories: after a quiet ~150 ms, also ask the full-text index for clips that weren't loaded. */
  const searchIndex = () => {
    clearTimeout(ftsTimer);
    const seq = ++ftsSeq; // also invalidates any request still in flight
    const { query, total, clips } = get();
    const q = query.trim();
    const { apps, text } = parseQuery(q);
    // An app:-only query also searches the index: old clips from that app were not loaded.
    if (total <= clips.length || (!text && apps.length === 0)) return;
    ftsTimer = setTimeout(() => {
      api.searchClips(q, FTS_LIMIT).then(
        (fts) => seq === ftsSeq && set({ fts, ftsQuery: q }),
        () => undefined, // already logged; the loaded clips still show
      );
    }, 150);
  };

  return {
    clips: [],
    total: 0,
    fts: [],
    ftsQuery: "",
    snippets: [],
    actionsOpen: false,
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
        const { clips, total } = await api.getClips();
        if (seq !== refreshSeq) return; // a newer refresh is in flight
        set((s) => {
          const top = selectTopOnRefresh;
          selectTopOnRefresh = false;
          if (s.kindFilter === "snippets") return { clips, total, selectedIndex: top ? 0 : s.selectedIndex };
          // Pins and new copies reorder the list: follow the selected clip, not its row number.
          const prevId = top ? undefined : selectSelected(s)?.id;
          const list = selectFiltered({ ...s, clips });
          const i = prevId === undefined ? -1 : list.findIndex((c) => c.id === prevId);
          return { clips, total, selectedIndex: i >= 0 ? i : clampIndex(s.selectedIndex, list.length) };
        });
        if (get().query) searchIndex(); // the index changed too
      } catch (e) {
        fail(e);
      }
    },

    loadSnippets: async () => {
      try {
        const snippets = await api.getSnippets();
        set((s) => ({ snippets, selectedIndex: s.kindFilter === "snippets" ? clampIndex(s.selectedIndex, snippets.length) : s.selectedIndex }));
      } catch (e) {
        fail(e);
      }
    },

    setActionsOpen: (actionsOpen) => set({ actionsOpen }),

    onShown: (view) => {
      set((s) => ({ query: "", kindFilter: "all", selectedIndex: 0, view, error: null, actionsOpen: false, shownCount: s.shownCount + 1 }));
      selectTopOnRefresh = true;
      void get().refresh();
      void get().refreshPaused();
    },

    setQuery: (query) => {
      set({ query, selectedIndex: 0 });
      searchIndex();
    },

    cycleKind: (step) =>
      set((s) => {
        const i = KIND_FILTERS.indexOf(s.kindFilter);
        return { kindFilter: KIND_FILTERS[(i + step + KIND_FILTERS.length) % KIND_FILTERS.length], selectedIndex: 0 };
      }),

    setKind: (kindFilter) => set({ kindFilter, selectedIndex: 0 }),

    select: (index) => set((s) => ({ selectedIndex: clampIndex(index, selectCount(s)) })),

    setView: (view) => set({ view, error: null }),

    togglePreview: () => set((s) => ({ showPreview: !s.showPreview })),

    paste: async (index, plain) => {
      const s = get();
      const snippet = selectSnippets(s)[index];
      const clip = selectFiltered(s)[index];
      try {
        // Snippets always paste as plain text (the backend expands their variables).
        if (snippet) await api.pasteSnippet(snippet.id);
        else if (clip) await api.pasteClip(clip.id, plain);
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
