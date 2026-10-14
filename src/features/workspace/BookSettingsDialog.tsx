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
  const [readingCover, setReadingCover] = useState(false);
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
        if (!saving && !readingCover) onClose();
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
  }, [onClose, saving, readingCover]);
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
  async function chooseCover(file: File | undefined) {
    if (!file) return;
    if (
      !["image/png", "image/jpeg", "image/gif"].includes(file.type) ||
      file.size > 8 * 1024 * 1024
    ) {
      setError("Choose a PNG, JPEG or GIF image under 8 MB.");
      return;
    }
    setReadingCover(true);
    setError(null);
    try {
      const data = await new Promise<string>((resolve, reject) => {
        const reader = new FileReader();
        reader.onload = () =>
          typeof reader.result === "string"
            ? resolve(reader.result)
            : reject(new Error("The cover could not be read."));
        reader.onerror = () =>
          reject(new Error("The cover could not be read. Choose the image again."));
        reader.readAsDataURL(file);
      });
      // Verify browser decoding before retaining the selection; Rust also
      // checks format signatures and size at the save boundary.
      await new Promise<void>((resolve, reject) => {
        const image = new Image();
        image.onload = () => resolve();
        image.onerror = () =>
          reject(new Error("This image could not be decoded. Choose another cover."));
        image.src = data;
      });
      setSettings((current) => (current ? { ...current, customCover: data } : current));
    } catch (failure) {
      setError(
        failure instanceof Error ? failure.message : "The cover could not be read.",
      );
    } finally {
      setReadingCover(false);
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
            disabled={saving || readingCover}
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
            <Icon name="book" />
            Metadata
          </button>
          <button
            type="button"
            className="btn"
            aria-pressed={tab === "instructions"}
            onClick={() => setTab("instructions")}
          >
            <Icon name="sparkles" />
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
                <div
                  className={`book-cover-settings${settings.customCover ? " book-cover-settings--custom" : ""}`}
                >
                  <span className="book-cover-settings__label">Export cover</span>
                  {settings.customCover ? (
                    <img
                      className="book-cover-settings__preview"
                      src={settings.customCover}
                      alt="Custom export cover preview"
                    />
                  ) : (
                    <p className="dialog__muted">
                      The original cover will be used, if the book has one.
                    </p>
                  )}
                  <label className="form-field">
                    <span>Choose custom cover</span>
                    <input
                      type="file"
                      accept="image/png,image/jpeg,image/gif"
                      disabled={saving || readingCover}
                      onChange={(event) => {
                        void chooseCover(event.target.files?.[0]);
                        event.target.value = "";
                      }}
                    />
                  </label>
                  {readingCover ? (
                    <StatusNotice kind="loading">Reading cover…</StatusNotice>
                  ) : null}
                  {settings.customCover ? (
                    <button
                      type="button"
                      className="btn"
                      disabled={saving || readingCover}
                      onClick={() => setSettings({ ...settings, customCover: null })}
                    >
                      <Icon name="keep" />
                      Use original cover
                    </button>
                  ) : null}
                  <p className="dialog__muted">
                    PNG, JPEG or GIF, up to 8 MB. Saved with Book Settings and included in
                    export. Cancel discards this selection.
                  </p>
                </div>
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
                        disabled={saving || readingCover}
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
                    disabled={saving || readingCover}
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
                    disabled={saving || readingCover}
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
              <button
                type="button"
                className="btn"
                disabled={saving || readingCover}
                onClick={onClose}
              >
                <Icon name="close" />
                Cancel
              </button>
              <button
                type="submit"
                className="btn btn--primary"
                disabled={saving || readingCover}
              >
                <Icon name="save" />
                {saving ? "Saving…" : "Save"}
              </button>
            </div>
          </form>
        ) : null}
      </div>
    </div>
  );
}
