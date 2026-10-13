# Milestone 2 — Side-by-side reading/translation UI

**Status:** Planning — not implemented  
**Source:** [Master plan](plan.md)  
**Repository rules:** [AGENTS.md](../AGENTS.md)

This file covers an implementation milestone within the manual-translation MVP (product Phase 1). File numbering follows the original milestone numbering; it does not change the product phases.

## Prerequisites

Build on [Milestone 1 — EPUB import and chapter listing](phase-1-epub-import.md).

## Tasks and acceptance criteria

- [ ] Design resizable 3-panel workspace (chapters, source, target).
- [ ] Render source chapter safely with images and applicable book CSS.
- [ ] Display target editor bound to stable source-block IDs.
- [ ] Support basic editing: Unicode text, paragraphs/headings, emphasis and inline links where safely mapped.
- [ ] Add keyboard navigation, selection/focus, and clear save/error state.
- [ ] Test Myanmar Unicode line wrapping, copy/paste and multi-line content.
- [ ] Validate EPUB.js/preview adapter compatibility with Tauri WebViews.

**Done when:** user can read one imported chapter and edit corresponding target blocks without altering source or breaking referenced images/links.

## Desktop UI/UX requirements

Implement the three-pane desktop workspace with resizable, keyboard-operable dividers and a collapsible chapter sidebar. Keep source read-only, target blocks editable and formatting limited to the validated schema. Apply the specified reading typography, active-block feedback, focus behavior and Myanmar input-method checks. Independent scrolling is the initial behavior.

Follow [Desktop UI/UX design](ui-ux-design.md) for layout dimensions, visual tokens, screen states, interaction rules and design acceptance checks. Requirements assigned to later milestones remain deferred.

## Supporting design

- [Section 2 — UX / screen architecture](plan.md)
- [Section 5 — Translation data model (proposed SQLite v1)](plan.md)
- [Section 11 — Known technical risks / mitigations](plan.md)

Follow the master plan’s quality gates and repository definition of done. Check tasks only after implementation and acceptance criteria are verified; keep completion status aligned with the master plan.
