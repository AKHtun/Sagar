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

### Settings-only slice — 2026-10-13 (explicitly authorized)

Home (Library) now has AI settings, explicitly requested by the user. Scope is
configuration and availability only — no translation calls, no book content
leaves the device:

- Built-in CLI providers (Codex, OpenCode, Claude Code): PATH-only
  availability in the list; an explicit Check runs `<binary> --version` with a
  timeout. No process is spawned and no network request is made without the
  user pressing Check.
- Custom OpenAI-compatible providers: name, base URL, API key and model in
  app-owned SQLite (migration 4); reads are masked to key presence, secrets
  are never logged or serialized. An explicit Test performs a single
  `GET {base}/models` with the stored key; validation rejects non-http(s),
  credential-bearing, and query/fragment URLs.
- None of the milestone tasks above are complete; provider interface,
  prompt/context building, proposals-as-drafts, batch progress and keyring
  migration all remain open.
