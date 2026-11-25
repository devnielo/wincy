import { Kbd } from "@heroui/react";
import { Pin, PinOff, Trash2 } from "lucide-react";
import { memo } from "react";
import { fileSrc } from "../api";
import { relativeTime, useI18n, type Translate } from "../i18n";
import { parseColor, toRgbString } from "../lib/color";
import { useClipboardStore } from "../store/useClipboardStore";
import type { ClipSummary } from "../types";
import { ActionButton } from "./ActionButton";
import { KIND_ICONS } from "./kindIcons";

export const ROW_HEIGHT = 44;

const fileName = (path: string) => path.split(/[\\/]/).filter(Boolean).pop() ?? path;

function label(clip: ClipSummary, t: Translate): string {
  if (clip.kind === "image") return clip.imageSize ? `${t("image")} ${clip.imageSize[0]}×${clip.imageSize[1]}` : t("image");
  if (clip.kind === "files" && clip.files?.length) {
    const more = clip.files.length - 1;
    return more > 0 ? `${fileName(clip.files[0])}  ${t("moreFiles", { n: more })}` : fileName(clip.files[0]);
  }
  return clip.preview.replace(/\s+/g, " ").trim();
}

function Leading({ clip }: { clip: ClipSummary }) {
  if (clip.kind === "color") {
    const rgba = parseColor(clip.preview);
    if (rgba) {
      return (
        <span
          className="size-5 shrink-0 rounded-md border border-(--wincy-stroke) shadow-xs"
          style={{ background: toRgbString(rgba) }}
          aria-hidden
        />
      );
    }
  }
  if (clip.kind === "image" && clip.thumbPath) {
    return (
      <img
        src={fileSrc(clip.thumbPath)}
        alt=""
        draggable={false}
        className="size-7 shrink-0 rounded-md border border-(--wincy-stroke) object-cover"
      />
    );
  }
  const Icon = KIND_ICONS[clip.kind];
  return <Icon size={16} className="mx-0.5 shrink-0 text-muted" aria-hidden />;
}

/** Pin/Delete on the selected (= hovered) row; they act on the selected clip. */
function RowActions({ pinned }: { pinned: boolean }) {
  const { t } = useI18n();
  const s = useClipboardStore.getState();
  return (
    // The row's click pastes: keep button clicks from reaching it.
    <span className="-my-1 flex shrink-0 gap-0.5" onClick={(e) => e.stopPropagation()}>
      <ActionButton
        icon={pinned ? PinOff : Pin}
        label={pinned ? t("a.unpin") : t("a.pin")}
        keys="Ctrl P"
        onPress={() => void s.togglePin()}
      />
      <ActionButton icon={Trash2} label={t("a.delete")} keys="Ctrl Delete" onPress={() => void s.deleteSelected()} className="hover:text-danger" />
    </span>
  );
}

interface Props {
  clip: ClipSummary;
  index: number;
  selected: boolean;
  onHover: (index: number) => void;
  onPick: (index: number) => void;
}

export const ClipItem = memo(function ClipItem({ clip, index, selected, onHover, onPick }: Props) {
  const { t, locale } = useI18n();
  return (
    <div
      id={`clip-${clip.id}`}
      role="option"
      aria-selected={selected}
      // Keep keyboard focus in the search input.
      onMouseDown={(e) => e.preventDefault()}
      onMouseMove={() => !selected && onHover(index)}
      onClick={() => onPick(index)}
      style={{ height: ROW_HEIGHT }}
      className={`relative mx-1.5 flex cursor-default items-center gap-3 rounded-md px-2.5 ${
        selected ? "bg-(--wincy-selected) before:absolute before:inset-y-3 before:left-0 before:w-[3px] before:rounded-full before:bg-accent" : ""
      }`}
    >
      <span className="flex w-7 shrink-0 justify-center">
        <Leading clip={clip} />
      </span>
      <span className={`min-w-0 flex-1 truncate ${clip.kind === "code" ? "font-mono text-[12.5px]" : "text-[13.5px]"}`}>
        {label(clip, t)}
      </span>
      {clip.pinned && !selected && <Pin size={13} className="shrink-0 rotate-45 text-accent" aria-label={t("pinned")} />}
      <span className="max-w-36 shrink-0 truncate text-xs text-muted">
        {clip.sourceApp ? `${clip.sourceApp.replace(/\.exe$/i, "")} · ` : ""}
        {relativeTime(clip.updatedAt, locale)}
      </span>
      {selected && <RowActions pinned={clip.pinned} />}
      {index < 9 && (
        <Kbd variant="light" className="shrink-0 text-[11px] text-muted [word-spacing:normal]" aria-hidden>
          <Kbd.Content>Alt {index + 1}</Kbd.Content>
        </Kbd>
      )}
    </div>
  );
});
