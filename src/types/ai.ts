/** AI provider settings DTOs from the Rust settings service. */

export type CliProviderStatus = {
  id: string;
  label: string;
  binaries: string[];
  available: boolean;
};

export type CliCheckResult = {
  id: string;
  label: string;
  available: boolean;
  connected: boolean;
  version: string | null;
  message: string;
};

export type CustomProviderSummary = {
  id: string;
  name: string;
  baseUrl: string;
  model: string;
  hasApiKey: boolean;
  createdAt: string;
  updatedAt: string;
};

export type AiProviderList = {
  cli: CliProviderStatus[];
  custom: CustomProviderSummary[];
};

export type CustomTestResult = {
  ok: boolean;
  modelCount: number | null;
  message: string;
};
