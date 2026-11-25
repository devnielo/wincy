import { Chip } from "@heroui/react";
import { ArrowDownToLine, ClipboardPaste, Eye, EyeOff, Pause, Pin, PinOff, Play, Settings, Trash2, Type, Zap } from "lucide-react";
import { useI18n } from "../i18n";
import { selectSelected, selectSelectedSnippet, useClipboardStore } from "../store/useClipboardStore";
import { ActionButton } from "./ActionButton";

/** Every palette shortcut as a button (contract: "Mouse-first actions"). */
export function Footer() {
  const { t } = useI18n();
  const error = useClipboardStore((s) => s.error);
  const paused = useClipboardStore((s) => s.paused);
  const showPreview = useClipboardStore((s) => s.showPreview);
  const hasUpdate = useClipboardStore((s) => s.update !== null);
  const selected = useClipboardStore(selectSelected);
  const hasSnippet = useClipboardStore((s) => !!selectSelectedSnippet(s));
  const s = useClipboardStore.getState();
  const none = !selected;
  // Live index: Footer doesn't re-render when only selectedIndex changes (same clip, new row).
  const paste = (plain: boolean) => {
    const st = useClipboardStore.getState();
    void st.paste(st.selectedIndex, plain);
  };

  return (
    <footer className="shrink-0 border-t border-(--wincy-stroke)">
      {error && (
        <p role="alert" className="truncate px-3 pt-1.5 text-[11.5px] text-danger" title={error}>
          {error}
        </p>
      )}
      <div className="flex h-10 items-center gap-0.5 px-1.5">
        <ActionButton
          icon={ClipboardPaste}
          label={t("a.paste")}
          text={t("a.paste")}
          keys="Enter"
          isDisabled={none && !hasSnippet}
          onPress={() => paste(false)}
        />
        <ActionButton
          icon={Type}
          label={t("a.plainLong")}
          text={t("a.plain")}
          keys="Shift Enter"
          isDisabled={none}
          onPress={() => paste(true)}
        />
        <ActionButton
          icon={Zap}
          label={t("a.actionsLong")}
          text={t("a.actions")}
          keys="Ctrl K"
          isDisabled={none}
          onPress={() => s.setActionsOpen(true)}
        />
        <ActionButton
          icon={selected?.pinned ? PinOff : Pin}
          label={selected?.pinned ? t("a.unpin") : t("a.pin")}
          keys="Ctrl P"
          isDisabled={none}
          onPress={() => void s.togglePin()}
        />
        <ActionButton
          icon={Trash2}
          label={t("a.delete")}
          keys="Ctrl Delete"
          isDisabled={none}
          onPress={() => void s.deleteSelected()}
          className="hover:text-danger"
        />
        <span className="flex-1" />
        {hasUpdate && (
          <button type="button" tabIndex={-1} onMouseDown={(e) => e.preventDefault()} onClick={() => s.setView("settings")} className="mr-1 shrink-0 rounded-full">
            <Chip size="sm" color="accent" variant="soft" className="cursor-pointer">
              <ArrowDownToLine size={11} aria-hidden />
              <Chip.Label>{t("updateAvailable")}</Chip.Label>
            </Chip>
          </button>
        )}
        <ActionButton
          icon={paused ? Play : Pause}
          label={paused ? t("a.resume") : t("a.pause")}
          text={paused ? t("a.paused") : undefined}
          pressed={paused}
          onPress={() => void s.setPaused(!paused)}
          className={paused ? "text-warning!" : ""}
        />
        <ActionButton
          icon={showPreview ? Eye : EyeOff}
          label={showPreview ? t("a.previewHide") : t("a.previewShow")}
          keys="Ctrl Space"
          pressed={showPreview}
          onPress={s.togglePreview}
        />
        <ActionButton icon={Settings} label={t("settings")} keys="Ctrl ," onPress={() => s.setView("settings")} />
      </div>
    </footer>
  );
}
