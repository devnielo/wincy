---
version: 1
slug: "src-components-landing-astro"
primary_target: "src/components/Landing.astro"
related_targets: ["src/components/Palette.astro","src/layouts/Base.astro","src/styles/global.css"]
---

# Wincy landing page (EN `/`, ES `/es/`)

Scope: the whole marketing page, both locales, built from `src/components/Landing.astro`, `Palette.astro`, `Base.astro`, `global.css` and the i18n dictionaries. Visitor mode: Persuade.

Audience and job: Windows developers, designers and keyboard power users deciding whether a small, unsigned, open-source utility is worth installing and trusting with their clipboard. Action: Download for Windows (GitHub latest release). Proof: the live palette demo that behaves like the real app, the honest SmartScreen story, local-only privacy facts, the Win+V comparison. Constraints: no invented claims, stats or testimonials; EN and ES in sync; static Astro; small JS; WCAG 2.2 AA; light and dark; 360px and up.

Run note: unattended subagent run. The decision round could not be put in front of the user, so the assigned direction was built with its materials translated to the brief-pinned Windows 11 look (PRODUCT.md: "must read as a native Windows 11 utility").

## Direction contract

THESIS: The page is filed the way Wincy files a clipboard: every capability is a record filed under the key that calls it, with its provenance attached, and the visitor retrieves things the way the app retrieves clips. It refuses the split hero, the 3x3 icon grid, the haze band and the closing line that repeats the hero.

OWN-WORLD: A card catalog translated into Windows 11 material. Neutral Mica greys are the cabinet, white records with 1px strokes and 8px corners are the cards, Schibsted Grotesk sets the record headings, Segoe UI and Cascadia set the live UI. Windows-accent blue is reserved for the chosen thing (selected clip, the one primary action, links). Pins are signal flags, kind filters are divider tabs, keycaps are call numbers, label/value lists are tracings. Brand violet-to-teal appears only as hard-edged material: the logo tile and the V keycap of the close.

STORY: The visitor sees the real palette at a pointer on a believable desktop, watches it riffle to "git" and paste, then tries it. Four records explain what they touched, each under its call number. The shelf list shows every key, the deep-blue band states what Wincy never files, Win+V is compared plainly, install is honest about SmartScreen, questions are answered open, and the close hands them Alt+V.

FIRST VIEWPORT: Fixed header with logo, five anchors, EN/ES, GitHub and a Download for Windows button at the right corner. Head row: h1 on cols 1-7 in two or three lines near 60px; lead, Download for Windows (primary, blue), View on GitHub and the fact line (version, Windows 10 and 11, MIT) on cols 8-12, feet aligned. Below, across the full wrap, a Windows desktop crop: flat hard-edged blue wallpaper, a pointer, the 600x460 palette at true scale at pointer plus 8px, the preview window 8px to its right at 1200px and up, a taskbar with the Wincy tray icon.

FORM: card catalog (library index cabinet), position 5 of the ordered list (1 Windows 11 desktop session, 2 shortcut reference cards, 3 spec redlines, 4 keycap kitting sheet, 5 card catalog, 6 office clipboard, 7 carbon-copy book). seed:9d875e0356853362

Raises kept from the declined challengers:
- From the algorave floor: what is about to happen is visible before it happens; the preview always shows exactly what Enter will paste.
- From the cutting-bench rail: every state is a mark, not only a hue (pin glyph, selection pill, check on paste, Play glyph and label when paused).
- From the Versailles vistas: one axis; every block hangs off the same left line and the same split line.
- From the memory quilt: provenance is stitched on; every clip shown anywhere carries its source app and age.
- From the doll atelier: one colour is reserved for the chosen thing.
- From the broadcast card: one fixed anchor; Download holds the header's right corner on every screen.

Signature interaction and motion grammar: the hero demo plays the ritual once after load (Alt+V pressed, palette opens beside the pointer, "git" typed, list ranks and narrows, preview swaps, Enter, "Pasted"), stops for good at the first touch, never runs under reduced motion. Second moment: the privacy statement lights up word by word with the scroll inside its own section. Everything else: hero entrance once, 1px button step and press, 2px lift only on linked cards, drawn underlines.

FINISH: unreviewed and undocumented is unfinished; this build ends with the finish review, the verdict, DESIGN.md, and every shipping raster carrying its provenance

## Build adaptations (cited)

- Autoplay types "fig", not "git": the colour clip's preview (large swatch with HEX/RGB/HSL) demonstrates type detection and the preview in one beat; the code clip's full preview is filed as the fourth record instead.
- The close's V keycap is Windows-accent blue, not the brand gradient: taste's no-violet rule for page design and the detector's ai-color-palette finding. The brand gradient now appears only in the logo tile and the share image's logo.
- Records are ruled entries (a 1px top rule, no white box): the white record card nested a wallpaper panel and a window (three boxes deep), which the craft floor refuses.
- Phones: rows in fragments and in the hero palette hide "app · age" (no room in a 300px window), the hero taskbar is hidden, and the shortcut list, compare table, tracings and FAQ reflow (FAQ answers fold below 900px).
- Tab in the demo cycles the kind filters like the app, but leaves the demo after "Files" (Shift+Tab after "All") so the page never traps keyboard focus (WCAG 2.1.2); the app itself wraps around.
- The demo's paste toast sits in the stage's top gutter, in the hint's place, so it never covers rows or the footer.

## Unresolved

- Win+V "no search" row is from the copy audit's knowledge, not a fresh check on current Windows 11.
- The demo's Spanish buttons use the app's own labels ("Texto plano", "Desfijar"); the site's prose says "sin formato" and "dejar de fijar". Aligning src/i18n.ts in the app would remove the mismatch (outside the website's scope).
