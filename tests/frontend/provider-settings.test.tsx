import { expect, mock, test } from "bun:test";
import { Window } from "happy-dom";
import { act, createElement } from "react";
import { createRoot } from "react-dom/client";

const calls: string[] = [];
let custom = [
  {
    id: "sample",
    name: "Sample provider",
    protocol: "gemini",
    baseUrl: "https://example.test/v1",
    model: "sample-model",
    hasApiKey: true,
    createdAt: "",
    updatedAt: "",
  },
];
mock.module("@tauri-apps/api/core", () => ({
  invoke: async (command: string) => {
    calls.push(command);
    if (command === "list_ai_providers")
      return {
        cli: [
          { id: "cli", label: "Test assistant", binaries: ["test-cli"], available: true },
        ],
        custom,
      };
    if (command === "test_ai_custom_provider")
      return {
        ok: false,
        modelCount: null,
        message: "Connection failed. Check the URL.",
      };
    if (command === "delete_ai_custom_provider") {
      custom = [];
      return;
    }
    throw new Error(command);
  },
}));
const { AiSettingsDialog } = await import("../../src/features/settings/AiSettingsDialog");

test("provider settings hide technical detail, expose failures and require deletion confirmation", async () => {
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
  const container = document.createElement("div");
  document.body.append(container);
  const root = createRoot(container);
  const button = (name: string) => {
    const found = Array.from(
      container.querySelectorAll<HTMLButtonElement>("button"),
    ).find(
      (b) => b.getAttribute("aria-label") === name || b.textContent?.trim() === name,
    );
    if (!found) throw new Error(`Missing button: ${name}`);
    return found;
  };
  try {
    await act(async () =>
      root.render(
        createElement(AiSettingsDialog, {
          onClose: () => {},
          appearance: {
            theme: "light",
            textSize: 18,
            onThemeChange: () => {},
            onTextSizeChange: () => {},
          },
        }),
      ),
    );
    await act(async () => button("Providers").click());
    expect(container.querySelector(".notice--info")).toBeNull();
    const details = button("Connection details · Sample provider");
    expect(details.getAttribute("aria-expanded")).toBe("false");
    expect(
      container.querySelector("#provider-details-sample")?.hasAttribute("hidden"),
    ).toBe(true);
    await act(async () => details.click());
    expect(details.getAttribute("aria-expanded")).toBe("true");
    expect(container.querySelector("#provider-details-sample")?.textContent).toContain(
      "https://example.test/v1",
    );
    await act(async () => button("Test Sample provider").click());
    expect(container.querySelector('[role="alert"]')?.textContent).toBe(
      "Connection failed. Check the URL.",
    );
    await act(async () => button("Delete Sample provider").click());
    expect(calls).not.toContain("delete_ai_custom_provider");
    await act(async () => button("Cancel").click());
    expect(container.querySelector(".provider-row__confirm")).toBeNull();
    await act(async () => button("Delete Sample provider").click());
    await act(async () => button("Confirm delete").click());
    expect(calls).toContain("delete_ai_custom_provider");
    expect(container.querySelector(".provider-list")?.textContent).toContain(
      "Test assistant",
    );
    expect(container.textContent).toContain("No custom providers yet.");
  } finally {
    await act(async () => root.unmount());
    container.remove();
  }
});
