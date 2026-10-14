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

### Translation Model tab — 2026-10-13 (explicitly authorized)

Settings gained a Translation Model tab: pick the agent (Codex, OpenCode,
Claude Code or a saved custom provider), then pick its model from a searchable
list. Model discovery per agent: Codex via `codex debug models --bundled`
(offline) with fallback to `codex debug models` plus a known-models catalog;
OpenCode via `opencode models` (local cache, no `--refresh`, so no network);
Claude Code via a built-in catalog of aliases and current full IDs (the CLI
offers no machine-readable list); custom providers via live `GET {base}/models`
with fallback to the saved model. Any model ID can also be entered manually.
The choice persists in app-owned SQLite (migration 5, `ai_translation_selection`)
for future translation work. Saving only remembers the choice — no translation
runs and no book content is sent.

### Assist panel — 2026-10-13 (explicitly authorized)

The Translation header is now Rich text | HTML | AI, keeping the reviewed and
keep-original actions. The AI view shows the saved agent/model, requires an
explicit per-run consent checkbox, and generates only on demand: the backend
spawns the installed Codex (`exec --model`), OpenCode (`run --model`) or
Claude Code (`-p --model --output-format text --max-turns 5`) CLI in an empty
temp directory with a 10-minute deadline, then validates the output against
the chapter schema. Runs are cancellable, terminal states arrive as
`ai-translation-event`s, and the proposal previews read-only with Insert into
draft (two-step confirm over existing content, always unreviewed) and Copy
text. The saved draft is never modified by a run; custom-API agents are
rejected for translation until a network provider path is designed. Prompt,
arg-shape, proposal-validation and stub-CLI runner/timeout/cancel coverage is
in `services/ai_translate.rs`; source loading in `storage/ai_translate.rs`.

### Structure-preserving redesign — 2026-10-13 (explicitly authorized)

Proposals now translate the chapter's original HTML body with identical
structure: only human-readable text may change. Tables, images, links, ids
and attributes must round-trip exactly — enforced by an element-skeleton
comparison (`services/ai_translate.rs`), with code-fence stripping and
`data:` image embedding for Rich-editor display. Any structural change rejects
the proposal with nothing inserted. Fresh (empty) drafts receive the result
straight into the Rich editor; chapters with content keep the preview +
explicit Insert flow. Stuck runs are diagnosable: a `started` event, an
elapsed ticker, surfaced event-subscription failures, and truncated tool
stderr on failures (local display only).

### Button + streaming dialog — 2026-10-13 (explicitly authorized)

AI is a separate header button, not a view tab (Rich text | HTML unchanged).
It opens a confirm dialog showing agent, model and direction with an explicit
Generate button. Runs stream the tool's stdout live (`stream` events with
full-so-far text into an auto-scrolling read-only view, ~750 ms throttle,
1 MB cap) instead of appearing suddenly; cancel/timeout/deadline behavior is
unchanged and the child is always killed and reaped.

### Proposal retry correction — 2026-10-13

The proposal dialog uses an accessible X close control. Retry carries the final
failed output back to the Rust service, which retains only complete top-level
XML sections matching the immutable source structure and requests the remaining
source sections. Truncated or structurally changed sections are regenerated;
combined output still requires full schema and structure validation. Partial
output is held in the open dialog, not persisted across closing/restart. A
complete but invalid proposal reports that there is no unfinished section to
continue. Native CLI/UI and cross-platform verification remain open.

### Reasoning effort selection — 2026-10-13 (explicitly authorized)

Translation Model now includes Default plus model-specific reasoning levels /
variants. Codex catalog parsing reads `slug` and `supported_reasoning_levels`;
OpenCode discovery uses local `models --verbose` metadata and enabled variant
names; Claude Code uses the documented per-model effort table. Models without
listed adjustable levels retain Default. Migration 6 adds nullable
`reasoning_effort` without replacing existing selections. Settings and proposal
summary display the saved effort, and Generate/Retry pass it to the CLI via
Codex `--config model_reasoning_effort="…"`, OpenCode `--variant`, or Claude
Code `--effort`. Rust checks the choice on save and execution; Default sends no
override. Codex catalog fallback/manual IDs allow documented level names with
model compatibility left to the CLI; OpenCode requires advertised variants.

Verified installed CLI help and local OpenCode metadata (Muse Spark 1.3 free
advertises minimal/low/medium/high/xhigh). Regression tests cover catalog
metadata, disabled variants, CLI arguments, invalid effort/model combinations,
and upgrading a saved v5 selection. Native model invocation/UI and
Windows/Linux remain the next unfinished verification tasks.

Sources: [Codex config reference](https://learn.chatgpt.com/docs/config-file/config-reference),
[OpenCode variants](https://opencode.ai/docs/models/#variants),
[Claude Code effort](https://code.claude.com/docs/en/model-config#adjust-effort-level).

### Codex temporary-directory fix — 2026-10-13

Codex translation runs now pass `exec --skip-git-repo-check`: their isolated
working directory is deliberately outside a Git repository. Regression tests
check this flag with both default and explicit reasoning effort. This skips
only the Git-directory check; it does not disable sandboxing or approval rules.

### Completed proposal validation correction — 2026-10-13

A completed proposal with a trailing `<hr/>` inside its credit paragraph was
rejected by the editor nesting check. The proposal boundary now uses parsed,
namespace-aware XML to move only trailing horizontal rules outside paragraphs,
preserving text, attributes, and rule order. The same normalization is used
for source/proposal comparison; the immutable source is never rewritten.
Other invalid nesting and active content remain rejected. Structural comparison
also includes element depth to catch reparented elements. Errors now include
validation details instead of hiding the reason.

Retry of a complete retained output revalidates it without launching another
translation; incomplete output still continues from matching complete sections.
The dialog buffers events arriving before the start command returns so a fast
local recovery cannot lose its terminal event. A synthetic Myanmar fixture
covers the credit paragraph, valid saved output, complete retry, multiple rules,
unsafe attributes and invalid nesting. All 112 Rust tests, frontend lint/build,
and formatting passed. The exact supplied chapter also passed a local validation
check; a recovered copy was saved beside its attachment without changing the
original. Native UI insertion and Windows/Linux remain unfinished verification.

### Disabled button contrast — 2026-10-13

Shared disabled buttons now use an opaque neutral background with muted text
and a visible border. This fixes Generate's gray-on-faded-blue label before
consent is checked, using the existing light/dark theme tokens. Consent and
disabled behavior are unchanged. Native visual verification remains open.

### Per-book metadata and translation instructions — 2026-10-13

Each workspace now has Book Settings beside the book title: export title,
primary author, publisher, description, source/target language and book-wide
AI guidance for consistent names, terminology and style. Migration 7 stores
metadata overrides and instructions separately from the immutable snapshot.
Validation precedes a transactional save; existing titles/drafts survive the
migration. Optional blank metadata retains the source value.

The AI proposal dialog shows saved book guidance and accepts an optional
current-run instruction. Rust reads book instructions by the chapter's project,
combines both sets with the standard translation/preservation prompt, and
limits each guidance field to 20,000 characters. Current-run naming/style
conflicts take precedence over book-wide guidance; preservation checks remain
mandatory. Changing run guidance clears retained partial output before Retry.
Instructions are sent only with an explicitly consented Generate/Retry and
are never written to the exported EPUB.

Export now applies saved metadata even when no new title is supplied in the
export request. Namespace-aware DC edits update only the primary creator,
retain IDs/refinements and additional creators, and add missing optional fields
with escaped Unicode/XML text. Integration tests reopen the resulting EPUB,
check all edited metadata, isolate guidance between two projects, reject
partial invalid saves, and confirm byte-identical source/snapshot preservation.
Frontend lint/build, cargo formatting and all 116 Rust tests passed. Native
Book Settings/AI interactions and cross-platform verification remain open;
full EPUBCheck conformance was not performed.

### Bundled-app CLI discovery fix — 2026-10-14

Release builds launched from Finder/Dock inherit a sparse `launchd` PATH, so
`codex` (nvm/npm shim), `opencode` (`~/.opencode/bin`) and `claude`
(`~/.local/bin`) reported "Not found on PATH" even when installed. CLI lookup
in `services/ai.rs` now searches process PATH, then the login-shell PATH
(bounded `printf %s "$PATH"` probe, constant args only), then well-known
per-user install directories (including nvm version bins) plus brew/system
prefixes, deduplicated with process-PATH precedence. Error messages no longer
claim PATH-only lookup and stay free of filesystem paths. Four regression
tests cover fallback listing, nvm expansion, shell-output parsing and
combined-directory resolution. Rust suite (101 unit, 19 export, 17 import),
frontend lint/build and cargo formatting passed; new code is clippy-clean
(six pre-existing clippy warnings in untouched files remain). Next unfinished
verification: rebuilt release app on macOS finding all three tools, plus
Windows/Linux checks.
