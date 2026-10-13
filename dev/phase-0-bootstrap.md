# Milestone 0 — Bootstrap & architecture foundation

**Status:** Planning — not implemented  
**Source:** [Master plan](plan.md)  
**Repository rules:** [AGENTS.md](../AGENTS.md)

This file covers an implementation milestone within the manual-translation MVP (product Phase 1). File numbering follows the original milestone numbering; it does not change the product phases.

## Prerequisites

Start here; use the existing scaffold.

## Tasks and acceptance criteria

- [ ] Verify generated Tauri 2 + React + TS project runs with **Bun** on the current development OS.
- [ ] Remove demo UI; create minimal app shell and navigation.
- [ ] Add strict TS checks, format/lint conventions and relevant scripts.
- [ ] Add Rust `domain`, `services`, `epub`, `storage` structure as functionality is introduced.
- [ ] Implement typed Tauri command bridge and reusable error reporting.
- [ ] Add SQLite schema/migration initialization with an automated persistence test.
- [ ] Add native dialog plugin and narrowly scoped capabilities for import/export; avoid broad filesystem permissions.

**Done when:** `bun tauri dev` launches, the frontend invokes a typed Rust test command, and a local SQLite migration test passes.

## Desktop UI/UX requirements

Define shared spacing, typography, colors and focus styles. Build a compact desktop library/workspace frame with labeled mock content, accessible controls and reusable loading/error feedback. Preserve native window behavior; implement only the shell needed by this milestone.

Follow [Desktop UI/UX design](ui-ux-design.md) for layout dimensions, visual tokens, screen states, interaction rules and design acceptance checks. Requirements assigned to later milestones remain deferred.

## Supporting design

- [Section 3 — Component architecture](plan.md)
- [Section 5 — Translation data model (proposed SQLite v1)](plan.md)
- [Section 6 — Tauri command contract (initial proposal)](plan.md)
- [Section 12 — Next action — first coding slice](plan.md)

Follow the master plan’s quality gates and repository definition of done. Check tasks only after implementation and acceptance criteria are verified; keep completion status aligned with the master plan.
