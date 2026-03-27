export type ProxyState = "stopped" | "starting" | "running" | "error";
export type RuntimeState =
  | "unknown"
  | "checking"
  | "installing"
  | "ready"
  | "repairing"
  | "error";

export interface AppSettings {
  pythonPath: string;
  port: number;
  masterKey: string;
  autoStartProxy: boolean;
}

export interface ProviderPreset {
  id: string;
  provider: string;
  apiKey: string;
  apiBase: string;
  models: ModelEntry[];
}

export interface ModelEntry {
  id: string;
  alias: string;
  litellmModel: string;
  extraParams: Record<string, string | number | boolean>;
}

export interface AppConfig {
  settings: AppSettings;
  providerGroups: ProviderPreset[];
}

export interface DiagnosticCheck {
  ok: boolean;
  detail: string;
  hint?: string;
}

export interface EnvironmentStatus {
  python: DiagnosticCheck;
  runtime: DiagnosticCheck;
  overallOk: boolean;
}

export interface RuntimeStatus {
  state: RuntimeState;
  detail: string;
  pythonPath: string;
  runtimeDir: string;
  version?: string | null;
  hint?: string | null;
}

export interface ProxyStatus {
  state: ProxyState;
  pid: number | null;
  port: number;
  endpoint: string;
  healthy: boolean;
  lastError?: string | null;
}

export interface LogEntry {
  stream: "stdout" | "stderr" | "system";
  line: string;
  timestamp: string;
}

export interface TestRequestPayload {
  port: number;
  masterKey: string;
  model: string;
  systemPrompt: string;
  userMessage: string;
}

export interface TestRequestResult {
  ok: boolean;
  status: number | null;
  durationMs: number;
  requestPreview: string;
  responseText?: string;
  responseJson?: string;
  error?: string;
}
