# Milestone 3 — Local autosave, progress and resume

**Status:** Implemented on macOS (verified 2026-10-12 via storage/integration tests and app smoke). Windows/Linux untested.  
**Source:** [Master plan](plan.md)  
**Repository rules:** [AGENTS.md](../AGENTS.md)

This file covers an implementation milestone within the manual-translation MVP (product Phase 1). File numbering follows the original milestone numbering; it does not change the product phases.

## Prerequisites

Build on [Milestone 2 — Side-by-side reading/translation UI](phase-2-translation-workspace.md).

## Tasks and acceptance criteria

- [x] Debounced autosave with revision conflict protection and retries.
- [x] Persist `pending`, `draft`, `reviewed`, `skipped` states.
- [x] Derive chapter status from required block review status.
- [x] Restore project and last chapter after a restart.
- [x] Flush saves before navigation and export.
- [x] Add recoverability and save-failure tests (including process interruption where feasible).

**Status notes**

- Conflicts keep the local draft and offer an explicit reload; committed
  revisions are the only recovery guarantee, matching the master plan.
- Process-kill-during-save is covered only indirectly (transactional
  single-statement writes); a dedicated kill test remains open in the
  release checklist.

**Done when:** edited translations survive app restart and chapter switching without data loss; failures are reported honestly.

## Desktop UI/UX requirements

Separate Saved/Saving/Unsaved/Error feedback from Pending/Draft/Reviewed/Keep original review states. Keep save failures visible, preserve local drafts and await acknowledged writes before navigation. Editing reviewed text returns it to Draft. Restore the active chapter; persist presentation preferences separately from authoritative translations.

Follow [Desktop UI/UX design](ui-ux-design.md) for layout dimensions, visual tokens, screen states, interaction rules and design acceptance checks. Requirements assigned to later milestones remain deferred.

## Supporting design

- [Section 5 — Translation data model (proposed SQLite v1)](plan.md)
- [Section 6 — Tauri command contract (initial proposal)](plan.md)
- [Section 7 — Autosave and recovery flow](plan.md)

Follow the master plan’s quality gates and repository definition of done. Check tasks only after implementation and acceptance criteria are verified; keep completion status aligned with the master plan.
