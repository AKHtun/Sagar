import { expect, test } from "bun:test";
import { Window } from "happy-dom";
import { act, createElement } from "react";

const browser = new Window();
Object.assign(globalThis, {
  window: browser, document: browser.document, navigator: browser.navigator,
  DOMParser: browser.DOMParser, Node: browser.Node, Element: browser.Element,
  HTMLElement: browser.HTMLElement, getComputedStyle: browser.getComputedStyle.bind(browser),
  requestAnimationFrame: browser.requestAnimationFrame.bind(browser),
  cancelAnimationFrame: browser.cancelAnimationFrame.bind(browser),
  IS_REACT_ACT_ENVIRONMENT: true,
});
const { createRoot } = await import("react-dom/client");
const { ChapterEditor } = await import("../../src/features/translator/ChapterEditor");
const { isEditorLink } = await import("../../src/features/translator/SelectionToolbar");
const { toggleBookInline } = await import("../../src/features/translator/inlineFormatting");
const { comparableDocument, documentFragment } = await import("../../src/features/translator/richText");
type Editor = import("@tiptap/react").Editor;

async function settle() {
  await act(async () => { await new Promise((resolve) => setTimeout(resolve, 50)); });
}

test("selection popup formats Myanmar text, saves links and dismisses without changing text", async () => {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  let editor!: Editor;
  let saved = "";
  const props = {
    html: '<p id="paragraph">မင်္ဂလာပါ world</p>', disabled: false, active: true,
    onChange: (html: string) => { saved = html; },
    onReady: (ready: Editor) => { editor = ready; }, onImage: () => {},
  };
  const button = (label: string) => document.querySelector<HTMLButtonElement>(
    `.selection-popup button[aria-label="${label}"]`,
  );
  try {
    await act(async () => { root.render(createElement(ChapterEditor, props)); });
    await settle();
    // happy-dom has no layout; provide the editor's DOM geometry only.
    editor.view.coordsAtPos = () => ({ left: 300, right: 400, top: 200, bottom: 225 });
    expect(document.querySelector(".selection-popup")).toBeNull();
    await act(async () => { editor.commands.focus(); editor.commands.setTextSelection({ from: 1, to: 10 }); });
    await settle();
    expect(button("Bold")).not.toBeNull();
    const originalText = editor.state.doc.textContent;
    for (const label of ["Bold", "Italic", "Underline", "Strikethrough", "Highlight", "Superscript", "Subscript"]) {
      await act(async () => { button(label)!.click(); });
      await settle();
      expect(button(label)?.getAttribute("aria-pressed")).toBe("true");
      expect(editor.state.selection.empty).toBe(false);
    }
    expect(saved).toContain("<mark>");
    expect(saved).toContain("<sup>");
    expect(saved).toContain("<sub>");
    expect(editor.state.doc.textContent).toBe(originalText);
    const snapshot = documentFragment(editor.getHTML());
    await act(async () => { editor.commands.setContent(snapshot, { parseOptions: { preserveWhitespace: true } }); });
    expect(comparableDocument(editor.getHTML())).toBe(comparableDocument(snapshot));
    await act(async () => { editor.commands.setTextSelection({ from: 1, to: 10 }); });
    await settle();
    await act(async () => { button("Edit link")!.click(); });
    await settle();
    let input = document.querySelector<HTMLInputElement>("#selection-link-url")!;
    expect(input).not.toBeNull();
    expect(document.activeElement).toBe(input);
    // Native input events update React state, as actual typing does.
    const enter = async (value: string) => {
      await act(async () => {
        const setter = Object.getOwnPropertyDescriptor(browser.HTMLInputElement.prototype, "value")!.set!;
        setter.call(input, value);
        input.dispatchEvent(new browser.Event("input", { bubbles: true }));
      });
    };
    await enter("javascript:alert(1)");
    await act(async () => { document.querySelector(".selection-popup form")!.dispatchEvent(new browser.Event("submit", { bubbles: true, cancelable: true })); });
    await settle();
    expect(document.querySelector('[role="alert"]')?.textContent).toContain("Use https");
    expect(editor.getHTML()).not.toContain("javascript:");
    expect(input.value).toBe("javascript:alert(1)");
    await enter("../notes.xhtml#note-1");
    await act(async () => { document.querySelector(".selection-popup form")!.dispatchEvent(new browser.Event("submit", { bubbles: true, cancelable: true })); });
    await settle();
    expect(saved).toContain('href="../notes.xhtml#note-1"');
    expect(saved).not.toContain("target=");
    expect(button("Edit link")?.getAttribute("aria-pressed")).toBe("true");
    await act(async () => { button("Edit link")!.click(); });
    await settle();
    input = document.querySelector<HTMLInputElement>("#selection-link-url")!;
    expect(input.value).toBe("../notes.xhtml#note-1");
    await act(async () => { document.querySelector<HTMLButtonElement>(".selection-popup .btn")!.click(); });
    await settle();
    expect(saved).not.toContain("href=");
    // Tab enters the popup; arrow keys move between controls; Escape restores editor focus.
    await act(async () => { editor.view.dom.dispatchEvent(new browser.KeyboardEvent("keydown", { key: "Tab", bubbles: true })); });
    expect(document.activeElement?.getAttribute("aria-label")).toBe("Bold");
    await act(async () => { document.activeElement!.dispatchEvent(new browser.KeyboardEvent("keydown", { key: "ArrowRight", bubbles: true })); });
    expect(document.activeElement?.getAttribute("aria-label")).toBe("Italic");
    await act(async () => { document.activeElement!.dispatchEvent(new browser.KeyboardEvent("keydown", { key: "Escape", bubbles: true })); });
    await settle();
    expect(document.querySelector(".selection-popup")).toBeNull();
    expect(editor.state.doc.textContent).toBe(originalText);
    await act(async () => { editor.commands.setTextSelection({ from: 2, to: 9 }); });
    await settle();
    expect(button("Bold")).not.toBeNull();
    await act(async () => { root.render(createElement(ChapterEditor, { ...props, disabled: true })); });
    await settle();
    expect(document.querySelector(".selection-popup")).toBeNull();
  } finally {
    await act(async () => root.unmount());
    container.remove();
  }
});

test("authored marks retain book attributes, anchors, inline elements and empty formatting on reload", async () => {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  let editor!: Editor;
  const html = '<p id="p1"><a id="note"></a><sup id="ref" epub:type="noteref"><a id="note-ref" href="#note">၁</a></sup> <mark title="မှတ်ချက်"><em>မြန်မာ</em></mark> <sub class="number">2</sub><sup id="empty"></sup>  </p>';
  try {
    await act(async () => { root.render(createElement(ChapterEditor, {
      html, disabled: false, active: true, onChange: () => {}, onImage: () => {},
      onReady: (ready: Editor) => { editor = ready; },
    })); });
    await settle();
    expect(comparableDocument(editor.getHTML())).toBe(comparableDocument(html));
    const saved = documentFragment(editor.getHTML());
    editor.commands.setContent(saved, { parseOptions: { preserveWhitespace: true } });
    expect(comparableDocument(editor.getHTML())).toBe(comparableDocument(saved));
    editor.view.coordsAtPos = () => ({ left: 300, right: 400, top: 200, bottom: 225 });
    await act(async () => { editor.commands.focus(); editor.commands.setTextSelection({ from: 3, to: 4 }); });
    await settle();
    await act(async () => { document.querySelector<HTMLButtonElement>('.selection-popup button[aria-label="Edit link"]')!.click(); });
    await settle();
    const remove = document.querySelector<HTMLButtonElement>(".selection-popup .btn")!;
    expect(remove.disabled).toBe(true);
    expect(remove.title).toContain("book attributes");
    expect(editor.getHTML()).toContain('id="note-ref"');
  } finally {
    await act(async () => root.unmount());
    container.remove();
  }
});

test("link entry accepts EPUB references and rejects active or absolute URLs", () => {
  for (const url of ["#note", "../notes.xhtml#note", "chapter.xhtml?x=1#p", "https://example.com", "mailto:reader@example.com", "a%20b.xhtml"]) expect(isEditorLink(url)).toBe(true);
  for (const url of ["", "javascript:alert(1)", "java\nscript:alert(1)", "data:text/html,x", "file:///tmp/a", "//example.com", "/etc/a", "a\\b", "a b"]) expect(isEditorLink(url)).toBe(false);
});

test("inline container tools toggle and undo without losing text or semantic attributes", async () => {
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  let editor!: Editor;
  const html = '<p>မြန်မာ &amp; two</p><p>Next paragraph</p>';
  try {
    await act(async () => { root.render(createElement(ChapterEditor, {
      html, disabled: false, active: false, onChange: () => {}, onImage: () => {},
      onReady: (ready: Editor) => { editor = ready; },
    })); });
    await settle();
    for (const name of ["book_mark", "book_sup", "book_sub"]) {
      editor.commands.setTextSelection({ from: 1, to: editor.state.doc.firstChild!.nodeSize - 1 });
      const before = comparableDocument(editor.getHTML());
      expect(toggleBookInline(editor, name)).toBe(true);
      const tag = name.slice(5);
      expect(editor.getHTML()).toContain(`<${tag}>မြန်မာ &amp; two</${tag}>`);
      expect(toggleBookInline(editor, name)).toBe(true);
      expect(comparableDocument(editor.getHTML())).toBe(before);
    }
    editor.commands.setTextSelection({ from: 1, to: 4 });
    expect(editor.commands.toggleCode()).toBe(true);
    expect(editor.getHTML()).toContain("<code>");
    expect(editor.commands.toggleCode()).toBe(true);
    expect(editor.getHTML()).not.toContain("<code>");
    editor.commands.setTextSelection({ from: 1, to: editor.state.doc.content.size - 1 });
    expect(toggleBookInline(editor, "book_mark", false)).toBe(false);
    editor.commands.setTextSelection(1);
    expect(toggleBookInline(editor, "book_sup", false)).toBe(false);
    editor.commands.setContent('<p><sup id="footnote">၁</sup></p>');
    editor.commands.setTextSelection({ from: 2, to: 3 });
    expect(toggleBookInline(editor, "book_sup", false)).toBe(false);
    expect(editor.getHTML()).toContain('id="footnote"');
    // A plain container addition is one undoable edit.
    editor.commands.setContent(html);
    editor.commands.setTextSelection({ from: 1, to: 4 });
    expect(toggleBookInline(editor, "book_mark")).toBe(true);
    expect(editor.commands.undo()).toBe(true);
    expect(comparableDocument(editor.getHTML())).toBe(comparableDocument(html));
  } finally {
    await act(async () => root.unmount());
    container.remove();
  }
});
