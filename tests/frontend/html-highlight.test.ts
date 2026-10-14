import { expect, test } from "bun:test";
import { highlightHtml } from "../../src/lib/html/highlight";

function decodedText(highlighted: string) {
  return highlighted.replace(/<[^>]*>/g, "")
    .replace(/&lt;/g, "<").replace(/&gt;/g, ">")
    .replace(/&quot;/g, '"').replace(/&#x27;/g, "'").replace(/&amp;/g, "&");
}

test("colors XHTML tags, attributes, strings and comments without changing text", () => {
  const source = '<?xml version="1.0"?>\n<!-- note -->\n<p id="မြန်မာ">မင်္ဂလာပါ &amp; text</p>\n';
  const result = highlightHtml(source);
  for (const token of ["name", "attr", "string", "comment"]) {
    expect(result).toContain(`hljs-${token}`);
  }
  expect(decodedText(result)).toBe(source);
});

test("escapes executable source markup and tolerates unfinished code", () => {
  for (const source of ['<script>alert("x")</script>', '<img src="x" onerror="alert(1)"/>', '<p title="unfinished', "", "\n\n"]) {
    const result = highlightHtml(source);
    expect(result).not.toContain("<script>");
    expect(result).not.toContain("<img ");
    expect(decodedText(result)).toBe(source);
  }
});
