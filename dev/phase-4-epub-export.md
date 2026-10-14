# Milestone 4 — EPUB export (vertical-slice first)

**Status:** Implemented on macOS (verified 2026-10-12 via export round-trip integration tests). EPUBCheck not run (no Java in this environment); Windows/Linux untested.  
**Source:** [Master plan](plan.md)  
**Repository rules:** [AGENTS.md](../AGENTS.md)

This file covers an implementation milestone within the manual-translation MVP (product Phase 1). File numbering follows the original milestone numbering; it does not change the product phases.

## Prerequisites

Build on [Milestone 3 — Local autosave, progress and resume](phase-3-autosave-and-resume.md).

## Tasks and acceptance criteria

- [x] Implement a 1-chapter translated EPUB round trip on an intentionally simple fixture.
- [x] Patch translated XHTML fragments into source resource tree without flattening assets.
- [x] Repackage EPUB with correct OCF entry ordering/compression.
- [x] Update title, language and navigation labels without breaking hrefs.
- [x] Add native Save dialog, explicit draft mode and unfinished-content review.
- [x] Validate output ZIP, XHTML parsing, manifest targets and links.
- [x] Reimport the exported file and compare chapter/resource counts and spine order.
- [x] Add regression fixtures for formatting, internal links, images and footnotes.

**Done when:** a fully manually translated fixture exports, passes the project's integrity checks, reopens in a reading app, and retains its structural assets. — Met in the automated round-trip test (re-import acts as the reader smoke test); EPUBCheck remains an open dev-environment step recorded in docs/release-checklist.md.

## Desktop UI/UX requirements

Provide an export review dialog with title/language fields, actionable missing-content counts and explicit draft acknowledgement. Use native Save dialogs, truthful validation feedback and safe overwrite behavior. Enable preview only when it reflects saved translations and explains retained original blocks.

Follow [Desktop UI/UX design](ui-ux-design.md) for layout dimensions, visual tokens, screen states, interaction rules and design acceptance checks. Requirements assigned to later milestones remain deferred.

## Supporting design

- [Section 6 — Tauri command contract (initial proposal)](plan.md)
- [Section 8 — Export pipeline — most important correctness requirement](plan.md)
- [Section 10 — Quality gates / test matrix](plan.md)

Follow the master plan’s quality gates and repository definition of done. Check tasks only after implementation and acceptance criteria are verified; keep completion status aligned with the master plan.

### Export review layout correction — 2026-10-13

Export uses a bounded dialog with fixed header, title/language settings,
summary, acknowledgement and actions. Only the chapter review region scrolls;
its table headings stay visible. Rows use readable status labels and state
whether draft export will include a saved translation or original content.
Disabled actions have a neutral background; the active export action uses the
accent with contrasting text. Keyboard focus stays in the modal and the
chapter list is keyboard-scrollable.

Draft exports include reviewed translations **and** saved unreviewed drafts.
Empty or intentionally skipped chapters retain original content. This is
explicit in the dialog and covered by an export regression test. Browser
layout checks use 758 chapters at 1280 × 800 and 960 × 640; native Windows/Linux
verification remains open.

### Translated-only draft option — 2026-10-13

Draft export offers “Include original content for untranslated chapters”
(checked by default, preserving the behavior above). When unchecked, chapters
without any saved translation — pending and “Keep original” alike — are
omitted from the output book (spine, manifest and EPUB 3 TOC entries removed,
files excluded) so the draft contains only finished (reviewed + draft)
chapters. Within included chapters, untranslated blocks still retain original
text. Completed exports always include every chapter. An empty filtered draft
is rejected (`nothing_to_export`); omission is reported via an
`excluded_untranslated` warning. Covered by round-trip regression tests.

### Export safety corrections — 2026-10-13

Imports persist the canonical original file path through migration 8. Export
rejects that path and the stored snapshot, and compares the import hash to
protect unchanged originals/identical copies from legacy projects. Legacy
projects cannot recover an unknown original path if its contents have changed.

Validation checks local manifest resources, parses all manifest XHTML including
navigation, and resolves XHTML href/src references and XHTML anchor IDs.
Translated-only drafts with links into removed chapters are rejected with an
actionable error; an existing destination remains untouched. External URLs are
not fetched. CSS/SVG resource references and full EPUBCheck conformance remain
outside these internal checks.

Authored chapter headings now supply navigation labels. Navigation hrefs are
resolved relative to their own document, including fragment and percent-encoded
paths, while original href bytes remain unchanged.

Verification: 16 export integration tests, the Rust suite, and frontend build
passed on macOS. Regression fixtures cover original-path/hash protection,
filtered broken links, preservation of existing output after validation failure,
missing assets/anchors, malformed navigation and nested navigation with Myanmar
heading text. No UI layout changed; native reader, Windows/Linux and EPUBCheck
verification remain the next unfinished release checks.

### Custom export cover — 2026-10-13

Book Settings → Metadata now accepts a custom PNG/JPEG/GIF cover up to 8 MB,
previews the selected image, and offers “Use original cover”. Selection is
saved with the other settings; Cancel leaves the saved cover unchanged and
save failures retain entered settings. Migration 9 stores the bounded image
separately in SQLite. Existing projects default to their original cover.
Library thumbnails refresh after Book Settings saves.

Export adds a new image resource, sets its EPUB 3 `cover-image` property,
updates an existing legacy cover metadata pointer, and removes the old cover
property without deleting original image assets. Existing XHTML/SVG cover
references are redirected to the custom image, including object/embed media
types. A book without a cover receives cover-image metadata; this does not add
a separate reading-order cover page. CSS background images remain preserved.
Source EPUB and snapshot are never edited.

Verification: Rust suite and frontend lint/build passed; export regressions
cover persistence, no-cover books, format changes, XHTML/SVG references, legacy
metadata, invalid-image rejection, restore-original and source immutability.
Browser UI fixtures verified custom selection, preview, Save, restore-original,
retained input after a save error, focus wrapping and Escape, with light/dark
screenshots at 1280×800 and 960×640. Native checks were unavailable because
the Mac was locked; Windows/Linux, native file selection and EPUBCheck remain
unfinished release verification. Temporary UI fixtures were removed.
