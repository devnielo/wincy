---
name: Wincy website
description: The marketing site for Wincy, a keyboard-first clipboard manager for Windows, set as a card catalog in Windows 11 material.
colors:
  cabinet-grey: "#f4f4f5"
  record-white: "#ffffff"
  drawer-grey: "#ececef"
  catalog-ink: "#1b1b1f"
  pencil-grey: "#56565f"
  rule-grey: "#e0e0e5"
  rule-strong: "#cbcbd2"
  windows-accent: "#005fb8"
  windows-accent-deep: "#004f9a"
  on-accent: "#ffffff"
  vault-blue: "#0b2747"
  vault-ink: "#f1f6fd"
  vault-ink-soft: "#b7c9e3"
  vault-dim: "#6487b4"
  app-ink: "#18181b"
  app-muted: "#5b5b64"
  app-accent: "#0485f7"
  app-danger: "#d12f33"
  app-warning: "#a85a00"
  smartscreen-blue: "#0067c0"
  night-cabinet: "#121214"
  night-record: "#1b1b1e"
  night-ink: "#f2f2f4"
  night-pencil: "#a8a8b2"
  night-accent: "#60cdff"
  night-on-accent: "#04121f"
  brand-violet: "#6d5bff"
  brand-teal: "#22b8cf"
  edge-shade: "rgb(0 0 0 / 0.06)"
  swatch-shade: "rgb(0 0 0 / 0.08)"
  knob-shade: "rgb(0 0 0 / 0.25)"
typography:
  display:
    fontFamily: "Schibsted Grotesk Variable, Segoe UI Variable Display, Segoe UI, system-ui, sans-serif"
    fontSize: "clamp(2.125rem, 1.15rem + 2.9vw, 3.75rem)"
    fontWeight: 500
    lineHeight: 1.02
    letterSpacing: "-0.035em"
  headline:
    fontFamily: "Schibsted Grotesk Variable, Segoe UI Variable Display, Segoe UI, system-ui, sans-serif"
    fontSize: "clamp(1.875rem, 1.25rem + 1.9vw, 3rem)"
    fontWeight: 500
    lineHeight: 1.06
    letterSpacing: "-0.03em"
  statement:
    fontFamily: "Schibsted Grotesk Variable, Segoe UI Variable Display, Segoe UI, system-ui, sans-serif"
    fontSize: "clamp(2.25rem, 1.3rem + 2.8vw, 3.75rem)"
    fontWeight: 500
    lineHeight: 1.04
    letterSpacing: "-0.035em"
  close:
    fontFamily: "Schibsted Grotesk Variable, Segoe UI Variable Display, Segoe UI, system-ui, sans-serif"
    fontSize: "clamp(2.375rem, 1.3rem + 3.4vw, 4.5rem)"
    fontWeight: 500
    lineHeight: 1
    letterSpacing: "-0.04em"
  title:
    fontFamily: "Schibsted Grotesk Variable, Segoe UI Variable Display, Segoe UI, system-ui, sans-serif"
    fontSize: "1.25rem"
    fontWeight: 500
    lineHeight: 1.3
    letterSpacing: "-0.015em"
  question:
    fontFamily: "Schibsted Grotesk Variable, Segoe UI Variable Display, Segoe UI, system-ui, sans-serif"
    fontSize: "1.125rem"
    fontWeight: 500
    lineHeight: 1.4
    letterSpacing: "-0.01em"
  lead:
    fontFamily: "Schibsted Grotesk Variable, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "clamp(1.0625rem, 1rem + 0.25vw, 1.25rem)"
    fontWeight: 400
    lineHeight: 1.5
  body:
    fontFamily: "Schibsted Grotesk Variable, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "1rem"
    fontWeight: 400
    lineHeight: 1.625
  small:
    fontFamily: "Schibsted Grotesk Variable, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "0.9375rem"
    fontWeight: 400
    lineHeight: 1.5
  label:
    fontFamily: "Schibsted Grotesk Variable, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "0.8125rem"
    fontWeight: 500
    lineHeight: 1
  label-small:
    fontFamily: "Schibsted Grotesk Variable, Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "0.75rem"
    fontWeight: 500
    lineHeight: 1
  app-ui:
    fontFamily: "Segoe UI Variable Text, Segoe UI, Schibsted Grotesk Variable, system-ui, sans-serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.4
  app-input:
    fontFamily: "Segoe UI Variable Text, Segoe UI, Schibsted Grotesk Variable, system-ui, sans-serif"
    fontSize: "15px"
    fontWeight: 400
    lineHeight: 1
  app-meta:
    fontFamily: "Segoe UI Variable Text, Segoe UI, Schibsted Grotesk Variable, system-ui, sans-serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.2
  app-code:
    fontFamily: "Cascadia Mono, Cascadia Code, Consolas, ui-monospace, monospace"
    fontSize: "13px"
    fontWeight: 400
    lineHeight: 1.625
  smartscreen-title:
    fontFamily: "Segoe UI Variable Text, Segoe UI, system-ui, sans-serif"
    fontSize: "19px"
    fontWeight: 300
    lineHeight: 1.2
rounded:
  selection: "3px"
  menu-item: "4px"
  key: "5px"
  row: "6px"
  control: "8px"
  window: "8px"
  panel: "12px"
  chip: "16px"
  big-key: "22px"
  pill: "999px"
spacing:
  gutter-phone: "24px"
  gutter-tablet: "48px"
  gutter-desktop: "80px"
  section-phone: "56px"
  section-tablet: "96px"
  section-desktop: "120px"
  header: "64px"
components:
  button-primary:
    backgroundColor: "{colors.windows-accent}"
    textColor: "{colors.on-accent}"
    rounded: "{rounded.control}"
    padding: "0 20px"
    height: "48px"
  button-primary-hover:
    backgroundColor: "{colors.windows-accent-deep}"
    textColor: "{colors.on-accent}"
  button-quiet:
    backgroundColor: "{colors.record-white}"
    textColor: "{colors.catalog-ink}"
    rounded: "{rounded.control}"
    padding: "0 20px"
    height: "48px"
  keycap:
    backgroundColor: "{colors.record-white}"
    textColor: "{colors.catalog-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.key}"
    height: "1.75rem"
  version-tag:
    backgroundColor: "{colors.drawer-grey}"
    textColor: "{colors.catalog-ink}"
    rounded: "{rounded.key}"
    padding: "1px 7px"
  privacy-band:
    backgroundColor: "{colors.vault-blue}"
    textColor: "{colors.vault-ink}"
---

# Design System: Wincy website

## Overview

**Creative North Star: "The Card Catalog in Windows 11 Material"**

The site files Wincy the way Wincy files a clipboard. Every capability is a ruled record with a heading, a call number made of keycaps (the shortcut that calls it) and the real window it describes; label and value lists are tracings; kind filters are divider tabs; pins are signal flags. The cabinet is neutral Mica grey, the records are ruled entries rather than boxes, and the product itself appears as live HTML copies of its own windows (palette, preview, settings, tray menu, SmartScreen dialog) sitting on a flat, hard-edged Windows wallpaper.

Density is calm and one-section-per-screen on a laptop: each section starts with a headline and short lead, then one structured body (records, shelf list, tracings, comparison, numbered steps, questions). The one ground change is the deep-blue privacy band. Motion is a single authored moment: the hero desktop opens on Alt+V, types "fig", narrows, previews and pastes once, then hands control to the visitor. The page refuses the stock SaaS template: no split hero, no icon-tile grid, no violet/cyan haze, no coloured glow, no closing line that repeats the hero.

**Key Characteristics:**
- Neutral greys with no hue tint; one Windows-accent blue reserved for the chosen thing.
- Schibsted Grotesk for the site's voice; Segoe UI Variable and Cascadia Mono for everything that depicts the app.
- Real product UI, never illustrations: every window is a faithful copy of the app's components.
- Keycaps as call numbers on records, rows of the shortcut list and the close.
- Hairline rules organise records, shortcuts, questions and tracings.

## Colors

A restrained Windows palette: neutral Mica greys, one accent blue, one deep-blue band, and the app's own tokens kept separate for its depicted windows.

### Primary
- **Windows Accent Blue** (#005fb8; #60cdff in dark): the one primary action (Download), links, focus rings, the selected-filter and paste states, the V keycap of the close. Dark mode inverts it to Windows' light accent with near-black text (#04121f).

### Secondary
- **Vault Blue** (#0b2747): the privacy band only, the same in both themes, with Vault Ink (#f1f6fd) for headings and Vault Ink Soft (#b7c9e3) for secondary text and tracing labels.

### Neutral
- **Cabinet Grey** (#f4f4f5; #121214 in dark): the page ground and the header.
- **Record White** (#ffffff; #1b1b1e in dark): keycaps, quiet buttons, the Wincy column of the comparison.
- **Drawer Grey** (#ececef): the version tag, hover fills, the install step's plain visual.
- **Catalog Ink** (#1b1b1f; #f2f2f4 in dark): headings and primary text.
- **Pencil Grey** (#56565f; #a8a8b2 in dark): leads, body copy in lists, secondary labels.
- **Rule Grey** (#e0e0e5) and **Rule Strong** (#cbcbd2): hairlines, keycap edges, record rules.

### The app's own palette (depictions only)
- **App Ink / App Muted** (#18181b / #5b5b64), **App Accent** (#0485f7), **App Danger** (#d12f33), **App Warning** (#a85a00), plus translucent Mica surface and stroke values: copied from the app's `src/index.css` and HeroUI defaults so the windows look like the shipped app. **SmartScreen Blue** (#0067c0) draws the Windows dialog as Windows draws it.

### Named Rules
**The Chosen Thing Rule.** Windows Accent Blue marks only what the visitor would choose or is choosing: the primary action, a link, a focus ring, the selected clip, the pressed filter. Nothing decorative is blue.

**The Logo-Only Gradient Rule.** The brand violet-to-teal (#6d5bff to #22b8cf) lives in the logo tile and nowhere else: no hazes, glows, gradient text or tinted grounds.

## Typography

**Display Font:** Schibsted Grotesk Variable (self-hosted via Fontsource, falling back to Segoe UI Variable)
**Body Font:** Schibsted Grotesk Variable
**App Fonts:** Segoe UI Variable Text and Cascadia Mono, used only inside depicted app windows

**Character:** A precise, slightly condensed grotesk at medium weight sets the site's voice; the app windows keep the system faces Windows users see in the real palette, so the product never looks restyled for marketing.

### Hierarchy
- **Display** (500, clamp 2.125rem to 3.75rem, 1.02, -0.035em): the hero h1 only, balanced over two or three lines.
- **Headline** (500, clamp 1.875rem to 3rem, 1.06, -0.03em): section headings; the privacy statement and the close run larger.
- **Title** (500, 1.25rem, 1.3): record and install-step headings; questions at 1.125rem.
- **Lead** (400, clamp 1.0625rem to 1.25rem, 1.5): the paragraph under a section heading, at most 40rem wide.
- **Body** (400, 1rem, 1.625): answers and step text, at most 62ch.
- **Statement and Close** (500, clamp 2.25rem to 3.75rem and 2.375rem to 4.5rem): the privacy statement and the closing line, the two moments set larger than a section heading.
- **Question** (500, 1.125rem, 1.4): FAQ questions; the hero lead also steps up to 1.125rem from 1024px.
- **Small** (400, 0.9375rem): header links, table heads, notes and the footer.
- **Label** (500, 0.8125rem; 0.75rem in the demo hint): keycap legends and the version tag.
- **App sizes** (Segoe UI Variable): 15px search field, 14px window text, 13.5px row labels, 12px meta, tooltips and chips, 11px Alt N hints; Cascadia Mono at 12.5px in rows and 13px in the preview; the SmartScreen title at 19px light, as Windows sets it.

### Named Rules
**The Two Voices Rule.** Schibsted speaks for the site; Segoe UI and Cascadia speak for the app. Never set site copy in Cascadia, never set app UI in Schibsted.

## Layout

A centred wrap of up to 1280px with gutters of 24px (phone), 48px (640px and up) and 80px (1280px and up). Sections pad 56px on phones, 96px from 640px and 120px at the top from 1280px; from 1024px wide and 700px tall each section fills at least one screen. A 12-column grid carries the hero head (h1 on 7 columns, side on 5 from 1280px; 6 and 6 from 1024px), the records (zigzag 7/5 then 5/7), the privacy band (6/6) and the close (7/5). The hero's desktop crop spans the whole wrap; the preview window appears from 1200px. Below 640px everything is one column: shortcut rows put keys and action side by side, the comparison becomes feature-then-two-answers rows, tracings stack, the FAQ folds (open from 900px), and the header keeps a short Download button.

### Named Rules
**The One Axis Rule.** Every block hangs off the wrap's left edge and the grid's split line; nothing is centred in the page except windows on their wallpaper.

## Elevation & Depth

The site itself is flat: grounds, rules and tonal fills carry hierarchy, and site buttons use only a 1px inner bottom shade. Depth belongs to the depicted app: windows carry the app shadow and a Mica blur over the wallpaper, exactly as they float on a real desktop.

### Shadow Vocabulary
- **App window** (`box-shadow: 0 18px 48px -14px rgb(6 28 66 / 0.42), 0 2px 8px rgb(6 28 66 / 0.12)`; darker in dark mode): palette, preview, settings, tray menu and fragments.
- **Big keycap** (`inset 0 -10px 0 <key side>, 0 22px 40px -22px rgb(0 0 0 / 0.45)`): the Alt and V keys of the close.

### Named Rules
**The Depth Is The App Rule.** Only things that are windows in the real product cast shadows or blur. No site card, button or band gets elevation.

## Shapes

Small, Windows-like corners: 5px keycaps and tags, 8px buttons and app windows (the Windows 11 window radius), 12px wallpaper panels and the hero stage, full pills for the app's ghost action buttons. Inside depicted windows the app's own radii hold: 3px selection pill, 4px menu items, 6px rows, tooltips and app chips, 16px status chips. The close's big keycaps take 22px. Keycaps take a 2px bottom border to read as physical keys. Hairline 1px rules separate records, shortcut rows, questions and tracings.

## Components

### Buttons
- **Shape:** gently rounded (8px), 48px tall (40px in the header).
- **Primary:** Windows Accent Blue with white text (near-black on the dark-mode accent), a download icon, and the label "Download for Windows".
- **Hover / Focus:** a 1px upward step and the deeper accent on hover, a press-down on active, the 2px accent focus ring offset by 2px.
- **Quiet:** Record White with a strong rule border (View on GitHub). **Tonal:** accent at 10% for the demo's reopen button.

### Keycaps (call numbers)
- **Style:** Record White, 1px strong rule with a 2px bottom edge, 5px radius, 1.75rem tall, medium label type; combos split on "+" into separate caps.
- **Use:** record call numbers, the shortcut list, the demo hint, and the big Alt + V of the close (V in accent).

### Records
- **Structure:** a 1px strong top rule, a title with its keycap call number at the right, a short body, then a 12px wallpaper panel holding a centred, non-interactive (inert) copy of an app window.

### The palette demo (signature component)
- **Fidelity:** a 600x460 copy of the app palette (search, kind filters, settings, rows with leading visual, label, pin flag, "app · age", row actions and Alt N, footer actions) and the preview window 8px beside it, on a wallpaper with a pointer and a taskbar holding Wincy's tray icon.
- **Behaviour:** fuzzy-scored search, Tab/Shift+Tab kind filters (leaving the demo past the ends), arrows, Enter and Shift+Enter, Alt+1..9, Ctrl+P, Ctrl+Delete, Ctrl+Space, Esc, Alt+V to reopen; hover selects; a check-marked toast in the stage's top gutter stands in for the paste.

### Navigation
- **Style:** a fixed 64px header on Cabinet Grey with a hairline: logo and name, five section anchors (from 1200px), an EN/ES segmented switch, GitHub, and Download holding the right corner on every width; below 1200px a full-screen popover menu.

## Do's and Don'ts

### Do:
- **Do** depict the product only with faithful copies of its real components, using the app tokens and the Segoe UI / Cascadia faces.
- **Do** file each capability as a ruled record under the keycaps that call it.
- **Do** keep Windows Accent Blue for the primary action, links, focus and selection.
- **Do** keep EN and ES in sync in `src/i18n`, with no copy in the components.
- **Do** keep every moving thing optional: the autoplay stops on first touch and never runs under reduced motion.

### Don't:
- **Don't** use the brand violet or teal outside the logo: no haze, glow, gradient text or tinted grounds.
- **Don't** put a box around a record or nest a card inside a card; windows sit on wallpaper, records are ruled.
- **Don't** add shadows or blur to site elements; depth belongs to depicted windows.
- **Don't** add illustrations, icon tiles, invented stats, testimonials or download counts.
