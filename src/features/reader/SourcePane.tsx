import { type MouseEvent, useState } from "react";
import { Icon } from "../../components/Icon";
import { StatusNotice } from "../../components/StatusNotice";
import type { ChapterContent } from "../../types/chapter";

/** Full chapter preview is allowlisted in Rust. Raw XHTML is displayed as text. */
export function SourcePane({
  content,
  loading,
  error,
  onRetry,
}: {
  content: ChapterContent | null;
  loading: boolean;
  error: string | null;
  onRetry: () => void;
}) {
  const [view, setView] = useState<"reader" | "html">("reader");
  return (
    <section className="pane pane--source" aria-labelledby="source-pane-heading">
      <div className="pane__header">
        <h2 className="pane__heading" id="source-pane-heading">
          Original <span className="pane__badge">Read only</span>
        </h2>
        <fieldset className="view-switch view-switch--icons" aria-label="Original view">
          <button
            type="button"
            className="icon-button"
            aria-label="Original reader view"
            title="Reader view"
            aria-pressed={view === "reader"}
            onClick={() => setView("reader")}
          >
            <Icon name="book" />
          </button>
          <button
            type="button"
            className="icon-button"
            aria-label="Original HTML view"
            title="HTML view"
            aria-pressed={view === "html"}
            onClick={() => setView("html")}
          >
            <Icon name="code" />
          </button>
        </fieldset>
      </div>
      <div className="pane__scroll">
        {loading ? <StatusNotice kind="loading">Loading chapter…</StatusNotice> : null}
        {error ? (
          <StatusNotice kind="error" onRetry={onRetry}>
            {error}
          </StatusNotice>
        ) : null}
        {!loading && !error && content ? (
          view === "html" ? (
            <pre className="source-code">
              <code>{content.sourceDocument}</code>
            </pre>
          ) : (
            <article
              className="book-text"
              aria-label="Original chapter content"
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
          <StatusNotice kind="info">No chapter selected.</StatusNotice>
        ) : null}
      </div>
    </section>
  );
}
function interceptLinkClicks(event: MouseEvent) {
  const target = event.target as HTMLElement | null;
  if (target?.closest("a")) event.preventDefault();
}
