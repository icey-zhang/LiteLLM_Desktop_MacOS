import { invoke } from "@tauri-apps/api/core";
import { listen } from "@tauri-apps/api/event";
import type {
  AppConfig,
  EnvironmentStatus,
  LogEntry,
  ProxyStatus,
  RuntimeStatus,
  TestRequestPayload,
  TestRequestResult,
} from "../types/config";
import { createDefaultConfig } from "./default-config";

const isTauriRuntime =
  typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;

const demoStatus: ProxyStatus = {
  state: "stopped",
  pid: null,
  port: 4000,
  endpoint: "http://127.0.0.1:4000",
  healthy: false,
  lastError: "当前运行在浏览器开发模式，未连接 Tauri 后端。",
};

const demoEnvironment: EnvironmentStatus = {
  python: {
    ok: false,
    detail: "仅在 Tauri 环境下可检测",
    hint: "请使用 `npm run tauri -- dev` 启动桌面端。",
  },
  runtime: {
    ok: false,
    detail: "仅在 Tauri 环境下可创建运行环境",
    hint: "请使用 `npm run tauri -- dev` 启动桌面端。",
  },
  overallOk: false,
};

const demoRuntimeStatus: RuntimeStatus = {
  state: "error",
  detail: "浏览器模式不会创建 LiteLLM 运行环境。",
  pythonPath: "",
  runtimeDir: "",
  version: null,
  hint: "请使用 `npm run tauri -- dev` 启动桌面端。",
};

export async function loadConfig(): Promise<AppConfig> {
  if (!isTauriRuntime) {
    return createDefaultConfig();
  }

  return invoke<AppConfig>("load_app_config");
}

export async function saveConfig(config: AppConfig): Promise<AppConfig> {
  if (!isTauriRuntime) {
    return config;
  }

  return invoke<AppConfig>("save_app_config", { config });
}

export async function checkEnvironment(): Promise<EnvironmentStatus> {
  if (!isTauriRuntime) {
    return demoEnvironment;
  }

  return invoke<EnvironmentStatus>("check_environment");
}

export async function getRuntimeStatus(): Promise<RuntimeStatus> {
  if (!isTauriRuntime) {
    return demoRuntimeStatus;
  }

  return invoke<RuntimeStatus>("get_runtime_status");
}

export async function bootstrapRuntime(): Promise<RuntimeStatus> {
  if (!isTauriRuntime) {
    return demoRuntimeStatus;
  }

  return invoke<RuntimeStatus>("bootstrap_runtime");
}

export async function getProxyStatus(): Promise<ProxyStatus> {
  if (!isTauriRuntime) {
    return demoStatus;
  }

  return invoke<ProxyStatus>("get_proxy_status");
}

export async function startProxy(): Promise<ProxyStatus> {
  if (!isTauriRuntime) {
    return {
      ...demoStatus,
      state: "error",
      lastError: "浏览器模式无法启动 LiteLLM 进程。",
    };
  }

  return invoke<ProxyStatus>("start_proxy");
}

export async function stopProxy(): Promise<ProxyStatus> {
  if (!isTauriRuntime) {
    return demoStatus;
  }

  return invoke<ProxyStatus>("stop_proxy");
}

export async function restartProxy(): Promise<ProxyStatus> {
  if (!isTauriRuntime) {
    return {
      ...demoStatus,
      state: "error",
      lastError: "浏览器模式无法重启 LiteLLM 进程。",
    };
  }

  return invoke<ProxyStatus>("restart_proxy");
}

export async function testProxyRequest(
  payload: TestRequestPayload,
): Promise<TestRequestResult> {
  if (!isTauriRuntime) {
    return {
      ok: false,
      status: null,
      durationMs: 0,
      requestPreview: JSON.stringify(payload, null, 2),
      error: "浏览器模式无法直接请求本地代理，请使用 Tauri 桌面端。",
    };
  }

  return invoke<TestRequestResult>("test_proxy_request", { payload });
}

export async function subscribeProxyLogs(
  handler: (entry: LogEntry) => void,
): Promise<() => void> {
  if (!isTauriRuntime) {
    return () => {};
  }

  const unlisten = await listen<LogEntry>("proxy-log", (event) => {
    handler(event.payload);
  });

  return () => {
    unlisten();
  };
}

export async function subscribeRuntimeStatus(
  handler: (status: RuntimeStatus) => void,
): Promise<() => void> {
  if (!isTauriRuntime) {
    return () => {};
  }

  const unlisten = await listen<RuntimeStatus>("runtime-status", (event) => {
    handler(event.payload);
  });

  return () => {
    unlisten();
  };
}
