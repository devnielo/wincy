import { Button } from "@heroui/react";
import { relaunch } from "@tauri-apps/plugin-process";
import { useState } from "react";
import { checkForUpdate, errorMessage, logError, openExternal } from "../api";
import { useI18n } from "../i18n";
import { useClipboardStore } from "../store/useClipboardStore";

const DOWNLOAD_URL = "https://github.com/devnielo/wincy/releases/latest";

type Status =
  | { kind: "idle" }
  | { kind: "checking" }
  | { kind: "upToDate" }
  | { kind: "downloading"; pct: number | null }
  | { kind: "installing" }
  | { kind: "error"; message: string; detail?: string };

export function UpdateSection() {
  const { t, locale } = useI18n();
  const update = useClipboardStore((s) => s.update);
  const setUpdate = useClipboardStore((s) => s.setUpdate);
  const installing = useClipboardStore((s) => s.installing);
  const setInstalling = useClipboardStore((s) => s.setInstalling);
  // The updater can't replace a portable exe in place: send the user to the release page instead.
  const portable = useClipboardStore((s) => s.status?.portable ?? false);
  const [status, setStatus] = useState<Status>({ kind: "idle" });
  const busy =
    installing || status.kind === "checking" || status.kind === "downloading" || status.kind === "installing";

  async function checkNow() {
    setStatus({ kind: "checking" });
    try {
      const found = await checkForUpdate();
      setUpdate(found);
      setStatus(found ? { kind: "idle" } : { kind: "upToDate" });
    } catch (e) {
      // The plugin's raw error ("Could not fetch a valid release JSON…") is English-only and cryptic; keep it as a tooltip.
      setStatus({ kind: "error", message: t("s.checkFailed"), detail: errorMessage(e) });
    }
  }

  async function install() {
    // Settings may have been closed and reopened mid-download; this state survives that.
    if (!update || useClipboardStore.getState().installing) return;
    setInstalling(true);
    let total = 0;
    let done = 0;
    setStatus({ kind: "downloading", pct: null });
    try {
      await update.downloadAndInstall((event) => {
        if (event.event === "Started") total = event.data.contentLength ?? 0;
        else if (event.event === "Progress") {
          done += event.data.chunkLength;
          setStatus({ kind: "downloading", pct: total ? Math.min(1, done / total) : null });
        } else setStatus({ kind: "installing" });
      });
      // On Windows the NSIS installer usually exits the app before this runs.
      await relaunch();
    } catch (e) {
      logError(`update install failed: ${errorMessage(e)}`);
      setInstalling(false);
      setStatus({ kind: "error", message: errorMessage(e) });
    }
  }

  const pct = (n: number | null) =>
    n === null ? "" : new Intl.NumberFormat(locale, { style: "percent", maximumFractionDigits: 0 }).format(n);

  const label =
    status.kind === "checking"
      ? t("s.checking")
      : status.kind === "downloading"
        ? t("s.downloading", { pct: pct(status.pct) })
        : status.kind === "installing" || installing
          ? t("s.installing")
          : update
            ? t("s.install", { version: update.version })
            : t("s.checkNow");

  if (portable && update) {
    return (
      <div className="flex items-center justify-between gap-3">
        <p className="text-xs text-muted" role="status">
          {t("s.portableUpdate", { version: update.version })}
        </p>
        <Button
          size="sm"
          variant="primary"
          className="shrink-0"
          onPress={() => openExternal(DOWNLOAD_URL).catch((e: unknown) => setStatus({ kind: "error", message: errorMessage(e) }))}
        >
          {t("s.downloadPage")}
        </Button>
      </div>
    );
  }

  return (
    <div className="flex items-center justify-between gap-3">
      <p
        className={`text-xs ${status.kind === "error" ? "text-danger" : "text-muted"}`}
        role="status"
        title={status.kind === "error" ? status.detail : undefined}
      >
        {status.kind === "error" ? status.message : status.kind === "upToDate" ? t("s.upToDate") : ""}
      </p>
      <Button size="sm" variant={update ? "primary" : "secondary"} isPending={busy} onPress={() => void (update ? install() : checkNow())}>
        {label}
      </Button>
    </div>
  );
}
