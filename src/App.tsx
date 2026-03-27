import { useEffect, useMemo, useState } from "react";
import { Shell } from "./components/shell";
import { sounds } from "./lib/sounds";
import {
  bootstrapRuntime,
  checkEnvironment,
  getRuntimeStatus,
  getProxyStatus,
  loadConfig,
  restartProxy,
  saveConfig,
  startProxy,
  stopProxy,
  subscribeProxyLogs,
  subscribeRuntimeStatus,
  testProxyRequest,
} from "./lib/api";
import { createDefaultConfig } from "./lib/default-config";
import type {
  AppConfig,
  EnvironmentStatus,
  LogEntry,
  ProxyStatus,
  RuntimeStatus,
  TestRequestResult,
} from "./types/config";

const fallbackStatus: ProxyStatus = {
  state: "stopped",
  pid: null,
  port: 4000,
  endpoint: "http://127.0.0.1:4000",
  healthy: false,
  lastError: null,
};

export default function App() {
  const [config, setConfig] = useState<AppConfig>(createDefaultConfig());
  const [environment, setEnvironment] = useState<EnvironmentStatus | null>(null);
  const [runtimeStatus, setRuntimeStatus] = useState<RuntimeStatus | null>(null);
  const [proxyStatus, setProxyStatus] = useState<ProxyStatus>(fallbackStatus);
  const [logs, setLogs] = useState<LogEntry[]>([]);
  const [testResult, setTestResult] = useState<TestRequestResult | null>(null);
  const [saveState, setSaveState] = useState<"idle" | "saving" | "saved" | "error">("idle");

  useEffect(() => {
    let alive = true;
    void (async () => {
      try {
        const nextConfig = await loadConfig();
        if (!alive) {
          return;
        }
        setConfig(nextConfig);
        setEnvironment(await checkEnvironment());
        const detectedRuntime = await getRuntimeStatus();
        if (!alive) {
          return;
        }
        setRuntimeStatus(detectedRuntime);
        if (detectedRuntime.state !== "ready") {
          setRuntimeStatus({
            ...detectedRuntime,
            state: detectedRuntime.state === "error" ? "repairing" : "installing",
          });
          const bootstrappedRuntime = await bootstrapRuntime();
          if (!alive) {
            return;
          }
          setRuntimeStatus(bootstrappedRuntime);
          const refreshedConfig = await loadConfig();
          if (!alive) {
            return;
          }
          setConfig(refreshedConfig);
          setEnvironment(await checkEnvironment());
        }
        setProxyStatus(await getProxyStatus());
      } catch (e) {
        console.error("Initialization failed", e);
      }
    })();
    return () => {
      alive = false;
    };
  }, []);

  useEffect(() => {
    let alive = true;

    void subscribeProxyLogs((entry) => {
      if (!alive) {
        return;
      }

      setLogs((current) => [...current.slice(-499), entry]);
    }).then((unsubscribe) => {
      if (!alive) {
        unsubscribe();
      }
    });

    void subscribeRuntimeStatus((status) => {
      if (!alive) {
        return;
      }

      setRuntimeStatus(status);
    }).then((unsubscribe) => {
      if (!alive) {
        unsubscribe();
      }
    });

    const interval = window.setInterval(() => {
      void getProxyStatus().then((status) => setProxyStatus(status));
    }, 2000);

    return () => {
      alive = false;
      window.clearInterval(interval);
    };
  }, []);

  const availableModels = useMemo(
    () => config.providerGroups.flatMap((group) => group.models),
    [config.providerGroups],
  );

  async function handleSave() {
    setSaveState("saving");
    try {
      const saved = await saveConfig(config);
      setConfig(saved);
      setSaveState("saved");
      setEnvironment(await checkEnvironment());
      setTimeout(() => setSaveState("idle"), 2000);
    } catch (error) {
      setSaveState("error");
      sounds.playError();
      setLogs((current) => [
        ...current,
        {
          stream: "system",
          line: error instanceof Error ? error.message : "保存配置失败",
          timestamp: new Date().toISOString(),
        },
      ]);
    }
  }

  async function handleStart() {
    if (runtimeStatus?.state !== "ready") {
      const nextRuntime = await bootstrapRuntime();
      setRuntimeStatus(nextRuntime);
      if (nextRuntime.state !== "ready") {
        sounds.playError();
        return;
      }
    }
    const status = await startProxy();
    setProxyStatus(status);
  }

  async function handleStop() {
    const status = await stopProxy();
    setProxyStatus(status);
  }

  async function handleRestart() {
    const status = await restartProxy();
    setProxyStatus(status);
  }

  async function handleRetestEnvironment() {
    setEnvironment(await checkEnvironment());
    setRuntimeStatus(await getRuntimeStatus());
  }

  async function handleBootstrapRuntime() {
    setRuntimeStatus((current) =>
      current
        ? {
            ...current,
            state: current.state === "ready" ? "repairing" : "installing",
          }
        : null,
    );
    const nextRuntime = await bootstrapRuntime();
    setRuntimeStatus(nextRuntime);
    setEnvironment(await checkEnvironment());
    setConfig(await loadConfig());
  }

  async function handleRunTest(
    model: string,
    systemPrompt: string,
    userMessage: string,
  ) {
    if (availableModels.length === 0) {
      setTestResult({
        ok: false,
        status: null,
        durationMs: 0,
        requestPreview: "",
        error: "请先在配置页配置至少一个模型。",
      });
      sounds.playError();
      return;
    }

    const result = await testProxyRequest({
      port: config.settings.port,
      masterKey: config.settings.masterKey,
      model,
      systemPrompt,
      userMessage,
    });
    setTestResult(result);
  }

  return (
    <Shell
      config={config}
      environment={environment}
      runtimeStatus={runtimeStatus}
      proxyStatus={proxyStatus}
      logs={logs}
      testResult={testResult}
      saveState={saveState}
      onConfigChange={setConfig}
      onSave={handleSave}
      onStart={handleStart}
      onStop={handleStop}
      onRestart={handleRestart}
      onRetestEnvironment={handleRetestEnvironment}
      onBootstrapRuntime={handleBootstrapRuntime}
      onClearLogs={() => setLogs([])}
      onRunTest={handleRunTest}
    />
  );
}
