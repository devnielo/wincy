import { Button } from "@heroui/react";
import { useVirtualizer } from "@tanstack/react-virtual";
import { Braces, ClipboardList, SearchX } from "lucide-react";
import { useCallback, useEffect, useRef } from "react";
import { useI18n } from "../i18n";
import { selectFiltered, selectSnippets, useClipboardStore } from "../store/useClipboardStore";
import { ClipItem, ROW_HEIGHT, SnippetItem } from "./ClipItem";

function Empty() {
  const { t } = useI18n();
  const snippetsMode = useClipboardStore((s) => s.kindFilter === "snippets");
  const hasItems = useClipboardStore((s) => (snippetsMode ? s.snippets.length : s.clips.length) > 0);
  if (snippetsMode && !hasItems) {
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-2 px-6 text-center text-sm text-muted" role="status">
        <Braces size={28} strokeWidth={1.5} aria-hidden />
        <span className="text-foreground">{t("snippet.empty")}</span>
        <span className="text-xs">{t("snippet.emptyHint")}</span>
        <Button size="sm" variant="secondary" preventFocusOnPress onPress={() => useClipboardStore.getState().setView("settings")} className="mt-1">
          {t("snippet.add")}
        </Button>
      </div>
    );
  }
  const Icon = hasItems ? SearchX : ClipboardList;
  return (
    <div className="flex flex-1 flex-col items-center justify-center gap-2 text-sm text-muted" role="status">
      <Icon size={28} strokeWidth={1.5} aria-hidden />
      {hasItems ? t("emptyResults") : t("emptyHistory")}
    </div>
  );
}

export function ClipList() {
  const { t } = useI18n();
  const clips = useClipboardStore(selectFiltered);
  const snippets = useClipboardStore(selectSnippets);
  const snippetsMode = useClipboardStore((s) => s.kindFilter === "snippets");
  const selectedIndex = useClipboardStore((s) => s.selectedIndex);
  const select = useClipboardStore((s) => s.select);
  const paste = useClipboardStore((s) => s.paste);
  const scrollRef = useRef<HTMLDivElement>(null);
  const pick = useCallback((index: number) => void paste(index, false), [paste]);
  const count = snippetsMode ? snippets.length : clips.length;

  const virtualizer = useVirtualizer({
    count,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 8,
    paddingStart: 6,
    paddingEnd: 6,
  });

  useEffect(() => {
    if (count) virtualizer.scrollToIndex(selectedIndex, { align: "auto" });
  }, [virtualizer, selectedIndex, count, clips, snippets]);

  if (!count) return <Empty />;

  return (
    <div ref={scrollRef} className="wincy-scroll min-h-0 flex-1 overflow-y-auto">
      <div id="clip-list" role="listbox" aria-label={snippetsMode ? t("kind.snippets") : t("clipList")} className="relative" style={{ height: virtualizer.getTotalSize() }}>
        {virtualizer.getVirtualItems().map((row) => {
          const props = { index: row.index, selected: row.index === selectedIndex, onHover: select, onPick: pick };
          const snippet = snippetsMode ? snippets[row.index] : undefined;
          return (
            <div
              key={snippet ? `s${snippet.id}` : clips[row.index].id}
              role="presentation"
              className="absolute inset-x-0 top-0"
              style={{ transform: `translateY(${row.start}px)` }}
            >
              {snippet ? <SnippetItem snippet={snippet} {...props} /> : <ClipItem clip={clips[row.index]} {...props} />}
            </div>
          );
        })}
      </div>
    </div>
  );
}
