import { type MouseEvent, type RefObject, useEffect, useRef, useState } from "react";
import { HtmlCode } from "../../components/HtmlCode";
import { Icon } from "../../components/Icon";
import { StatusNotice } from "../../components/StatusNotice";
import { useTranslation } from "../../lib/i18n";
import type { ChapterContent } from "../../types/chapter";

/** Full chapter preview is allowlisted in Rust. Raw XHTML is displayed as text. */
export function SourcePane({
  content,
  loading,
  error,
  onRetry,
  scrollRef,
}: {
  content: ChapterContent | null;
  loading: boolean;
  error: string | null;
  onRetry: () => void;
  scrollRef?: RefObject<HTMLDivElement | null>;
}) {
  const { t } = useTranslation();
  const [view, setView] = useState<"reader" | "html">("reader");
  const [copyState, setCopyState] = useState<"idle" | "copying" | "copied" | "error">(
    "idle",
  );
  const copyRequest = useRef(0);
  // biome-ignore lint/correctness/useExhaustiveDependencies: Reset feedback and invalidate pending clipboard results when the displayed chapter or view changes.
  useEffect(() => {
    setCopyState("idle");
    copyRequest.current += 1;
    return () => {
      copyRequest.current += 1;
    };
  }, [content, loading, error, view]);

  async function copyHtml() {
    if (!content || loading || error || copyState === "copying") return;
    const request = ++copyRequest.current;
    setCopyState("copying");
    try {
      await navigator.clipboard.writeText(content.sourceDocument);
      if (request === copyRequest.current) setCopyState("copied");
    } catch {
      if (request === copyRequest.current) setCopyState("error");
    }
  }
  return (
    <section className="pane pane--source" aria-labelledby="source-pane-heading">
      <div className="pane__header">
        <h2 className="pane__heading" id="source-pane-heading">
          {t("Original")} <span className="pane__badge">{t("Read only")}</span>
        </h2>
        <fieldset
          className="view-switch view-switch--icons"
          aria-label={t("Original view")}
        >
          {view === "html" ? (
            <button
              type="button"
              className="icon-button"
              aria-label={t("Copy original HTML")}
              title={copyState === "copied" ? "HTML copied" : t("Copy original HTML")}
              disabled={loading || !!error || !content || copyState === "copying"}
              onClick={() => void copyHtml()}
            >
              <Icon name={copyState === "copied" ? "check" : "copy"} />
            </button>
          ) : null}
          <button
            type="button"
            className="icon-button"
            aria-label={t("Original reader view")}
            title={t("Reader view")}
            aria-pressed={view === "reader"}
            onClick={() => setView("reader")}
          >
            <Icon name="book" />
          </button>
          <button
            type="button"
            className="icon-button"
            aria-label={t("Original HTML view")}
            title={t("HTML view")}
            aria-pressed={view === "html"}
            onClick={() => setView("html")}
          >
            <Icon name="code" />
          </button>
        </fieldset>
      </div>
      <div className="pane__scroll" ref={scrollRef}>
        {copyState === "copied" ? (
          <StatusNotice kind="success">{t("Original HTML copied.")}</StatusNotice>
        ) : null}
        {copyState === "error" ? (
          <StatusNotice
            kind="error"
            onRetry={() => void copyHtml()}
            retryLabel="Copy again"
          >
            {t("Could not copy HTML. Try again or select the code and copy it manually.")}
          </StatusNotice>
        ) : null}
        {loading ? (
          <StatusNotice kind="loading">{t("Loading chapter…")}</StatusNotice>
        ) : null}
        {error ? (
          <StatusNotice kind="error" onRetry={onRetry}>
            {error}
          </StatusNotice>
        ) : null}
        {!loading && !error && content ? (
          view === "html" ? (
            <HtmlCode value={content.sourceDocument} />
          ) : (
            <article
              className="book-text"
              aria-label={t("Original chapter content")}
              onClick={interceptLinkClicks}
              onKeyDown={(event) => {
                if (event.key === "Enter" && (event.target as HTMLElement).closest("a"))
                  event.preventDefault();
              }}
            >
              <div dangerouslySetInnerHTML={{ __html: content.previewHtml }} />
            </article>
          )
        ) : null}
        {!loading && !error && !content ? (
          <StatusNotice kind="info">{t("No chapter selected.")}</StatusNotice>
        ) : null}
      </div>
    </section>
  );
}
function interceptLinkClicks(event: MouseEvent) {
  const target = event.target as HTMLElement | null;
  if (target?.closest("a")) event.preventDefault();
}
