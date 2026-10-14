# Plan.md — EPUB Translation Studio

### Codex App Server text streaming — 2026-10-14

Supersedes the stderr display approach, which the user confirmed only showed
startup text until completion. Codex translation and HTML repair now use a
Rust stdio client for codex app-server: initialize, ephemeral read-only thread,
selected model/effort, and a single turn. item/agentMessage/delta feeds translated
text into the existing single output box; final messages are deduplicated and
successful turn completion is required before the existing XHTML/skeleton checks.
Removed the separate diagnostics box. Other providers retain their adapters.
No Node/Python runtime, TCP listener, source mutation or user config changes.
The process is reaped on success, cancellation, timeout and protocol errors;
messages/output are bounded, stale turns/commentary are ignored, and unexpected
interactive actions fail explicitly rather than approving tools.
Verified the installed protocol schema and an authenticated Rust adapter probe
using gpt-6.1-sol / low: six text deltas, first at 4.17s, completion at 4.42s
for synthetic short text. This is evidence of live deltas, not a chapter speed
benchmark. Rust formatting, 125 unit tests, 20 export and 18 import tests passed;
the live probe passed separately (normally ignored), and the performance sweep
remains ignored. Frontend lint/build and 18 tests passed, including early/stale
stream routing, single textarea, Myanmar text, Cancel/Escape and focus restore.
Existing bundle-size warning remains. Next unfinished task: rebuild/restart and
verify the same chapter/model/effort in the native app, with a timing comparison.
Native themes/minimum-size/IME and Windows/Linux remain unverified. No checkboxes
changed.

### Codex live stderr progress — 2026-10-14

Preserved Codex's restored plain exec arguments and streamed its stderr into a
separate Codex progress box in both translation and HTML repair dialogs. Stdout
alone supplies the proposal, validation and retry content. Progress activity
resets the silence timer; stale jobs are ignored and early HTML repair events
are queued until the start response returns. Diagnostic pipes and retained
history are bounded (128 KiB tail); progress updates are throttled and flushed
at completion. No saved draft/source mutation or JSON mode reintroduction.
Rust formatting and all 31 translation-service tests passed. Regression coverage
verifies progress before final stdout, proposal separation, stderr flooding,
cancellation and continued structured output for other providers. Frontend
lint/build and all 18 tests passed; the dialog regression covers early/stale
progress, Myanmar text, readonly output, Cancel/Escape and focus restoration.
Existing bundle-size warning remains. No running Sagar window was available;
native light/dark, minimum-window, IME and Windows/Linux behavior remain
unverified. Next unfinished task: rebuild/restart and verify live progress with
the same chapter/model/effort, including a timing comparison. No checkboxes changed.

### Codex streaming rollback — 2026-10-14

User reported a chapter run increasing from about 140 seconds to over 200
seconds after enabling JSON mode. Restored Codex's previous exec arguments
without --json and its plain stdout execution path for translation and HTML
repair. Other providers retain structured streaming. The timing cause is not
confirmed; no same-chapter performance claim is made. Rust formatting and all
28 translation-service tests passed, including a plain Codex process regression.
Next unfinished task: rebuild/restart Sagar and time the same chapter with the
same model/effort; an already-running job is unaffected by source changes.
Native and cross-platform verification remain unverified. No checkboxes changed.

### Structured streaming for existing CLI providers — 2026-10-14

Codex now uses exec --json, OpenCode uses run --format json, and Claude Code
uses stream-json with verbose and include-partial-messages. Provider decoders
feed response text into the existing translation/HTML repair progress box,
ignore reasoning/tools, deduplicate text snapshots, and require successful
completion before validating proposals. Codex uses the last completed agent
message; OpenCode retains the final assistant message's text parts.
Live synthetic probes verified all three installed CLIs. Claude emitted text
deltas; OpenCode emitted a completed text part; Codex emitted only a completed
agent message. Codex exec therefore does not promise token-by-token output.
Rust formatter and translation service tests, plus frontend lint/build passed;
the existing bundle-size warning remains. Native dialog, keyboard/IME and
Windows/Linux verification remain unverified. Next unfinished task: native
verification and, if requested, a Codex transport with token delta support.
No milestone checkboxes changed.

### Antigravity CLI provider — 2026-10-14

Added Antigravity (agy) to built-in provider discovery/checks, live `agy models`
listing and saved translation selection, with low/medium/high effort controls.
Translation and HTML repair run headless with stream-json; only agent response
deltas appear in progress and the successful terminal response is validated.
Malformed, failed or missing results are rejected. Existing cancellation,
timeouts, isolated working directories and proposal review remain in use.
Verified installed agy 1.1.23 version/model probes. Frontend lint/build and
17 frontend tests passed; Rust unit/export/import tests passed (performance
sweep ignored). Build retains the existing bundle-size warning.
Next unfinished task: native provider selection and authenticated translation/
repair verification at normal/minimum sizes in both themes; Windows/Linux and
native keyboard/IME behavior remain unverified. No milestone checkboxes changed.

### Blockquote and code authoring — 2026-10-14

Enabled inline code and standard pre/code blocks in the chapter editor while
retaining plain pre elements and imported code attributes. Formatting toolbar
now has Blockquote, Inline code and Code block actions with icons, tooltips,
selected states and capability-aware disabled states. Slash menu supports
/blockquote (also /quote), /code and /inline code; applying a command removes
its query and formats in one undoable transaction. Added theme-token styling
for quotes/code, wrapping code lines without losing saved indentation.
All 17 frontend tests, frontend lint/build, and 13 Rust rich-text validator tests
passed. The build retains the existing bundle-size warning. Regression checks
cover Myanmar text, HTML escaping, pre/code round trips, slash-query removal,
undo, inline-code input, formatting toggles and disabled editors.
Native light/dark, minimum-window, toolbar/slash keyboard interaction and IME
checks remain unverified because no current running Sagar build was available.
Next unfinished task: native verification of these controls; broader XHTML
preservation work remains tracked below. No milestone checkboxes changed.


### Rich XHTML preservation expansion — 2026-10-14

The chapter Rich editor now retains structural containers (including imprint
section/header IDs and classes), text-only container variants, definition lists,
preformatted text, inline spans/b/i/code/sup/sub/ruby/rt/rp, inline images,
EPUB semantics, xml:lang, image srcset/sizes and layout attributes. SVG and picture
subtrees use protected nodes with inert editor placeholders and retained markup;
they remain editable in HTML. SVG namespaces are kept during serialization.
Disabled automatic trailing paragraphs and retained significant preformatted
whitespace so changing views does not rewrite these structures.
Regression coverage includes imprint markup, Myanmar text and links, inline
semantics/images, definition lists/pre, SVG namespaces and responsive images,
reload/edit cycles, and inert SVG node views. Frontend lint/build and all 12
frontend tests passed; build retains the existing bundle-size warning. Rust tests
passed (107 unit, 20 export, 18 import; performance sweep ignored).
No running Sagar window was available: native IME, keyboard, minimum-size,
light/dark and cross-platform verification remain unverified.
This is an expansion, not a claim of universal EPUB XHTML support. Next unfinished
task: deliberate models for complex table sections and mixed block/inline
containers, followed by safe MathML/media/form preservation and native verification.
The Rust safety policy and loss-aware view-switch guard remain in force. No
milestone checkboxes changed and no existing unrelated edits were reverted.


### Imprint markup comparison regression — 2026-10-14

Chapter proposal comparison, continuation, normalization and image embedding now
use the same XHTML/EPUB/XLink namespace context as the restricted editor.
This fixes inherited epub:type declarations being lost when extracting a body.
A synthetic imprint regression covers initial runs, partial retries, Myanmar text,
image srcset/link preservation, changed attributes and unknown-prefix rejection.
Rust formatting and 107 unit, 20 export and 18 import tests passed; one performance
sweep remains ignored. Frontend build passed with the existing bundle-size warning.
Combined frontend checks stopped on existing richText.ts formatting differences.
Next unfinished task: verify the imprint chapter in a rebuilt native app;
full proposal acceptance and Windows/Linux behavior remain unverified.
No milestone checkboxes changed.


### XHTML 1.1 import compatibility — 2026-10-14

The shared XML parser accepts the standard inert XHTML 1.1 PUBLIC declaration
without fetching a DTD; internal entity declarations and arbitrary external
DTDs remain rejected. Synthetic coverage checks editable Myanmar content,
preview, draft export, byte preservation and source immutability. Rust formatting
and 106 unit, 20 export and 18 import tests passed (performance sweep ignored).
The supplied book additionally uses named nbsp/copy entities in its verso page,
which remain unsupported in the strict parser. A separate compatibility copy
removes legacy declarations and replaces those references with numeric equivalents
using XML events; all 24 chapters load, 1706 blocks are editable, and draft
export validates. Original source files remain unchanged. Next unfinished task:
native import/edit verification of the compatibility copy; general named-entity
and EPUB 2/NCX compatibility remain open. No milestone checkboxes changed.

### Unpacked EPUB folder import feedback — 2026-10-14

Import now explains that an EPUB folder must be packaged as a .epub archive.
A regression confirms rejection preserves the folder and creates no project
storage. Rust formatting and 18 import tests passed (one performance sweep
ignored). A separate archive of the supplied local folder passed Sagar's
parser with 24 chapters and two book warnings; the source folder was unchanged.
Next unfinished verification: import that archive in the native app and review
its content warnings. Native/cross-platform UI and EPUBCheck remain unverified;
no milestone checkboxes changed.

### SVG cover preview and DOCTYPE export regression — 2026-10-14

Source XML parsing accepts the inert HTML5 doctype while rejecting internal
entity declarations and external DTDs. Chapter loading, segmentation, navigation,
patching and export validation share this behavior. Preview supports a restricted
passive SVG subset, rewrites image href/xlink:href to local snapshot URLs, and
drops scripts, foreignObject, animations, CSS and URL paints. Original SVG bytes
remain preserved in exports; arbitrary SVG authoring is still unsupported.
Synthetic regressions cover cover loading, sanitization, export validation and
resource/source byte preservation. Rust checks passed: 93 unit, 19 export and
17 import tests (large-book sweep ignored). A temporary diagnostic also imported,
loaded the cover and exported the supplied pg11-images-3.epub successfully.
Next unfinished verification: rebuilt native preview/export in both themes and
an external EPUB reader/EPUBCheck; cross-platform rendering remains unverified.
No milestone checkboxes changed.

### AI italic/bold formatting instructions — 2026-10-14

Translation instructions identify em/strong as editable italic/bold formatting
and prohibit introducing unsupported tags while retaining source structure.
AI Fix HTML explicitly converts i/b to em/strong without losing text, valid
attributes or formatting. All five prompt tests passed. Next unfinished check:
verify a native AI Fix HTML run on a draft containing i tags; no milestone
checkboxes changed.

### Reviewed chapter indicator — 2026-10-14

The chapter review button stays green with a tinted background when the draft
status is reviewed, including hover. Its pressed state and accessible label/
tooltip identify Chapter reviewed. Editing resets status to draft through the
existing save service and clears the green state. Lint/build and all four
frontend tests passed. Next verification: native reviewed/edit/reload states
in both themes; no milestone checkboxes changed.

### Running-action button — 2026-10-14

Both AI dialogs use the shared red/white Cancel button with the close icon.
Frontend lint/build and four tests passed. Native rendered verification remains
pending on a current build; no milestone checkboxes changed.

### ZIP directory export regression — 2026-10-14

Export copies validated filenames rather than treating the filtered file list
as raw ZIP indices. Explicit directory entries previously truncated the copy
loop and dropped trailing resources. The supplied local project reproduced a
missing toc.ncx; export from an isolated database/snapshot copy now succeeds,
retains every source resource and preserves navigation/CSS bytes. A synthetic
integration regression covers multiple directory entries and source immutability.
All 18 export tests passed. Next unfinished verification: restart the native
app, export and reopen in a reader; EPUBCheck/cross-platform checks remain open.

### HTML view simplification — 2026-10-14

The user's latest preference supersedes displaying the read-only document
wrapper: HTML view shows only editable chapter content, with no wrapper
explanation. Original head, stylesheet links and body attributes remain
preserved by export. Native verification remains pending on a current build.

### Button consistency — 2026-10-14

All visible text buttons pair their labels with shared icons; icon-only actions
remain supported. Language and export dialog header dismissal uses an accessible
X button, and language saving uses Save/Saving…. Shared tab, heading and language
control spacing accommodates icon/text groups. AGENTS.md records this convention.
Frontend lint/build and all four tests passed; a source audit found no remaining
text-only buttons. The running native app still shows an older build, so current
native light/dark, minimum-size and keyboard verification remains unfinished.
Next task: verify the updated buttons in a rebuilt desktop app. No milestones
are advanced.

### Full XHTML authoring context — 2026-10-14

Translation HTML view shows the original XML declaration, document head,
stylesheet links and opening body above the editable chapter body, followed
by the original closing wrapper. The wrapper is read-only and comes from
namespace-aware source parsing in Rust, with original bytes retained for
ordinary bodies. Empty bodies are displayed with expanded opening/closing
tags. Saved drafts remain body content; export retains the source wrapper.
Regression tests verify stylesheet/head/body-class preservation and prefixed
and empty bodies. Frontend lint/build and four tests, Rust formatting and
the Rust suite passed (large-book sweep remains ignored). Next unfinished task:
restart the desktop development build and verify this view at normal/minimum
sizes in both themes, with keyboard and Myanmar IME checks. Milestones remain
unchanged.

### EPUB anchor preservation — 2026-10-14

Rich chapter editing now represents empty `a[id]` navigation targets as inline
nodes and preserves paragraph whitespace when loading or converting XHTML.
HTML-to-Rich applies the exact parsed document that passed the existing loss
check. A synthetic Myanmar regression fixture covers anchors, links, trailing
spaces, edits and reloads; the supplied chapter also passed a local round-trip
check. Native IME, minimum-window, light/dark and cross-OS checks remain open.
No milestone checkboxes are advanced. Next unfinished task: verify this
conversion and Myanmar composition in the running desktop build.

**Status:** Milestones 0–4 implemented and verified on macOS (import → translate → autosave → export). Milestone 5 is partially done: polish/docs verified on macOS; installers and cross-OS checks remain open (see docs/release-checklist.md).  
**Target:** Windows, macOS and Linux  
**Phase 1:** Manual side-by-side EPUB translation and EPUB export  
**Chosen scaffold:** Tauri 2 + React + TypeScript + **Bun**  
**Phase 2 (deferred):** AI-assisted translation and review

## 1. Product overview

Desktop application for manually translating a book chapter by chapter, without losing the original EPUB formatting or file structure.

**Primary user journey**

```text
Open EPUB
  -> Validate / import source snapshot
  -> Read metadata, OPF spine, TOC and resources
  -> Create local translation project
  -> Choose chapter
  -> View original | edit translated text
  -> Autosave progress and review missing blocks
  -> Repeat for all chapters
  -> Preview output / update language and title
  -> Export translated EPUB (separate file)
  -> Validate and reopen exported book
```

### MVP goals

- Open a **DRM-free, text-heavy, reflowable EPUB 3** book from disk.
- Provide a split editor: **original (left, read-only)** and **translation (right, editable)**.
- Support a chapter list and navigation based on the book's reading order.
- Save each chapter's translation automatically; restore a project after restarting the app.
- Preserve cover, image assets, chapter order, CSS where possible, link targets and footnotes.
- Track status for blocks and chapters; show incomplete content before export.
- Export a standalone translated EPUB file with updated book title/language metadata.
- Work entirely offline with no AI service or login.

### MVP exclusions

AI translation and provider keys, EPUB 2/NCX (until compatibility milestone), fixed-layout, DRM bypass, PDF/MOBI, collaborative editing, cloud storage, embedded authoring/source-code IDE, arbitrary CSS editing and full EPUB accessibility authoring tools.

## 2. UX / screen architecture

Detailed visual and interaction requirements: [Desktop UI/UX design](ui-ux-design.md). This specification supplements the screen architecture below and maps delivery to Milestones 0–5.

### A. Project Library

- **Open EPUB** button and native file picker.
- Recent projects, title/cover/author, source/target language, last edited, completion count.
- Resume an existing project. Re-importing the same source warns about duplicates by content hash.
- Clearly identify unsupported EPUBs and specific reasons.

### B. Translation Workspace

```text
+---------------------------------------------------------------------+
| Book title           Save: Saved           Preview       Export EPUB |
+----------------------+-------------------------+--------------------+
| Chapters             | ORIGINAL                | TRANSLATION        |
|  01  Completed       | read-only EPUB content  | editable blocks    |
|  02  In progress     | source heading/image    | target heading     |
| >03  In progress     | paragraph A             | translation A      |
|  04  Not started     | paragraph B             | translation B      |
|                      | paragraph C             | translation C      |
|                      |                         |                    |
| Book progress        | synchronized block      | status per block   |
+----------------------+-------------------------+--------------------+
| Chapter 3 of 20      | 8 / 12 blocks reviewed   | Autosaved 12:30 PM|
+---------------------------------------------------------------------+
```

- Sidebar: chapters, status (`not_started`, `in_progress`, `completed`), progress.
- Resizable left/right panes. Comfortable Myanmar font and line-height; independent scrolling initially. Block-linked scrolling later, after editor stability.
- Original pane renders protected/read-only content, including images, headings and links.
- Target pane presents a chapter document with restricted rich text, a shared formatting toolbar, and local/current-chapter image insertion. Original pane offers rendered Reader and read-only HTML modes.
- Target pane edits supported content blocks. Users can copy original text into target, save, undo, and mark blocks reviewed.
- Search inside the selected chapter (post-core milestone if necessary).
- Never claim an empty block is complete merely because its chapter was visited.
- Chapter changes wait for or flush pending writes; warn on persistent save failures.

### C. Export / Review

- Set translated book title and target language (`my`, `en`, etc.; store BCP 47 tags).
- Show completed/pending/unsupported block and chapter counts.
- Prevent silently exporting an unfinished book; allow explicit **Export draft** with warning and defined fallback behavior.
- Open native Save dialog, write to a new path, validate ZIP/OPF/reference integrity, and show success/error.
- Never replace the original imported file by accident.

## 3. Component architecture

```text
React + TypeScript (Bun/Vite)                    Tauri 2 / Rust
+---------------------------------+            +------------------------+
| ProjectLibrary                   |            | Tauri commands         |
| TranslationWorkspace            | <--------> | Application services   |
|  - ChapterSidebar               | typed      |  - ProjectService      |
|  - SourceChapterView            | commands   |  - ImportService       |
|  - TargetBlockEditor            |            |  - TranslationService  |
|  - SaveState / Progress         |            |  - ExportService       |
| ExportReviewDialog              |            +----------+-------------+
+---------------------------------+                       |
                                                          v
                                      +---------------------------------+
                                      | EPUB engine                     |
                                      | ZIP/OCF + OPF + XHTML + nav    |
                                      | source resource map / rewrite  |
                                      +----------------+----------------+
                                                       |
                                      +----------------v----------------+
                                      | Local project storage          |
                                      | SQLite + immutable source EPUB |
                                      | drafts / original resources    |
                                      +---------------------------------+
```

### Responsibilities

| Layer | Owns | Must not own |
| --- | --- | --- |
| React UI | display, navigation, editing UX | ZIP parsing, SQLite queries |
| Typed Tauri bridge | DTOs and command calls | business rules |
| Rust services | import, save, status, export orchestration | JSX/UI state |
| EPUB engine | archive, OPF/spine/nav, XHTML transforms | frontend behavior |
| SQLite repository | transactions and migrations | EPUB rendering |

**Initial Rust modules**: `commands`, `domain`, `services`, `epub`, `storage`, `security` (create as implemented, not as empty placeholders).

## 4. EPUB import design

**Import steps**

1. Native open dialog (`.epub` filter).
2. Rust validates file accessibility, ZIP structure, archive sizes/counts, path normalization, duplicate/symlink entries, and `mimetype`.
3. Read `META-INF/container.xml` to locate the OPF package, respecting relative paths.
4. Parse OPF metadata, manifest, spine and EPUB 3 navigation document with namespace-aware XML processing.
5. Resolve spine item IDs to XHTML resource paths. A TOC entry may point to an anchor within a chapter; do not assume one nav entry = one file.
6. Create immutable source snapshot under app project storage and compute `source_sha256`.
7. Extract an ordered block map for editable headings, paragraphs, list items, block quotes etc. Retain the original resource path and stable structural locator for each block.
8. Create project/chapter/block rows and a translation draft transactionally.
9. Display project overview; surface unsupported content with actionable warnings.

**Security:** never extract ZIP paths unchecked or allow embedded scripts to run in a privileged WebView. Restrict local asset serving to app-owned imported resources; prevent network fetches from book content.

**EPUB.js decision spike:** verify chapter preview, inline images, CSS, relative links, Burmese rendering and Tauri WebView security on all three desktop OSes. Prefer a safe document renderer if the preview library cannot satisfy these constraints. Reader rendering must never become the authoritative export source.

## 5. Translation data model (proposed SQLite v1)

The persisted block model is authoritative for manual edits. This schema is a starting point; migrations may refine it after the first import/export spike.

```sql
CREATE TABLE projects (
  id TEXT PRIMARY KEY,
  source_sha256 TEXT NOT NULL,
  source_snapshot_path TEXT NOT NULL,
  title TEXT NOT NULL,
  author TEXT,
  source_language TEXT,
  target_language TEXT NOT NULL,
  translated_title TEXT,
  created_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  last_chapter_id TEXT,
  schema_version INTEGER NOT NULL DEFAULT 1
);

CREATE TABLE chapters (
  id TEXT PRIMARY KEY,
  project_id TEXT NOT NULL REFERENCES projects(id) ON DELETE CASCADE,
  spine_index INTEGER NOT NULL,
  manifest_item_id TEXT NOT NULL,
  resource_href TEXT NOT NULL,
  title TEXT NOT NULL,
  status TEXT NOT NULL DEFAULT 'not_started',
  UNIQUE(project_id, spine_index)
);

CREATE TABLE blocks (
  id TEXT PRIMARY KEY,
  chapter_id TEXT NOT NULL REFERENCES chapters(id) ON DELETE CASCADE,
  block_index INTEGER NOT NULL,
  structural_locator TEXT NOT NULL,
  element_name TEXT NOT NULL,
  source_xhtml_fragment TEXT NOT NULL,
  source_plain_text TEXT NOT NULL,
  is_translatable INTEGER NOT NULL DEFAULT 1,
  support_status TEXT NOT NULL DEFAULT 'supported',
  UNIQUE(chapter_id, block_index)
);

CREATE TABLE translations (
  block_id TEXT PRIMARY KEY REFERENCES blocks(id) ON DELETE CASCADE,
  target_xhtml_fragment TEXT,
  target_plain_text TEXT,
  status TEXT NOT NULL DEFAULT 'pending',
  revision INTEGER NOT NULL DEFAULT 0,
  updated_at TEXT,
  reviewed_at TEXT
);

CREATE INDEX idx_chapters_project ON chapters(project_id, spine_index);
CREATE INDEX idx_blocks_chapter ON blocks(chapter_id, block_index);
```

**Implementation notes**

- Enable SQLite foreign keys, WAL mode and migrations; keep SQLite in app-owned storage.
- Use UUIDs or equivalent stable IDs assigned **once at import**, never frontend array indexes.
- `structural_locator` is a stable-in-snapshot DOM locator, not a fragile displayed paragraph number. Store resource-relative URI and IDs necessary to remap references.
- `source_xhtml_fragment` represents the original content; keep original EPUB resource bytes as source of truth for export.
- `target_xhtml_fragment` should be a sanitized **inner fragment for a supported block**, not arbitrary XML from the browser. Validate element/attribute allowlists and namespaces before serialization.
- Manage changes with optimistic `revision` checking so a delayed autosave cannot replace a newer edit.
- Do not duplicate full book text in analytics or logs. SQLite stores translation working data only.

### Block/chapter status rules

- Translation block: `pending` (no reviewed content), `draft` (saved but unreviewed), `reviewed` (user confirmed), `skipped` (user intentionally retains original).
- Chapter: `not_started`, `in_progress`, `completed` (derived from required block states; optional cached status must be kept consistent).
- Unsupported content: tracked separately; cannot be auto-marked complete.
- Completion percentage: `reviewed + intentional skipped` / total supported required blocks, with unsupported warnings shown separately.

**Important editorial trade-off:** paragraph-to-paragraph alignment makes progress/scrolling reliable, but some translations need to merge or split sentences. Keep alignment at **block level**, allow free editing *within* a target block; defer cross-block merge/split to a deliberate later milestone. Nontrivial structures (tables, ruby, MathML, complex footnotes) must be detected rather than corrupted.

## 6. Tauri command contract (initial proposal)

TypeScript wrappers should return typed DTOs and map domain errors to user-readable states. Command names/DTOs may evolve, but the responsibilities should remain stable.

```ts
type ProjectId = string;
type ChapterId = string;
type BlockId = string;

type TranslationStatus = 'pending' | 'draft' | 'reviewed' | 'skipped';

type BlockDTO = {
  id: BlockId;
  sourceHtml: string;      // sanitized for display, never executable
  sourceText: string;
  targetHtml: string | null;
  status: TranslationStatus;
  revision: number;
  supportStatus: 'supported' | 'unsupported';
};

type ExportOptions = {
  title: string;
  targetLanguage: string;
  mode: 'completed' | 'draft';
};
```

| Command | Input | Output |
| --- | --- | --- |
| `import_epub` | selected source path | `ProjectSummary`, import warnings |
| `list_projects` | none | project summaries |
| `get_project` | `projectId` | project metadata/progress |
| `list_chapters` | `projectId` | ordered chapter summaries |
| `get_chapter` | `chapterId` | blocks + read-only resource locator |
| `save_translation` | `blockId`, target fragment, status, expected revision | saved revision + timestamp |
| `update_project_metadata` | `projectId`, translated title, language | project |
| `review_export` | `projectId` | missing/unsupported/invalid content report |
| `export_epub` | `projectId`, destination path, options | output path + validation report |

**Do not trust paths or HTML passed from React.** Rust validates file selection permissions, canonicalized paths, and output safety independently. Keep long imports/exports off the UI thread and report progress or cancellation when appropriate.

## 7. Autosave and recovery flow

```text
User edits a target block
  -> local UI draft updates immediately
  -> debounce (e.g. 400-800 ms; tune after testing)
  -> save_translation(blockId, content, status, expectedRevision)
  -> SQLite transaction updates content + revision + timestamp
  -> UI confirms Saved / shows failure and retry

Before chapter change / export / app close:
  -> flush pending edits where possible
  -> wait for acknowledged writes
  -> abort export and warn if save failed

On reopen:
  -> load project, selected chapter, saved translations
  -> resume from latest committed revisions
```

Avoid relying solely on an app-close callback (crashes and forced closes skip it). Consider a small in-memory unsaved-draft queue for transient failures, but don't represent uncommitted drafts as safely stored. Add recovery tests for killed/restarted processes once core persistence exists.

## 8. Export pipeline — most important correctness requirement

```text
User clicks Export
  -> flush saves
  -> review missing, skipped and unsupported blocks
  -> collect metadata and draft/complete decision
  -> copy immutable source EPUB into a temporary work archive
  -> map each translated block to its original XHTML resource
  -> replace only supported, intended content fragments
  -> update translated title/language + TOC labels where editable
  -> update nav/NCX as applicable; preserve spine/IDs/hrefs/assets
  -> produce OCF-correct ZIP
  -> validate internal references, XML, resource files
  -> EPUBCheck (optional developer/integration step)
  -> reopen output as a reader smoke test
  -> finalize destination file atomically where supported
  -> report export result and any warnings
```

### Export rules

- For Phase 1, **do not convert** the entire source book to Tiptap JSON and back. Patch only approved translated blocks into a copy of the original XHTML tree.
- Maintain original outer tags and stable resource IDs where possible; preserve embedded anchors/footnotes and reject edits that would break them.
- Rewriting a block must be XML-safe: escape text, serialize valid XHTML, maintain namespaces, and preserve required semantic attributes.
- Resource-level originals (covers, images, fonts, CSS, package data) stay unchanged unless deliberately edited.
- Package metadata: update `dc:language`, translated `dc:title` if provided; retain authors, publication data, identifiers by default; choose edition/identifier policy explicitly before publicly distributing translated editions.
- Translate visible TOC labels as a defined field or from reviewed headings; maintain each target `href`. Never blindly translate nav XML text without verifying its intended mapping.
- Write OCF `mimetype` entry **first, uncompressed**. Preserve `META-INF/container.xml`, manifest entries and spine order.
- For **draft export**, untranslated supported blocks retain original text with a clear warning/report; unsupported content is preserved unchanged. For **completed export**, block if required untranslated/unsupported items remain unless the user has explicitly reviewed a skip policy.
- Save to a different path than the source file or its immutable snapshot; confirm before replacing any unrelated existing output.
- Use temporary output and safe rename/copy fallback, avoiding half-written destination files.
- Export failures must not modify saved project translations.

## 9. Implementation roadmap

Checkboxes are checked only after the functionality exists and its acceptance criteria are verified.

### Phase files

The files below break down the existing roadmap without changing its scope or completion status. Numbered phase files follow Milestones 0–5 within product Phase 1; compatibility and AI remain deferred tracks.

- [x] [Milestone 0 — Bootstrap & architecture foundation](phase-0-bootstrap.md)
- [x] [Milestone 1 — EPUB import and chapter listing](phase-1-epub-import.md)
- [x] [Milestone 2 — Side-by-side reading/translation UI](phase-2-translation-workspace.md) (independent chapter rich text documents and image insertion; native IME checks pending)
- [x] [Milestone 3 — Local autosave, progress and resume](phase-3-autosave-and-resume.md)
- [x] [Milestone 4 — EPUB export (vertical-slice first)](phase-4-epub-export.md) (EPUBCheck pending)
- [ ] [Milestone 5 — Polish & desktop release](phase-5-desktop-release.md) (macOS polish done; installers/cross-OS open)
- [Milestone 2 — Side-by-side reading/translation UI](phase-2-translation-workspace.md)
- [Milestone 3 — Local autosave, progress and resume](phase-3-autosave-and-resume.md)
- [Milestone 4 — EPUB export (vertical-slice first)](phase-4-epub-export.md)
- [Milestone 5 — Polish & desktop release](phase-5-desktop-release.md)
- [Later — EPUB compatibility improvements](later-epub-compatibility.md)
- [Later — AI translation (explicitly NOT Phase 1)](later-ai-translation.md)

### Milestone 0 — Bootstrap & architecture foundation

- [x] Verify generated Tauri 2 + React + TS project runs with **Bun** on the current development OS.
- [x] Remove demo UI; create minimal app shell and navigation.
- [x] Add strict TS checks, format/lint conventions and relevant scripts.
- [x] Add Rust `domain`, `services`, `epub`, `storage` structure as functionality is introduced.
- [x] Implement typed Tauri command bridge and reusable error reporting.
- [x] Add SQLite schema/migration initialization with an automated persistence test.
- [x] Add native dialog plugin and narrowly scoped capabilities for import/export; avoid broad filesystem permissions.

**Done when:** `bun tauri dev` launches, the frontend invokes a typed Rust test command, and a local SQLite migration test passes.

### Milestone 1 — EPUB import and chapter listing

- [x] Implement archive and untrusted-input safety limits.
- [x] Parse container.xml + OPF manifest/spine/metadata for EPUB 3.
- [x] Parse EPUB navigation and resolve ordered chapter titles/anchors.
- [x] Copy immutable source snapshot, hash it, and create project rows.
- [x] Build stable editable block map for supported XHTML elements.
- [x] Surface unsupported EPUB feature warnings instead of failing silently.
- [x] Show imported project in library and chapter sidebar.
- [x] Add fixtures: simple multi-chapter, nested paths, images, links, missing OPF, malformed ZIP.

**Done when:** importing a fixture shows correct title, author, source language, cover if present, and spine-ordered chapters. Imported original is not modified.

### Milestone 2 — Side-by-side reading/translation UI

- [x] Design resizable 3-panel workspace (chapters, source, target).
- [x] Render source chapter safely with images and applicable book CSS.
- [x] Display target editor bound to stable source-block IDs.
- [ ] Support basic editing: Unicode text, paragraphs/headings, emphasis and inline links where safely mapped.
- [x] Add keyboard navigation, selection/focus, and clear save/error state.
- [ ] Test Myanmar Unicode line wrapping, copy/paste and multi-line content.
- [x] Validate EPUB.js/preview adapter compatibility with Tauri WebViews.

**Done when:** user can read one imported chapter and edit corresponding target blocks without altering source or breaking referenced images/links.

### Milestone 3 — Local autosave, progress and resume

- [x] Debounced autosave with revision conflict protection and retries.
- [x] Persist `pending`, `draft`, `reviewed`, `skipped` states.
- [x] Derive chapter status from required block review status.
- [x] Restore project and last chapter after a restart.
- [x] Flush saves before navigation and export.
- [x] Add recoverability and save-failure tests (including process interruption where feasible).

**Done when:** edited translations survive app restart and chapter switching without data loss; failures are reported honestly.

### Milestone 4 — EPUB export (vertical-slice first)

- [x] Implement a 1-chapter translated EPUB round trip on an intentionally simple fixture.
- [x] Patch translated XHTML fragments into source resource tree without flattening assets.
- [x] Repackage EPUB with correct OCF entry ordering/compression.
- [x] Update title, language and navigation labels without breaking hrefs.
- [x] Add native Save dialog, explicit draft mode and unfinished-content review.
- [x] Validate output ZIP, XHTML parsing, manifest targets and links.
- [x] Reimport the exported file and compare chapter/resource counts and spine order.
- [x] Add regression fixtures for formatting, internal links, images and footnotes.

**Done when:** a fully manually translated fixture exports, passes the project's integrity checks, reopens in a reading app, and retains its structural assets. EPUBCheck should also pass on supported compliant fixtures in CI/dev test environments.

### Milestone 5 — Polish & desktop release

- [x] UX review: dark mode, pane resizing, text zoom, shortcuts, error dialogs.
- [ ] Accessibility and Burmese text rendering cross-OS tests.
- [x] Performance checks with large books (bounded RAM/use lazy loading; 5,001 chapters / 56.9 MB macOS sweep, see Milestone 5 evidence).
- [ ] Package and smoke-test installers for Windows/macOS/Linux.
      (macOS .app/.dmg built and smoke-tested; Windows/Linux pending.)
- [x] Document backups, privacy, supported EPUB limits, and recovery.
- [x] Release checklist covering data-loss cases and export validation.

**Done when:** build artifacts install/open/export reliably on all supported desktop systems and known limitations are documented.

### Later — EPUB compatibility improvements

- [ ] EPUB 2 + NCX, and complex navigation structures.
- [ ] Advanced tables, ruby, semantic footnotes, SVG/MathML, media overlays as deliberate supported cases.
- [ ] Multi-block translation restructuring (join/split) with stable export mappings.
- [ ] Glossary/terminology, find/replace, side-by-side block scroll synchronization.
- [ ] Project import/export/backup files and manual revision history.

### Later — AI translation (explicitly NOT Phase 1)

- [ ] Provider-independent `TranslationProvider` interface with cancellable jobs.
- [ ] Prompt/context builder with glossary and per-block identity.
- [ ] API key security and opt-in network/privacy disclosures.
- [ ] Proposals stored as separate drafts, not automatic overwrites of reviewed manual translations.
- [ ] Batch progress, retry, costs, rate limits and user review.

## 10. Quality gates / test matrix

| Test case | Expected result |
| --- | --- |
| EPUB 3 book with non-alphabetical filenames | Correct OPF spine order |
| Nested OPF/XHTML/assets paths | References resolve correctly |
| Cover + images + CSS | Visible in source, retained in export |
| Bold/italic + internal link + footnote | Original semantics/targets survive export |
| Very long chapter | Smooth navigation, bounded memory |
| Burmese/Myanmar combining characters | No text corruption; UTF-8 intact |
| App closed during autosave | Last committed translation loads; unsaved state not falsely marked saved |
| Chapter changed quickly after typing | Buffered edits are flushed |
| ZIP traversal / huge decompressed payload | Safe rejection with meaningful error |
| Fixed-layout, DRM or unsupported block | Explicit warning; no silent corruption |
| Export with empty translations | Clear draft warning; never mislabel as complete |
| Export onto original input path | Rejected |
| Completed translated EPUB | ZIP valid, navigation and content readable, EPUBCheck passes for supported fixtures |
| Windows/macOS/Linux reader variations | Fonts, links and layout remain usable |

### Suggested command checks

```bash
bun install
bun tauri dev
bun run build
cd src-tauri && cargo fmt --check && cargo test
# Add these once configured in package.json/CI:
# bun run lint
# bun run test
# cd src-tauri && cargo clippy -- -D warnings
# java -jar <path-to-epubcheck.jar> <exported.epub>
```

## 11. Known technical risks / mitigations

| Risk | Mitigation |
| --- | --- |
| WYSIWYG serialization drops EPUB-specific inline markup | Restricted editor schema, original DOM preservation, explicit compatibility warnings and round-trip fixtures |
| Nav entries don't map 1:1 to XHTML files | Maintain spine item/resource/anchor models separately |
| WebView differences across OSes | Cross-OS EPUB.js spike and automated smoke fixtures |
| Tauri privilege exposure to imported HTML | Sandboxed/sanitized preview, restrictive CSP/capabilities, no network access by default |
| Crashes or delayed autosave | Transactional revisions, save state, flush on navigation, periodic persistence |
| Corrupt or unsafe EPUB ZIP | Input size/path/resource limits, XML validation |
| Valid-looking ZIP fails EPUB standard | OCF checks plus independent EPUBCheck fixture validation |
| Myanmar font shaping/line breaking differs by OS | Font fallback strategy, cross-platform visual tests, no string indexing by code unit |
| Huge books cause memory spikes | Lazy chapter load, storage-backed resource map, bounded parsing and previews |

## 12. Next action — first coding slice

**Start with Milestone 0** in the already-created React/TypeScript/Bun scaffold. Do **not** recreate the scaffold.

1. Confirm `package.json`, `src-tauri/tauri.conf.json`, and `src-tauri/Cargo.toml` and run `bun install` / `bun tauri dev`.
2. Create minimal Project Library and Translation Workspace shell (mock data permitted for UI only, clearly marked).
3. Add a typed Tauri command (e.g., `health_check`) and Rust service wiring.
4. Add local SQLite initialization and a migration test.
5. Implement a small EPUB **import/export proof of concept** on a synthetic EPUB 3 fixture before committing to a rich text schema. Verify lossless preservation of links, inline formatting and images.

**Do not start AI integration until the user explicitly changes the phase.**

## Technical references

- Tauri 2 create project & Bun commands: https://v2.tauri.app/start/create-project/
- Tauri file dialog: https://v2.tauri.app/plugin/dialog/
- Tauri permissions: https://v2.tauri.app/learn/security/using-plugin-permissions/
- EPUB overview / package, spine and navigation: https://www.w3.org/TR/epub-overview-34/
- EPUB specification: https://www.w3.org/publishing/epub/
- EPUBCheck validator: https://github.com/w3c/epubcheck

### Workspace revision — 2026-10-13

Milestone 2 follow-up replaces plain text fields with restricted Tiptap passage editors presented as a document, adds Reader/HTML source views and image insertion with EPUB manifest packaging. See [Milestone 2](phase-2-translation-workspace.md) for verified behavior and limitations. Native Myanmar IME/copy-paste and Windows/Linux remain unchecked; no release milestones were advanced.

### Chapter authoring correction — 2026-10-13

The user's clarified workflow supersedes the block-aligned editor/merge-split
restriction in this plan: each chapter has one independent, initially empty
rich text document. Paragraph selection is not required. Chapter IDs own saved
drafts, review and export; immutable source blocks remain for import and legacy
recovery. Export replaces the corresponding XHTML body with the authored
document. See Milestone 2 for preservation limits and verification.

Translation HTML authoring is now explicitly requested: the target pane has
an editable HTML body view sharing chapter autosave with Rich text, with
validation and a loss-aware check before converting back. This supersedes
the earlier HTML-authoring exclusion for the supported chapter schema; the
source HTML view stays read-only.

### Writer shortcuts — 2026-10-13

The editor has a `/` block menu for Text, H1–H3, Table and Image, with keyboard
filtering/selection and pointer selection at the caret. Basic authored tables
are now an explicitly requested feature, using the existing chapter save and
EPUB export services. See Milestone 2 for validation and remaining native checks.

### Per-book settings follow-up — 2026-10-13

Explicitly requested book metadata and AI instruction settings are implemented
as an extension of the export/workspace slice and the authorized AI assist
work. See [AI follow-ups](later-ai-translation.md) for fields, prompt behavior,
preservation tests and remaining native/cross-platform checks. Release and
future AI milestone checkboxes are unchanged.

### Custom cover follow-up — 2026-10-13

The explicitly requested custom cover feature is implemented in Book Settings
and EPUB export using local SQLite storage and an export-only asset override.
See Milestone 4 for behavior, regression tests and native verification limits.
Release checkboxes remain unchanged.

### Library views follow-up — 2026-10-13

The Home library supports the existing table and a responsive cover grid through
accessible view buttons beside Search. The view preference is saved locally;
search, open, progress, Book Settings and removal share the same project data
and actions in both layouts. This is a Milestone 5 UI polish follow-up.
`bun run check` passed (lint, TypeScript and production build); Vite reports a
large bundle warning. No running development build was available for rendered
light/dark, minimum-window, keyboard or native interaction verification.
Next unfinished task: verify both layouts in the desktop app at 1280×800 and
960×640 in both themes; Windows/Linux release checks remain open.
