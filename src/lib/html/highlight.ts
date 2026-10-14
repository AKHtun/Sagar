import hljs from "highlight.js/lib/core";
import xml from "highlight.js/lib/languages/xml";

hljs.registerLanguage("xml", xml);

/** Presentation only: highlight.js escapes source before adding its token spans. */
export function highlightHtml(source: string): string {
  return hljs.highlight(source, { language: "xml", ignoreIllegals: true }).value;
}
