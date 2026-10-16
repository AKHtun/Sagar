import { invoke } from "@tauri-apps/api/core";
import type { AppendSelectionResult, SelectionState } from "../../types/aiSelection";
import { toCommandError } from "./errors";

async function command<T>(name: string, args: Record<string, unknown>): Promise<T> {
  try {
    return await invoke<T>(name, args);
  } catch (error) {
    throw toCommandError(error);
  }
}

export function getAiSelectionState(chapterId: string): Promise<SelectionState> {
  return command("get_ai_selection_state", { chapterId });
}

export function prepareAiSelectionRun(input: {
  chapterId: string;
  unitIds: string[];
  agent: string;
  model: string;
  reasoningEffort: string | null;
  currentInstructions: string;
}): Promise<string> {
  return command("prepare_ai_selection_run", { input });
}

export function controlAiSelectionRun(
  runId: string,
  action: "start" | "retry" | "smaller" | "skip" | "cancel",
  consent: boolean,
): Promise<string> {
  return command("control_ai_selection_run", { runId, action, consent });
}

export type AppendSelectionInput = {
  chapterId: string;
  proposalIds: string[];
  expectedRevision: number;
  requestId: string;
  allowAgain: boolean;
  confirmOutOfOrder: boolean;
};

export function appendAiSelectionResults(
  input: AppendSelectionInput,
): Promise<AppendSelectionResult> {
  return command("append_ai_selection_results", { input });
}

export function deleteAiSelectionRun(chapterId: string, runId: string): Promise<void> {
  return command("delete_ai_selection_run", { chapterId, runId });
}
