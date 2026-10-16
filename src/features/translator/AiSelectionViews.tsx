import { convertFileSrc } from "@tauri-apps/api/core";
import { useState } from "react";
import { Icon } from "../../components/Icon";
import { useTranslation } from "../../lib/i18n";
import { languageName } from "../../lib/languages";
import { preparePreviewImages } from "../../lib/tauri/preview";
import type { SelectionItem, SelectionRun, SelectionUnit } from "../../types/aiSelection";

export type WorkflowStep = "select" | "progress" | "review";
export function stepForRun(run: SelectionRun | null): WorkflowStep {
  return !run ? "select" : run.state === "done" ? "review" : "progress";
}
export const readyItems = (run: SelectionRun | null) =>
  (
    run?.items.filter((item) => item.state === "ready" && item.previewHtml !== null) ?? []
  ).sort((a, b) => a.order - b.order);

export type ChapterProposal = {
  item: SelectionItem;
  run: SelectionRun;
  runNumber: number;
};

/** Runs arrive newest first; one current successful proposal per source occurrence. */
export function chapterProposals(runs: SelectionRun[]): ChapterProposal[] {
  const current = new Map<string, ChapterProposal>();
  runs.forEach((run, index) => {
    for (const item of readyItems(run)) {
      if (!current.has(item.unitId))
        current.set(item.unitId, { item, run, runNumber: runs.length - index });
    }
  });
  return [...current.values()].sort((a, b) => a.item.order - b.item.order);
}

/** Both inputs are inert HTML from the Rust source/proposal sanitizer. */
export function SelectionPreview({ html, label }: { html: string; label: string }) {
  return (
    <section
      className="ai-selection__preview"
      aria-label={label}
      onClickCapture={(event) => {
        if (event.target instanceof Element && event.target.closest("a"))
          event.preventDefault();
      }}
      // biome-ignore lint/security/noDangerouslySetInnerHtml: Rust allowlisted local-resource preview
      dangerouslySetInnerHTML={{
        __html: preparePreviewImages(html, (path) => convertFileSrc(path, "epubres")),
      }}
    />
  );
}

export function SourceRows({
  units,
  selected,
  disabled,
  toggle,
}: {
  units: SelectionUnit[];
  selected: Set<string>;
  disabled: boolean;
  toggle: (id: string, checked: boolean) => void;
}) {
  const { t } = useTranslation();
  return (
    <section className="ai-selection__list" aria-label={t("Source selections")}>
      {units.map((unit) => (
        <article className="ai-selection__item" key={unit.id}>
          <label className="ai-selection__choice">
            <input
              type="checkbox"
              checked={selected.has(unit.id)}
              disabled={disabled || !!unit.unavailable}
              onChange={(event) => toggle(unit.id, event.target.checked)}
            />
            <span className="ai-selection__row-copy">
              <strong>
                {unit.order + 1}. <span className="ai-selection__kind">{unit.kind}</span>
              </strong>
              <span className="ai-selection__excerpt">
                {unit.text || "Image · included locally"}
              </span>
              <small>
                {unit.unavailable ??
                  (unit.added
                    ? "Added before"
                    : unit.hasProposal
                      ? "Saved proposal"
                      : t("Not translated"))}
              </small>
            </span>
          </label>
          {(unit.kind === "image" || unit.kind === "figure") && !unit.unavailable ? (
            <div className="ai-selection__thumbnail">
              <SelectionPreview
                html={unit.previewHtml}
                label={`Image ${unit.order + 1}`}
              />
            </div>
          ) : null}
          <details>
            <summary>{t("Expand formatted source")}</summary>
            <SelectionPreview
              html={unit.previewHtml}
              label={`Source item ${unit.order + 1}`}
            />
          </details>
        </article>
      ))}
      {!units.length ? <p>{t("No selectable source content in this chapter.")}</p> : null}
    </section>
  );
}

function ReviewRow({
  item,
  unit,
  checked,
  disabled,
  source,
  target,
  toggle,
}: {
  item: SelectionItem;
  unit?: SelectionUnit;
  checked: boolean;
  disabled: boolean;
  source: string;
  target: string;
  toggle: (checked: boolean) => void;
}) {
  const { t } = useTranslation();
  const [expanded, setExpanded] = useState(true);
  // Markup length is only an expansion heuristic; content is never truncated or reserialized.
  const long = item.sourcePreviewHtml.length + (item.previewHtml?.length ?? 0) > 1600;
  return (
    <article className="ai-selection__item ai-review-row">
      <label className="ai-selection__choice">
        <input
          type="checkbox"
          checked={checked}
          disabled={disabled}
          onChange={(event) => toggle(event.target.checked)}
        />
        <strong>
          {t("Include result")} {item.order + 1} · {unit?.kind ?? "Content"} ·{" "}
          {unit?.added ? "Added before" : "Ready"}
        </strong>
      </label>
      {long && !expanded ? (
        <p className="dialog__muted">
          {t("Long content — expand to compare the complete original and translation.")}
        </p>
      ) : null}
      <div className="ai-review-pair" hidden={long && !expanded}>
        <section>
          <h4>{source}</h4>
          <SelectionPreview
            html={item.sourcePreviewHtml}
            label={`Original item ${item.order + 1}`}
          />
        </section>
        <section>
          <h4>{target}</h4>
          <SelectionPreview
            html={item.previewHtml ?? ""}
            label={`Translated item ${item.order + 1}`}
          />
        </section>
      </div>
      {long ? (
        <button
          type="button"
          className="btn"
          aria-expanded={expanded}
          onClick={() => setExpanded(!expanded)}
        >
          <Icon name={expanded ? "minus" : "plus"} />
          {expanded ? "Collapse full text" : "Show full text"}
        </button>
      ) : null}
    </article>
  );
}

export function ReviewRows({
  entries,
  units,
  accepted,
  disabled,
  toggle,
}: {
  entries: ChapterProposal[];
  units: SelectionUnit[];
  accepted: Set<string>;
  disabled: boolean;
  toggle: (id: string, checked: boolean) => void;
}) {
  const { t } = useTranslation();
  return (
    <section aria-label={t("Paired translation results")}>
      {entries.map(({ item, run, runNumber }) => (
        <div key={item.unitId}>
          <p className="dialog__muted">
            {t("Batch")} {runNumber} · {run.config.model || "Local images"}
          </p>
          <ReviewRow
            item={item}
            unit={units.find((unit) => unit.id === item.unitId)}
            source={languageName(run.config.sourceLanguage) ?? "Original"}
            target={languageName(run.config.targetLanguage) ?? "Translation"}
            checked={accepted.has(item.id)}
            disabled={disabled}
            toggle={(checked) => toggle(item.id, checked)}
          />
        </div>
      ))}
    </section>
  );
}
