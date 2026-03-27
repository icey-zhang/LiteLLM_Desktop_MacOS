import { sounds } from "../lib/sounds";
import type {
  EnvironmentStatus,
  ProxyStatus,
  RuntimeStatus,
} from "../types/config";

interface OverviewPanelProps {
  environment: EnvironmentStatus | null;
  runtimeStatus: RuntimeStatus | null;
  proxyStatus: ProxyStatus;
  onStart: () => Promise<void>;
  onStop: () => Promise<void>;
  onRestart: () => Promise<void>;
  onRetestEnvironment: () => Promise<void>;
  onBootstrapRuntime: () => Promise<void>;
}

function stateLabel(state: ProxyStatus["state"]) {
  switch (state) {
    case "running":
      return "运行中";
    case "starting":
      return "启动中";
    case "error":
      return "异常";
    default:
      return "未启动";
  }
}

export function OverviewPanel({
  environment,
  runtimeStatus,
  proxyStatus,
  onStart,
  onStop,
  onRestart,
  onRetestEnvironment,
  onBootstrapRuntime,
}: OverviewPanelProps) {
  const handleStart = async () => {
    sounds.playClick();
    await onStart();
    sounds.playSuccess();
  };

  const handleStop = async () => {
    sounds.playClick();
    await onStop();
    sounds.playSwitch();
  };

  const handleRestart = async () => {
    sounds.playClick();
    await onRestart();
    sounds.playSuccess();
  };

  const handleRetest = async () => {
    sounds.playClick();
    await onRetestEnvironment();
    sounds.playSuccess();
  };

  const handleRepairRuntime = async () => {
    sounds.playClick();
    await onBootstrapRuntime();
    sounds.playSuccess();
  };

  return (
    <section className="panel-stack">
      <header className="panel-header">
        <div>
          <p className="eyebrow">概览</p>
          <h2>状态与环境</h2>
        </div>
        <div className="action-row">
          <button className="btn btn-secondary" onClick={() => void handleRetest()}>
            重新检测环境
          </button>
          <button className="btn btn-secondary" onClick={() => void handleRepairRuntime()}>
            修复运行环境
          </button>
        </div>
      </header>

      <div className="status-grid">
        <article className="status-card">
          <p className="label">Python</p>
          <strong>{environment?.python.ok ? "已就绪" : "未就绪"}</strong>
          <p>{environment?.python.detail ?? "等待检测"}</p>
        </article>

        <article className="status-card">
          <p className="label">运行环境</p>
          <strong>
            {runtimeStatus?.state === "ready"
              ? "已就绪"
              : runtimeStatus?.state === "installing"
                ? "安装中"
                : runtimeStatus?.state === "repairing"
                  ? "修复中"
                  : runtimeStatus?.state === "checking"
                    ? "检查中"
                    : "未就绪"}
          </strong>
          <p>{runtimeStatus?.detail ?? "等待检测"}</p>
        </article>

        <article className="status-card">
          <p className="label">代理状态</p>
          <strong>{stateLabel(proxyStatus.state)}</strong>
          <p>{proxyStatus.endpoint}</p>
        </article>
      </div>

      <div className="action-row">
        <button
          className="btn btn-primary"
          onClick={() => void handleStart()}
          disabled={runtimeStatus?.state !== "ready"}
        >
          启动代理
        </button>
        <button
          className="btn btn-secondary"
          onClick={() => void handleRestart()}
          disabled={runtimeStatus?.state !== "ready"}
        >
          重启代理
        </button>
        <button className="btn btn-outline-danger" onClick={() => void handleStop()}>
          停止代理
        </button>
      </div>
    </section>
  );
}
