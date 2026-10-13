# Milestone 1 — EPUB import and chapter listing

**Status:** Planning — not implemented  
**Source:** [Master plan](plan.md)  
**Repository rules:** [AGENTS.md](../AGENTS.md)

This file covers an implementation milestone within the manual-translation MVP (product Phase 1). File numbering follows the original milestone numbering; it does not change the product phases.

## Prerequisites

Build on [Milestone 0 — Bootstrap & architecture foundation](phase-0-bootstrap.md).

## Tasks and acceptance criteria

- [ ] Implement archive and untrusted-input safety limits.
- [ ] Parse container.xml + OPF manifest/spine/metadata for EPUB 3.
- [ ] Parse EPUB navigation and resolve ordered chapter titles/anchors.
- [ ] Copy immutable source snapshot, hash it, and create project rows.
- [ ] Build stable editable block map for supported XHTML elements.
- [ ] Surface unsupported EPUB feature warnings instead of failing silently.
- [ ] Show imported project in library and chapter sidebar.
- [ ] Add fixtures: simple multi-chapter, nested paths, images, links, missing OPF, malformed ZIP.

**Done when:** importing a fixture shows correct title, author, source language, cover if present, and spine-ordered chapters. Imported original is not modified.

## Desktop UI/UX requirements

Implement the library’s empty, importing, success and error states; preserve focus on picker cancellation. Use compact project rows and spine-ordered chapter navigation. Show duplicate-source choices and actionable unsupported-content warnings without exposing internal logs.

Follow [Desktop UI/UX design](ui-ux-design.md) for layout dimensions, visual tokens, screen states, interaction rules and design acceptance checks. Requirements assigned to later milestones remain deferred.

## Supporting design

- [Section 4 — EPUB import design](plan.md)
- [Section 5 — Translation data model (proposed SQLite v1)](plan.md)
- [Section 6 — Tauri command contract (initial proposal)](plan.md)

Follow the master plan’s quality gates and repository definition of done. Check tasks only after implementation and acceptance criteria are verified; keep completion status aligned with the master plan.
