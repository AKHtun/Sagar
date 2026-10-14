const BLOCK_TAGS =
  "p|h1|h2|h3|h4|h5|h6|ul|ol|li|blockquote|table|thead|tbody|tfoot|tr|td|th";

/**
 * Pretty-print a chapter body fragment so the translation code view reads
 * like the original source: one block per line.
 *
 * The stored draft is a flat list of body-inner blocks (`<h1>`, `<p>`,
 * `<ul>`, `<table>` …). It is often saved as a single line
 * (`</h1><p>`), which is hard to scan. This inserts a single newline
 * between block boundaries only — never inside inline markup or text —
 * so the output stays valid XHTML and renders identically.
 *
 * Whitespace between block elements is insignificant for rendering and is
 * accepted by the Rust `validate_document` checks (including table
 * structure, which only rejects non-whitespace text). The function is
 * idempotent: already-formatted input is unchanged.
 */
export function formatChapterHtml(html: string): string {
  if (!html?.trim()) return "";
  let out = html.trim();
  // Repeat until stable so adjacent container boundaries (e.g.
  // <table><tbody><tr>) all split even though matches overlap.
  // Only spaces/tabs are consumed — never newlines — so already-split
  // boundaries are skipped and the next unsplit boundary still matches.
  for (let pass = 0; pass < 5; pass += 1) {
    const before = out;
    // </p><p> , </h1><p> , </ul><p> … → newline between top-level blocks.
    out = out.replace(
      new RegExp(`</(${BLOCK_TAGS})>[ \\t]*<(?!/)((?:${BLOCK_TAGS})[\\s>/])`, "gi"),
      "</$1>\n<$2",
    );
    // <hr/><p> → newline after a self-closing block.
    out = out.replace(/(<hr[^>]*\/>)[ \t]*<(?!\/)/gi, "$1\n<");
    // <ul><li> , <table><tbody> , <tr><td> … → newline after a container opens.
    out = out.replace(
      new RegExp(
        `(<(?:ul|ol|table|thead|tbody|tfoot|tr|blockquote)[^>]*>)[ \\t]*<(?!/)((?:${BLOCK_TAGS})[\\s>/])`,
        "gi",
      ),
      "$1\n<$2",
    );
    // </li></ul> , </td></tr> … → newline before a container closes.
    out = out.replace(
      /<\/(li|td|th|tr|thead|tbody|tfoot)>[ \t]*(<\/(?:ul|ol|table|thead|tbody|tfoot|tr)>)/gi,
      "</$1>\n$2",
    );
    if (out === before) break;
  }
  return out;
}
