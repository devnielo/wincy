import { Button, Input } from "@heroui/react";
import { Plus, X } from "lucide-react";
import { useState, type FormEvent } from "react";
import { useI18n } from "../i18n";
import { normalize } from "../lib/apps";
import { useClipboardStore } from "../store/useClipboardStore";

export function IgnoredApps() {
  const { t } = useI18n();
  const apps = useClipboardStore((s) => s.settings?.ignoredApps ?? []);
  const saveSettings = useClipboardStore((s) => s.saveSettings);
  const [draft, setDraft] = useState("");
  const [error, setError] = useState<string | null>(null);

  async function add(e: FormEvent) {
    e.preventDefault();
    const app = normalize(draft);
    if (!app) return;
    if (apps.some((a) => a.toLowerCase() === app)) {
      setError(t("s.appExists"));
      return;
    }
    const err = await saveSettings({ ignoredApps: [...apps, app] });
    setError(err);
    if (!err) setDraft("");
  }

  async function remove(app: string) {
    setError(await saveSettings({ ignoredApps: apps.filter((a) => a !== app) }));
  }

  return (
    <div className="flex flex-col gap-2">
      <ul className="flex flex-wrap gap-1.5">
        {apps.map((app) => (
          <li
            key={app}
            className="flex h-7 items-center gap-1 rounded-md border border-(--wincy-stroke) bg-(--wincy-field) pr-0.5 pl-2 text-xs"
          >
            {app}
            <Button
              isIconOnly
              size="sm"
              variant="ghost"
              aria-label={t("s.removeApp", { app })}
              onPress={() => void remove(app)}
              className="size-6 min-w-6"
            >
              <X size={12} />
            </Button>
          </li>
        ))}
      </ul>
      <form onSubmit={(e) => void add(e)} className="flex gap-2">
        <Input
          value={draft}
          onChange={(e) => {
            setDraft(e.target.value);
            setError(null);
          }}
          placeholder={t("s.appPlaceholder")}
          aria-label={t("s.ignoredApps")}
          aria-describedby={error ? "ignored-apps-error" : undefined}
          spellCheck={false}
          autoComplete="off"
          className="h-8 flex-1 text-sm"
        />
        <Button type="submit" size="sm" variant="secondary" isDisabled={!normalize(draft)}>
          <Plus size={14} />
          {t("s.addApp")}
        </Button>
      </form>
      {error && (
        <p id="ignored-apps-error" role="alert" className="text-xs text-danger">
          {error}
        </p>
      )}
    </div>
  );
}
