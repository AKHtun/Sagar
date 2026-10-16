import { useEffect, useRef, useState } from "react";
import { Icon } from "../../components/Icon";
import { StatusNotice } from "../../components/StatusNotice";
import { useTranslation } from "../../lib/i18n";
import { formatDirection, languageSuggestions } from "../../lib/languages";
import { getProject, updateProjectMetadata } from "../../lib/tauri/commands";
import { CommandError } from "../../lib/tauri/errors";
import type { ProjectSummary } from "../../types/project";

type LanguageDialogProps = {
  project: ProjectSummary;
  onClose: () => void;
  onSaved: (project: ProjectSummary) => void;
};

const TAG_PATTERN = /^[A-Za-z0-9-]+$/;

/**
 * Translation language dialog: shows the book's translation direction and
 * lets the user correct the source/target tags. Source comes from EPUB
 * metadata at import (often missing or wrong) and previously had no edit
 * path; target was only editable at export time.
 */
export function LanguageDialog({ project, onClose, onSaved }: LanguageDialogProps) {
  const { t } = useTranslation();
  const [source, setSource] = useState(project.sourceLanguage ?? "");
  const [target, setTarget] = useState(project.targetLanguage);
  const [fieldErrors, setFieldErrors] = useState<Record<string, string>>({});
  const [saveError, setSaveError] = useState<string | null>(null);
  const [saving, setSaving] = useState(false);
  const dialogRef = useRef<HTMLDivElement | null>(null);
  const previousFocus = useRef<HTMLElement | null>(null);
  const suggestions = useRef(languageSuggestions());

  useEffect(() => {
    previousFocus.current = document.activeElement as HTMLElement | null;
    dialogRef.current?.focus();
    return () => {
      previousFocus.current?.focus();
    };
  }, []);

  useEffect(() => {
    const onKeyDown = (event: KeyboardEvent) => {
      if (event.key === "Escape") {
        event.stopPropagation();
        if (!saving) onClose();
      }
      if (event.key === "Tab") {
        const controls = Array.from(
          dialogRef.current?.querySelectorAll<HTMLElement>(
            'button:not(:disabled), input:not(:disabled), [tabindex="0"]',
          ) ?? [],
        );
        const first = controls[0];
        const last = controls[controls.length - 1];
        if (
          event.shiftKey &&
          (document.activeElement === first ||
            document.activeElement === dialogRef.current)
        ) {
          event.preventDefault();
          last?.focus();
        } else if (!event.shiftKey && document.activeElement === last) {
          event.preventDefault();
          first?.focus();
        }
      }
    };
    window.addEventListener("keydown", onKeyDown, true);
    return () => window.removeEventListener("keydown", onKeyDown, true);
  }, [onClose, saving]);

  async function handleSave() {
    const errors: Record<string, string> = {};
    if (target.trim() === "" || !TAG_PATTERN.test(target.trim())) {
      errors.target = "Enter a language tag such as “my” or “en-US”.";
    }
    if (source.trim() !== "" && !TAG_PATTERN.test(source.trim())) {
      errors.source = "Enter a language tag such as “en”, or leave blank for unknown.";
    }
    setFieldErrors(errors);
    if (Object.keys(errors).length > 0) {
      return;
    }
    setSaving(true);
    setSaveError(null);
    try {
      await updateProjectMetadata(project.id, null, target.trim(), source.trim());
      onSaved(await getProject(project.id));
      onClose();
    } catch (error) {
      setSaveError(
        error instanceof CommandError
          ? error.message
          : "The languages could not be saved.",
      );
    } finally {
      setSaving(false);
    }
  }

  return (
    <div className="dialog-backdrop">
      <div
        className="dialog dialog--narrow"
        role="dialog"
        aria-modal="true"
        aria-labelledby="language-dialog-title"
        tabIndex={-1}
        ref={dialogRef}
      >
        <div className="panel__header">
          <h2 className="panel__heading" id="language-dialog-title">
            {t("Translation languages")}
          </h2>
          <button
            type="button"
            className="icon-button"
            aria-label={t("Close")}
            title={t("Close")}
            disabled={saving}
            onClick={onClose}
          >
            <Icon name="close" />
          </button>
        </div>

        <p className="dialog__muted" role="status">
          {formatDirection(
            source.trim() === "" ? null : source,
            target.trim() === "" ? "…" : target,
          )}
        </p>

        <div className="form-grid">
          <label className="form-field">
            <span>{t("Source language (original book)")}</span>
            <input
              type="text"
              value={source}
              maxLength={32}
              list="language-suggestions"
              placeholder="en"
              onChange={(event) => {
                setSource(event.target.value);
                setFieldErrors((current) => {
                  const next = { ...current };
                  delete next.source;
                  return next;
                });
              }}
              aria-invalid={fieldErrors.source !== undefined}
            />
            {fieldErrors.source ? (
              <span className="form-field__error">{fieldErrors.source}</span>
            ) : null}
          </label>
          <label className="form-field">
            <span>{t("Target language (translation)")}</span>
            <input
              type="text"
              value={target}
              maxLength={32}
              list="language-suggestions"
              placeholder="my"
              onChange={(event) => {
                setTarget(event.target.value);
                setFieldErrors((current) => {
                  const next = { ...current };
                  delete next.target;
                  return next;
                });
              }}
              aria-invalid={fieldErrors.target !== undefined}
            />
            {fieldErrors.target ? (
              <span className="form-field__error">{fieldErrors.target}</span>
            ) : null}
          </label>
        </div>
        <datalist id="language-suggestions">
          {suggestions.current.map((suggestion) => (
            <option key={suggestion.value} value={suggestion.value}>
              {suggestion.label}
            </option>
          ))}
        </datalist>
        <p className="dialog__muted">
          {t(
            "Leave the source blank when the original language is unknown. Changing languages only relabels the project and future exports — book text is never rewritten.",
          )}
        </p>

        {saveError ? <StatusNotice kind="error">{saveError}</StatusNotice> : null}

        <div className="button-row dialog__actions">
          <button
            type="button"
            className="btn btn--primary"
            disabled={saving}
            onClick={() => void handleSave()}
          >
            <Icon name="save" />
            {saving ? t("Saving…") : t("Save")}
          </button>
        </div>
      </div>
    </div>
  );
}
