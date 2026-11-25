import { Button, Kbd, Tooltip } from "@heroui/react";
import type { LucideIcon } from "lucide-react";

interface Props {
  icon: LucideIcon;
  label: string;
  /** Shown in the tooltip, e.g. "Ctrl P". */
  keys?: string;
  /** Visible text next to the icon; icon-only (label in aria + tooltip) when omitted. */
  text?: string;
  onPress: () => void;
  isDisabled?: boolean;
  pressed?: boolean;
  className?: string;
}

/** Ghost button with a tooltip that never takes focus from the search input. */
export function ActionButton({ icon: Icon, label, keys, text, onPress, isDisabled, pressed, className = "" }: Props) {
  return (
    <Tooltip delay={500} closeDelay={0}>
      <Button
        size="sm"
        variant="ghost"
        isIconOnly={!text}
        aria-label={label}
        aria-pressed={pressed}
        isDisabled={isDisabled}
        preventFocusOnPress
        onPress={onPress}
        className={`${text ? "h-7 gap-1.5 px-2 text-xs" : "size-7 min-w-7"} ${pressed ? "text-accent" : "text-muted"} ${className}`}
      >
        <Icon size={15} aria-hidden />
        {text}
      </Button>
      <Tooltip.Content className="flex items-center gap-2 text-xs">
        {label}
        {keys && (
          <Kbd variant="light" className="text-[11px] [word-spacing:normal]">
            <Kbd.Content>{keys}</Kbd.Content>
          </Kbd>
        )}
      </Tooltip.Content>
    </Tooltip>
  );
}
