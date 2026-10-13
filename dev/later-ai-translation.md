# Later — AI translation (explicitly NOT Phase 1)

**Status:** Deferred — requires explicit user authorization; outside product Phase 1  
**Source:** [Master plan](plan.md)  
**Repository rules:** [AGENTS.md](../AGENTS.md)

This file covers a deferred roadmap track. It does not authorize implementation or change the MVP scope.

## Prerequisites

Begin only after the user explicitly changes the product phase to permit AI integration.

## Tasks and acceptance criteria

- [ ] Provider-independent `TranslationProvider` interface with cancellable jobs.
- [ ] Prompt/context builder with glossary and per-block identity.
- [ ] API key security and opt-in network/privacy disclosures.
- [ ] Proposals stored as separate drafts, not automatic overwrites of reviewed manual translations.
- [ ] Batch progress, retry, costs, rate limits and user review.

## Supporting design

- [Section 3 — Component architecture](plan.md)
- [Section 5 — Translation data model (proposed SQLite v1)](plan.md)
- [Section 6 — Tauri command contract (initial proposal)](plan.md)

Follow the master plan’s quality gates and repository definition of done. Check tasks only after implementation and acceptance criteria are verified; keep completion status aligned with the master plan.
