# Milestone 5 — Polish & desktop release

**Status:** Planning — not implemented  
**Source:** [Master plan](plan.md)  
**Repository rules:** [AGENTS.md](../AGENTS.md)

This file covers an implementation milestone within the manual-translation MVP (product Phase 1). File numbering follows the original milestone numbering; it does not change the product phases.

## Prerequisites

Build on [Milestone 4 — EPUB export (vertical-slice first)](phase-4-epub-export.md).

## Tasks and acceptance criteria

- [ ] UX review: dark mode, pane resizing, text zoom, shortcuts, error dialogs.
- [ ] Accessibility and Burmese text rendering cross-OS tests.
- [ ] Performance checks with large books (bounded RAM/use lazy loading).
- [ ] Package and smoke-test installers for Windows/macOS/Linux.
- [ ] Document backups, privacy, supported EPUB limits, and recovery.
- [ ] Release checklist covering data-loss cases and export validation.

**Done when:** build artifacts install/open/export reliably on all supported desktop systems and known limitations are documented.

## Desktop UI/UX requirements

Verify system/light/dark appearance, text zoom, pane sizing and layout persistence. Polish platform shortcuts and focus behavior; check minimum-window and high-DPI layouts. Record keyboard, screen-reader, contrast and Myanmar rendering findings for each tested OS.

Follow [Desktop UI/UX design](ui-ux-design.md) for layout dimensions, visual tokens, screen states, interaction rules and design acceptance checks. Requirements assigned to later milestones remain deferred.

## Supporting design

- [Section 2 — UX / screen architecture](plan.md)
- [Section 10 — Quality gates / test matrix](plan.md)
- [Section 11 — Known technical risks / mitigations](plan.md)

Follow the master plan’s quality gates and repository definition of done. Check tasks only after implementation and acceptance criteria are verified; keep completion status aligned with the master plan.
