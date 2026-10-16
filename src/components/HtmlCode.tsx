import { useMemo, useRef } from "react";
import { highlightHtml } from "../lib/html/highlight";
import { useTranslation } from "../lib/i18n";

export function HtmlCode({ value }: { value: string }) {
  const highlighted = useMemo(() => highlightHtml(value), [value]);
  return (
    <pre className="source-code html-syntax">
      {/* biome-ignore lint/security/noDangerouslySetInnerHtml: highlightHtml escapes all source markup; only generated token spans are HTML. */}
      <code dangerouslySetInnerHTML={{ __html: highlighted }} />
    </pre>
  );
}

/** A native textarea owns all editing; the inert layer only adds syntax colors. */
export function HtmlCodeEditor({
  value,
  disabled,
  onChange,
}: {
  value: string;
  disabled: boolean;
  onChange: (value: string) => void;
}) {
  const { t } = useTranslation();
  const backdrop = useRef<HTMLPreElement>(null);
  const highlighted = useMemo(() => highlightHtml(value), [value]);
  return (
    <div className="html-code-editor">
      <pre
        ref={backdrop}
        className="html-code-editor__highlight html-syntax"
        aria-hidden="true"
      >
        {/* biome-ignore lint/security/noDangerouslySetInnerHtml: Escaped highlighter output, never raw chapter HTML. */}
        <code dangerouslySetInnerHTML={{ __html: `${highlighted}\n` }} />
      </pre>
      <textarea
        className="html-editor"
        aria-label={t("Translation HTML code")}
        value={value}
        spellCheck={false}
        autoCapitalize="off"
        autoCorrect="off"
        disabled={disabled}
        onChange={(event) => onChange(event.target.value)}
        onScroll={(event) => {
          if (!backdrop.current) return;
          backdrop.current.scrollTop = event.currentTarget.scrollTop;
          backdrop.current.scrollLeft = event.currentTarget.scrollLeft;
        }}
      />
    </div>
  );
}
