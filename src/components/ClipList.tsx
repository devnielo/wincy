import { useVirtualizer } from "@tanstack/react-virtual";
import { ClipboardList, SearchX } from "lucide-react";
import { useCallback, useEffect, useRef } from "react";
import { useI18n } from "../i18n";
import { selectFiltered, useClipboardStore } from "../store/useClipboardStore";
import { ClipItem, ROW_HEIGHT } from "./ClipItem";

export function ClipList() {
  const { t } = useI18n();
  const clips = useClipboardStore(selectFiltered);
  const selectedIndex = useClipboardStore((s) => s.selectedIndex);
  const hasHistory = useClipboardStore((s) => s.clips.length > 0);
  const select = useClipboardStore((s) => s.select);
  const paste = useClipboardStore((s) => s.paste);
  const scrollRef = useRef<HTMLDivElement>(null);
  const pick = useCallback((index: number) => void paste(index, false), [paste]);

  const virtualizer = useVirtualizer({
    count: clips.length,
    getScrollElement: () => scrollRef.current,
    estimateSize: () => ROW_HEIGHT,
    overscan: 8,
    paddingStart: 6,
    paddingEnd: 6,
  });

  useEffect(() => {
    if (clips.length) virtualizer.scrollToIndex(selectedIndex, { align: "auto" });
  }, [virtualizer, selectedIndex, clips]);

  if (!clips.length) {
    const Icon = hasHistory ? SearchX : ClipboardList;
    return (
      <div className="flex flex-1 flex-col items-center justify-center gap-2 text-sm text-muted" role="status">
        <Icon size={28} strokeWidth={1.5} aria-hidden />
        {hasHistory ? t("emptyResults") : t("emptyHistory")}
      </div>
    );
  }

  return (
    <div ref={scrollRef} className="wincy-scroll min-h-0 flex-1 overflow-y-auto">
      <div id="clip-list" role="listbox" aria-label={t("clipList")} className="relative" style={{ height: virtualizer.getTotalSize() }}>
        {virtualizer.getVirtualItems().map((row) => (
          <div key={clips[row.index].id} role="presentation" className="absolute inset-x-0 top-0" style={{ transform: `translateY(${row.start}px)` }}>
            <ClipItem
              clip={clips[row.index]}
              index={row.index}
              selected={row.index === selectedIndex}
              onHover={select}
              onPick={pick}
            />
          </div>
        ))}
      </div>
    </div>
  );
}
