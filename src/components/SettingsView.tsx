import { Alert, AlertDialog, Button, Description, Label, ListBox, NumberField, Select, Switch, toast } from "@heroui/react";
import { ArrowLeft, Check, ClipboardCopy, Coffee, Download, ExternalLink, FileText, FolderOpen, Trash2, Upload } from "lucide-react";
import { useEffect, useRef, useState, type ReactNode } from "react";
import markSmall from "../../assets/brand/mark-small.svg";
import { api, appVersion, chooseExportPath, chooseImportPath, errorMessage, openExternal } from "../api";
import { useI18n, type MessageKey } from "../i18n";
import { useClipboardStore } from "../store/useClipboardStore";
import type { Language, Settings } from "../types";
import { AccentSwatches, ThemeCards } from "./AppearancePicker";
import { IgnoredApps } from "./IgnoredApps";
import { ShortcutRecorder } from "./ShortcutRecorder";
import { SnippetSettings, type Confirm } from "./SnippetSettings";
import { UpdateSection } from "./UpdateSection";

const REPO_URL = "https://github.com/devnielo/wincy";
const SUPPORT_URL = "https://buymeacoffee.com/devnielo";
const LANGUAGES: Language[] = ["system", "en", "es"];
const EXPIRE_DAYS = [0, 1, 7, 30, 90];

function Section({ title, children }: { title: string; children: ReactNode }) {
  return (
    <section className="flex flex-col gap-1">
      <h2 className="px-1 pb-1 text-xs font-semibold tracking-wide text-muted uppercase">{title}</h2>
      <div className="flex flex-col divide-y divide-(--wincy-stroke) rounded-lg border border-(--wincy-stroke) bg-(--wincy-card)">
        {children}
      </div>
    </section>
  );
}

function Row({ id, label, hint, children }: { id: string; label: string; hint?: string; children: ReactNode }) {
  return (
    <div className="flex min-h-12 items-center justify-between gap-4 px-3 py-2">
      <div className="flex min-w-0 flex-col">
        <span id={id} className="text-sm">
          {label}
        </span>
        {hint && <span className="text-xs text-muted">{hint}</span>}
      </div>
      {children}
    </div>
  );
}

type BoolKey = { [K in keyof Settings]: Settings[K] extends boolean ? K : never }[keyof Settings];

function Toggle({ field, label, hint }: { field: BoolKey; label: MessageKey; hint?: MessageKey }) {
  const { t } = useI18n();
  const value = useClipboardStore((s) => s.settings?.[field] ?? false);
  const saveSettings = useClipboardStore((s) => s.saveSettings);
  const setError = useClipboardStore((s) => s.setError);
  return (
    <Switch
      isSelected={value}
      onChange={(on) => void saveSettings({ [field]: on }).then(setError)}
      className="flex min-h-12 w-full items-center justify-between gap-4 px-3 py-2"
    >
      <Switch.Content className="flex w-full items-center justify-between gap-4">
        <span className="flex min-w-0 flex-col">
          <Label className="text-sm font-normal">{t(label)}</Label>
          {hint && <Description className="text-xs font-normal text-muted">{t(hint)}</Description>}
        </span>
        <Switch.Control>
          <Switch.Thumb />
        </Switch.Control>
      </Switch.Content>
    </Switch>
  );
}

function Choice<T extends string>({ labelId, value, options, label, onChange }: {
  labelId: string;
  value: T;
  options: T[];
  label: (option: T) => string;
  onChange: (value: T) => void;
}) {
  return (
    <Select
      aria-labelledby={labelId}
      value={value}
      onChange={(key) => key !== null && onChange(key as T)}
      className="w-40"
    >
      <Select.Trigger>
        <Select.Value />
        <Select.Indicator />
      </Select.Trigger>
      <Select.Popover>
        <ListBox>
          {options.map((o) => (
            <ListBox.Item key={o} id={o} textValue={label(o)}>
              {label(o)}
              <ListBox.ItemIndicator />
            </ListBox.Item>
          ))}
        </ListBox>
      </Select.Popover>
    </Select>
  );
}

function ShortcutBanner({ shortcut }: { shortcut: string }) {
  const { t } = useI18n();
  const error = useClipboardStore((s) => s.status?.shortcutError);
  if (!error) return null;
  return (
    <Alert status="danger" role="alert" className="rounded-lg border border-danger/30 bg-danger-soft shadow-none">
      <Alert.Indicator />
      <Alert.Content>
        <Alert.Title>{t("s.shortcutTaken", { shortcut: shortcut.split("+").join(" + ") })}</Alert.Title>
        <Alert.Description className="text-xs text-foreground/80" title={error}>
          {t("s.shortcutTakenBody")}
        </Alert.Description>
      </Alert.Content>
    </Alert>
  );
}

function ConflictsBanner() {
  const { t } = useI18n();
  const conflicts = useClipboardStore((s) => s.status?.shortcutConflicts);
  if (!conflicts?.length) return null;
  return (
    <Alert status="warning" className="rounded-lg border border-warning/30 bg-warning-soft shadow-none">
      <Alert.Indicator />
      <Alert.Content>
        <Alert.Title>{t("s.conflictsTitle")}</Alert.Title>
        <Alert.Description className="text-xs text-foreground/80">
          {t("s.conflictsBody")}
          <ul className="mt-1 list-disc pl-4">
            {conflicts.map((c) => (
              <li key={c}>
                {c.replace(/ \((pin|snippet)\)$/, (_, k) => ` (${t(k === "pin" ? "s.conflictPin" : "s.conflictSnippet")})`)}
              </li>
            ))}
          </ul>
        </Alert.Description>
      </Alert.Content>
    </Alert>
  );
}

/** Export / import of clips and snippets (`*.wincy.json`). */
function Backup() {
  const { t } = useI18n();
  const [includeImages, setIncludeImages] = useState(true);
  const [busy, setBusy] = useState<"export" | "import" | null>(null);

  async function run<T>(kind: "export" | "import", pick: () => Promise<string | null>, go: (path: string) => Promise<T>, done: (r: T) => void) {
    setBusy(kind);
    try {
      const path = await pick();
      if (path) done(await go(path));
    } catch (e) {
      toast.danger(kind === "export" ? t("s.exportFailed") : t("s.importFailed"), { description: errorMessage(e), timeout: 0 });
    } finally {
      setBusy(null);
    }
  }

  return (
    <div className="flex flex-col gap-2 px-3 py-2.5">
      <div className="flex flex-col">
        <span className="text-sm">{t("s.backup")}</span>
        <span className="text-xs text-muted">{t("s.backupHint")}</span>
      </div>
      <div className="flex flex-wrap items-center gap-2">
        <Button
          size="sm"
          variant="secondary"
          isPending={busy === "export"}
          isDisabled={busy !== null}
          onPress={() =>
            void run("export", chooseExportPath, (path) => api.exportHistory(path, includeImages), (r) =>
              toast.success(t("s.exportDone"), { description: t("s.exportBody", { clips: r.clips, snippets: r.snippets }) }),
            )
          }
        >
          <Upload size={14} />
          {t("s.export")}
        </Button>
        <Button
          size="sm"
          variant="secondary"
          isPending={busy === "import"}
          isDisabled={busy !== null}
          onPress={() =>
            void run("import", chooseImportPath, api.importHistory, (r) =>
              toast.success(t("s.importDone"), { description: t("s.importBody", { ...r }) }),
            )
          }
        >
          <Download size={14} />
          {t("s.import")}
        </Button>
        <Switch isSelected={includeImages} onChange={setIncludeImages} size="sm" className="ml-auto">
          <Switch.Content className="flex items-center gap-2">
            <Switch.Control>
              <Switch.Thumb />
            </Switch.Control>
            <Label className="text-xs font-normal">{t("s.includeImages")}</Label>
          </Switch.Content>
        </Switch>
      </div>
    </div>
  );
}

function Diagnostics({ run }: { run: (action: () => Promise<unknown>) => void }) {
  const { t } = useI18n();
  const [copied, setCopied] = useState(false);

  useEffect(() => {
    if (!copied) return;
    const timer = setTimeout(() => setCopied(false), 2000);
    return () => clearTimeout(timer);
  }, [copied]);

  return (
    <div className="flex flex-col gap-2 px-3 py-2.5">
      <div className="flex flex-col">
        <span className="text-sm">{t("s.diagnostics")}</span>
        <span className="text-xs text-muted">{t("s.diagnosticsHint")}</span>
      </div>
      <div className="flex flex-wrap gap-2">
        <Button size="sm" variant="secondary" onPress={() => run(() => api.copyDiagnostics().then(() => setCopied(true)))}>
          {copied ? <Check size={14} className="text-success" /> : <ClipboardCopy size={14} />}
          <span aria-live="polite">{copied ? t("s.copied") : t("s.copyDiagnostics")}</span>
        </Button>
        <Button size="sm" variant="secondary" onPress={() => run(api.openLogDir)}>
          <FileText size={14} />
          {t("s.openLogDir")}
        </Button>
      </div>
    </div>
  );
}

export function SettingsView() {
  const { t } = useI18n();
  const settings = useClipboardStore((s) => s.settings);
  const saveSettings = useClipboardStore((s) => s.saveSettings);
  const error = useClipboardStore((s) => s.error);
  const setError = useClipboardStore((s) => s.setError);
  const setView = useClipboardStore((s) => s.setView);
  const portable = useClipboardStore((s) => s.status?.portable ?? false);
  const store = useClipboardStore((s) => s.status?.store ?? false);
  const [version, setVersion] = useState("");
  // Kept after closing so the dialog keeps its text during the exit animation.
  const [confirm, setConfirm] = useState<Confirm | null>(null);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const ask = (c: Confirm) => {
    setConfirm(c);
    setConfirmOpen(true);
  };
  const rootRef = useRef<HTMLDivElement>(null);

  useEffect(() => {
    rootRef.current?.focus();
    appVersion().then(setVersion, () => setVersion("?"));
  }, []);

  useEffect(() => {
    // Overlays and the shortcut recorder consume their own Esc (preventDefault); the
    // confirm dialog closes itself on Esc.
    const onKeyDown = (e: KeyboardEvent) =>
      e.key === "Escape" && !e.defaultPrevented && !confirmOpen && setView("palette");
    window.addEventListener("keydown", onKeyDown);
    return () => window.removeEventListener("keydown", onKeyDown);
  }, [setView, confirmOpen]);

  const run = (action: () => Promise<unknown>) =>
    void action().then(
      () => setError(null),
      (e: unknown) => setError(errorMessage(e)),
    );
  const save = (patch: Partial<Settings>) => void saveSettings(patch).then(setError);

  return (
    <div ref={rootRef} tabIndex={-1} className="flex min-h-0 flex-1 flex-col outline-none">
      <header className="flex h-12 shrink-0 items-center gap-2 border-b border-(--wincy-stroke) px-2">
        <Button isIconOnly size="sm" variant="ghost" aria-label={t("back")} onPress={() => setView("palette")}>
          <ArrowLeft size={16} />
        </Button>
        <h1 className="text-sm font-semibold">{t("settings")}</h1>
        {error && (
          <p role="alert" className="ml-auto min-w-0 truncate text-xs text-danger" title={error}>
            {error}
          </p>
        )}
      </header>

      {settings && (
        <div className="wincy-scroll flex min-h-0 flex-1 flex-col gap-4 overflow-y-auto p-4">
          <ShortcutBanner shortcut={settings.shortcut} />
          <ConflictsBanner />

          <Section title={t("s.general")}>
            <Row id="set-shortcut" label={t("s.shortcut")} hint={t("s.shortcutHint")}>
              <ShortcutRecorder
                labelId="set-shortcut"
                value={settings.shortcut}
                onChange={(shortcut) => saveSettings({ shortcut: shortcut ?? settings.shortcut })}
              />
            </Row>
            {/* Portable copies never register autostart: the exe may live on a removable drive. */}
            {!portable && <Toggle field="launchAtStartup" label="s.launchAtStartup" />}
            <Toggle field="pasteDirectly" label="s.pasteDirectly" hint="s.pasteDirectlyHint" />
          </Section>

          <Section title={t("s.snippets")}>
            <SnippetSettings ask={ask} />
          </Section>

          <Section title={t("s.appearance")}>
            <div className="flex flex-col gap-2.5 px-3 py-3">
              <span id="set-theme" className="text-sm">
                {t("s.theme")}
              </span>
              <ThemeCards labelId="set-theme" settings={settings} save={save} />
            </div>
            <div className="flex flex-col gap-2.5 px-3 py-3">
              <span className="text-sm">
                <span id="set-accent">{t("s.accent")}</span>
                <span className="text-muted"> · {t(`accent.${settings.accent}`)}</span>
              </span>
              <AccentSwatches labelId="set-accent" settings={settings} save={save} />
            </div>
            <Row id="set-lang" label={t("s.language")}>
              <Choice
                labelId="set-lang"
                value={settings.language}
                options={LANGUAGES}
                label={(o) => t(`lang.${o}`)}
                onChange={(language) => save({ language })}
              />
            </Row>
          </Section>

          <Section title={t("s.history")}>
            <Row id="set-max" label={t("s.maxItems")} hint={t("s.maxItemsHint")}>
              <NumberField
                aria-labelledby="set-max"
                value={settings.maxItems}
                minValue={50}
                maxValue={10000}
                step={50}
                onChange={(n) => {
                  if (!Number.isFinite(n) || n === settings.maxItems) return;
                  // Lowering deletes history right away, so it needs a confirmation.
                  if (n > settings.maxItems) save({ maxItems: n });
                  else
                    ask({
                      title: t("s.trimTitle", { n }),
                      body: t("s.trimBody"),
                      action: t("s.trim"),
                      run: () => save({ maxItems: n }),
                    });
                }}
                isWheelDisabled
                className="w-36"
              >
                <NumberField.Group>
                  <NumberField.DecrementButton />
                  <NumberField.Input className="w-16 text-center" />
                  <NumberField.IncrementButton />
                </NumberField.Group>
              </NumberField>
            </Row>
            <Row id="set-expire" label={t("s.expireDays")} hint={t("s.expireDaysHint")}>
              <Choice
                labelId="set-expire"
                value={String(settings.expireDays)}
                // A hand-edited value outside the presets still shows as itself.
                options={[...new Set([...EXPIRE_DAYS, settings.expireDays])].sort((a, b) => a - b).map(String)}
                label={(o) => (o === "0" ? t("expire.never") : o === "1" ? t("expire.day") : t("expire.days", { n: o }))}
                onChange={(value) => {
                  const days = Number(value);
                  const old = settings.expireDays;
                  // A shorter window deletes history right away, so it needs a confirmation.
                  if (days === 0 || (old !== 0 && days > old)) save({ expireDays: days });
                  else
                    ask({
                      title: t("s.expireTitle"),
                      body: t("s.trimBody"),
                      action: t("s.trim"),
                      run: () => save({ expireDays: days }),
                    });
                }}
              />
            </Row>
            <Toggle field="captureImages" label="s.captureImages" />
            <Toggle field="ocrImages" label="s.ocrImages" hint="s.ocrImagesHint" />
          </Section>

          <Section title={t("s.privacy")}>
            <Toggle field="ignoreSecrets" label="s.ignoreSecrets" hint="s.ignoreSecretsHint" />
            <div className="flex flex-col gap-2 px-3 py-2.5">
              <div className="flex flex-col">
                <span className="text-sm">{t("s.ignoredApps")}</span>
                <span className="text-xs text-muted">{t("s.ignoredAppsHint")}</span>
              </div>
              <IgnoredApps />
            </div>
          </Section>

          <Section title={t("s.updates")}>
            {store ? (
              <p className="px-3 py-2.5 text-sm">{t("s.storeUpdates")}</p>
            ) : (
              <>
                <Toggle field="checkUpdates" label="s.checkUpdates" />
                <div className="px-3 py-2">
                  <UpdateSection />
                </div>
              </>
            )}
          </Section>

          <Section title={t("s.data")}>
            <div className="flex flex-wrap gap-2 px-3 py-2.5">
              <Button size="sm" variant="secondary" onPress={() => run(api.openDataDir)}>
                <FolderOpen size={14} />
                {t("s.openDataDir")}
              </Button>
              <Button
                size="sm"
                variant="danger-soft"
                onPress={() =>
                  ask({
                    title: t("s.clearTitle"),
                    body: t("s.clearBody"),
                    action: t("s.clear"),
                    run: () => run(api.clearHistory),
                  })
                }
              >
                <Trash2 size={14} />
                {t("s.clearHistory")}
              </Button>
            </div>
            <Backup />
            <Diagnostics run={run} />
          </Section>

          <footer className="flex items-center justify-between px-1 pb-1 text-xs text-muted">
            <span className="inline-flex items-center gap-1.5"><img src={markSmall} width={16} height={16} alt="" />Wincy · {t("s.version", { version })}</span>
            <div className="flex gap-1">
              <Button size="sm" variant="ghost" onPress={() => run(() => openExternal(SUPPORT_URL))}>
                <Coffee size={12} />
                {t("s.support")}
              </Button>
              <Button size="sm" variant="ghost" onPress={() => run(() => openExternal(REPO_URL))}>
                {t("s.github")}
                <ExternalLink size={12} />
              </Button>
            </div>
          </footer>
        </div>
      )}

      <AlertDialog.Backdrop isOpen={confirmOpen} onOpenChange={setConfirmOpen} isKeyboardDismissDisabled={false}>
        <AlertDialog.Container>
          <AlertDialog.Dialog className="sm:max-w-[380px]">
            <AlertDialog.Header>
              <AlertDialog.Icon status="danger" />
              <AlertDialog.Heading>{confirm?.title}</AlertDialog.Heading>
            </AlertDialog.Header>
            <AlertDialog.Body>
              <p className="text-sm">{confirm?.body}</p>
            </AlertDialog.Body>
            <AlertDialog.Footer>
              <Button slot="close" variant="tertiary">
                {t("s.cancel")}
              </Button>
              <Button slot="close" variant="danger" onPress={confirm?.run}>
                {confirm?.action}
              </Button>
            </AlertDialog.Footer>
          </AlertDialog.Dialog>
        </AlertDialog.Container>
      </AlertDialog.Backdrop>
    </div>
  );
}
