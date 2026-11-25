import { Input, toast } from "@heroui/react";
import {
  Binary,
  BookmarkPlus,
  Braces,
  CaseLower,
  CaseSensitive,
  CaseUpper,
  ExternalLink,
  FolderOpen,
  Keyboard,
  Minimize2,
  Percent,
  Pipette,
  Scissors,
  Space,
  Unlink,
  WrapText,
  type LucideIcon,
} from "lucide-react";
import { useEffect, useMemo, useState, type KeyboardEvent, type ReactNode } from "react";
import { api, errorMessage, openExternal, revealFile } from "../api";
import { useI18n, type Translate } from "../i18n";
import { parseColor, toHex, toHslString, toRgbString } from "../lib/color";
import { applicableTransforms, type TransformId } from "../lib/transforms";
import { browserUrl } from "../lib/url";
import { selectSelected, useClipboardStore } from "../store/useClipboardStore";
import type { Clip, ClipSummary } from "../types";
import { keycap, ShortcutRecorder } from "./ShortcutRecorder";

interface Action {
  id: string;
  group: "transform" | "clip";
  icon: LucideIcon;
  label: string;
  /** Right-aligned detail: the transformed text, a keycap… */
  detail?: string;
  run: () => unknown;
}

const TRANSFORM_ICONS: Record<TransformId, LucideIcon> = {
  upper: CaseUpper,
  lower: CaseLower,
  title: CaseSensitive,
  trim: Scissors,
  collapse: Space,
  joinLines: WrapText,
  jsonFormat: Braces,
  jsonMinify: Minimize2,
  base64Encode: Binary,
  base64Decode: Binary,
  urlEncode: Percent,
  urlDecode: Percent,
};

const oneLine = (text: string) => text.replace(/\s+/g, " ").trim().slice(0, 80);

function defaultSnippetName(text: string): string {
  const line = text.split(/\r?\n/).find((l) => l.trim()) ?? "";
  return Array.from(line.trim()).slice(0, 80).join(""); // code points: never split an emoji
}

type Mode = "list" | "snippet" | "shortcut";

interface Handlers {
  pasteText: (text: string) => void;
  open: (url: string) => void;
  reveal: (path: string) => void;
  setMode: (mode: Mode) => void;
  removeShortcut: () => void;
}

function buildActions(clip: Clip, t: Translate, h: Handlers): Action[] {
  const actions: Action[] = [];
  const text = clip.text;
  if (clip.kind === "color" && text !== null) {
    const rgba = parseColor(text);
    if (rgba) {
      for (const [format, value] of [["HEX", toHex(rgba)], ["RGB", toRgbString(rgba)], ["HSL", toHslString(rgba)]]) {
        if (value.toLowerCase() === text.trim().toLowerCase()) continue;
        actions.push({ id: `color-${format}`, group: "clip", icon: Pipette, label: t("act.copyAs", { format }), detail: value, run: () => h.pasteText(value) });
      }
    }
  }
  const url = clip.kind === "url" && text !== null ? browserUrl(text) : null;
  if (url) actions.push({ id: "open", group: "clip", icon: ExternalLink, label: t("act.open"), run: () => h.open(url) });
  const file = clip.kind === "files" ? clip.files?.[0] : undefined;
  if (file) actions.push({ id: "reveal", group: "clip", icon: FolderOpen, label: t("act.reveal"), run: () => h.reveal(file) });
  if (text !== null) actions.push({ id: "snippet", group: "clip", icon: BookmarkPlus, label: t("act.saveSnippet"), run: () => h.setMode("snippet") });
  if (clip.pinned) {
    actions.push({
      id: "shortcut",
      group: "clip",
      icon: Keyboard,
      label: t("act.setShortcut"),
      detail: clip.shortcut ? keycap(clip.shortcut) : undefined,
      run: () => h.setMode("shortcut"),
    });
    if (clip.shortcut) actions.push({ id: "unshortcut", group: "clip", icon: Unlink, label: t("act.removeShortcut"), run: h.removeShortcut });
  }
  if (text !== null) {
    for (const { id, result } of applicableTransforms(text)) {
      actions.push({ id, group: "transform", icon: TRANSFORM_ICONS[id], label: t(`t.${id}`), detail: oneLine(result), run: () => h.pasteText(result) });
    }
  }
  return actions;
}

/** The Ctrl+K panel over the palette. It acts on the clip that was selected when it opened. */
export function QuickActions() {
  const { t } = useI18n();
  const close = () => useClipboardStore.getState().setActionsOpen(false);
  const fail = (e: unknown) => useClipboardStore.getState().setError(errorMessage(e));
  const [target] = useState<ClipSummary | undefined>(() => selectSelected(useClipboardStore.getState()));
  const [clip, setClip] = useState<Clip | null>(null);
  const [mode, setMode] = useState<Mode>("list");
  const [query, setQuery] = useState("");
  const [active, setActive] = useState(0);

  useEffect(() => {
    if (!target) return close();
    let cancelled = false;
    api.getClip(target.id).then(
      (c) => !cancelled && setClip(c),
      (e: unknown) => {
        fail(e);
        close();
      },
    );
    return () => {
      cancelled = true;
    };
  }, [target]);

  const actions = useMemo(() => {
    if (!clip) return [];
    const done = (p: Promise<unknown>) => void p.then(close, (e: unknown) => (fail(e), close()));
    return buildActions(clip, t, {
      pasteText: (text) => done(api.pasteText(text)),
      open: (url) => done(openExternal(url)),
      reveal: (path) => done(revealFile(path)),
      setMode,
      removeShortcut: () =>
        done(api.setClipShortcut(clip.id, null).then(() => toast(t("toast.shortcutRemoved")))),
    });
  }, [clip, t]);

  const needle = query.trim().toLocaleLowerCase();
  const visible = needle ? actions.filter((a) => a.label.toLocaleLowerCase().includes(needle)) : actions;
  const current = Math.min(active, visible.length - 1);

  function onListKey(e: KeyboardEvent) {
    const move = (to: number) => setActive(Math.max(0, Math.min(to, visible.length - 1)));
    switch (e.key) {
      case "ArrowDown": move(current + 1); break;
      case "ArrowUp": move(current - 1); break;
      case "PageDown": case "End": move(visible.length - 1); break;
      case "PageUp": case "Home": move(0); break;
      case "Enter": visible[current]?.run(); break;
      case "Escape": close(); break;
      case "Tab": break; // keep focus in the panel
      default:
        if (!(e.ctrlKey && e.key.toLowerCase() === "k")) return;
        close();
    }
    e.preventDefault();
  }

  let body: ReactNode;
  if (!clip) {
    body = <p className="px-3 py-6 text-center text-sm text-muted" role="status">{t("actions.loading")}</p>;
  } else if (mode === "snippet") {
    body = <SnippetNameForm clip={clip} onBack={() => setMode("list")} onDone={close} />;
  } else if (mode === "shortcut") {
    body = (
      <div
        className="flex flex-col gap-3 px-3 py-3"
        // The recorder stops the Esc that cancels recording; the next one goes back. Tab must
        // not move focus behind the panel, where no key handler would reach it.
        onKeyDown={(e) => {
          if (e.key === "Escape") setMode("list");
          else if (e.key !== "Tab") return;
          e.preventDefault();
        }}
      >
        <div className="flex flex-col">
          <span id="qa-shortcut" className="text-sm">{t("act.shortcutTitle")}</span>
          <span className="text-xs text-muted">{t("act.shortcutHint")}</span>
        </div>
        <ShortcutRecorder
          labelId="qa-shortcut"
          value={clip.shortcut}
          autoFocus
          onChange={(shortcut) =>
            api.setClipShortcut(clip.id, shortcut).then(
              () => {
                toast.success(shortcut ? t("toast.shortcutSet", { shortcut: keycap(shortcut) }) : t("toast.shortcutRemoved"));
                close();
                return null;
              },
              (e: unknown) => errorMessage(e),
            )
          }
        />
      </div>
    );
  } else {
    body = (
      <>
        <Input
          autoFocus
          value={query}
          onChange={(e) => {
            setQuery(e.target.value);
            setActive(0);
          }}
          onKeyDown={onListKey}
          placeholder={t("actions.search")}
          aria-label={t("actions.search")}
          role="combobox"
          aria-expanded
          aria-controls="qa-list"
          aria-activedescendant={visible[current] ? `qa-${visible[current].id}` : undefined}
          spellCheck={false}
          autoComplete="off"
          className="h-10 shrink-0 rounded-none border-0 border-b border-(--wincy-stroke) bg-transparent px-3 text-sm shadow-none outline-none focus:ring-0 focus-visible:ring-0"
        />
        <div id="qa-list" role="listbox" aria-label={t("a.actionsLong")} className="wincy-scroll min-h-0 overflow-y-auto p-1">
          {visible.length === 0 && <p className="px-3 py-4 text-center text-sm text-muted">{t("actions.empty")}</p>}
          {visible.map((a, i) => (
            <div key={a.id}>
              {a.group !== visible[i - 1]?.group && (
                <div role="presentation" className="px-2 pt-1.5 pb-1 text-[11px] font-semibold tracking-wide text-muted uppercase">
                  {a.group === "transform" ? t("actions.transform") : t("actions.clip")}
                </div>
              )}
              <div
                id={`qa-${a.id}`}
                role="option"
                aria-selected={i === current}
                onMouseDown={(e) => e.preventDefault()}
                onMouseMove={() => i !== current && setActive(i)}
                onClick={() => a.run()}
                className={`flex h-8 cursor-default items-center gap-2.5 rounded-md px-2 text-[13px] ${i === current ? "bg-(--wincy-selected)" : ""}`}
              >
                <a.icon size={15} className={`shrink-0 ${i === current ? "text-accent" : "text-muted"}`} aria-hidden />
                <span className="shrink-0">{a.label}</span>
                {a.detail && <span className="min-w-0 flex-1 truncate text-right font-mono text-[11.5px] text-muted">{a.detail}</span>}
              </div>
            </div>
          ))}
        </div>
      </>
    );
  }

  return (
    // Click outside the panel closes it.
    <div className="absolute inset-0 z-20 flex items-end justify-end bg-black/15 p-2 pb-11" onMouseDown={(e) => e.target === e.currentTarget && close()}>
      <section
        aria-label={t("a.actionsLong")}
        // Clicks on headers, padding or text must not take focus from the input that handles the keys.
        onMouseDown={(e) => !(e.target as Element).closest("input,[tabindex]") && e.preventDefault()}
        className="flex max-h-[340px] w-[380px] flex-col overflow-hidden rounded-lg border border-(--wincy-stroke) bg-(--wincy-popover) shadow-xl"
      >
        {body}
      </section>
    </div>
  );
}

function SnippetNameForm({ clip, onBack, onDone }: { clip: Clip; onBack: () => void; onDone: () => void }) {
  const { t } = useI18n();
  const [name, setName] = useState(() => defaultSnippetName(clip.text ?? ""));
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);

  async function save() {
    if (saving) return;
    setSaving(true);
    try {
      const saved = await api.saveSnippet({ id: null, name: name.trim(), body: clip.text ?? "", shortcut: null });
      toast.success(t("toast.snippetSaved", { name: saved.name }));
      onDone();
    } catch (e) {
      setError(errorMessage(e));
      setSaving(false);
    }
  }

  return (
    <div className="flex flex-col gap-2 px-3 py-3">
      <label htmlFor="qa-snippet-name" className="text-sm">{t("act.snippetName")}</label>
      <Input
        id="qa-snippet-name"
        autoFocus
        value={name}
        maxLength={80}
        onChange={(e) => {
          setName(e.target.value);
          setError(null);
        }}
        onFocus={(e) => e.target.select()}
        onKeyDown={(e) => {
          if (e.key === "Enter") void save();
          else if (e.key === "Escape") onBack();
          else if (e.key !== "Tab") return; // Tab: keep focus in the panel
          e.preventDefault();
        }}
        aria-describedby={error ? "qa-snippet-error" : "qa-snippet-hint"}
        aria-invalid={!!error}
        spellCheck={false}
        autoComplete="off"
        className="h-8 text-sm"
      />
      {error ? (
        <p id="qa-snippet-error" role="alert" className="text-xs text-danger">{error}</p>
      ) : (
        <p id="qa-snippet-hint" className="text-xs text-muted">{t("act.snippetNameHint")}</p>
      )}
    </div>
  );
}
