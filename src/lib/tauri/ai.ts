import { invoke } from "@tauri-apps/api/core";
import type {
  AiProviderList,
  CliCheckResult,
  CustomProviderSummary,
  CustomTestResult,
} from "../../types/ai";
import { toCommandError } from "./errors";

/** Typed wrappers for the AI provider settings commands (settings only). */

export async function listAiProviders(): Promise<AiProviderList> {
  try {
    return await invoke<AiProviderList>("list_ai_providers");
  } catch (error) {
    throw toCommandError(error);
  }
}

export async function checkAiCli(providerId: string): Promise<CliCheckResult> {
  try {
    return await invoke<CliCheckResult>("check_ai_cli", { providerId });
  } catch (error) {
    throw toCommandError(error);
  }
}

export async function saveAiCustomProvider(input: {
  id?: string;
  name: string;
  baseUrl: string;
  apiKey?: string | null;
  model: string;
}): Promise<CustomProviderSummary> {
  try {
    return await invoke<CustomProviderSummary>("save_ai_custom_provider", {
      id: input.id ?? null,
      name: input.name,
      baseUrl: input.baseUrl,
      apiKey: input.apiKey ?? null,
      model: input.model,
    });
  } catch (error) {
    throw toCommandError(error);
  }
}

export async function deleteAiCustomProvider(id: string): Promise<void> {
  try {
    await invoke("delete_ai_custom_provider", { id });
  } catch (error) {
    throw toCommandError(error);
  }
}

export async function testAiCustomProvider(id: string): Promise<CustomTestResult> {
  try {
    return await invoke<CustomTestResult>("test_ai_custom_provider", { id });
  } catch (error) {
    throw toCommandError(error);
  }
}
