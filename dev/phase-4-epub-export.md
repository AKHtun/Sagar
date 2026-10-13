# Milestone 4 — EPUB export (vertical-slice first)

**Status:** Planning — not implemented  
**Source:** [Master plan](plan.md)  
**Repository rules:** [AGENTS.md](../AGENTS.md)

This file covers an implementation milestone within the manual-translation MVP (product Phase 1). File numbering follows the original milestone numbering; it does not change the product phases.

## Prerequisites

Build on [Milestone 3 — Local autosave, progress and resume](phase-3-autosave-and-resume.md).

## Tasks and acceptance criteria

- [ ] Implement a 1-chapter translated EPUB round trip on an intentionally simple fixture.
- [ ] Patch translated XHTML fragments into source resource tree without flattening assets.
- [ ] Repackage EPUB with correct OCF entry ordering/compression.
- [ ] Update title, language and navigation labels without breaking hrefs.
- [ ] Add native Save dialog, explicit draft mode and unfinished-content review.
- [ ] Validate output ZIP, XHTML parsing, manifest targets and links.
- [ ] Reimport the exported file and compare chapter/resource counts and spine order.
- [ ] Add regression fixtures for formatting, internal links, images and footnotes.

**Done when:** a fully manually translated fixture exports, passes the project's integrity checks, reopens in a reading app, and retains its structural assets. EPUBCheck should also pass on supported compliant fixtures in CI/dev test environments.

## Desktop UI/UX requirements

Provide an export review dialog with title/language fields, actionable missing-content counts and explicit draft acknowledgement. Use native Save dialogs, truthful validation feedback and safe overwrite behavior. Enable preview only when it reflects saved translations and explains retained original blocks.

Follow [Desktop UI/UX design](ui-ux-design.md) for layout dimensions, visual tokens, screen states, interaction rules and design acceptance checks. Requirements assigned to later milestones remain deferred.

## Supporting design

- [Section 6 — Tauri command contract (initial proposal)](plan.md)
- [Section 8 — Export pipeline — most important correctness requirement](plan.md)
- [Section 10 — Quality gates / test matrix](plan.md)

Follow the master plan’s quality gates and repository definition of done. Check tasks only after implementation and acceptance criteria are verified; keep completion status aligned with the master plan.
