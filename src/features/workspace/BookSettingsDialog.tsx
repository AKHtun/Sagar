import { useEffect, useRef, useState } from "react";
import { Icon } from "../../components/Icon";
import { StatusNotice } from "../../components/StatusNotice";
import { getBookSettings, getProject, saveBookSettings } from "../../lib/tauri/commands";
import type { BookSettings, ProjectSummary } from "../../types/project";

export function BookSettingsDialog({
  project,
  onClose,
  onSaved,
}: {
  project: ProjectSummary;
  onClose: () => void;
  onSaved: (project: ProjectSummary) => void;
}) {
  const [settings, setSettings] = useState<BookSettings | null>(null);
  const [tab, setTab] = useState<"metadata" | "instructions">("metadata");
  const [error, setError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const [retry, setRetry] = useState(0);
  const dialog = useRef<HTMLDivElement>(null);
  useEffect(() => {
    const previous = document.activeElement as HTMLElement | null;
    dialog.current?.focus();
    return () => previous?.focus();
  }, []);
  // biome-ignore lint/correctness/useExhaustiveDependencies: retry explicitly reloads settings after an I/O failure.
  useEffect(() => {
    let disposed = false;
    setError(null);
    getBookSettings(project.id)
      .then((value) => {
        if (!disposed) setSettings(value);
      })
      .catch((failure: unknown) => {
        if (!disposed)
          setError(
            failure instanceof Error
              ? failure.message
              : "Book settings could not be loaded.",
          );
      });
    return () => {
      disposed = true;
    };
  }, [project.id, retry]);
  useEffect(() => {
    const onKey = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        if (!saving) onClose();
      }
      if (event.key !== "Tab") return;
      const controls = Array.from(
        dialog.current?.querySelectorAll<HTMLElement>(
          "button:not(:disabled), input:not(:disabled), textarea:not(:disabled), select:not(:disabled)",
        ) ?? [],
      ).filter((control) => control.getClientRects().length > 0);
      const first = controls[0];
      const last = controls[controls.length - 1];
      if (
        event.shiftKey &&
        (document.activeElement === first || document.activeElement === dialog.current)
      ) {
        event.preventDefault();
        last?.focus();
      } else if (!event.shiftKey && document.activeElement === last) {
        event.preventDefault();
        first?.focus();
      }
    };
    window.addEventListener("keydown", onKey, true);
    return () => window.removeEventListener("keydown", onKey, true);
  }, [onClose, saving]);
  async function save() {
    if (!settings || saving) return;
    setSaving(true);
    setError(null);
    try {
      await saveBookSettings(project.id, settings);
      onSaved(await getProject(project.id));
      onClose();
    } catch (failure) {
      setError(
        failure instanceof Error ? failure.message : "Book settings could not be saved.",
      );
    } finally {
      setSaving(false);
    }
  }
  function update(key: keyof BookSettings, value: string) {
    setSettings((current) => (current ? { ...current, [key]: value } : current));
  }
  return (
    <div className="dialog-backdrop">
      <div
        className="dialog"
        role="dialog"
        aria-modal="true"
        aria-labelledby="book-settings-title"
        tabIndex={-1}
        ref={dialog}
      >
        <div className="panel__header">
          <h2 className="panel__heading" id="book-settings-title">
            Book settings · {project.title}
          </h2>
          <button
            type="button"
            className="icon-button"
            aria-label="Close book settings"
            title="Close"
            disabled={saving}
            onClick={onClose}
          >
            <Icon name="close" />
          </button>
        </div>
        <fieldset
          className="book-settings-sections button-row"
          aria-label="Book settings sections"
        >
          <button
            type="button"
            className="btn"
            aria-pressed={tab === "metadata"}
            onClick={() => setTab("metadata")}
          >
            Metadata
          </button>
          <button
            type="button"
            className="btn"
            aria-pressed={tab === "instructions"}
            onClick={() => setTab("instructions")}
          >
            AI instructions
          </button>
        </fieldset>
        {!settings && !error ? (
          <StatusNotice kind="loading">Loading book settings…</StatusNotice>
        ) : null}
        {error ? (
          <StatusNotice
            kind="error"
            onRetry={!settings ? () => setRetry((value) => value + 1) : undefined}
          >
            {error}
          </StatusNotice>
        ) : null}
        {settings ? (
          <form
            onSubmit={(event) => {
              event.preventDefault();
              void save();
            }}
          >
            {tab === "metadata" ? (
              <>
                <p className="dialog__muted">
                  These fields describe the exported EPUB. The original book is preserved.
                  Leave author, publisher or description blank to keep its source value.
                </p>
                <div className="form-grid">
                  {(
                    [
                      ["title", "Export title"],
                      ["author", "Primary author"],
                      ["publisher", "Publisher"],
                      ["sourceLanguage", "Original language"],
                      ["targetLanguage", "Translation language"],
                    ] as const
                  ).map(([key, label]) => (
                    <label className="form-field" key={key}>
                      <span>{label}</span>
                      <input
                        value={settings[key]}
                        required={key === "title" || key === "targetLanguage"}
                        maxLength={key.includes("Language") ? 64 : 1024}
                        disabled={saving}
                        onChange={(event) => update(key, event.target.value)}
                      />
                    </label>
                  ))}
                </div>
                <label className="form-field">
                  <span>Description</span>
                  <textarea
                    rows={4}
                    value={settings.description}
                    maxLength={20000}
                    disabled={saving}
                    onChange={(event) => update("description", event.target.value)}
                  />
                </label>
              </>
            ) : (
              <>
                <p className="dialog__muted">
                  Applied to every AI translation in this book, together with the current
                  run's instructions. Use this for consistent names, terminology and
                  writing style.
                </p>
                <label className="form-field">
                  <span>Book translation instructions</span>
                  <textarea
                    rows={10}
                    value={settings.instructions}
                    maxLength={20000}
                    disabled={saving}
                    placeholder="Use these character names consistently…"
                    onChange={(event) => update("instructions", event.target.value)}
                  />
                </label>
                <p className="dialog__muted">
                  Saved locally. Sent to the selected agent only when you generate a
                  translation; not included in the exported EPUB.
                </p>
              </>
            )}
            <div className="button-row dialog__actions">
              <button type="button" className="btn" disabled={saving} onClick={onClose}>
                Cancel
              </button>
              <button type="submit" className="btn btn--primary" disabled={saving}>
                {saving ? "Saving…" : "Save book settings"}
              </button>
            </div>
          </form>
        ) : null}
      </div>
    </div>
  );
}
