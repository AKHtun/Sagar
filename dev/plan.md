# Plan.md — EPUB Translation Studio

**Status:** Planning / no application features are claimed implemented yet  
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

Checkboxes are intentionally **unchecked** because no feature implementation has been verified.

### Phase files

The files below break down the existing roadmap without changing its scope or completion status. Numbered phase files follow Milestones 0–5 within product Phase 1; compatibility and AI remain deferred tracks.

- [Milestone 0 — Bootstrap & architecture foundation](phase-0-bootstrap.md)
- [Milestone 1 — EPUB import and chapter listing](phase-1-epub-import.md)
- [Milestone 2 — Side-by-side reading/translation UI](phase-2-translation-workspace.md)
- [Milestone 3 — Local autosave, progress and resume](phase-3-autosave-and-resume.md)
- [Milestone 4 — EPUB export (vertical-slice first)](phase-4-epub-export.md)
- [Milestone 5 — Polish & desktop release](phase-5-desktop-release.md)
- [Later — EPUB compatibility improvements](later-epub-compatibility.md)
- [Later — AI translation (explicitly NOT Phase 1)](later-ai-translation.md)

### Milestone 0 — Bootstrap & architecture foundation

- [ ] Verify generated Tauri 2 + React + TS project runs with **Bun** on the current development OS.
- [ ] Remove demo UI; create minimal app shell and navigation.
- [ ] Add strict TS checks, format/lint conventions and relevant scripts.
- [ ] Add Rust `domain`, `services`, `epub`, `storage` structure as functionality is introduced.
- [ ] Implement typed Tauri command bridge and reusable error reporting.
- [ ] Add SQLite schema/migration initialization with an automated persistence test.
- [ ] Add native dialog plugin and narrowly scoped capabilities for import/export; avoid broad filesystem permissions.

**Done when:** `bun tauri dev` launches, the frontend invokes a typed Rust test command, and a local SQLite migration test passes.

### Milestone 1 — EPUB import and chapter listing

- [ ] Implement archive and untrusted-input safety limits.
- [ ] Parse container.xml + OPF manifest/spine/metadata for EPUB 3.
- [ ] Parse EPUB navigation and resolve ordered chapter titles/anchors.
- [ ] Copy immutable source snapshot, hash it, and create project rows.
- [ ] Build stable editable block map for supported XHTML elements.
- [ ] Surface unsupported EPUB feature warnings instead of failing silently.
- [ ] Show imported project in library and chapter sidebar.
- [ ] Add fixtures: simple multi-chapter, nested paths, images, links, missing OPF, malformed ZIP.

**Done when:** importing a fixture shows correct title, author, source language, cover if present, and spine-ordered chapters. Imported original is not modified.

### Milestone 2 — Side-by-side reading/translation UI

- [ ] Design resizable 3-panel workspace (chapters, source, target).
- [ ] Render source chapter safely with images and applicable book CSS.
- [ ] Display target editor bound to stable source-block IDs.
- [ ] Support basic editing: Unicode text, paragraphs/headings, emphasis and inline links where safely mapped.
- [ ] Add keyboard navigation, selection/focus, and clear save/error state.
- [ ] Test Myanmar Unicode line wrapping, copy/paste and multi-line content.
- [ ] Validate EPUB.js/preview adapter compatibility with Tauri WebViews.

**Done when:** user can read one imported chapter and edit corresponding target blocks without altering source or breaking referenced images/links.

### Milestone 3 — Local autosave, progress and resume

- [ ] Debounced autosave with revision conflict protection and retries.
- [ ] Persist `pending`, `draft`, `reviewed`, `skipped` states.
- [ ] Derive chapter status from required block review status.
- [ ] Restore project and last chapter after a restart.
- [ ] Flush saves before navigation and export.
- [ ] Add recoverability and save-failure tests (including process interruption where feasible).

**Done when:** edited translations survive app restart and chapter switching without data loss; failures are reported honestly.

### Milestone 4 — EPUB export (vertical-slice first)

- [ ] Implement a 1-chapter translated EPUB round trip on an intentionally simple fixture.
- [ ] Patch translated XHTML fragments into source resource tree without flattening assets.
- [ ] Repackage EPUB with correct OCF entry ordering/compression.
- [ ] Update title, language and navigation labels without breaking hrefs.
- [ ] Add native Save dialog, explicit draft mode and unfinished-content review.
- [ ] Validate output ZIP, XHTML parsing, manifest targets and links.
- [ ] Reimport the exported file and compare chapter/resource counts and spine order.
- [ ] Add regression fixtures for formatting, internal links, images and footnotes.

**Done when:** a fully manually translated fixture exports, passes the project's integrity checks, reopens in a reading app, and retains its structural assets. EPUBCheck should also pass on supported compliant fixtures in CI/dev test environments.

### Milestone 5 — Polish & desktop release

- [ ] UX review: dark mode, pane resizing, text zoom, shortcuts, error dialogs.
- [ ] Accessibility and Burmese text rendering cross-OS tests.
- [ ] Performance checks with large books (bounded RAM/use lazy loading).
- [ ] Package and smoke-test installers for Windows/macOS/Linux.
- [ ] Document backups, privacy, supported EPUB limits, and recovery.
- [ ] Release checklist covering data-loss cases and export validation.

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
