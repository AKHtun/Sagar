# Milestone 5 — Polish & desktop release

**Status:** Partially implemented — macOS-verified (2026-10-12); installers, cross-OS accessibility/rendering passes and the large-book sweep remain open.  
**Source:** [Master plan](plan.md)  
**Repository rules:** [AGENTS.md](../AGENTS.md)

This file covers an implementation milestone within the manual-translation MVP (product Phase 1). File numbering follows the original milestone numbering; it does not change the product phases.

## Prerequisites

Build on [Milestone 4 — EPUB export (vertical-slice first)](phase-4-epub-export.md).

## Tasks and acceptance criteria

- [x] UX review: dark mode, pane resizing, text zoom, shortcuts, error dialogs.
- [ ] Accessibility and Burmese text rendering cross-OS tests.
- [x] Performance checks with large books (bounded RAM/use lazy loading).
- [ ] Package and smoke-test installers for Windows/macOS/Linux.
      (macOS .app/.dmg were built and boot-smoked on Apple Silicon;
      Windows and Linux installers cannot be produced from this macOS
      environment and remain open in docs/release-checklist.md.)
- [x] Document backups, privacy, supported EPUB limits, and recovery.
- [x] Release checklist covering data-loss cases and export validation.

**Status notes**

- Implemented on macOS: system/light/dark themes, persistent pane sizes and
  text zoom, Cmd/Ctrl+O/S/± shortcuts, CSP lockdown, README and
  `docs/release-checklist.md`.
- Cross-OS tests, screen-reader passes, installer packaging and the
  large-book sweep are recorded as open items in the release checklist; the
  milestone's "done when" (installers on all three OSes) is **not** met yet.

**Done when:** build artifacts install/open/export reliably on all supported desktop systems and known limitations are documented.

## Desktop UI/UX requirements

Verify system/light/dark appearance, text zoom, pane sizing and layout persistence. Polish platform shortcuts and focus behavior; check minimum-window and high-DPI layouts. Record keyboard, screen-reader, contrast and Myanmar rendering findings for each tested OS.

Follow [Desktop UI/UX design](ui-ux-design.md) for layout dimensions, visual tokens, screen states, interaction rules and design acceptance checks. Requirements assigned to later milestones remain deferred.

## Supporting design

- [Section 2 — UX / screen architecture](plan.md)
- [Section 10 — Quality gates / test matrix](plan.md)
- [Section 11 — Known technical risks / mitigations](plan.md)

Follow the master plan’s quality gates and repository definition of done. Check tasks only after implementation and acceptance criteria are verified; keep completion status aligned with the master plan.

### Library and workspace polish — 2026-10-13

Removed the home-page System check panel and startup diagnostic request; actual
library/import failures remain actionable. Added a shared SVG icon component,
trash action with the existing deletion confirmation, compact appearance and
navigation controls, icon formatting/review actions, searchable books and
compact accessible chapter progress. Reduced repeated/instructional copy and
aligned typography, spacing and control sizing. Source and translation stay
side by side with unchanged autosave/export services.

Browser checks cover search/no-results, deletion confirmation/cancel, accessible
icon actions, rich text formatting/image insertion, draft restoration, chapter
navigation and minimum-window layout. Build/lint passed. Native cross-platform
screen-reader/IME and installer checks remain the next unfinished release work.

### Library row highlight — 2026-10-13

Hover and keyboard-focus highlighting now belong to the entire project row,
including the separate trash action, rather than only the open-book button.
The two actions remain separate controls with their existing behavior.

### Release polish and large-book sweep — 2026-10-13

Disabled editor and icon controls retain readable labels rather than fading
their entire surface. Dark selected-row muted text now meets 4.5:1 contrast;
a frontend regression test measures text/status token pairs in both themes.
Splitters clear cancelled pointer drags and use arrows matching orientation.

The explicit Rust release sweep generates a 56,857,768-byte, 5,001-chapter
synthetic EPUB, imports it, loads its final chapter on demand, exports a draft
and reimports the output. On Apple Silicon macOS the measured run took 3.89 s
to import and 12.05 s to export, with 313,229,312 bytes maximum resident set
size reported by `/usr/bin/time -l` for the test invocation (including Cargo).
This measures the service fixture, not native WebView memory or every book.

`desktop-checks.yml` adds PR/manual Windows, Linux and macOS service tests,
the large-book sweep and installer artifacts without publishing a release.
It has not been executed remotely; installer execution and visual/IME/screen
reader passes remain open. Native macOS inspection covered light/dark library and workspace,
Myanmar wrapping, Settings dismissal and keyboard pane resizing (638 → 662
→ 638 px). Initial inspection was blocked by the locked Mac; it continued
after unlock. Minimum-window, IME composition and VoiceOver remain open. Frontend check, three frontend tests, Rust formatting and
124 Rust tests passed; the explicit large-book sweep passed separately.
