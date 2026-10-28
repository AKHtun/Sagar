# WEcoSagar — Sovereign EPUB & PDF Translation Studio

> ### 🌸 Attribution & Acknowledgements
> **WEcoSagar** is an extended sovereign edition built upon **[Sagar](https://github.com/saturngod/Sagar)**, the open-source EPUB Translation Studio originally created by **[saturngod](https://github.com/saturngod)** (Saturngod / Htain Lin Shwe).
> 
> We extend deep gratitude and honor to **saturngod** for envisioning and engineering the original offline-first, dual-pane EPUB translation studio for Myanmar translators. The original vision of privacy-first, zero-telemetry local translation remains the foundational bedrock of this tool.
>
> **Enhancements in this WEcoS edition:**
> - **Tri-Engine Sovereign Integration**: Intel NPU dense embedding retrieval (Engine 1), Intel Arc Dawei Guard security & orthography gate (Engine 2), and local/gated LLM cascade.
> - **TagCodec**: Reversible inline XHTML tag dehydration/rehydration protecting complex formatting attributes against LLM drift.
> - **Direct PDF Ingestion**: Native PyMuPDF converter generating valid EPUB3 archives from technical PDF manuals and books.
> - **Myanmar Canonical Orthography Normalizer**: Eliminates dotted-circle rendering artifacts and diacritic inversion.
> - **Font Auto-Embedding**: Canonical Pyidaungsu and epigraphical font packaging for seamless multi-platform e-reader display.
> - **Standalone CLI Utilities**: Headless import/export and batch processing binaries.

An offline-first desktop app for manually translating DRM-free EPUB books,
side by side, without losing the original formatting. Read the original
chapter on the left, write the translation on the right, review chapters, and export a separate translated EPUB.

![](./icon.png)

Built with Tauri 2, React, TypeScript and Rust; SQLite stores project data
locally. **No** **account,** **no** **network:** **book** **content** **never** **leaves** **the** **machine.**

## Development (Bun + Rust)

```bash
bun install
bun tauri dev        # run the desktop app in development mode
bun run check        # Biome lint + TypeScript + frontend build
cd src-tauri
cargo fmt --check && cargo test
cargo clippy --all-targets -- -D warnings
```

Requires the **Tauri** **2** **prerequisites** for your OS (see
[https://v2.tauri.app/start/prerequisites/](https://v2.tauri.app/start/prerequisites/) ).

## Release (version tags)

The version lives in 4 files that must always agree: `package.json`,
`src-tauri/tauri.conf.json`, `src-tauri/Cargo.toml`, and
`src-tauri/Cargo.lock` (the `sagar` entry). Never edit them by hand â€”
one command bumps all four, commits, and tags:

```bash
bun run release:tag -- 0.1.4 -m "version 0.1.4"  # bumps all 4 files, commits, tags v0.1.4
git push origin <branch> v0.1.4                   # pre-push re-validates before it leaves
```

Pushing a `v*` tag triggers the release workflow, which builds for
macOS/Windows/Linux. Helpers:

```bash
bun run version:check          # verify the 4 files agree
bun run version:set -- 0.1.4   # bump files without committing/tagging
bun run hooks:install          # reinstall the pre-push guard (auto-runs on bun install)
```

There is no git hook for tag creation, so the `pre-push` hook validates
the tagged commit itself and blocks mismatched tags before they reach CI.

## How it works

1. **Import** â€” open a DRM-free, reflowable EPUB 3 file. The app validates the
archive safely, reads metadata/manifest/spine/navigation, copies an
immutable snapshot into app-owned storage, and detects translatable blocks
(headings, paragraphs, list items, captions).
1. **Translate** â€” chapters are listed in reading order. Each chapter shows
the original (read-only) next to an editable translation. Chapter documents autosave
with revision protection; you can mark chapters **reviewed** or choose
**keep** **original**. Text structures the editor cannot translate safely
(tables, preformatted text, MathMLâ€¦) are preserved unchanged and listed as
import notes.
1. **Export** â€” a review dialog shows exactly what is translated. A
completed export requires every chapter reviewed or intentionally
kept original; a draft export keeps untranslated chapters in the original
language after explicit acknowledgement. The exported book is a new file:
translated fragments are spliced into a copy of the original XHTML, so
images, links, footnotes and structure are preserved, and the output is
re-validated before the app reports success.

## Where your data lives

- Database and immutable book snapshots: `~/Library/Application Support/app.myanmars.sagar`
(macOS; equivalent app-data directories on Windows/Linux).
- Delete that folder to reset the library (this also deletes translations).
- The original EPUB file you open is never modified.
- Presentation preferences (theme, text size, pane widths) live in the
WebView's local storage, separate from translations.

## Translate using AI

The AI dialog supports two workflows:

- **Whole**** ****chapter** keeps the original Generate/Retry workflow for short chapters.
- **Selected**** ****paragraphs** lets you choose passages, whole tables, and standalone
images. Text is translated in small sequential requests; long paragraphs and
tables are reconstructed locally. Preview completed results and choose
**Append**** ****selected**** ****results** to add them to the end of your current draft.

Images are included in previews, the draft, and EPUB export without sending
their bytes to the translation provider. Figures keep their captions; captions
are translated as text. Unsupported or missing resources are shown explicitly.

Selected-mode proposals and completed parts are saved locally until accepted.
Translating the same source paragraph again replaces its previous saved proposal
only after the new result succeeds and validates, even with a different model.
Failed or cancelled attempts keep the previous proposal; there is no version picker.
After a successful append, their saved output and older ready versions of the
same source items are removed; the chapter draft and small insertion receipts
remain. Unadded proposals stay available across batches and restarts. If you
later remove accepted text from the draft, select its source to translate again.
A failed batch
pauses the queue: Retry the failed batch, Retry with smaller batches, or Skip and
continue. Resume uses the saved run's original model, language direction and
instructions. Changing these settings starts a new run. Closing cancels active
work, and restarting never resumes a provider request automatically.

Appending preserves existing writing, saves a chapter draft, and does not mark
it reviewed. Insertion history covers selected mode only; it does not infer
what you subsequently edit or delete. Unselected source content is not copied
into the authored chapter, so review completeness and links before exporting.

Use your own subscription instead of using Token.
Sagar is supporting 
- Codex
- Claude Code
- Open Code
![](./assets/screen1.png)

![](./assets/screen2.png)
![](./assets/screen3.png)
It allow to copy and paste in your MS Word , too.
![](./assets/screen4.png)