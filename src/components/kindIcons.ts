import { Braces, Code, Files, Image, Layers, Link, Palette, Type, type LucideIcon } from "lucide-react";
import type { KindFilter } from "../store/useClipboardStore";

export const KIND_ICONS: Record<KindFilter, LucideIcon> = {
  all: Layers,
  text: Type,
  url: Link,
  color: Palette,
  code: Code,
  image: Image,
  files: Files,
  snippets: Braces,
};
