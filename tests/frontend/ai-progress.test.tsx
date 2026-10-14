import { expect, mock, test } from "bun:test";
import { Window } from "happy-dom";
import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import type { AiHtmlFixEvent } from "../../src/types/ai";

let receive: (event: { payload: AiHtmlFixEvent }) => void = () => {};
const calls: string[] = [];
const progressEvent: AiHtmlFixEvent = {
  jobId: "job", chapterId: "chapter", kind: "stream", fixedHtml: null,
  partialText: "<p>မြန်မာ</p>", message: "",
};
mock.module("@tauri-apps/api/event", () => ({
  listen: async (_name: string, listener: typeof receive) => {
    receive = listener;
    return () => { receive = () => {}; };
  },
}));
mock.module("@tauri-apps/api/core", () => ({
  invoke: async (command: string) => {
    calls.push(command);
    if (command === "list_ai_providers") return {
      cli: [{ id: "codex", label: "Codex", binaries: ["codex"], available: true }], custom: [],
    };
    if (command === "get_translation_selection") return {
      agent: "codex", model: "example", reasoningEffort: null, updatedAt: "",
    };
    if (command === "start_ai_html_fix") {
      // Exercise progress arriving before the start command returns its job ID.
      receive({ payload: progressEvent });
      return { jobId: "job" };
    }
    if (command === "cancel_ai_translation") return;
    throw new Error(`Unexpected command: ${command}`);
  },
}));
const { AiFixHtmlDialog } = await import("../../src/features/translator/AiFixHtmlDialog");

test("repair dialog displays early streamed text in one box and keeps cancel/close usable", async () => {
  const window = new Window();
  Object.assign(globalThis, {
    window, document: window.document, navigator: window.navigator,
    HTMLElement: window.HTMLElement, Node: window.Node, Element: window.Element,
    requestAnimationFrame: window.requestAnimationFrame.bind(window),
    IS_REACT_ACT_ENVIRONMENT: true,
  });
  const container = document.createElement("div");
  document.body.append(container);
  const previous = document.createElement("button");
  document.body.append(previous);
  previous.focus();
  let closed = false;
  let applied = "";
  const root = createRoot(container);
  try {
    await act(async () => root.render(createElement(AiFixHtmlDialog, {
      chapterId: "chapter", html: "<p>original</p>",
      onApply: (html: string) => { applied = html; }, onClose: () => { closed = true; },
    })));
    expect(document.activeElement?.getAttribute("role")).toBe("dialog");
    const consent = container.querySelector<HTMLInputElement>('input[type="checkbox"]');
    expect(consent).not.toBeNull();
    await act(async () => consent?.click());
    const start = [...container.querySelectorAll<HTMLButtonElement>("button")]
      .find(button => button.textContent?.includes("Yes, fix with AI"));
    expect(start?.disabled).toBe(false);
    await act(async () => start?.click());
    const output = container.querySelector<HTMLTextAreaElement>('[aria-label="Live repair output"]');
    expect(container.querySelectorAll("textarea").length).toBe(1);
    expect(output?.value).toBe("<p>မြန်မာ</p>");
    expect(output?.readOnly).toBe(true);
    await act(async () => receive({ payload: { ...progressEvent, jobId: "stale", partialText: "ignore" } }));
    expect(output?.value).toBe("<p>မြန်မာ</p>");
    await act(async () => receive({ payload: { ...progressEvent, kind: "stream", partialText: "<p>fixed</p>" } }));
    expect(output?.value).toBe("<p>fixed</p>");
    expect(applied).toBe("");
    const cancel = [...container.querySelectorAll<HTMLButtonElement>("button")]
      .find(button => button.textContent?.trim() === "Cancel");
    await act(async () => cancel?.click());
    expect(calls).toContain("cancel_ai_translation");
    await act(async () => window.dispatchEvent(new window.KeyboardEvent("keydown", { key: "Escape", bubbles: true })));
    expect(closed).toBe(true);
  } finally {
    await act(async () => root.unmount());
    expect(document.activeElement).toBe(previous);
    container.remove();
    previous.remove();
    await window.happyDOM.close();
  }
});
