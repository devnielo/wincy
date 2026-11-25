import { Kbd } from "@heroui/react";
import { useState, type KeyboardEvent } from "react";
import { useI18n } from "../i18n";
import { acceleratorFromEvent } from "../lib/accelerator";
import { useClipboardStore } from "../store/useClipboardStore";

export function ShortcutRecorder({ labelId }: { labelId: string }) {
  const { t } = useI18n();
  const shortcut = useClipboardStore((s) => s.settings?.shortcut ?? "");
  const saveSettings = useClipboardStore((s) => s.saveSettings);
  const [recording, setRecording] = useState(false);
  const [error, setError] = useState<string | null>(null);

  function start() {
    setError(null);
    setRecording(true);
  }

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
    e.stopPropagation(); // Esc here cancels recording, it must not leave the settings view
    if (e.key === "Escape") {
      setRecording(false);
      return;
    }
    const accelerator = acceleratorFromEvent(e);
    if (!accelerator) return; // keep waiting for a complete combo
    setRecording(false);
    // Re-recording the same combo is allowed: it retries a registration that failed earlier.
    void saveSettings({ shortcut: accelerator }).then(setError);
  }

  return (
    <div className="flex flex-col items-end gap-1">
      <div
        role="button"
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
        ) : (
          <Kbd variant="light" className="text-inherit [word-spacing:normal]">
            <Kbd.Content>{shortcut.split("+").join(" + ")}</Kbd.Content>
          </Kbd>
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
