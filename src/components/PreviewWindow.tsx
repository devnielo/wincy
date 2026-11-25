import { Chip } from "@heroui/react";
import { FileX } from "lucide-react";
import { useEffect, useState, type ReactNode } from "react";
import { api, errorMessage, fileSrc, onEvent } from "../api";
import { useAppearance } from "../hooks/useAppearance";
import { formatDate, useI18n } from "../i18n";
import { parseColor, toHex, toHslString, toRgbString } from "../lib/color";
import { useClipboardStore } from "../store/useClipboardStore";
import type { Clip } from "../types";

function Body({ clip }: { clip: Clip }) {
  const { t } = useI18n();
  const text = clip.text ?? clip.preview;
  switch (clip.kind) {
    case "color": {
      const rgba = parseColor(text);
      if (!rgba) break;
      return (
        <div className="flex flex-col gap-4">
          <div className="h-48 rounded-lg border border-(--wincy-stroke) shadow-sm" style={{ background: toRgbString(rgba) }} />
          <dl className="grid grid-cols-[auto_1fr] gap-x-4 gap-y-1.5 font-mono text-sm select-text">
            <dt className="text-muted">HEX</dt>
            <dd>{toHex(rgba)}</dd>
            <dt className="text-muted">{rgba.a < 1 ? "RGBA" : "RGB"}</dt>
            <dd>{toRgbString(rgba)}</dd>
            <dt className="text-muted">{rgba.a < 1 ? "HSLA" : "HSL"}</dt>
            <dd>{toHslString(rgba)}</dd>
          </dl>
        </div>
      );
    }
    case "image":
      if (!clip.imagePath) break;
      if (!clip.ocrText) return <img src={fileSrc(clip.imagePath)} alt="" draggable={false} className="size-full rounded-md object-contain" />;
      return (
        <div className="flex flex-col gap-4">
          <img src={fileSrc(clip.imagePath)} alt="" draggable={false} className="max-h-56 w-full rounded-md object-contain" />
          <section aria-labelledby="ocr-heading" className="flex flex-col gap-1.5">
            <h2 id="ocr-heading" className="text-xs font-semibold tracking-wide text-muted uppercase">
              {t("preview.ocr")}
            </h2>
            <p className="text-[14px] leading-relaxed break-words whitespace-pre-wrap select-text">{clip.ocrText}</p>
          </section>
        </div>
      );
    case "files":
      return (
        <ul className="flex flex-col gap-1.5 text-[13px] select-text">
          {(clip.files ?? []).map((f) => (
            <li key={f} className="break-all">
              {f}
            </li>
          ))}
        </ul>
      );
    case "url":
      return <p className="text-[15px] break-all text-accent select-text">{text}</p>;
    case "code":
      // No wrapping: the scroll container scrolls horizontally.
      return <pre className="w-max font-mono text-[13px] leading-relaxed whitespace-pre select-text">{text}</pre>;
  }
  return <p className="text-[15px] leading-relaxed break-words whitespace-pre-wrap select-text">{text}</p>;
}

function Meta({ label, children }: { label: string; children: ReactNode }) {
  return (
    <>
      <dt className="text-muted">{label}</dt>
      <dd className="truncate">{children}</dd>
    </>
  );
}

function Details({ clip }: { clip: Clip }) {
  const { t, locale } = useI18n();
  const text = clip.text;
  return (
    <footer className="shrink-0 border-t border-(--wincy-stroke) px-4 py-2.5 text-xs">
      <dl className="grid grid-cols-[auto_1fr_auto_1fr] gap-x-3 gap-y-1">
        <Meta label={t("preview.source")}>{clip.sourceApp ?? t("preview.unknownApp")}</Meta>
        <Meta label={t("preview.uses")}>{clip.useCount}</Meta>
        <Meta label={t("preview.created")}>{formatDate(clip.createdAt, locale)}</Meta>
        <Meta label={t("preview.updated")}>{formatDate(clip.updatedAt, locale)}</Meta>
        {text != null && (
          <>
            <Meta label={t("preview.chars")}>{text.length.toLocaleString(locale)}</Meta>
            <Meta label={t("preview.lines")}>{text.split("\n").length.toLocaleString(locale)}</Meta>
          </>
        )}
        {clip.imageSize && (
          <Meta label={t("preview.size")}>
            {clip.imageSize[0]}×{clip.imageSize[1]}
          </Meta>
        )}
        {clip.files && clip.files.length > 1 && <Meta label={t("kind.files")}>{t("preview.files", { n: clip.files.length })}</Meta>}
      </dl>
      {(clip.pinned || clip.hasHtml) && (
        <div className="mt-2 flex gap-1.5">
          {clip.pinned && (
            <Chip size="sm" color="accent" variant="soft">
              {t("pinned")}
            </Chip>
          )}
          {clip.hasHtml && (
            <Chip size="sm" variant="soft">
              {t("rich")}
            </Chip>
          )}
        </div>
      )}
    </footer>
  );
}

/**
 * The `preview` window: the full content of the clip the palette selected (`preview-clip`).
 * It never focuses itself; the backend keeps it non-activating.
 */
export function PreviewWindow() {
  const { t, locale } = useI18n();
  useAppearance();
  const [id, setId] = useState<number | null>(null);
  const [version, setVersion] = useState(0);
  const [clip, setClip] = useState<Clip | null>(null);
  const [error, setError] = useState<string | null>(null);

  useEffect(() => {
    void useClipboardStore.getState().loadSettings();
    void useClipboardStore.getState().loadStatus();
    const subscriptions = [
      onEvent("preview-clip", (p) => setId(p.id)),
      // Re-copy, paste, pin and delete all change the shown clip; refetch it.
      onEvent("clips-changed", () => setVersion((v) => v + 1)),
      onEvent("settings-changed", (s) => useClipboardStore.getState().setSettings(s)),
      onEvent("status-changed", (s) => useClipboardStore.getState().setStatus(s)),
    ];
    return () => subscriptions.forEach((p) => void p.then((unlisten) => unlisten()));
  }, []);

  useEffect(() => {
    document.documentElement.lang = locale;
  }, [locale]);

  useEffect(() => {
    if (id === null) return;
    let cancelled = false;
    api.getClip(id).then(
      (c) => {
        if (cancelled) return;
        setClip(c);
        setError(null);
      },
      (e: unknown) => {
        if (cancelled) return;
        // A refetch of the shown clip that was just deleted keeps it up until the palette sends
        // the next one; a new id that fails shows the empty state.
        setClip((prev) => (prev?.id === id ? prev : null));
        setError(errorMessage(e));
      },
    );
    return () => {
      cancelled = true;
    };
  }, [id, version]);

  if (id === null) return null;

  return (
    <main className="wincy-surface flex h-full flex-col overflow-hidden text-foreground">
      {/* The previous clip stays up while the next one loads, so hovering down the list doesn't flash. */}
      {clip ? (
        <>
          <div key={clip.id} className="wincy-scroll min-h-0 flex-1 overflow-auto p-4">
            <Body clip={clip} />
          </div>
          <Details clip={clip} />
        </>
      ) : (
        <div className="flex flex-1 flex-col items-center justify-center gap-2 p-6 text-center text-sm text-muted" role="status">
          {error ? (
            <>
              <FileX size={28} strokeWidth={1.5} aria-hidden />
              {t("preview.gone")}
            </>
          ) : (
            t("preview.loading")
          )}
        </div>
      )}
    </main>
  );
}
