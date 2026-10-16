import { expect, mock, test } from "bun:test";
import { Window } from "happy-dom";
import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import type { ProjectSummary } from "../../src/types/project";

const project: ProjectSummary = {
  id: "language-test",
  title: "Sample",
  author: null,
  sourceLanguage: "en-GB",
  targetLanguage: "x-local",
  reviewedBlocks: 0,
  totalBlocks: 0,
  unsupportedBlocks: 0,
  createdAt: "",
  updatedAt: "",
};
const saves: Record<string, unknown>[] = [];
let fail = true;
mock.module("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string) => path,
  invoke: async (command: string, args: Record<string, unknown>) => {
    if (command === "get_project") return project;
    if (command === "update_project_metadata") {
      saves.push(args);
      if (fail) {
        fail = false;
        throw { code: "save_failed", message: "Try again." };
      }
      return;
    }
    throw new Error(command);
  },
}));
const { LanguageDialog } = await import("../../src/features/workspace/LanguageDialog");

test("named choices preserve custom tags and unknown sources through save retry", async () => {
  const window = new Window();
  Object.assign(globalThis, {
    window,
    document: window.document,
    navigator: window.navigator,
    HTMLElement: window.HTMLElement,
    Node: window.Node,
    Element: window.Element,
    IS_REACT_ACT_ENVIRONMENT: true,
  });
  const trigger = document.createElement("button");
  document.body.append(trigger);
  trigger.focus();
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  let closes = 0;
  try {
    await act(async () =>
      root.render(
        createElement(LanguageDialog, {
          project,
          onClose: () => {
            closes++;
          },
          onSaved: () => {},
        }),
      ),
    );
    const source = container.querySelector<HTMLSelectElement>("#language-source");
    const target = container.querySelector<HTMLSelectElement>("#language-target");
    if (!source || !target) throw new Error("Language choices missing");
    expect(source.selectedOptions[0].textContent).toContain("English");
    expect(target.value).toBe("custom");
    expect(container.querySelector("input")?.value).toBe("x-local");
    await act(async () => {
      source.value = "";
      source.dispatchEvent(new window.Event("change", { bubbles: true }));
    });
    await act(async () => {
      target.value = "ja";
      target.dispatchEvent(new window.Event("change", { bubbles: true }));
    });
    expect(container.querySelector("input")).toBeNull();
    const submit = () =>
      container
        .querySelector("form")
        ?.dispatchEvent(new window.Event("submit", { bubbles: true, cancelable: true }));
    await act(async () => submit());
    expect(saves[0]).toMatchObject({ sourceLanguage: "", targetLanguage: "ja" });
    expect(container.querySelector('[role="alert"]')?.textContent).toContain(
      "Try again.",
    );
    expect(target.value).toBe("ja");
    expect(closes).toBe(0);
    await act(async () => submit());
    expect(saves[1]).toEqual(saves[0]);
    expect(closes).toBe(1);
  } finally {
    await act(async () => root.unmount());
    expect(document.activeElement).toBe(trigger);
    container.remove();
    trigger.remove();
  }
});
