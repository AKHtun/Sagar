import { expect, test } from "bun:test";
import { Window } from "happy-dom";
import { Editor } from "@tiptap/react";
import StarterKit from "@tiptap/starter-kit";
import Image from "@tiptap/extension-image";
import { Table, TableRow } from "@tiptap/extension-table";
import {
  BookStructure,
  BookTableCell,
  BookTableHeader,
} from "../../src/features/translator/bookStructure";
import { BookAttributes } from "../../src/features/translator/bookAttributes";
import { BookAnchor } from "../../src/features/translator/bookAnchor";
import { comparableDocument, documentFragment } from "../../src/features/translator/richText";
import { runSlashCommand } from "../../src/features/translator/useSlashMenu";

const window = new Window();
Object.assign(globalThis, {
  window, document: window.document, navigator: window.navigator,
  DOMParser: window.DOMParser, Node: window.Node, Element: window.Element,
  HTMLElement: window.HTMLElement,
  getComputedStyle: window.getComputedStyle.bind(window),
  requestAnimationFrame: window.requestAnimationFrame.bind(window),
  cancelAnimationFrame: window.cancelAnimationFrame.bind(window),
});

const ChapterTable = Table.extend({
  renderHTML({ HTMLAttributes }: { HTMLAttributes: Record<string, unknown> }) {
    return ["table", HTMLAttributes, ["tbody", 0]];
  },
});

function createEditor(html: string) {
  return new Editor({
    extensions: [ ...BookStructure, BookAttributes, BookAnchor,
      ChapterTable.configure({ resizable: false }),
      TableRow, BookTableCell, BookTableHeader,
      StarterKit.configure({ trailingNode: false, link: { openOnClick: false, autolink: false } }),
      Image.configure({ allowBase64: true }),
    ],
    content: html,
    parseOptions: { preserveWhitespace: true },
  });
}

for (const [label, html] of [
  ["imprint", '<section class="imprint" id="imprint" epub:type="imprint" xmlns:epub="http://www.idpf.org/2007/ops"><header><h2 epub:type="title">ထုတ်ဝေမှု</h2><img src="logo.png" srcset="logo-2x.png 2x, logo.png 1x" sizes="100vw" alt="Logo"/></header><p>မြန်မာ <a href="uncopyright.xhtml">စာ</a></p></section>'],
  ["inline semantics", '<p><span class="small"><b>မြန်မာ</b> <i>စာ</i></span><sup id="note">1</sup><sub>2</sub><code>text</code><ruby>字<rt>ji</rt><rp>(</rp></ruby></p>'],
  ["inline image", '<p>Before <img src="a.png" alt="A" srcset="a.png 1x, b.png 2x"/> after</p>'],
  ["blockquote and code", '<blockquote id="quote"><p>မြန်မာ <code class="sample">x &lt; y</code></p></blockquote><pre id="sample"><code>one\n  two &amp; three</code></pre>'],
  ["definition list", '<dl><dt>Term</dt><dd><p>Meaning</p></dd></dl><pre>one\n  two</pre>'],
  ["SVG", '<svg xmlns="http://www.w3.org/2000/svg" xmlns:xlink="http://www.w3.org/1999/xlink" viewBox="0 0 10 10"><g><title>Cover</title><image xlink:href="cover.png"/><path d="M0 0L10 10"/></g></svg><p>မြန်မာ</p>'],
  ["responsive image", '<figure><picture><source srcset="a.webp" type="image/webp"/><img src="a.png" alt="A"/></picture><figcaption>Caption</figcaption></figure>'],
  ["table cell align", '<table border="1" cellpadding="4"><tbody><tr><td align="center" valign="top"><p>Hi</p></td></tr></tbody></table>'],
  ["myanmar table", '<table><tbody><tr><th><p>ခေါင်းစဉ်</p></th></tr><tr><td><p>မြန်မာ စာသား</p></td></tr></tbody></table>'],
]) {
  test(`Rich preserves ${label} through editing and reloading`, () => {
    const editor = createEditor(html);
    try {
      expect(comparableDocument(editor.getHTML())).toBe(comparableDocument(html));
      const saved = documentFragment(editor.getHTML());
      editor.commands.setContent(saved, { parseOptions: { preserveWhitespace: true } });
      expect(comparableDocument(editor.getHTML())).toBe(comparableDocument(html));
      const textPosition: number[] = [];
      editor.state.doc.descendants((node, position) => {
        if (node.isText && textPosition.length === 0) textPosition.push(position);
      });
      if (textPosition.length) {
        editor.commands.insertContentAt(textPosition[0], "မင်္ဂလာပါ ");
        expect(editor.getHTML()).toContain("မင်္ဂလာပါ ");
      }
    } finally { editor.destroy(); }
  });
}

test("SVG node view never mounts imported executable content", () => {
  const editor = createEditor('<svg><script>alert(1)</script><path d="M0 0"/></svg>');
  try {
    expect(editor.view.dom.querySelector("svg, script")).toBeNull();
    expect(editor.getHTML()).not.toContain("<script");
    expect(editor.view.dom.textContent).toContain("SVG artwork");
  } finally { editor.destroy(); }
});

test("Cell align attributes round-trip verbatim without injected styles", () => {
  const editor = createEditor(
    '<table border="1" cellpadding="4"><tbody><tr><td align="center" valign="top"><p>Hi</p></td></tr></tbody></table>',
  );
  try {
    // Regression: the stock cell parsed align="center" into a style
    // ("text-align: center"), so AI-translated chapters with aligned cells
    // failed the HTML → Rich preservation check and trapped users in HTML.
    expect(editor.getHTML()).not.toContain("text-align");
    expect(comparableDocument(editor.getHTML())).toBe(
      comparableDocument(
        '<table border="1" cellpadding="4"><tbody><tr><td align="center" valign="top"><p>Hi</p></td></tr></tbody></table>',
      ),
    );
    editor.commands.setTextSelection(5);
    expect(editor.chain().focus().addRowAfter().run()).toBe(true);
  } finally {
    editor.destroy();
  }
});

test("Header/footer/caption sections stay loss-aware (no silent collapse)", () => {
  // prosemirror-tables rewrites unknown section wrappers on every
  // transaction, so thead/tfoot/caption intentionally do NOT round-trip:
  // the HTML → Rich check must keep them in HTML view instead of silently
  // collapsing them into the body. Flattening belongs to AI Fix HTML.
  for (const html of [
    '<table><thead><tr><th><p>H</p></th></tr></thead><tbody><tr><td><p>C</p></td></tr></tbody></table>',
    '<table><caption>Cap</caption><tbody><tr><td><p>C</p></td></tr></tbody></table>',
  ]) {
    const editor = createEditor(html);
    try {
      expect(comparableDocument(editor.getHTML())).not.toBe(comparableDocument(html));
    } finally {
      editor.destroy();
    }
  }
});

for (const [command, query, nodeName] of [
  ["blockquote", "/blockquote", "blockquote"],
  ["code", "/code", "codeBlock"],
] as const) {
  test(`Slash ${command} removes the query, preserves text, and undoes as one edit`, () => {
    const editor = createEditor(`<p>${query} မြန်မာ</p>`);
    try {
      editor.commands.setTextSelection(query.length + 1);
      expect(runSlashCommand(editor, command, { from: 1, to: query.length + 1 }, () => {})).toBe(true);
      expect(editor.isActive(nodeName)).toBe(true);
      expect(editor.state.doc.textContent).toBe(" မြန်မာ");
      const saved = documentFragment(editor.getHTML());
      editor.commands.undo();
      expect(editor.state.doc.textContent).toBe(`${query} မြန်မာ`);
      editor.commands.setContent(saved, { parseOptions: { preserveWhitespace: true } });
      expect(comparableDocument(editor.getHTML())).toBe(comparableDocument(saved));
    } finally { editor.destroy(); }
  });
}

test("Toolbar inline code toggles selected Myanmar text and preserves escaping", () => {
  const editor = createEditor("<p>မြန်မာ &lt;code&gt;</p>");
  try {
    editor.commands.selectAll();
    expect(editor.commands.toggleCode()).toBe(true);
    expect(editor.getHTML()).toContain("<code>မြန်မာ &lt;code&gt;</code>");
    expect(editor.commands.toggleCode()).toBe(true);
    expect(editor.getHTML()).not.toContain("<code>");
    editor.commands.toggleBlockquote();
    expect(editor.isActive("blockquote")).toBe(true);
    editor.commands.setTextSelection(3);
    editor.commands.toggleBlockquote();
    expect(editor.isActive("blockquote")).toBe(false);
    editor.commands.toggleCodeBlock();
    expect(editor.isActive("codeBlock")).toBe(true);
    editor.commands.toggleCodeBlock();
    expect(editor.isActive("paragraph")).toBe(true);
  } finally { editor.destroy(); }
});

test("Slash inline code formats subsequent input and ignores a disabled editor", () => {
  const editor = createEditor("<p>/inline code</p>");
  try {
    editor.commands.setTextSelection(13);
    expect(runSlashCommand(editor, "inlineCode", { from: 1, to: 13 }, () => {})).toBe(true);
    editor.view.dispatch(editor.state.tr.insertText("မြန်မာ"));
    expect(editor.getHTML()).toContain("<code>မြန်မာ</code>");
    editor.commands.setContent("<p>/code</p>");
    editor.setEditable(false);
    expect(runSlashCommand(editor, "code", { from: 1, to: 6 }, () => {})).toBe(false);
    expect(editor.state.doc.textContent).toBe("/code");
  } finally { editor.destroy(); }
});
