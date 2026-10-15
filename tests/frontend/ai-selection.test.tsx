import { expect, mock, test } from "bun:test";
import { Window } from "happy-dom";
import { act, createElement } from "react";
import { createRoot } from "react-dom/client";
import type { SelectionEvent, SelectionRun, SelectionState } from "../../src/types/aiSelection";

let receive: (event: { payload: SelectionEvent }) => void = () => {};
let state: SelectionState;
let commands: { command: string; args?: Record<string, unknown> }[] = [];
let generationMode: "done" | "running" = "done";
let selectionLoads = 0;
let appendCalls = 0;
let revision = 7;
let deferStart = false;
let releaseStart: (() => void) | null = null;
let deferAppend = false;
let releaseAppend: (() => void) | null = null;
let savedDraft = "<p>Manual writing.</p>";

mock.module("@tauri-apps/api/event", () => ({ listen: async (name: string, callback: typeof receive) => {
  if (name === "ai-selection-event") receive = callback;
  return () => {};
} }));
mock.module("@tauri-apps/api/core", () => ({
  convertFileSrc: (path: string, protocol: string) => `http://${protocol}.localhost/${path}`,
  invoke: async (command: string, args?: Record<string, unknown>) => {
    commands.push({ command, args });
    if (command === "list_ai_providers") return { cli: [], custom: [] };
    if (command === "get_translation_selection") return { agent: "custom:test", model: "chosen-model", reasoningEffort: "low" };
    if (command === "get_project") return { id: "p", sourceLanguage: "en", targetLanguage: "my" };
    if (command === "get_book_settings") return { instructions: "Keep names consistent." };
    if (command === "get_ai_selection_state") { selectionLoads++; return structuredClone(state); }
    if (command === "prepare_ai_selection_run") {
      const input = args?.input as { unitIds: string[] };
      state.runs = [makeRun(input.unitIds)];
      return "run";
    }
    if (command === "control_ai_selection_run") {
      if (args?.action === "cancel") return "";
      const run = state.runs[0];
      run.state = generationMode;
      if (generationMode === "done") run.items.forEach((i) => { i.state = "ready"; i.partsDone = 1; });
      // Emit before invoke resolves: the panel must buffer early events.
      receive({ payload: event(generationMode === "done" ? "done" : "progress") });
      if (deferStart) await new Promise<void>((resolve) => { releaseStart = resolve; });
      return "job";
    }
    if (command === "append_ai_selection_results") {
      appendCalls++;
      if (deferAppend) await new Promise<void>((resolve) => { releaseAppend = resolve; });
      state.units.forEach((u) => { u.added = true; });
      return { chapterId: "c", revision: ++revision, status: "draft", html: savedDraft + '<p>မြန်မာ</p><img src="images/photo.png" alt="River"/>' };
    }
    if (command === "save_chapter_document") { savedDraft = args?.html as string; return { revision: ++revision, status: "draft" }; }
    throw new Error(`Unexpected command: ${command}`);
  },
}));

const window = new Window();
Object.assign(globalThis, {
  window, document: window.document, navigator: window.navigator, DOMParser: window.DOMParser,
  Node: window.Node, Element: window.Element, HTMLElement: window.HTMLElement,
  getComputedStyle: window.getComputedStyle.bind(window), requestAnimationFrame: window.requestAnimationFrame.bind(window),
  cancelAnimationFrame: window.cancelAnimationFrame.bind(window), IS_REACT_ACT_ENVIRONMENT: true,
});
const { AiGenerateDialog } = await import("../../src/features/translator/AiGenerateDialog");
const { useChapterEditor } = await import("../../src/features/translator/useChapterEditor");
const { bookImageUrl } = await import("../../src/features/translator/bookImages");
const content = { chapterId: "c", document: { html: "<p>Manual writing.</p>", status: "draft", revision: 7 } };
let editorApi: ReturnType<typeof useChapterEditor> | null = null;
let closed = false;
function Harness() {
  const editor = useChapterEditor(content as never);
  editorApi = editor;
  return createElement(AiGenerateDialog, { projectId: "p", chapterId: "c", editor, onClose: () => { closed = true; } });
}
function makeRun(ids = ["text", "image"]): SelectionRun {
  return { id: "run", chapterId: "c", state: "paused", message: null, budget: 4096,
    config: { agent: "custom:test", model: "chosen-model", reasoningEffort: "low", sourceLanguage: "en", targetLanguage: "my", bookInstructions: "Book guidance", currentInstructions: "Saved guidance", fingerprint: "snapshot", projectId: "p", resourceHref: "chapter.xhtml" },
    items: ids.map((id, index) => ({ id: `proposal-${id}`, unitId: id, order: index, state: "pending", partsDone: 0, partsTotal: 1, failedOutput: null,
      sourcePreviewHtml: id === "image" ? '<img src="epubres://p/images/photo.png" alt="River"/>' : "<p>Original</p>",
      previewHtml: id === "image" ? '<img src="epubres://p/images/photo.png" alt="River"/>' : "<p>မြန်မာ</p>" })) };
}
function event(kind: SelectionEvent["kind"], overrides: Partial<SelectionEvent> = {}): SelectionEvent {
  return { runId: "run", chapterId: "c", jobId: "job", itemId: "proposal-text", partId: "part", kind, message: "Progress saved", partialText: null, ...overrides };
}
function reset() {
  commands = []; appendCalls = 0; selectionLoads = 0; closed = false; revision = 7; generationMode = "done";
  deferStart = false; releaseStart = null; deferAppend = false; releaseAppend = null; savedDraft = "<p>Manual writing.</p>";
  state = { units: [
    { id: "text", order: 0, kind: "paragraph", text: "Original", previewHtml: "<p>Original</p>", unavailable: null, added: false, hasProposal: false },
    { id: "image", order: 1, kind: "image", text: "", previewHtml: '<img src="epubres://p/images/photo.png" alt="River"/>', unavailable: null, added: false, hasProposal: false },
    { id: "unsafe", order: 2, kind: "image", text: "", previewHtml: "", unavailable: "Missing image resource", added: false, hasProposal: false },
  ], runs: [] };
}
function button(container: HTMLElement, label: string) {
  const value = [...container.querySelectorAll<HTMLButtonElement>("button")].find((b) => b.textContent?.trim() === label);
  if (!value) throw new Error(`Button missing: ${label}`);
  return value;
}
async function click(container: HTMLElement, label: string) { await act(async () => button(container, label).click()); }
async function mount() {
  const container = document.createElement("div"); document.body.append(container); const root = createRoot(container);
  await act(async () => root.render(createElement(Harness)));
  return { container, root };
}

test("Whole chapter stays default; selected text and images preview before an explicit saved append", async () => {
  reset(); const { container, root } = await mount();
  try {
    expect(button(container, "Whole chapter").getAttribute("aria-pressed")).toBe("true");
    expect(selectionLoads).toBe(0);
    await click(container, "Selected paragraphs");
    expect(container.textContent).toContain("0 selected");
    expect(container.querySelector('img[alt="River"]')?.getAttribute("src")).toBe("http://epubres.localhost/p/images/photo.png");
    await click(container, "Select all available");
    expect(container.textContent).toContain("2 selected");
    expect(button(container, "Translate selected").disabled).toBe(true);
    const consent = [...container.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')].find((e) => e.parentElement?.textContent?.includes("I understand selected text"));
    await act(async () => consent?.click());
    await click(container, "Translate selected");
    expect(appendCalls).toBe(0);
    expect(editorApi?.draft?.html).toContain("Manual writing.");
    expect(editorApi?.draft?.html).not.toContain("မြန်မာ");
    expect(container.textContent).toContain("Preview results");
    await click(container, "Select ready results");
    await click(container, "Append selected results");
    expect(appendCalls).toBe(1);
    expect(editorApi?.draft?.html).toContain("Manual writing.");
    expect(editorApi?.draft?.html).toContain("မြန်မာ");
    expect(editorApi?.draft?.html).toContain('src="images/photo.png"');
    expect(editorApi?.draft?.dirty).toBe(false);
    expect(editorApi?.draft?.status).toBe("draft");
    expect(closed).toBe(false);
    const prepared = commands.find((c) => c.command === "prepare_ai_selection_run")?.args?.input as { unitIds: string[] };
    expect(prepared.unitIds.sort()).toEqual(["image", "text"]);
    const appended = commands.find((c) => c.command === "append_ai_selection_results")?.args?.input as { expectedRevision: number };
    expect(appended.expectedRevision).toBe(7);
    await click(container, "Whole chapter");
    expect(button(container, "Whole chapter").getAttribute("aria-pressed")).toBe("true");
  } finally { await act(async () => root.unmount()); container.remove(); }
});

test("active selections lock the mode and ignore stale events; close cancels the owned run", async () => {
  reset(); generationMode = "running"; const { container, root } = await mount();
  try {
    await click(container, "Selected paragraphs");
    await click(container, "Select all available");
    const consent = [...container.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')].find((e) => e.parentElement?.textContent?.includes("I understand selected text"));
    await act(async () => consent?.click()); await click(container, "Translate selected");
    expect(button(container, "Whole chapter").disabled).toBe(true);
    await act(async () => receive({ payload: event("stream", { jobId: "old-job", partialText: "STALE" }) }));
    expect(container.textContent).not.toContain("STALE");
    await act(async () => receive({ payload: event("stream", { partialText: "<p>မြန်မာ</p>" }) }));
    expect(container.querySelector<HTMLTextAreaElement>('[aria-label="Selected translation live output"]')?.value).toContain("မြန်မာ");
  } finally { await act(async () => root.unmount()); container.remove(); }
  expect(commands.some((c) => c.command === "control_ai_selection_run" && c.args?.action === "cancel")).toBe(true);
});

test("relative images use local native URLs without allowing external fetches", () => {
  expect(bookImageUrl("../images/river.png", "epubres://p/OEBPS/chapter.xhtml")).toBe("http://epubres.localhost/p/images/river.png");
  for (const value of ["https://example.com/a.png", "//example.com/a.png", "javascript:bad()", "\\remote\\a.png", "data:image/svg+xml,<svg/>"]) expect(bookImageUrl(value, "epubres://p/ch.xhtml")).toBeNull();
});

test("closing while a start command is pending cancels again after that command returns", async () => {
  reset(); generationMode = "running"; deferStart = true;
  const { container, root } = await mount();
  await click(container, "Selected paragraphs"); await click(container, "Select all available");
  const consent = [...container.querySelectorAll<HTMLInputElement>('input[type="checkbox"]')].find((e) => e.parentElement?.textContent?.includes("I understand selected text"));
  await act(async () => consent?.click()); await click(container, "Translate selected");
  expect(releaseStart).not.toBeNull();
  await act(async () => root.unmount());
  await act(async () => { releaseStart?.(); });
  expect(commands.filter((c) => c.command === "control_ai_selection_run" && c.args?.action === "cancel").length).toBe(2);
  container.remove();
});

test("append flushes pending manual edits, reports saving, and holds navigation until commit", async () => {
  reset(); state.runs = [makeRun()]; state.runs[0].state = "done"; state.runs[0].items.forEach((i) => {i.state = "ready";});
  deferAppend = true;
  const { container, root } = await mount();
  try {
    await click(container, "Selected paragraphs"); await click(container, "Select ready results");
    await act(async () => editorApi?.setHtml("<p>New manual writing.</p>"));
    await click(container, "Append selected results");
    expect(releaseAppend).not.toBeNull();
    expect(editorApi?.indicator).toBe("saving");
    expect(container.querySelector<HTMLButtonElement>('button[aria-label="Close"]')?.disabled).toBe(true);
    let flushed = false;
    const navigation = editorApi?.flushAll().then(() => { flushed = true; });
    await act(async () => { await Promise.resolve(); });
    expect(flushed).toBe(false);
    await act(async () => { releaseAppend?.(); await navigation; });
    expect(flushed).toBe(true);
    expect(editorApi?.draft?.html).toContain("New manual writing.");
    expect(editorApi?.draft?.html).toContain("မြန်မာ");
    const appended = commands.find((c) => c.command === "append_ai_selection_results")?.args?.input as { expectedRevision: number };
    expect(appended.expectedRevision).toBe(8);
    expect(editorApi?.draft?.dirty).toBe(false);
  } finally { await act(async () => root.unmount()); container.remove(); }
});
