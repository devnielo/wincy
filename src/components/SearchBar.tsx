import { Button, Input } from "@heroui/react";
import { Search, Settings } from "lucide-react";
import { useEffect, useRef } from "react";
import { useI18n } from "../i18n";
import { KIND_FILTERS, selectSelected, useClipboardStore } from "../store/useClipboardStore";
import { ActionButton } from "./ActionButton";
import { KIND_ICONS } from "./kindIcons";

export function SearchBar() {
  const { t } = useI18n();
  const query = useClipboardStore((s) => s.query);
  const setQuery = useClipboardStore((s) => s.setQuery);
  const kindFilter = useClipboardStore((s) => s.kindFilter);
  const setKind = useClipboardStore((s) => s.setKind);
  const setView = useClipboardStore((s) => s.setView);
  const shownCount = useClipboardStore((s) => s.shownCount);
  const activeId = useClipboardStore((s) => selectSelected(s)?.id);
  const inputRef = useRef<HTMLInputElement>(null);

  useEffect(() => inputRef.current?.focus(), [shownCount]);

  useEffect(() => {
    const focus = () => inputRef.current?.focus();
    window.addEventListener("focus", focus);
    return () => window.removeEventListener("focus", focus);
  }, []);

  return (
    // "deep": the whole row drags the window except inputs and buttons (Tauri skips clickable elements).
    <div data-tauri-drag-region="deep" className="flex h-12 shrink-0 items-center gap-2 border-b border-(--wincy-stroke) pr-2 pl-3.5">
      <Search size={16} className="shrink-0 text-muted" aria-hidden />
      {/* Plain Input rather than SearchField: SearchField swallows Esc/Enter, which belong to the palette keymap. */}
      <Input
        ref={inputRef}
        autoFocus
        value={query}
        onChange={(e) => setQuery(e.target.value)}
        placeholder={t("searchPlaceholder")}
        aria-label={t("searchLabel")}
        role="combobox"
        aria-expanded
        aria-controls="clip-list"
        aria-autocomplete="list"
        aria-activedescendant={activeId === undefined ? undefined : `clip-${activeId}`}
        spellCheck={false}
        autoComplete="off"
        className="h-9 min-w-0 flex-1 border-0 bg-transparent px-1 text-[15px] shadow-none outline-none focus:ring-0 focus-visible:ring-0"
      />
      <div role="group" aria-label={t("filterBy", { kind: t(`kind.${kindFilter}`) })} className="flex shrink-0 gap-0.5">
        {KIND_FILTERS.map((kind) => {
          const Icon = KIND_ICONS[kind];
          const active = kind === kindFilter;
          return (
            <Button
              key={kind}
              isIconOnly
              size="sm"
              variant={active ? "secondary" : "ghost"}
              aria-label={t(`kind.${kind}`)}
              aria-pressed={active}
              preventFocusOnPress
              onPress={() => setKind(kind)}
              className={`size-7 min-w-7 ${active ? "text-accent" : "text-muted"}`}
            >
              <Icon size={15} />
            </Button>
          );
        })}
      </div>
      <span className="h-5 w-px shrink-0 bg-(--wincy-stroke)" aria-hidden />
      <ActionButton icon={Settings} label={t("settings")} keys="Ctrl ," onPress={() => setView("settings")} />
    </div>
  );
}
