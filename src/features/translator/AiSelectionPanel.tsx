import { convertFileSrc } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import { useCallback, useEffect, useRef, useState } from "react";
import { Icon } from "../../components/Icon";
import { StatusNotice } from "../../components/StatusNotice";
import { formatDirection } from "../../lib/languages";
import {
  controlAiSelectionRun,
  getAiSelectionState,
  prepareAiSelectionRun,
} from "../../lib/tauri/aiSelection";
import { preparePreviewImages } from "../../lib/tauri/preview";
import type { TranslationSelection } from "../../types/ai";
import type { SelectionEvent, SelectionState } from "../../types/aiSelection";
import { ApiRequestDetails } from "./ApiRequestDetails";
import type { useChapterEditor } from "./useChapterEditor";

const PAGE_SIZE = 50;
const message = (error: unknown) =>
  error instanceof Error
    ? error.message
    : "The selected translation could not be completed.";

/** Every HTML preview comes from Rust's inert source/proposal sanitizer. */
function Preview({ html, label }: { html: string; label: string }) {
  return (
    <section
      className="ai-selection__preview"
      aria-label={label}
      onClickCapture={(event) => {
        if (event.target instanceof Element && event.target.closest("a"))
          event.preventDefault();
      }}
      // biome-ignore lint/security/noDangerouslySetInnerHtml: Rust allowlisted, local-resource preview
      dangerouslySetInnerHTML={{
        __html: preparePreviewImages(html, (path) => convertFileSrc(path, "epubres")),
      }}
    />
  );
}

export function AiSelectionPanel({
  chapterId,
  selection,
  bookInstructions,
  currentInstructions,
  onInstructions,
  editor,
  onBusy,
}: {
  chapterId: string;
  selection: TranslationSelection | null;
  bookInstructions: string;
  currentInstructions: string;
  onInstructions: (value: string) => void;
  editor: ReturnType<typeof useChapterEditor>;
  onBusy: (busy: boolean) => void;
}) {
  const [data, setData] = useState<SelectionState | null>(null);
  const [selected, setSelected] = useState<Set<string>>(new Set());
  const [accepted, setAccepted] = useState<Set<string>>(new Set());
  const [runId, setRunId] = useState<string | null>(null);
  const [consent, setConsent] = useState(false);
  const [busy, setBusy] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [notice, setNotice] = useState<string | null>(null);
  const [stream, setStream] = useState("");
  const [trace, setTrace] = useState<string[]>([]);
  const [page, setPage] = useState(0);
  const [previewPage, setPreviewPage] = useState(0);
  const [allowAgain, setAllowAgain] = useState(false);
  const [confirmOrder, setConfirmOrder] = useState(false);
  const [eventsUnavailable, setEventsUnavailable] = useState(false);
  const mounted = useRef(true);
  const activeRun = useRef<string | null>(null);
  const currentJob = useRef<string | null>(null);
  const starting = useRef(false);
  const bufferedEvents = useRef<SelectionEvent[]>([]);
  const receiveEvent = useRef<(event: SelectionEvent) => void>(() => {});
  const loadVersion = useRef(0);
  const appendRequest = useRef<{ key: string; id: string } | null>(null);
  const run = data?.runs.find((r) => r.id === runId) ?? null;
  const running = busy || run?.state === "running";
  const chosen = data?.units.filter((u) => selected.has(u.id)) ?? [];
  const needsProvider = chosen.some((u) => u.text.length > 0);
  const localOnly = chosen.length > 0 && !needsProvider;
  const generateLabel = localOnly
    ? chosen.every((u) => u.kind === "image" || u.kind === "figure")
      ? "Preview selected images"
      : "Preview selected content"
    : "Translate selected";

  const reload = useCallback(async () => {
    const version = ++loadVersion.current;
    const next = await getAiSelectionState(chapterId);
    if (!mounted.current || version !== loadVersion.current) return;
    setData(next);
    setRunId((id) => id ?? next.runs[0]?.id ?? null);
  }, [chapterId]);

  useEffect(() => {
    mounted.current = true;
    void reload().catch((e: unknown) => {
      if (mounted.current) setError(message(e));
    });
    let unlisten: (() => void) | undefined;
    const receive = (payload: SelectionEvent) => {
      if (
        !mounted.current ||
        payload.chapterId !== chapterId ||
        payload.runId !== activeRun.current
      )
        return;
      if (starting.current) {
        bufferedEvents.current.push(payload);
        return;
      }
      if (payload.jobId !== currentJob.current) return;
      if (payload.kind === "stream") setStream(payload.partialText ?? "");
      else if (payload.kind === "status")
        setTrace((old) => [...old.slice(-19), payload.message]);
      else {
        setNotice(payload.message);
        if (["done", "failed", "paused"].includes(payload.kind)) setBusy(false);
        void reload().catch((e: unknown) => {
          if (mounted.current) setError(message(e));
        });
      }
    };
    receiveEvent.current = receive;
    void listen<SelectionEvent>("ai-selection-event", ({ payload }) => receive(payload))
      .then((fn) => {
        if (mounted.current) unlisten = fn;
        else fn();
      })
      .catch(() => {
        if (mounted.current) setEventsUnavailable(true);
      });
    return () => {
      mounted.current = false;
      unlisten?.();
      if (activeRun.current)
        void controlAiSelectionRun(activeRun.current, "cancel", false).catch(() => {});
    };
  }, [chapterId, reload]);

  useEffect(() => {
    onBusy(running || editor.appending);
  }, [running, editor.appending, onBusy]);
  // Persisted checkpoints recover missed events and terminal-state races.
  useEffect(() => {
    if (!running) return;
    const timer = setInterval(() => {
      void reload().catch((e: unknown) => {
        if (mounted.current) setError(message(e));
      });
    }, 2000);
    return () => clearInterval(timer);
  }, [running, reload]);
  useEffect(() => {
    if (run?.state === "running") activeRun.current = run.id;
    if (!starting.current && run && run.state !== "running") setBusy(false);
  }, [run]);

  const start = async () => {
    if (starting.current || running) return;
    starting.current = true;
    bufferedEvents.current = [];
    setBusy(true);
    setError(null);
    setNotice(null);
    setStream("");
    setTrace([]);
    try {
      const id = await prepareAiSelectionRun({
        chapterId,
        unitIds: [...selected],
        agent: selection?.agent ?? "",
        model: selection?.model ?? "",
        reasoningEffort: selection?.reasoningEffort ?? null,
        currentInstructions,
      });
      if (!mounted.current) return;
      activeRun.current = id;
      setRunId(id);
      setAccepted(new Set());
      setPreviewPage(0);
      currentJob.current = await controlAiSelectionRun(id, "start", consent);
      if (!mounted.current) {
        void controlAiSelectionRun(id, "cancel", false).catch(() => {});
        return;
      }
      starting.current = false;
      for (const event of bufferedEvents.current) receiveEvent.current(event);
      bufferedEvents.current = [];
      await reload();
    } catch (e) {
      starting.current = false;
      if (mounted.current) {
        setError(message(e));
        setBusy(false);
        await reload().catch(() => {});
      }
    }
  };

  const control = async (action: "start" | "retry" | "smaller" | "skip" | "cancel") => {
    if (!runId) return;
    if (action !== "cancel" && starting.current) return;
    setError(null);
    setNotice(null);
    setStream("");
    if (action !== "cancel") setBusy(true);
    if (action !== "cancel") {
      starting.current = true;
      bufferedEvents.current = [];
    }
    activeRun.current = runId;
    try {
      const job = await controlAiSelectionRun(runId, action, consent);
      if (!mounted.current) {
        if (action !== "cancel")
          void controlAiSelectionRun(runId, "cancel", false).catch(() => {});
        return;
      }
      if (action !== "cancel") currentJob.current = job;
      starting.current = false;
      for (const event of bufferedEvents.current) receiveEvent.current(event);
      bufferedEvents.current = [];
      await reload();
    } catch (e) {
      starting.current = false;
      if (mounted.current) {
        setError(message(e));
        setBusy(false);
      }
    }
  };

  const append = async () => {
    const proposalIds = [...accepted].sort();
    const key = JSON.stringify(proposalIds);
    if (appendRequest.current?.key !== key)
      appendRequest.current = { key, id: crypto.randomUUID() };
    setError(null);
    setNotice(null);
    try {
      await editor.appendProposals({
        proposalIds,
        requestId: appendRequest.current.id,
        allowAgain,
        confirmOutOfOrder: confirmOrder,
      });
      if (!mounted.current) return;
      appendRequest.current = null;
      setAccepted(new Set());
      setSelected(new Set());
      setConfirmOrder(false);
      setNotice("Selected results were appended and saved as a draft.");
      await reload();
    } catch (e) {
      if (mounted.current) setError(message(e));
    }
  };

  const toggle = (id: string, checked: boolean, target: "source" | "preview") => {
    const setter = target === "source" ? setSelected : setAccepted;
    setter((old) => {
      const next = new Set(old);
      if (checked) next.add(id);
      else next.delete(id);
      return next;
    });
  };
  const ready = run?.items.filter((i) => i.state === "ready") ?? [];
  const added = new Set(data?.units.filter((u) => u.added).map((u) => u.id));
  const hasPendingText = run?.items.some(
    (i) =>
      !["ready", "skipped"].includes(i.state) &&
      data?.units.some((u) => u.id === i.unitId && u.text.length > 0),
  );

  return (
    <section className="ai-selection" aria-label="Selected paragraphs translation">
      <p className="dialog__muted">
        Select text, whole tables, and images. Images stay local. Only selected text and a
        little neighboring context go to your provider. Accepted results append to the end
        of your draft; unselected content is not copied.
      </p>
      <p className="dialog__muted">
        {selection
          ? `${selection.agent} · ${selection.model} · ${selection.reasoningEffort ?? "Default effort"}`
          : "Choose an AI model in Settings to translate text. Images can be included without a provider."}
      </p>
      {bookInstructions ? (
        <details>
          <summary>Book instructions</summary>
          <p className="book-instructions">{bookInstructions}</p>
        </details>
      ) : null}
      <label className="form-field">
        <span>Instructions for this run (optional)</span>
        <textarea
          rows={2}
          maxLength={20000}
          disabled={running || editor.appending}
          value={currentInstructions}
          onChange={(e) => onInstructions(e.target.value)}
        />
      </label>
      <p className="dialog__muted">
        New selections use these instructions. Resume and Retry use the saved run's
        original model, languages, and instructions.
      </p>
      {error ? (
        <StatusNotice
          kind="error"
          onRetry={() => {
            setError(null);
            void reload().catch((e: unknown) => setError(message(e)));
          }}
        >
          {error}
        </StatusNotice>
      ) : null}
      {notice ? <StatusNotice kind="info">{notice}</StatusNotice> : null}
      {!data ? (
        error ? null : (
          <StatusNotice kind="loading">Loading source selections…</StatusNotice>
        )
      ) : (
        <>
          <div className="button-row">
            <button
              type="button"
              className="btn"
              disabled={running}
              onClick={() =>
                setSelected(
                  new Set(
                    data.units
                      .filter((u) => !u.unavailable && (!u.added || allowAgain))
                      .map((u) => u.id),
                  ),
                )
              }
            >
              <Icon name="check" />
              Select all available
            </button>
            <button
              type="button"
              className="btn"
              disabled={running}
              onClick={() =>
                setSelected(
                  new Set(
                    data.units
                      .filter((u) => !u.unavailable && !u.added && !u.hasProposal)
                      .map((u) => u.id),
                  ),
                )
              }
            >
              <Icon name="paragraph" />
              Select untranslated
            </button>
            <button
              type="button"
              className="btn"
              disabled={running}
              onClick={() => setSelected(new Set())}
            >
              <Icon name="close" />
              Clear selection
            </button>
          </div>
          <p role="status">
            {selected.size} selected · {data.units.length} source items
          </p>
          <section className="ai-selection__list" aria-label="Source selections">
            {data.units.slice(page * PAGE_SIZE, (page + 1) * PAGE_SIZE).map((unit) => (
              <div className="ai-selection__item" key={unit.id}>
                <label className="ai-selection__choice">
                  <input
                    type="checkbox"
                    checked={selected.has(unit.id)}
                    disabled={
                      running || !!unit.unavailable || (unit.added && !allowAgain)
                    }
                    onChange={(e) => toggle(unit.id, e.target.checked, "source")}
                  />
                  <span>
                    <strong>
                      {unit.order + 1}. {unit.kind}
                    </strong>{" "}
                    — {unit.text || "Include image"}
                    <small>
                      {unit.unavailable ??
                        (unit.added
                          ? "Added earlier"
                          : unit.hasProposal
                            ? "Saved proposal available"
                            : unit.text
                              ? "Pending"
                              : "Include image")}
                    </small>
                  </span>
                </label>
                <details>
                  <summary>Show source</summary>
                  <Preview
                    html={unit.previewHtml}
                    label={`Source item ${unit.order + 1}`}
                  />
                </details>
                {unit.kind === "image" || (unit.kind === "figure" && !unit.text) ? (
                  <Preview html={unit.previewHtml} label={`Image ${unit.order + 1}`} />
                ) : null}
              </div>
            ))}
            {!data.units.length ? (
              <p>No selectable source content in this chapter.</p>
            ) : null}
          </section>
          {data.units.length > PAGE_SIZE ? (
            <div className="button-row">
              <button
                type="button"
                className="btn"
                disabled={page === 0}
                onClick={() => setPage(page - 1)}
              >
                <Icon name="left" />
                Previous items
              </button>
              <span>
                Page {page + 1} of {Math.ceil(data.units.length / PAGE_SIZE)}
              </span>
              <button
                type="button"
                className="btn"
                disabled={(page + 1) * PAGE_SIZE >= data.units.length}
                onClick={() => setPage(page + 1)}
              >
                <Icon name="right" />
                Next items
              </button>
            </div>
          ) : null}
          <label className="export-ack">
            <input
              type="checkbox"
              checked={allowAgain}
              disabled={running || editor.appending}
              onChange={(e) => {
                setAllowAgain(e.target.checked);
                setSelected(new Set());
                setAccepted(new Set());
              }}
            />
            <span>
              Add again: allow selecting items added earlier. Repeated anchor IDs are
              still rejected. History covers this mode only and does not track later
              manual deletions.
            </span>
          </label>
          <label className="export-ack">
            <input
              type="checkbox"
              checked={consent}
              disabled={running}
              onChange={(e) => setConsent(e.target.checked)}
            />
            <span>
              I understand selected text and neighboring context will be sent to the
              chosen provider. Image bytes are never sent.
            </span>
          </label>
          {data.runs.length ? (
            <label className="form-field">
              <span>Saved proposals and runs</span>
              <select
                value={runId ?? ""}
                disabled={running || editor.appending}
                onChange={(e) => {
                  setRunId(e.target.value);
                  setAccepted(new Set());
                  setPreviewPage(0);
                  setStream("");
                  setTrace([]);
                  setError(null);
                }}
              >
                {data.runs.map((r, i) => (
                  <option key={r.id} value={r.id}>
                    Run {data.runs.length - i} · {r.config.model || "Images only"} ·{" "}
                    {r.state}
                  </option>
                ))}
              </select>
            </label>
          ) : null}
          {run ? (
            <section aria-label="Saved run">
              <p>
                {formatDirection(run.config.sourceLanguage, run.config.targetLanguage)} ·{" "}
                {run.config.model || "Images only"} ·{" "}
                {run.items.filter((i) => i.state === "ready").length}/{run.items.length}{" "}
                ready
              </p>
              <details>
                <summary>Saved run instructions and model</summary>
                <p>
                  {run.config.agent} · {run.config.model} ·{" "}
                  {run.config.reasoningEffort ?? "Default effort"}
                </p>
                <p className="book-instructions">
                  {run.config.bookInstructions || "No book instructions"}
                </p>
                <p className="book-instructions">
                  {run.config.currentInstructions || "No additional instructions"}
                </p>
              </details>
              {run.message ? (
                <StatusNotice kind={run.state === "failed" ? "error" : "info"}>
                  {run.message}
                </StatusNotice>
              ) : null}
              {running ? (
                <div className="button-row">
                  <StatusNotice kind="loading">
                    Translating… completed parts are saved locally.
                  </StatusNotice>
                  <button
                    type="button"
                    className="btn"
                    onClick={() => void control("cancel")}
                  >
                    <Icon name="close" />
                    Cancel
                  </button>
                </div>
              ) : run.state !== "done" ? (
                <div className="button-row">
                  <button
                    type="button"
                    className="btn"
                    disabled={hasPendingText && !consent}
                    onClick={() =>
                      void control(run.state === "failed" ? "retry" : "start")
                    }
                  >
                    <Icon name="refresh" />
                    {run.state === "failed" ? "Retry failed batch" : "Resume saved run"}
                  </button>
                  {run.state === "failed" ? (
                    <>
                      <button
                        type="button"
                        className="btn"
                        disabled={!consent || run.budget <= 512}
                        onClick={() => void control("smaller")}
                      >
                        <Icon name="minus" />
                        Retry with smaller batches
                      </button>
                      <button
                        type="button"
                        className="btn"
                        disabled={hasPendingText && !consent}
                        onClick={() => void control("skip")}
                      >
                        <Icon name="right" />
                        Skip and continue
                      </button>
                    </>
                  ) : null}
                </div>
              ) : null}
              <ApiRequestDetails messages={trace} />
              {eventsUnavailable ? (
                <p role="status">
                  Live output is unavailable. Saved progress is refreshed while the run is
                  active.
                </p>
              ) : null}
              {stream ? (
                <textarea
                  className="ai-stream"
                  aria-label="Selected translation live output"
                  readOnly
                  value={stream}
                />
              ) : null}
              {run.items
                .filter((i) => i.state !== "ready")
                .slice(0, PAGE_SIZE)
                .map((item) => (
                  <div key={item.id} className="ai-selection__item">
                    <p>
                      Item {item.order + 1}: {item.state} · {item.partsDone}/
                      {item.partsTotal} parts saved
                    </p>
                    {item.failedOutput ? (
                      <details>
                        <summary>Recoverable provider output</summary>
                        <textarea
                          readOnly
                          value={item.failedOutput}
                          aria-label={`Recoverable output for item ${item.order + 1}`}
                        />
                      </details>
                    ) : null}
                  </div>
                ))}
              {ready.length ? (
                <>
                  <h3>Preview results</h3>
                  <div className="button-row">
                    <button
                      type="button"
                      className="btn"
                      disabled={editor.appending}
                      onClick={() =>
                        setAccepted(
                          new Set(
                            ready
                              .filter((i) => !added.has(i.unitId) || allowAgain)
                              .map((i) => i.id),
                          ),
                        )
                      }
                    >
                      <Icon name="check" />
                      Select ready results
                    </button>
                  </div>
                  {ready
                    .slice(previewPage * PAGE_SIZE, (previewPage + 1) * PAGE_SIZE)
                    .map((item) => (
                      <div key={item.id} className="ai-selection__item">
                        <label className="ai-selection__choice">
                          <input
                            type="checkbox"
                            checked={accepted.has(item.id)}
                            disabled={
                              editor.appending || (added.has(item.unitId) && !allowAgain)
                            }
                            onChange={(e) => toggle(item.id, e.target.checked, "preview")}
                          />
                          <strong>
                            Include result {item.order + 1}
                            {added.has(item.unitId) ? " — Added earlier" : ""}
                          </strong>
                        </label>
                        <details>
                          <summary>Compare source</summary>
                          <Preview
                            html={item.sourcePreviewHtml}
                            label={`Original item ${item.order + 1}`}
                          />
                        </details>
                        <Preview
                          html={item.previewHtml ?? ""}
                          label={`Translated item ${item.order + 1}`}
                        />
                      </div>
                    ))}
                  {ready.length > PAGE_SIZE ? (
                    <div className="button-row">
                      <button
                        type="button"
                        className="btn"
                        disabled={!previewPage}
                        onClick={() => setPreviewPage(previewPage - 1)}
                      >
                        <Icon name="left" />
                        Previous results
                      </button>
                      <button
                        type="button"
                        className="btn"
                        disabled={(previewPage + 1) * PAGE_SIZE >= ready.length}
                        onClick={() => setPreviewPage(previewPage + 1)}
                      >
                        <Icon name="right" />
                        Next results
                      </button>
                    </div>
                  ) : null}
                  <p className="dialog__muted">
                    Adds {accepted.size} results to the end of your current draft,
                    preserving existing writing. Review chapter completeness and links to
                    content you did not include before export.
                  </p>
                  <label className="export-ack">
                    <input
                      type="checkbox"
                      checked={confirmOrder}
                      onChange={(e) => setConfirmOrder(e.target.checked)}
                    />
                    <span>Allow appending earlier source items after later content.</span>
                  </label>
                </>
              ) : null}
            </section>
          ) : null}
          <div className="button-row ai-selection__actions">
            <button
              type="button"
              className="btn btn--primary"
              disabled={
                running ||
                editor.appending ||
                !selected.size ||
                (needsProvider && (!consent || !selection))
              }
              onClick={() => void start()}
            >
              <Icon name={localOnly ? "image" : "sparkles"} />
              {generateLabel}
            </button>
            {ready.length ? (
              <button
                type="button"
                className={accepted.size ? "btn btn--primary" : "btn"}
                disabled={
                  !accepted.size ||
                  running ||
                  editor.appending ||
                  !!editor.draft?.conflict
                }
                onClick={() => void append()}
              >
                <Icon name="plus" />
                {editor.appending ? "Saving…" : "Append selected results"}
              </button>
            ) : null}
          </div>
        </>
      )}
    </section>
  );
}
