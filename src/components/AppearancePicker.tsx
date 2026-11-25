import { Radio, RadioGroup } from "@heroui/react";
import { Check, LayoutGrid } from "lucide-react";
import { useDark } from "../hooks/useAppearance";
import { useI18n } from "../i18n";
import { ACCENT_NAMES, resolveAccent } from "../lib/accent";
import { useClipboardStore } from "../store/useClipboardStore";
import type { Accent, Settings, Theme } from "../types";

const THEMES: Theme[] = ["system", "light", "dark"];

/** A tiny palette window in one mode, drawn with that mode's own colors whatever the current theme is. */
function MiniPalette({ dark, accent }: { dark: boolean; accent: string }) {
  const line = dark ? "rgb(255 255 255 / 0.45)" : "rgb(0 0 0 / 0.3)";
  return (
    <div className="absolute inset-0 flex flex-col gap-1.5 p-2" style={{ background: dark ? "#202020" : "#f3f3f3" }}>
      <div className="h-2.5 rounded-sm" style={{ background: dark ? "rgb(255 255 255 / 0.08)" : "#ffffff" }} />
      <div className="flex items-center gap-1.5 rounded-sm px-1 py-1" style={{ background: dark ? "rgb(255 255 255 / 0.07)" : "rgb(0 0 0 / 0.055)" }}>
        <span className="h-2.5 w-0.5 rounded-full" style={{ background: accent }} />
        <span className="h-1 w-3/5 rounded-full" style={{ background: line }} />
      </div>
      <span className="ml-2.5 h-1 w-2/5 rounded-full" style={{ background: line }} />
      <span className="ml-2.5 h-1 w-1/2 rounded-full" style={{ background: line }} />
    </div>
  );
}

export function ThemeCards({ labelId, settings, save }: { labelId: string; settings: Settings; save: (patch: Partial<Settings>) => void }) {
  const { t } = useI18n();
  const systemAccent = useClipboardStore((s) => s.status?.systemAccent ?? null);
  const accentFor = (dark: boolean) => resolveAccent(settings.accent, systemAccent, dark).color;
  return (
    <RadioGroup
      aria-labelledby={labelId}
      orientation="horizontal"
      value={settings.theme}
      onChange={(theme) => save({ theme: theme as Theme })}
      className="grid grid-cols-3 gap-2"
    >
      {THEMES.map((mode) => (
        <Radio key={mode} value={mode} className="min-w-0">
          <Radio.Content className="group flex w-full flex-col items-stretch gap-1.5 text-xs font-normal">
            <span className="relative block h-16 overflow-hidden rounded-md border border-(--wincy-stroke) outline-offset-2 group-data-focus-visible:outline-2 group-data-focus-visible:outline-(--focus) group-data-selected:border-accent group-data-selected:ring-1 group-data-selected:ring-accent">
              {mode !== "dark" && <MiniPalette dark={false} accent={accentFor(false)} />}
              {mode !== "light" && (
                <span className="absolute inset-0" style={mode === "system" ? { clipPath: "polygon(60% 0, 100% 0, 100% 100%, 40% 100%)" } : undefined}>
                  <MiniPalette dark accent={accentFor(true)} />
                </span>
              )}
            </span>
            <span className="flex items-center justify-center gap-1 group-data-selected:font-semibold">{t(`theme.${mode}`)}</span>
          </Radio.Content>
        </Radio>
      ))}
    </RadioGroup>
  );
}

export function AccentSwatches({ labelId, settings, save }: { labelId: string; settings: Settings; save: (patch: Partial<Settings>) => void }) {
  const { t } = useI18n();
  const systemAccent = useClipboardStore((s) => s.status?.systemAccent ?? null);
  const dark = useDark();
  return (
    <RadioGroup
      aria-labelledby={labelId}
      orientation="horizontal"
      value={settings.accent}
      onChange={(accent) => save({ accent: accent as Accent })}
      className="flex flex-wrap gap-2.5"
    >
      {ACCENT_NAMES.map((name) => {
        const { color, foreground } = resolveAccent(name, systemAccent, dark);
        const label = t(`accent.${name}`);
        return (
          <Radio key={name} value={name}>
            <Radio.Content className="group rounded-full">
              <span className="sr-only">{label}</span>
              <span
                title={label}
                className="flex size-7 items-center justify-center rounded-full ring-1 ring-black/10 outline-offset-2 ring-inset group-data-selected:outline-2 group-data-selected:outline-(--swatch) group-data-focus-visible:outline-2 group-data-focus-visible:outline-(--focus)"
                style={{ background: color, color: foreground, ["--swatch" as string]: color }}
              >
                <Check size={14} strokeWidth={3} className="hidden group-data-selected:block" aria-hidden />
                {name === "windows" && <LayoutGrid size={12} className="group-data-selected:hidden" aria-hidden />}
              </span>
            </Radio.Content>
          </Radio>
        );
      })}
    </RadioGroup>
  );
}
