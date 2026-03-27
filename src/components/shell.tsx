import { useMemo, useState } from "react";
import { sounds } from "../lib/sounds";
import type {
  AppConfig,
  EnvironmentStatus,
  LogEntry,
  ProxyStatus,
  RuntimeStatus,
  TestRequestResult,
} from "../types/config";
import { ConfigPanel } from "./config-panel";
import { LogsPanel } from "./logs-panel";
import { OverviewPanel } from "./overview-panel";
import { TestPanel } from "./test-panel";

type TabId = "overview" | "config" | "logs" | "tester";

interface ShellProps {
  config: AppConfig;
  environment: EnvironmentStatus | null;
  runtimeStatus: RuntimeStatus | null;
  proxyStatus: ProxyStatus;
  logs: LogEntry[];
  testResult: TestRequestResult | null;
  saveState: "idle" | "saving" | "saved" | "error";
  onConfigChange: (config: AppConfig) => void;
  onSave: () => Promise<void>;
  onStart: () => Promise<void>;
  onStop: () => Promise<void>;
  onRestart: () => Promise<void>;
  onRetestEnvironment: () => Promise<void>;
  onBootstrapRuntime: () => Promise<void>;
  onClearLogs: () => void;
  onRunTest: (
    model: string,
    systemPrompt: string,
    userMessage: string,
  ) => Promise<void>;
}

const tabs: Array<{ id: TabId; label: string; kicker: string }> = [
  { id: "overview", label: "概览", kicker: "状态面板" },
  { id: "config", label: "配置", kicker: "模型与凭据" },
  { id: "logs", label: "日志", kicker: "实时输出" },
  { id: "tester", label: "请求测试", kicker: "验证代理" },
];

export function Shell(props: ShellProps) {
  const [activeTab, setActiveTab] = useState<TabId>("overview");
  const headline = useMemo(
    () =>
      props.proxyStatus.state === "running"
        ? "本机代理在线"
        : "把 LiteLLM 变成桌面可控服务",
    [props.proxyStatus.state],
  );
  const availableModels = useMemo(
    () => props.config.providerGroups.flatMap((group) => group.models),
    [props.config.providerGroups],
  );

  const handleTabChange = (id: TabId) => {
    if (id !== activeTab) {
      sounds.playSwitch();
      setActiveTab(id);
    }
  };

  return (
    <div className="shell">
      <aside className="sidebar">
        <div className="brand-card">
          <p className="brand-kicker">LiteLLM Desktop</p>
          <h1>{headline}</h1>
          <p className="brand-copy">
            用一个本地窗口管理配置、代理进程、日志和测试请求，不再手工切命令行。
          </p>
        </div>

        <nav className="tab-list" aria-label="主导航">
          {tabs.map((tab) => (
            <button
              key={tab.id}
              className={tab.id === activeTab ? "tab-button active" : "tab-button"}
              onClick={() => handleTabChange(tab.id)}
              type="button"
            >
              <span>{tab.label}</span>
              <small>{tab.kicker}</small>
            </button>
          ))}
        </nav>
      </aside>

      <main className="content">
        {activeTab === "overview" ? (
          <OverviewPanel
            environment={props.environment}
            runtimeStatus={props.runtimeStatus}
            proxyStatus={props.proxyStatus}
            onStart={props.onStart}
            onStop={props.onStop}
            onRestart={props.onRestart}
            onRetestEnvironment={props.onRetestEnvironment}
            onBootstrapRuntime={props.onBootstrapRuntime}
          />
        ) : null}

        {activeTab === "config" ? (
          <ConfigPanel
            config={props.config}
            runtimePythonPath={props.runtimeStatus?.pythonPath ?? props.config.settings.pythonPath}
            saveState={props.saveState}
            onChange={props.onConfigChange}
            onSave={props.onSave}
          />
        ) : null}

        {activeTab === "logs" ? (
          <LogsPanel logs={props.logs} onClear={props.onClearLogs} />
        ) : null}

        {activeTab === "tester" ? (
          <TestPanel
            models={availableModels}
            port={props.config.settings.port}
            masterKey={props.config.settings.masterKey}
            result={props.testResult}
            onRunTest={props.onRunTest}
          />
        ) : null}
      </main>
    </div>
  );
}
