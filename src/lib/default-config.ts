import type { AppConfig, ModelEntry, ProviderPreset } from "../types/config";

function createId(prefix: string): string {
  return `${prefix}-${Math.random().toString(36).slice(2, 10)}`;
}

export function createProviderPreset(): ProviderPreset {
  return {
    id: createId("provider"),
    provider: "openai",
    apiKey: "",
    apiBase: "",
    models: [createModelEntry()],
  };
}

export function createModelEntry(): ModelEntry {
  return {
    id: createId("model"),
    alias: "gpt-4o-mini",
    litellmModel: "openai/gpt-4o-mini",
    extraParams: {},
  };
}

export function createDefaultConfig(): AppConfig {
  return {
    settings: {
      pythonPath: "python3",
      port: 4000,
      masterKey: "",
      autoStartProxy: false,
    },
    providerGroups: [createProviderPreset()],
  };
}
