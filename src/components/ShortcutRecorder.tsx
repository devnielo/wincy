import { Button, Kbd } from "@heroui/react";
import { X } from "lucide-react";
import { useEffect, useRef, useState, type KeyboardEvent } from "react";
import { onEvent } from "../api";
import { useI18n } from "../i18n";
import { acceleratorFromEvent } from "../lib/accelerator";

export const keycap = (shortcut: string) => shortcut.split("+").join(" + ");

interface Props {
  labelId: string;
  value: string | null;
  /** Saves the recorded combo (null = removed); resolves to an error message, or null on success. */
  onChange: (shortcut: string | null) => Promise<string | null>;
  /** Shows a remove button while a shortcut is set. */
  clearable?: boolean;
  autoFocus?: boolean;
}

export function ShortcutRecorder({ labelId, value, onChange, clearable, autoFocus }: Props) {
  const { t } = useI18n();
  const [recording, setRecording] = useState(!!autoFocus);
  const [error, setError] = useState<string | null>(null);
  const ref = useRef<HTMLDivElement>(null);

  useEffect(() => {
    if (autoFocus) ref.current?.focus(); // React's autoFocus only covers form controls
  }, [autoFocus]);

  // A combo Wincy already registered never reaches the WebView; the backend reports the press.
  useEffect(() => {
    if (!recording) return;
    const unlisten = onEvent("shortcut-in-use", () => {
      setRecording(false);
      setError(t("s.shortcutInUse"));
    });
    return () => void unlisten.then((off) => off());
  }, [recording, t]);

  function start() {
    setError(null);
    setRecording(true);
  }

  const save = (shortcut: string | null) => void onChange(shortcut).then(setError);

  // A div rather than <button>: a native button fires click on Space keyup, which would
  // restart recording right after capturing a combo like Ctrl+Space.
  function onKeyDown(e: KeyboardEvent) {
    if (!recording) {
      if (e.key === "Enter" || e.key === " ") {
        e.preventDefault();
        start();
      }
      return;
    }
    if (e.key === "Tab" && !e.ctrlKey && !e.altKey && !e.metaKey) {
      setRecording(false); // let focus move on
      return;
    }
    e.preventDefault();
    e.stopPropagation(); // Esc here cancels recording, it must not leave the view
    if (e.key === "Escape") {
      setRecording(false);
      return;
    }
    const accelerator = acceleratorFromEvent(e);
    if (!accelerator) return; // keep waiting for a complete combo
    setRecording(false);
    // Re-recording the same combo is allowed: it retries a registration that failed earlier.
    save(accelerator);
  }

  return (
    <div className="flex flex-col items-end gap-1">
      <div className="flex items-center gap-1">
        <div
          role="button"
          ref={ref}
          tabIndex={0}
          aria-labelledby={labelId}
          aria-pressed={recording}
          aria-describedby={error ? `${labelId}-error` : undefined}
          onClick={start}
          onKeyDown={onKeyDown}
          onBlur={() => setRecording(false)}
          className={`flex h-8 min-w-32 cursor-pointer items-center justify-center rounded-lg border px-3 text-sm outline-none select-none focus-visible:ring-2 focus-visible:ring-accent ${
            recording ? "border-accent bg-accent/10 text-accent" : "border-(--wincy-stroke) bg-field text-field-foreground"
          }`}
        >
          {recording ? (
            t("s.recording")
          ) : value ? (
            <Kbd variant="light" className="text-inherit [word-spacing:normal]">
              <Kbd.Content>{keycap(value)}</Kbd.Content>
            </Kbd>
          ) : (
            <span className="text-muted">{t("s.noShortcut")}</span>
          )}
        </div>
        {clearable && value && (
          <Button isIconOnly size="sm" variant="ghost" aria-label={t("s.removeShortcut")} onPress={() => save(null)} className="size-8 min-w-8">
            <X size={14} />
          </Button>
        )}
      </div>
      {error && (
        <p id={`${labelId}-error`} role="alert" className="max-w-64 text-right text-xs text-danger">
          {error}
        </p>
      )}
    </div>
  );
}
