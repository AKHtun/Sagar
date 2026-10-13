# Milestone 2 — Side-by-side reading/translation UI

**Status:** Revised 2026-10-13 with a balanced original/translation workspace and restricted Tiptap editor. Browser interaction and Rust round-trip checks pass; native IME and Windows/Linux verification remain open.  
**Source:** [Master plan](plan.md)  
**Repository rules:** [AGENTS.md](../AGENTS.md)

This file covers an implementation milestone within the manual-translation MVP (product Phase 1). File numbering follows the original milestone numbering; it does not change the product phases.

## Prerequisites

Build on [Milestone 1 — EPUB import and chapter listing](phase-1-epub-import.md).

## Tasks and acceptance criteria

- [x] Design resizable 3-panel workspace (chapters, source, target).
- [x] Render source chapter safely with images and applicable book CSS.
- [x] Display one independent rich text document per stable chapter ID.
- [ ] Support basic editing: Unicode text, paragraphs/headings, emphasis and inline links where safely mapped.
- [x] Add keyboard navigation, selection/focus, and clear save/error state.
- [ ] Test Myanmar Unicode line wrapping, copy/paste and multi-line content.
- [x] Validate EPUB.js/preview adapter compatibility with Tauri WebViews.

**Status notes**

- The reader decision was resolved in favor of a **custom sanitized
  renderer** (`epubres://` snapshot protocol + Rust-side fragment sanitizer)
  instead of EPUB.js: EPUB.js executes book scripts/styles inside the
  privileged WebView, while the custom renderer keeps the preview inert
  (no script execution, no network fetches, event handlers stripped).
- Book CSS is *not* applied in the preview (app typography only); CSS
  containment is deferred with the rich-text editor work.
- The target is **one empty rich text editor per chapter**, independent of
  source paragraphs. It works for image-only chapters and allows arbitrary
  paragraphs, headings, lists, emphasis and inserted images. No passage
  selection, per-block controls or preset source-derived content is required.
- SQLite migration 3 adds revision-checked `chapter_documents`. Existing
  block records remain intact; saved legacy translations are assembled into
  a chapter draft when first loaded. New chapters begin empty.
- Autosave, restart recovery, review and intentional skips now apply to the
  whole chapter. Keeping the original retains an existing draft for later.
- Export uses the authored document as the full XHTML body, preserving the
  source head/body attributes, reading order, archive resources and original
  source snapshot. Removed source anchor IDs are retained as empty spans at
  the chapter start; their old paragraph locations cannot be inferred from
  independently authored content. Unedited legacy drafts continue through
  their existing export path until saved as chapter documents.
- The original has **Reader / HTML** modes. Reader uses the full sanitized
  chapter body (including standalone images); HTML shows immutable XHTML as
  escaped, read-only text. Neither mode executes imported scripts or fetches
  external resources. Original book CSS remains excluded.
- **Insert image** supports local PNG/JPEG/GIF files up to 8 MB and images from
  the current chapter. Images persist with translation fragments; export
  extracts them into deduplicated resources and declares them in the OPF
  manifest. Existing source assets remain unchanged.
- The chapter sidebar collapses by default; a chapter selector and previous/
  next controls keep navigation available. Both panes initially have equal
  width and independent scrolling. Reflowable EPUBs use chapter navigation,
  not fixed physical pages.
- Chapter documents are namespace-aware XML validated in Rust, including
  safe formatting, list structure, images and nesting. Autosave serializes
  writes and flushes before switching chapters, returning to the library or
  exporting. Completion/export review counts chapters rather than passages.
- Regression tests cover an initially empty image-only chapter, arbitrary
  heading/paragraph/list authoring, image packaging, save/reload, stale
  revisions, reviewed/keep-original statuses, preserved legacy drafts,
  immutable source files and exported EPUB reopening. A mocked Tauri browser
  pass verifies exactly one empty editor, an enabled toolbar before selection,
  typing during autosave, image insertion and returning to a saved chapter.
- Myanmar storage/round-trip is covered by UTF-8 fixtures; a visual
  WebView pass (wrapping, combining marks, IME composition) is still open.

**Done when:** user can read one imported chapter and edit corresponding target blocks without altering source or breaking referenced images/links. — Met for reading, editing and asset preservation on macOS; images render from the snapshot protocol and untranslated blocks keep their markup byte-for-byte (integration-tested).

## Desktop UI/UX requirements

Implement the three-pane desktop workspace with resizable, keyboard-operable dividers and a collapsible chapter sidebar. Keep source read-only, target blocks editable and formatting limited to the validated schema. Apply the specified reading typography, active-block feedback, focus behavior and Myanmar input-method checks. Independent scrolling is the initial behavior.

Follow [Desktop UI/UX design](ui-ux-design.md) for layout dimensions, visual tokens, screen states, interaction rules and design acceptance checks. Requirements assigned to later milestones remain deferred.

## Supporting design

- [Section 2 — UX / screen architecture](plan.md)
- [Section 5 — Translation data model (proposed SQLite v1)](plan.md)
- [Section 11 — Known technical risks / mitigations](plan.md)

Follow the master plan’s quality gates and repository definition of done. Check tasks only after implementation and acceptance criteria are verified; keep completion status aligned with the master plan.

### Editable translation HTML view — 2026-10-13

The translation pane now offers **Rich text / HTML**. HTML edits update the
same chapter draft and autosave through Rust document validation. Original
HTML remains read-only. User-requested translation HTML authoring supersedes
the earlier source-code-editing exclusion for this supported chapter schema.
Malformed/unsupported HTML stays in the code editor with a save error and
cannot silently replace the saved document. Switching back validates XHTML
and checks its round-trip through the rich text schema; non-representable
structure remains in HTML view without being stripped. Ordered list start
numbers survive conversion. Empty code can return to the blank editor.

Browser checks cover code edits, headings/Unicode/list numbering in rich text,
invalid-code retention and switching back from an empty HTML document.
Native Myanmar IME and Windows/Linux checks remain open.

### Original XHTML copy/paste regression — 2026-10-13

Complete XHTML documents (including XML declaration and head) can be pasted
into translation HTML. Rust selects and validates the body; the immutable
source head continues to supply the chapter stylesheet during export. Source
class/ID/title/language/direction attributes and allowlisted typography styles
are preserved in code, rich text and export. Error messages now distinguish
malformed XML (with a position) and unsupported inline CSS. A synthetic TOC
fixture covers full-document input, class/style preservation and chapter links;
the exact user-supplied TOC was also tested locally without storing it in the
repository. Other unsupported structures retain the existing explicit errors.

### Direct paragraph/heading controls — 2026-10-13

The formatting dropdown is replaced with visible **Paragraph / H1 / H2 / H3**
buttons. They set the current block format before typing or at the caret without
requiring text selection. Clicking retains editor selection; active formatting
is highlighted with `aria-pressed`. Browser checks cover H1 before typing,
H2/H3 conversion at the cursor, Paragraph conversion, retained text and active
states. Build/lint pass; native cross-platform IME/accessibility remain open.

### Slash commands and authored tables — 2026-10-13

Typing `/` at the start of a block or after whitespace opens a menu at the
caret: Text, H1, H2, H3, Table and Image. Typing filters commands; Up/Down choose,
Enter/Tab insert, and Escape dismisses without deleting the literal slash.
Clicking a command retains editor focus and replaces only the slash query.
URLs and non-command slashes do not trigger the menu. HTML mode and IME
composition suppress it. The popup follows the caret and stays in the viewport.

Table inserts a 3×3 grid with a header row. A contextual icon toolbar adds or
removes rows/columns or deletes the grid. Tiptap table support is pinned to the
existing 3.31.4 editor version. Authored table XHTML, rows, header/data cells
and bounded cell spans are validated in Rust, retained through code/rich-text
switching, saved, exported and re-opened. Empty tables count as authored
content, preventing their loss on restart. This explicitly requested table
authoring supersedes the earlier restricted editor schema; arbitrary imported
complex-table preservation still requires the existing loss-aware checks.

Rust fixtures cover table structure, rejected nesting/spans and export/reopen
with source immutability. Browser checks cover slash filtering, keyboard
navigation, all requested insertions, Escape, table controls, HTML switching,
chapter persistence and the minimum-window layout. Native cross-platform IME,
screen-reader behavior and full EPUBCheck remain open.
