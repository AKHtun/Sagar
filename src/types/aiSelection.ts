export type SelectionUnit = {
  id: string;
  order: number;
  kind: string;
  text: string;
  previewHtml: string;
  unavailable: string | null;
  added: boolean;
  hasProposal: boolean;
};

export type SelectionItem = {
  id: string;
  unitId: string;
  order: number;
  state:
    | "pending"
    | "translating"
    | "ready"
    | "failed"
    | "skipped"
    | "added"
    | "superseded";
  previewHtml: string | null;
  sourcePreviewHtml: string;
  failedOutput: string | null;
  partsDone: number;
  partsTotal: number;
};

export type SelectionRun = {
  id: string;
  chapterId: string;
  state: "paused" | "running" | "done" | "failed";
  message: string | null;
  budget: number;
  config: {
    agent: string;
    model: string;
    reasoningEffort: string | null;
    sourceLanguage: string | null;
    targetLanguage: string;
    bookInstructions: string;
    currentInstructions: string;
    fingerprint: string;
    projectId: string;
    resourceHref: string;
  };
  items: SelectionItem[];
};

export type SelectionState = { units: SelectionUnit[]; runs: SelectionRun[] };
export type SelectionEvent = {
  runId: string;
  jobId: string;
  chapterId: string;
  itemId: string | null;
  partId: string | null;
  kind: "progress" | "stream" | "status" | "checkpoint" | "done" | "failed" | "paused";
  message: string;
  partialText: string | null;
};
export type AppendSelectionResult = {
  chapterId: string;
  html: string | null;
  revision: number;
  status: "draft";
};
