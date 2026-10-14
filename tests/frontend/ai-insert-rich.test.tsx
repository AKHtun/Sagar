import { expect, mock, test } from "bun:test";
import { Window } from "happy-dom";
import { act, createElement, useMemo } from "react";
import { createRoot } from "react-dom/client";

mock.module("@tauri-apps/api/event", () => ({
  listen: async () => () => {},
}));
let rejectValidation = false;
mock.module("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => path,
  invoke: async (command: string, args?: Record<string, unknown>) => {
    if (command === "save_chapter_document") return { revision: 2, status: "draft" };
    // Simulate the Rust validator accepting well-formed fragments verbatim.
    if (command === "validate_chapter_html") {
      // Real Rust AppError failures arrive as { code, message }.
      if (rejectValidation)
        throw { code: "invalid_translation", message: "Unsupported <marquee> element." };
      return (args?.html as string) ?? "";
    }
    throw new Error(`Unexpected command: ${command}`);
  },
}));
mock.module("@tauri-apps/plugin-dialog", () => ({
  open: async () => null,
  save: async () => null,
}));

const { TargetPane } = await import("../../src/features/translator/TargetPane");
const { useChapterEditor } = await import("../../src/features/translator/useChapterEditor");
type EditorApi = ReturnType<typeof useChapterEditor>;

let api: EditorApi | null = null;

function Harness({ proposal }: { proposal: string | null }) {
  const content = useMemo(
    () => ({
      chapterId: "ch1",
      title: "Chapter 1",
      // Fresh chapter: no translation yet.
      document: { html: "", status: "not_started", revision: 1 },
      images: [],
    }),
    [],
  );
  const editor = useChapterEditor(content as never);
  api = editor;
  // Simulate AiGenerateDialog's empty-draft path: proposal goes straight in.
  if (proposal !== null && (editor.draft?.html ?? "") !== proposal) {
    editor.setHtml(proposal);
  }
  return createElement(TargetPane, {
    projectId: "p1",
    content: content as never,
    loading: false,
    error: null,
    onRetry: () => {},
    onReloadAfterConflict: () => {},
    editor,
  });
}

async function settle(window: Window, rounds = 8) {
  for (let i = 0; i < rounds; i += 1) {
    await act(async () => {
      await new Promise((resolve) => setTimeout(resolve, 25));
    });
  }
  // let happy-dom + tiptap flush
  await act(async () => {});
}

test("failed validation lands in HTML view with a visible error, views stay usable", async () => {
  const window = new Window();
  Object.assign(globalThis, {
    window,
    document: window.document,
    navigator: window.navigator,
    DOMParser: window.DOMParser,
    Node: window.Node,
    Element: window.Element,
    HTMLElement: window.HTMLElement,
    getComputedStyle: window.getComputedStyle.bind(window),
    requestAnimationFrame: window.requestAnimationFrame.bind(window),
    cancelAnimationFrame: window.cancelAnimationFrame.bind(window),
    IS_REACT_ACT_ENVIRONMENT: true,
  });
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  rejectValidation = true;
  try {
    await act(async () => {
      root.render(createElement(Harness, { proposal: null }));
    });
    await settle(window);
    await act(async () => {
      root.render(createElement(Harness, { proposal: "<p>Has unsupported markup.</p>" }));
    });
    await settle(window);
    // Rejected content is kept in the code view with the reason visible…
    const code = container.querySelector('textarea[aria-label="Translation HTML code"]');
    expect(code).not.toBeNull();
    expect(code?.value).toContain("Has unsupported markup.");
    expect(container.textContent).toContain("Unsupported <marquee> element.");
    // …and neither view button is left wedged disabled.
    const richButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="Rich text view"]',
    );
    const htmlButton = container.querySelector<HTMLButtonElement>(
      'button[aria-label="HTML view"]',
    );
    expect(richButton?.disabled).toBe(false);
    expect(htmlButton?.disabled).toBe(false);
  } finally {
    rejectValidation = false;
    await act(async () => root.unmount());
    container.remove();
    await window.happyDOM.close();
  }
});

for (const proposal of [
  "<p>Translated one.</p><p>Translated two.</p>",
  "<h2>Title</h2><p>Body with <em>emphasis</em> and မြန်မာ text.</p>",
]) {
  test(`AI proposal reaches the Rich editor: ${proposal.slice(0, 30)}`, async () => {
    const window = new Window();
    Object.assign(globalThis, {
      window,
      document: window.document,
      navigator: window.navigator,
      DOMParser: window.DOMParser,
      Node: window.Node,
      Element: window.Element,
      HTMLElement: window.HTMLElement,
      getComputedStyle: window.getComputedStyle.bind(window),
      requestAnimationFrame: window.requestAnimationFrame.bind(window),
      cancelAnimationFrame: window.cancelAnimationFrame.bind(window),
      IS_REACT_ACT_ENVIRONMENT: true,
    });
    const container = document.createElement("div");
    document.body.append(container);
    const root = createRoot(container);
    api = null;
    try {
      await act(async () => {
        root.render(createElement(Harness, { proposal: null }));
      });
      await settle(window);
      // Rich editor mounted and ready.
      const richButton = container.querySelector<HTMLButtonElement>(
        'button[aria-label="Rich text view"]',
      );
      expect(richButton).not.toBeNull();
      expect(richButton?.disabled).toBe(false);

      // AI done on an empty draft: straight into the draft.
      await act(async () => {
        root.render(createElement(Harness, { proposal }));
      });
      await settle(window);

      const richEditor = container.querySelector(".chapter-editor");
      expect(richEditor).not.toBeNull();
      expect(richEditor?.textContent).toContain(
        proposal.includes("Translated") ? "Translated one." : "Body with",
      );
      // Still in Rich view, no HTML trap, no error notice.
      expect(
        container.querySelector('textarea[aria-label="Translation HTML code"]'),
      ).toBeNull();
      expect(container.querySelector(".rich-block, .chapter-editor")).not.toBeNull();
      expect(richButton?.disabled).toBe(false);
    } finally {
      await act(async () => root.unmount());
      container.remove();
      await window.happyDOM.close();
    }
  });
}
