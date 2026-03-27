import { useEffect, useState } from "react";
import { sounds } from "../lib/sounds";
import type { ModelEntry, TestRequestResult } from "../types/config";

interface TestPanelProps {
  models: ModelEntry[];
  port: number;
  masterKey: string;
  result: TestRequestResult | null;
  onRunTest: (
    model: string,
    systemPrompt: string,
    userMessage: string,
  ) => Promise<void>;
}

export function TestPanel({
  models,
  port,
  masterKey,
  result,
  onRunTest,
}: TestPanelProps) {
  const [selectedModel, setSelectedModel] = useState(models[0]?.alias ?? "");
  const [systemPrompt, setSystemPrompt] = useState("你是一个本地连通性测试助手。");
  const [userMessage, setUserMessage] = useState("请回答：LiteLLM 代理已连接。");
  const [loading, setLoading] = useState(false);

  useEffect(() => {
    if (models.length === 0) {
      setSelectedModel("");
    } else if (!models.some((model) => model.alias === selectedModel)) {
      setSelectedModel(models[0].alias);
    }
  }, [models, selectedModel]);

  const handleRunTest = async () => {
    sounds.playClick();
    setLoading(true);
    try {
      await onRunTest(selectedModel, systemPrompt, userMessage);
      sounds.playSuccess();
    } catch (e) {
      sounds.playError();
    } finally {
      setLoading(false);
    }
  };

  return (
    <section className="panel-stack">
      <header className="panel-header">
        <div>
          <p className="eyebrow">验证</p>
          <h2>发送测试请求</h2>
        </div>
        <p className="support-copy">
          目标: <strong>127.0.0.1:{port}</strong>
        </p>
      </header>

      <div className="form-grid">
        <label className="field">
          <span>测试模型</span>
          <select
            value={selectedModel}
            onChange={(event) => setSelectedModel(event.target.value)}
          >
            {models.map((model) => (
              <option key={model.id} value={model.alias}>
                {model.alias}
              </option>
            ))}
          </select>
        </label>

        <div className="field"></div>

        <label className="field">
          <span>System Prompt</span>
          <textarea
            value={systemPrompt}
            onChange={(event) => setSystemPrompt(event.target.value)}
          />
        </label>

        <label className="field">
          <span>User Message</span>
          <textarea
            value={userMessage}
            onChange={(event) => setUserMessage(event.target.value)}
          />
        </label>
      </div>

      <div className="action-row">
        <button
          className="btn btn-primary"
          onClick={() => void handleRunTest()}
          disabled={models.length === 0 || loading}
        >
          {loading ? "发送中..." : "发送测试请求"}
        </button>
      </div>

      <section className="result-card">
        <div className="card-header">
          <h4>响应结果</h4>
          {result && (
            <span className={`badge ${result.ok ? "success" : "error"}`}>
              {result.ok ? "SUCCESS" : "ERROR"}
            </span>
          )}
        </div>

        {result ? (
          <div className="panel-stack" style={{ gap: "1rem" }}>
            <div className="status-grid">
              <div className="status-card">
                <span className="label">HTTP</span>
                <strong>{result.status ?? "---"}</strong>
              </div>
              <div className="status-card">
                <span className="label">LATENCY</span>
                <strong>{result.durationMs}ms</strong>
              </div>
            </div>

            {result.error && <p className="badge error">{result.error}</p>}
            {result.responseText && <pre>{result.responseText}</pre>}
            {result.responseJson && <pre>{result.responseJson}</pre>}
          </div>
        ) : (
          <p className="support-copy">尚未发送请求。</p>
        )}
      </section>
    </section>
  );
}
