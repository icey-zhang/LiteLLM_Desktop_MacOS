import { sounds } from "../lib/sounds";
import { createModelEntry, createProviderPreset } from "../lib/default-config";
import type { AppConfig, ModelEntry, ProviderPreset } from "../types/config";

interface ConfigPanelProps {
  config: AppConfig;
  runtimePythonPath: string;
  saveState: "idle" | "saving" | "saved" | "error";
  onChange: (config: AppConfig) => void;
  onSave: () => Promise<void>;
}

function updateProvider(
  providers: ProviderPreset[],
  providerId: string,
  key: keyof ProviderPreset,
  value: string,
) {
  return providers.map((provider) =>
    provider.id === providerId ? { ...provider, [key]: value } : provider,
  );
}

function updateModel(
  models: ModelEntry[],
  modelId: string,
  key: keyof ModelEntry,
  value: string,
) {
  return models.map((model) =>
    model.id === modelId ? { ...model, [key]: value } : model,
  );
}

export function ConfigPanel({
  config,
  runtimePythonPath,
  saveState,
  onChange,
  onSave,
}: ConfigPanelProps) {
  const handleSave = async () => {
    sounds.playClick();
    try {
      await onSave();
      sounds.playSuccess();
    } catch (e) {
      sounds.playError();
    }
  };

  const handleAddProvider = () => {
    sounds.playSwitch();
    onChange({
      ...config,
      providerGroups: [...config.providerGroups, createProviderPreset()],
    });
  };

  const handleAddModel = (providerId: string) => {
    sounds.playSwitch();
    onChange({
      ...config,
      providerGroups: config.providerGroups.map((provider) =>
        provider.id === providerId
          ? { ...provider, models: [...provider.models, createModelEntry()] }
          : provider,
      ),
    });
  };

  const handleDeleteModel = (providerId: string, modelId: string) => {
    sounds.playSwitch();
    onChange({
      ...config,
      providerGroups: config.providerGroups.map((provider) =>
        provider.id === providerId
          ? {
              ...provider,
              models: provider.models.filter((entry) => entry.id !== modelId),
            }
          : provider,
      ),
    });
  };

  const handleDeleteProvider = (providerId: string) => {
    sounds.playSwitch();
    onChange({
      ...config,
      providerGroups: config.providerGroups.filter(
        (provider) => provider.id !== providerId,
      ),
    });
  };

  return (
    <section className="panel-stack">
      <header className="panel-header">
        <div>
          <p className="eyebrow">配置</p>
          <h2>全局设置</h2>
        </div>
        <button
          className="btn btn-primary"
          onClick={() => void handleSave()}
          disabled={saveState === "saving"}
        >
          {saveState === "saving" ? "保存中..." : "保存配置"}
        </button>
      </header>

      <div className="form-grid">
        <label className="field">
          <span>运行环境 Python</span>
          <input readOnly value={runtimePythonPath || "等待环境就绪"} />
        </label>

        <label className="field">
          <span>代理端口</span>
          <input
            type="number"
            value={config.settings.port}
            onChange={(event) =>
              onChange({
                ...config,
                settings: {
                  ...config.settings,
                  port: Number(event.target.value) || 4000,
                },
              })
            }
          />
        </label>
      </div>

      <header className="panel-header">
        <div>
          <p className="eyebrow">管理</p>
          <h2>Provider 与模型</h2>
        </div>
        <button className="btn btn-secondary" onClick={handleAddProvider}>
          + 新增 Provider 组
        </button>
      </header>

      <div className="stack-list">
        {config.providerGroups.map((provider) => (
          <div className="provider-group-card" key={provider.id}>
            <div className="card-header">
              <h4>{provider.provider || "未命名 Provider"}</h4>
              <button
                className="btn btn-danger"
                onClick={() => handleDeleteProvider(provider.id)}
              >
                删除
              </button>
            </div>

            <div className="form-grid">
              <label className="field">
                <span>名称 (openai, azure...)</span>
                <input
                  placeholder="openai"
                  value={provider.provider}
                  onChange={(event) =>
                    onChange({
                      ...config,
                      providerGroups: updateProvider(
                        config.providerGroups,
                        provider.id,
                        "provider",
                        event.target.value,
                      ),
                    })
                  }
                />
              </label>
              <label className="field">
                <span>API Key</span>
                <input
                  type="password"
                  placeholder="sk-..."
                  value={provider.apiKey}
                  onChange={(event) =>
                    onChange({
                      ...config,
                      providerGroups: updateProvider(
                        config.providerGroups,
                        provider.id,
                        "apiKey",
                        event.target.value,
                      ),
                    })
                  }
                />
              </label>
              <label className="field field-wide">
                <span>API Base (URL)</span>
                <input
                  placeholder="https://api.openai.com/v1"
                  value={provider.apiBase}
                  onChange={(event) =>
                    onChange({
                      ...config,
                      providerGroups: updateProvider(
                        config.providerGroups,
                        provider.id,
                        "apiBase",
                        event.target.value,
                      ),
                    })
                  }
                />
              </label>
            </div>
            <section className="nested-models">
              <div className="card-header">
                <span className="eyebrow">模型列表</span>
                <button
                  className="btn btn-secondary"
                  onClick={() => handleAddModel(provider.id)}
                >
                  + 新增模型
                </button>
              </div>

              {provider.models.map((model) => (
                <div className="model-card nested" key={model.id}>
                  <div className="field">
                    <span>别名</span>
                    <input
                      placeholder="gpt-4"
                      value={model.alias}
                      onChange={(event) =>
                        onChange({
                          ...config,
                          providerGroups: config.providerGroups.map((entry) =>
                            entry.id === provider.id
                              ? {
                                  ...entry,
                                  models: updateModel(
                                    entry.models,
                                    model.id,
                                    "alias",
                                    event.target.value,
                                  ),
                                }
                              : entry,
                          ),
                        })
                      }
                    />
                  </div>
                  <div className="field">
                    <span>LiteLLM 路径</span>
                    <input
                      placeholder="openai/gpt-4"
                      value={model.litellmModel}
                      onChange={(event) =>
                        onChange({
                          ...config,
                          providerGroups: config.providerGroups.map((entry) =>
                            entry.id === provider.id
                              ? {
                                  ...entry,
                                  models: updateModel(
                                    entry.models,
                                    model.id,
                                    "litellmModel",
                                    event.target.value,
                                  ),
                                }
                              : entry,
                          ),
                        })
                      }
                    />
                  </div>
                  <button
                    className="btn btn-danger"
                    onClick={() => handleDeleteModel(provider.id, model.id)}
                  >
                    删除
                  </button>
                </div>
              ))}
            </section>
          </div>
        ))}
      </div>
    </section>
  );
}
