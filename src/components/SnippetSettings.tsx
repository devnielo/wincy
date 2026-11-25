import { Button, Input, Kbd, TextArea } from "@heroui/react";
import { Pencil, Plus, Trash2 } from "lucide-react";
import { useState, type KeyboardEvent } from "react";
import { api, errorMessage } from "../api";
import { useI18n } from "../i18n";
import { useClipboardStore } from "../store/useClipboardStore";
import type { Snippet, SnippetInput } from "../types";
import { keycap, ShortcutRecorder } from "./ShortcutRecorder";

export interface Confirm {
  title: string;
  body: string;
  action: string;
  run: () => void;
}

function Editor({ initial, onDone }: { initial: SnippetInput; onDone: () => void }) {
  const { t } = useI18n();
  const [draft, setDraft] = useState(initial);
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const update = (patch: Partial<SnippetInput>) => {
    setDraft((d) => ({ ...d, ...patch }));
    setError(null);
  };

  async function save() {
    setSaving(true);
    try {
      await api.saveSnippet({ ...draft, name: draft.name.trim() });
      onDone();
    } catch (e) {
      // The backend validates (length, shortcut rules and uniqueness); show its message as is.
      setError(errorMessage(e));
      setSaving(false);
    }
  }

  // Esc cancels the edit instead of leaving Settings; Ctrl+Enter saves from the text area.
  function onKeyDown(e: KeyboardEvent) {
    if (e.key === "Escape" && !e.defaultPrevented) {
      e.preventDefault();
      onDone();
    } else if (e.key === "Enter" && e.ctrlKey) {
      e.preventDefault();
      void save();
    }
  }

  const id = draft.id ?? "new";
  return (
    <form
      onSubmit={(e) => {
        e.preventDefault();
        void save();
      }}
      onKeyDown={onKeyDown}
      className="flex flex-col gap-2.5 px-3 py-3"
      aria-label={draft.id === null ? t("s.newSnippet") : t("s.editSnippet", { name: initial.name })}
    >
      <label className="flex flex-col gap-1">
        <span className="text-xs text-muted">{t("s.snippetName")}</span>
        <Input
          autoFocus
          value={draft.name}
          maxLength={80}
          onChange={(e) => update({ name: e.target.value })}
          spellCheck={false}
          autoComplete="off"
          className="h-8 text-sm"
        />
      </label>
      <label className="flex flex-col gap-1">
        <span className="text-xs text-muted">{t("s.snippetBody")}</span>
        <TextArea
          value={draft.body}
          onChange={(e) => update({ body: e.target.value })}
          rows={5}
          spellCheck={false}
          className="resize-y font-mono text-[12.5px]"
        />
      </label>
      <div className="flex items-center justify-between gap-3">
        <span id={`snippet-shortcut-${id}`} className="text-xs text-muted">
          {t("s.snippetShortcut")}
        </span>
        <ShortcutRecorder
          labelId={`snippet-shortcut-${id}`}
          value={draft.shortcut}
          clearable
          onChange={(shortcut) => {
            update({ shortcut });
            return Promise.resolve(null);
          }}
        />
      </div>
      {error && (
        <p role="alert" className="text-xs text-danger">
          {error}
        </p>
      )}
      <div className="flex justify-end gap-2">
        <Button size="sm" variant="tertiary" onPress={onDone}>
          {t("s.cancel")}
        </Button>
        <Button type="submit" size="sm" variant="primary" isPending={saving} isDisabled={!draft.name.trim() || !draft.body}>
          {t("s.save")}
        </Button>
      </div>
    </form>
  );
}

/** Settings → Snippets: list, add, edit and delete. */
export function SnippetSettings({ ask }: { ask: (c: Confirm) => void }) {
  const { t } = useI18n();
  const snippets = useClipboardStore((s) => s.snippets);
  const setError = useClipboardStore((s) => s.setError);
  // null: no editor open; id null: a new snippet.
  const [editing, setEditing] = useState<SnippetInput | null>(null);
  const edit = (s: Snippet) => setEditing({ id: s.id, name: s.name, body: s.body, shortcut: s.shortcut });

  const remove = (s: Snippet) =>
    ask({
      title: t("s.deleteSnippetTitle", { name: s.name }),
      body: t("s.deleteSnippetBody"),
      action: t("a.delete"),
      run: () => void api.deleteSnippet(s.id).then(() => setError(null), (e: unknown) => setError(errorMessage(e))),
    });

  return (
    <>
      <p className="px-3 py-2.5 text-xs text-muted">{t("s.snippetsHint")}</p>
      {snippets.length === 0 && editing?.id !== null && <p className="px-3 py-3 text-sm text-muted">{t("s.noSnippets")}</p>}
      {snippets.map((s) =>
        editing?.id === s.id ? (
          <Editor key={s.id} initial={editing} onDone={() => setEditing(null)} />
        ) : (
          <div key={s.id} className="flex min-h-12 items-center gap-3 px-3 py-1.5">
            <button type="button" onClick={() => edit(s)} className="flex min-w-0 flex-1 cursor-pointer flex-col text-left outline-none focus-visible:underline">
              <span className="truncate text-sm">{s.name}</span>
              <span className="truncate text-xs text-muted">{s.body.slice(0, 200).replace(/\s+/g, " ")}</span>
            </button>
            {s.shortcut && (
              <Kbd variant="light" className="shrink-0 text-[11px] [word-spacing:normal]">
                <Kbd.Content>{keycap(s.shortcut)}</Kbd.Content>
              </Kbd>
            )}
            <div className="flex shrink-0 gap-0.5">
              <Button isIconOnly size="sm" variant="ghost" aria-label={t("s.editSnippet", { name: s.name })} onPress={() => edit(s)} className="size-7 min-w-7 text-muted">
                <Pencil size={14} />
              </Button>
              <Button isIconOnly size="sm" variant="ghost" aria-label={t("s.deleteSnippet", { name: s.name })} onPress={() => remove(s)} className="size-7 min-w-7 text-muted hover:text-danger">
                <Trash2 size={14} />
              </Button>
            </div>
          </div>
        ),
      )}
      {editing?.id === null ? (
        <Editor initial={editing} onDone={() => setEditing(null)} />
      ) : (
        <div className="px-3 py-2.5">
          <Button size="sm" variant="secondary" onPress={() => setEditing({ id: null, name: "", body: "", shortcut: null })}>
            <Plus size={14} />
            {t("s.newSnippet")}
          </Button>
        </div>
      )}
    </>
  );
}
