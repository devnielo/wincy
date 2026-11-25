import { AlertDialog, Button, Description, Label, ListBox, NumberField, Select, Switch } from "@heroui/react";
import { ArrowLeft, ExternalLink, FolderOpen, Trash2 } from "lucide-react";
import { useEffect, useRef, useState, type ReactNode } from "react";
import { api, appVersion, errorMessage, openExternal } from "../api";
import { useI18n, type MessageKey } from "../i18n";
import { useClipboardStore } from "../store/useClipboardStore";
import type { Language, Settings, Theme } from "../types";
import { IgnoredApps } from "./IgnoredApps";
import { ShortcutRecorder } from "./ShortcutRecorder";
import { UpdateSection } from "./UpdateSection";

const REPO_URL = "https://github.com/devnielo/wincy";
const THEMES: Theme[] = ["system", "light", "dark"];
const LANGUAGES: Language[] = ["system", "en", "es"];

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

function Choice<T extends string>({ labelId, value, options, prefix, onChange }: {
  labelId: string;
  value: T;
  options: T[];
  prefix: "theme" | "lang";
  onChange: (value: T) => void;
}) {
  const { t } = useI18n();
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
            <ListBox.Item key={o} id={o} textValue={t(`${prefix}.${o}` as MessageKey)}>
              {t(`${prefix}.${o}` as MessageKey)}
              <ListBox.ItemIndicator />
            </ListBox.Item>
          ))}
        </ListBox>
      </Select.Popover>
    </Select>
  );
}

export function SettingsView() {
  const { t } = useI18n();
  const settings = useClipboardStore((s) => s.settings);
  const saveSettings = useClipboardStore((s) => s.saveSettings);
  const error = useClipboardStore((s) => s.error);
  const setError = useClipboardStore((s) => s.setError);
  const setView = useClipboardStore((s) => s.setView);
  const [version, setVersion] = useState("");
  // Kept after closing so the dialog keeps its text during the exit animation.
  const [confirm, setConfirm] = useState<{ title: string; body: string; action: string; run: () => void } | null>(null);
  const [confirmOpen, setConfirmOpen] = useState(false);
  const ask = (c: NonNullable<typeof confirm>) => {
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
          <Section title={t("s.general")}>
            <Row id="set-shortcut" label={t("s.shortcut")} hint={t("s.shortcutHint")}>
              <ShortcutRecorder labelId="set-shortcut" />
            </Row>
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
            <Toggle field="launchAtStartup" label="s.launchAtStartup" />
            <Toggle field="pasteDirectly" label="s.pasteDirectly" hint="s.pasteDirectlyHint" />
            <Toggle field="captureImages" label="s.captureImages" />
          </Section>

          <Section title={t("s.appearance")}>
            <Row id="set-theme" label={t("s.theme")}>
              <Choice labelId="set-theme" prefix="theme" value={settings.theme} options={THEMES} onChange={(theme) => save({ theme })} />
            </Row>
            <Row id="set-lang" label={t("s.language")}>
              <Choice labelId="set-lang" prefix="lang" value={settings.language} options={LANGUAGES} onChange={(language) => save({ language })} />
            </Row>
          </Section>

          <Section title={t("s.privacy")}>
            <div className="flex flex-col gap-2 px-3 py-2.5">
              <div className="flex flex-col">
                <span className="text-sm">{t("s.ignoredApps")}</span>
                <span className="text-xs text-muted">{t("s.ignoredAppsHint")}</span>
              </div>
              <IgnoredApps />
            </div>
          </Section>

          <Section title={t("s.updates")}>
            <Toggle field="checkUpdates" label="s.checkUpdates" />
            <div className="px-3 py-2">
              <UpdateSection />
            </div>
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
          </Section>

          <footer className="flex items-center justify-between px-1 pb-1 text-xs text-muted">
            <span>Wincy · {t("s.version", { version })}</span>
            <Button size="sm" variant="ghost" onPress={() => run(() => openExternal(REPO_URL))}>
              {t("s.github")}
              <ExternalLink size={12} />
            </Button>
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
