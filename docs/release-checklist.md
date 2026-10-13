# Release checklist — Sagar EPUB Translation Studio

Status legend: [x] verified, [ ] open. "Verified" means executed with
recorded evidence on the named OS; nothing is marked verified without it.

## Data-loss cases

- [x] Editing a block, switching chapters, reopening the app restores the
      last committed translation (autosave flush on navigation; revision
      checks; covered by storage/integration tests).
- [x] Save failure keeps the local draft and shows a persistent Retry state
      (unit-tested revision conflicts; UI shows per-block errors).
- [x] Interrupted import leaves no partial project (single transaction).
- [x] Export never writes to the source file or the stored snapshot
      (integration-tested rejection).
- [ ] App killed mid-autosave: recovery tested by process interruption
      (partially covered by commit-only recovery tests; a dedicated
      kill-during-save test is open).
- [ ] Database upgrade path from future schema changes re-tested with a
      real user database.

## Export validation

- [x] Translated fixture round trip: patch, repackage (mimetype first,
      stored), validate, re-import and compare spine/titles (integration
      tests).
- [x] Untranslated blocks and unsupported content preserved byte-for-byte.
- [x] Metadata (dc:title, dc:language) and nav labels updated without
      breaking hrefs.
- [ ] EPUBCheck run on the exported fixture set (needs Java; not installed
      in this environment). The app does **not** claim full EPUB
      conformance from its internal checks.
- [ ] Exported book opened in at least one third-party reader per OS.

## Platforms

- [x] macOS (Apple Silicon): unit/integration suites, dev boot, UI smoke.
- [ ] Windows: install, run, import/export, Myanmar shaping (untested).
- [ ] Linux: install, run, import/export, Myanmar shaping (untested).
- [x] macOS (Apple Silicon) bundle: `bun tauri build` produced
      `sagar.app` (6.4 MiB) and `sagar_0.1.0_aarch64.dmg` (3.1 MiB);
      launch smoke passed in dev; installer opened on a clean machine is
      still recommended before distribution.
- [ ] Windows/Linux installers (cannot be built or tested from macOS here).

## Accessibility & typography

- [x] Keyboard: chapter list, splitters, editor, dialogs operable without
      the mouse; focus visible; dialogs restore focus on Escape.
- [ ] Screen reader pass (VoiceOver at minimum) on real content.
- [x] Contrast tokens defined for light/dark; manual contrast spot-check
      against WCAG 4.5:1 pending final palette sign-off.
- [x] Myanmar text input, wrapping and storage verified via synthetic
      fixtures at the storage level (UTF-8 intact; combining marks are text).
- [ ] Myanmar rendering check with real Padauk/Myanmar Text fonts on each
      target OS (WebView shaping differs across platforms).

## Privacy

- [x] No network access in the request path: book content renders only from
      the app-owned snapshot protocol; external book URLs are never fetched;
      CSP restricts the WebView.
- [x] No telemetry, no logging of book text or filesystem paths in errors.
- [ ] CSP reviewed again after any new WebView feature (current policy in
      `src-tauri/tauri.conf.json`).

## Performance

- [x] Chapters load on demand; export streams per-resource from the
      snapshot.
- [ ] Large-book sweep (5k+ chapters, 50 MB EPUB) for import/export time
      and memory bounds.
