import type { Editor } from "@tiptap/react";
import { Icon } from "../../components/Icon";
import type { BlockView } from "../../types/chapter";
import { RichBlockEditor } from "./RichBlockEditor";
import { hasContent, plainFragment } from "./richText";
import type { EditorBlock } from "./useTranslationEditor";

const STATUS_LABEL: Record<BlockView["status"], string> = {
  pending: "Not translated",
  draft: "Draft",
  reviewed: "Reviewed",
  skipped: "Keep original",
};

type BlockEditorRowProps = {
  block: BlockView;
  editorState: EditorBlock | undefined;
  onFocus: (editor: Editor) => void;
  onText: (blockId: string, text: string) => void;
  onMarkReviewed: (blockId: string) => void;
  onKeepOriginal: (blockId: string) => void;
  onTranslateAgain: (blockId: string, sourceText: string) => void;
  onCopyOriginal: (blockId: string, sourceText: string) => void;
  onRetry: (blockId: string) => void;
  onReloadAfterConflict: () => void;
};

/** One editable translation row bound to a stable source block id. */
export function BlockEditorRow({
  block,
  editorState,
  onFocus,
  onText,
  onMarkReviewed,
  onKeepOriginal,
  onTranslateAgain,
  onCopyOriginal,
  onRetry,
  onReloadAfterConflict,
}: BlockEditorRowProps) {
  const state = editorState;
  const status = state?.status ?? block.status;
  const text = state?.text ?? block.targetHtml ?? "";
  const hasText = hasContent(text);

  return (
    <div
      className={`target-block target-block--${status} target-block--${block.elementName}`}
    >
      <div className="target-block__toolbar">
        <span className={`target-block__chip target-block__chip--${status}`}>
          {STATUS_LABEL[status]}
          {state?.dirty ? " · unsaved" : ""}
        </span>
        {block.hasInlineMarkup ? (
          <span
            className="target-block__note"
            title="The original text contains inline formatting or links. Check formatting, links and images when replacing this content; “Keep original” preserves the source fully."
          >
            Check original formatting
          </span>
        ) : null}
        <span className="target-block__actions">
          <button
            type="button"
            className="btn btn--small"
            onClick={() => {
              if (block.sourceEditorHtml !== null && block.sourceEditorHtml !== undefined)
                onText(block.id, block.sourceEditorHtml);
              else onCopyOriginal(block.id, block.sourcePlainText);
            }}
            disabled={state?.conflict ?? false}
          >
            <Icon name="copy" />
            {block.sourceEditorHtml !== null ? "Copy original" : "Copy text only"}
          </button>
          <button
            type="button"
            className="btn btn--small"
            onClick={() => onMarkReviewed(block.id)}
            disabled={!hasText || (state?.conflict ?? false)}
            title={
              hasText
                ? "Confirm this translation as reviewed"
                : "Enter a translation first"
            }
          >
            <Icon name="check" />
            Mark reviewed
          </button>
          {status === "skipped" ? (
            <button
              type="button"
              className="btn btn--small"
              onClick={() => onTranslateAgain(block.id, block.sourcePlainText)}
            >
              <Icon name="edit" />
              Translate instead
            </button>
          ) : (
            <button
              type="button"
              className="btn btn--small"
              onClick={() => onKeepOriginal(block.id)}
              disabled={state?.conflict ?? false}
              title="Keep the original text for this block in the exported book"
            >
              <Icon name="keep" />
              Keep original
            </button>
          )}
        </span>
      </div>
      <RichBlockEditor
        html={
          status === "skipped"
            ? (block.sourceEditorHtml ?? plainFragment(block.sourcePlainText))
            : text
        }
        disabled={status === "skipped" || (state?.conflict ?? false)}
        label={`Translation for ${block.elementName}, section ${block.blockIndex + 1}`}
        onChange={(html) => onText(block.id, html)}
        onFocus={onFocus}
      />
      {state?.conflict ? (
        <div className="target-block__error" role="alert">
          <span>
            A newer saved version of this block exists. Your draft is kept above.
          </span>
          <button
            type="button"
            className="btn btn--small"
            onClick={onReloadAfterConflict}
          >
            <Icon name="refresh" />
            Reload saved content
          </button>
        </div>
      ) : state?.error ? (
        <div className="target-block__error" role="alert">
          <span>{state.error}</span>
          <button
            type="button"
            className="btn btn--small"
            onClick={() => onRetry(block.id)}
          >
            <Icon name="refresh" />
            Retry
          </button>
        </div>
      ) : null}
    </div>
  );
}
