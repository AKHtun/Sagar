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

### Translation direction correction — 2026-10-13

The workspace footer now states the translation direction in readable form
(e.g. “English (en) → Myanmar (my)”) instead of only the target tag, and it
opens a Translation languages dialog. Source language comes from EPUB metadata
at import and was previously uneditable; both tags are now correctable per
project with the existing tag validation (blank source clears back to
unknown). Changing languages only relabels the project and future exports —
book text is never rewritten. Covered by a service-level round-trip test.

### Empty chapter caret correction — 2026-10-13

The empty chapter hint is positioned over the paragraph instead of floating
in the text flow. Clicking it places the caret at the paragraph start; the
hint remains while focused and disappears when content is typed. The hint is
CSS-only and never enters the saved document. Frontend lint/build passed;
native caret/focus and Myanmar IME verification remain open.

### AI result display correction — 2026-10-13

The mounted Rich editor now synchronizes external chapter-draft updates such
as an AI proposal immediately, through the same Rust validation and loss-aware
round-trip check used when switching from HTML. Unsupported content opens the
HTML view with an explanation instead of silently discarding formatting.
Equivalent content from normal editor typing is not reloaded, preserving the
caret and undo history; stale async validation cannot overwrite newer text.
Frontend lint/build passed. Native AI completion/insert verification remains
open, along with the existing cross-platform checks.

### Workspace toolbar simplification — 2026-10-13

The book title, chapter picker/sidebar toggle, previous/next navigation,
save state and Export now share one workspace header. Original/Translation
view controls remain in the pane headers. The duplicate source chapter-title
row is removed, and formatting is hidden until the user opens Format (also
hidden in HTML view). Keyboard formatting shortcuts and the slash menu remain
available. On narrow windows the chapter picker remains available while
previous/next icons are hidden to avoid crowding. Frontend lint/build passed;
native visual and keyboard verification remains open.

### Pane controls as icons — 2026-10-13

Reader/Rich text and HTML, AI proposal and Format now use consistent 34px
square icon controls, matching the review actions. Tooltips and accessible
labels identify each action; selected views and expanded formatting retain
highlighted states. Both pane headers share this sizing. Frontend lint/build
passed; native visual verification remains open.

The Paragraph formatting button now uses a compact pilcrow (¶) icon with its
existing tooltip, accessible label, active state and keyboard shortcut.

### Appearance preferences in Settings — 2026-10-13

Library Settings now includes an Appearance tab for Follow system / Light /
Dark and reading text size with an English/Myanmar preview. The unexplained
appearance and +/− buttons are removed from the library header. Preferences
still use the existing local keys, apply immediately, and keep Cmd/Ctrl +/−
shortcuts. Provider and Translation Model tabs remain available. Settings tabs
support arrow-key navigation with focus tracking. Frontend lint/build passed;
native theme/size and cross-platform verification remain open.

Library book rows now also expose the same Book Settings dialog through a
settings icon beside Trash, without entering the workspace. Saving updates
the library's project data in place and retains search/focus state. Frontend
lint/build passed; native library-dialog verification remains open.

### App display name — 2026-10-13

Tauri productName, the native window title and HTML title now use “Sagar”.
The existing app identifier and storage identity remain unchanged. A rebuilt
and relaunched app is required for the macOS application menu label to update.

### Searchable chapter picker — 2026-10-13

The native chapter select is replaced by a bounded, searchable combobox.
Search matches reading-order numbers and titles; the current chapter is
marked. Arrow keys navigate results, Enter selects, Escape closes and restores
focus, and Tab/outside click dismiss the popup. The popup scrolls to the active
result and limits list height for large books. Selection continues through the
existing autosave-flush navigation path. Frontend lint/build passed; native
keyboard, pointer and light/dark visual checks remain open.

### Sidebar search and progress filters — 2026-10-13

The sidebar searches chapter titles and reading-order numbers, with All,
Incomplete, Not started, In progress and Completed filters. Filtered chapters
keep reading order; the active chapter uses its live draft progress. Search
and filter controls remain above the scrollable list, with a result count and
Clear filters empty state. Arrow/Home/End keys move focus within visible rows;
Enter/Space selects through the existing autosave-flush path. A visible row
remains tabbable even when the active chapter is filtered out. Frontend
lint/build passed; native sidebar interaction/visual checks remain open.

Workspace Book Settings is now after Export at the right end of the header.
Export uses the shared square download icon button with its accessible label
and tooltip, following the user's explicit preference for this toolbar.

### HTML syntax highlighting — 2026-10-14

Original and translation HTML views share an offline HTML/XML highlighter.
Only escaped highlighter output is rendered; imported markup remains inert.
Translation editing retains its native textarea, with an inert, scroll-linked
syntax layer, preserving the existing draft/autosave path and native selection,
undo and IME handling. Colors use shared light/dark tokens; forced-color mode
uses ordinary visible input text. Original Copy HTML still copies raw source.

Two Bun regression tests pass for token coloring, exact Unicode/whitespace
preservation, incomplete code and escaping executable markup. Frontend lint and
build pass (existing bundle-size warning). No current Sagar window was available
for visual checks. Normal/minimum-window layout, scrolling/caret alignment,
light/dark appearance, native undo/IME and Windows/Linux verification remain
open; milestone checkboxes are unchanged.
