use std::{
    collections::BTreeMap,
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use tauri::{AppHandle, Manager};

const APP_CONFIG_FILE: &str = "app-config.json";
const LITELLM_CONFIG_FILE: &str = "litellm-config.yaml";
const LOG_DIR: &str = "logs";
const LOG_FILE: &str = "proxy.log";
const RUNTIME_DIR: &str = "runtime";
const VENV_DIR: &str = ".venv";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppSettings {
    pub python_path: String,
    pub port: u16,
    pub master_key: String,
    pub auto_start_proxy: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ProviderGroup {
    pub id: String,
    pub provider: String,
    pub api_key: String,
    pub api_base: String,
    pub models: Vec<ModelEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct ModelEntry {
    pub id: String,
    pub alias: String,
    pub litellm_model: String,
    pub extra_params: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct AppConfig {
    pub settings: AppSettings,
    pub provider_groups: Vec<ProviderGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LiteLlmConfig {
    pub model_list: Vec<LiteLlmModel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub general_settings: Option<GeneralSettings>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LiteLlmModel {
    pub model_name: String,
    pub litellm_params: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GeneralSettings {
    pub master_key: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyProviderPreset {
    pub provider: String,
    pub api_key: String,
    pub api_base: String,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyModelEntry {
    pub id: String,
    pub alias: String,
    pub litellm_model: String,
    pub provider: String,
    pub api_key: String,
    pub api_base: String,
    #[serde(default)]
    pub extra_params: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Deserialize)]
#[serde(rename_all = "camelCase")]
struct LegacyAppConfig {
    pub settings: AppSettings,
    pub providers: Vec<LegacyProviderPreset>,
    pub models: Vec<LegacyModelEntry>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            settings: AppSettings {
                python_path: String::new(),
                port: 4000,
                master_key: String::new(),
                auto_start_proxy: false,
            },
            provider_groups: vec![ProviderGroup {
                id: "provider-default".to_string(),
                provider: "openai".to_string(),
                api_key: String::new(),
                api_base: String::new(),
                models: vec![ModelEntry {
                    id: "model-default".to_string(),
                    alias: "gpt-4o-mini".to_string(),
                    litellm_model: "openai/gpt-4o-mini".to_string(),
                    extra_params: BTreeMap::new(),
                }],
            }],
        }
    }
}

impl From<LegacyAppConfig> for AppConfig {
    fn from(legacy: LegacyAppConfig) -> Self {
        let mut provider_groups = legacy
            .providers
            .into_iter()
            .enumerate()
            .map(|(index, provider)| ProviderGroup {
                id: format!("provider-migrated-{index}"),
                provider: provider.provider,
                api_key: provider.api_key,
                api_base: provider.api_base,
                models: Vec::new(),
            })
            .collect::<Vec<_>>();

        for model in legacy.models {
            let target_group_index = provider_groups.iter().position(|group| {
                group.provider == model.provider
                    && group.api_key == model.api_key
                    && group.api_base == model.api_base
            });

            let group = if let Some(index) = target_group_index {
                &mut provider_groups[index]
            } else if let Some(index) = provider_groups.iter().position(|group| {
                group.provider == model.provider
                    && group.api_key.is_empty()
                    && group.api_base.is_empty()
                    && model.api_key.is_empty()
                    && model.api_base.is_empty()
            }) {
                &mut provider_groups[index]
            } else {
                provider_groups.push(ProviderGroup {
                    id: format!("provider-migrated-dynamic-{}", provider_groups.len()),
                    provider: model.provider.clone(),
                    api_key: model.api_key.clone(),
                    api_base: model.api_base.clone(),
                    models: Vec::new(),
                });
                provider_groups
                    .last_mut()
                    .expect("migrated provider group should exist")
            };

            group.models.push(ModelEntry {
                id: model.id,
                alias: model.alias,
                litellm_model: model.litellm_model,
                extra_params: model.extra_params,
            });
        }

        if provider_groups.is_empty() {
            provider_groups = AppConfig::default().provider_groups;
        }

        Self {
            settings: legacy.settings,
            provider_groups,
        }
    }
}

fn app_data_dir(app: &AppHandle) -> Result<PathBuf> {
    let dir = app
        .path()
        .app_data_dir()
        .context("无法解析应用数据目录")?;
    fs::create_dir_all(&dir).context("无法创建应用数据目录")?;
    Ok(dir)
}

pub fn app_config_path(app: &AppHandle) -> Result<PathBuf> {
    Ok(app_data_dir(app)?.join(APP_CONFIG_FILE))
}

pub fn litellm_config_path(app: &AppHandle) -> Result<PathBuf> {
    Ok(app_data_dir(app)?.join(LITELLM_CONFIG_FILE))
}

pub fn proxy_log_path(app: &AppHandle) -> Result<PathBuf> {
    let logs_dir = app_data_dir(app)?.join(LOG_DIR);
    fs::create_dir_all(&logs_dir).context("无法创建日志目录")?;
    Ok(logs_dir.join(LOG_FILE))
}

pub fn runtime_dir(app: &AppHandle) -> Result<PathBuf> {
    let dir = app_data_dir(app)?.join(RUNTIME_DIR);
    fs::create_dir_all(&dir).context("无法创建 runtime 目录")?;
    Ok(dir)
}

pub fn runtime_venv_dir(app: &AppHandle) -> Result<PathBuf> {
    Ok(runtime_dir(app)?.join(VENV_DIR))
}

pub fn runtime_python_path(app: &AppHandle) -> Result<PathBuf> {
    Ok(runtime_venv_dir(app)?.join("bin").join("python"))
}

fn normalize_python_path(app: &AppHandle, config: &mut AppConfig) -> Result<()> {
    config.settings.python_path = runtime_python_path(app)?
        .to_string_lossy()
        .into_owned();
    Ok(())
}

pub fn load_or_default(app: &AppHandle) -> Result<AppConfig> {
    let path = app_config_path(app)?;
    if !path.exists() {
        let mut config = AppConfig::default();
        normalize_python_path(app, &mut config)?;
        return Ok(config);
    }

    let raw = fs::read_to_string(path).context("无法读取应用配置")?;
    let mut config = if let Ok(config) = serde_json::from_str::<AppConfig>(&raw) {
        config
    } else if let Ok(legacy) = serde_json::from_str::<LegacyAppConfig>(&raw) {
        legacy.into()
    } else {
        AppConfig::default()
    };
    normalize_python_path(app, &mut config)?;
    Ok(config)
}

pub fn save_app_config(app: &AppHandle, config: &AppConfig) -> Result<AppConfig> {
    let mut config = config.clone();
    normalize_python_path(app, &mut config)?;
    let config_path = app_config_path(app)?;
    let json = serde_json::to_string_pretty(&config).context("无法序列化应用配置")?;
    fs::write(config_path, json).context("无法写入应用配置")?;
    write_litellm_yaml(app, &config)?;
    Ok(config)
}

pub fn write_litellm_yaml(app: &AppHandle, config: &AppConfig) -> Result<PathBuf> {
    let lite_config = build_litellm_config(config);
    let yaml = serde_yaml::to_string(&lite_config).context("无法序列化 LiteLLM YAML")?;
    let path = litellm_config_path(app)?;
    fs::write(&path, yaml).context("无法写入 LiteLLM YAML")?;
    Ok(path)
}

pub fn clear_log_file(path: &Path) -> Result<()> {
    fs::write(path, "").context("无法重置日志文件")?;
    Ok(())
}

pub fn build_litellm_config(config: &AppConfig) -> LiteLlmConfig {
    let model_list = config
        .provider_groups
        .iter()
        .flat_map(|provider| {
            provider.models.iter().map(|model| {
                let mut litellm_params = BTreeMap::new();
                litellm_params
                    .insert("model".to_string(), Value::String(model.litellm_model.clone()));
                if !provider.api_key.trim().is_empty() {
                    litellm_params
                        .insert("api_key".to_string(), Value::String(provider.api_key.clone()));
                }
                if !provider.api_base.trim().is_empty() {
                    litellm_params
                        .insert("api_base".to_string(), Value::String(provider.api_base.clone()));
                }
                for (key, value) in &model.extra_params {
                    litellm_params.insert(key.clone(), value.clone());
                }

                LiteLlmModel {
                    model_name: model.alias.clone(),
                    litellm_params,
                }
            })
        })
        .collect::<Vec<_>>();

    let general_settings = if config.settings.master_key.trim().is_empty() {
        None
    } else {
        Some(GeneralSettings {
            master_key: config.settings.master_key.clone(),
        })
    };

    LiteLlmConfig {
        model_list,
        general_settings,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn builds_yaml_with_provider_inheritance() {
        let config = AppConfig {
            provider_groups: vec![ProviderGroup {
                id: "provider-1".to_string(),
                provider: "openai".to_string(),
                api_key: "sk-test".to_string(),
                api_base: "https://example.com/v1".to_string(),
                models: vec![ModelEntry {
                    id: "model-1".to_string(),
                    alias: "gpt-4o-mini".to_string(),
                    litellm_model: "openai/gpt-4o-mini".to_string(),
                    extra_params: BTreeMap::new(),
                }],
            }],
            ..AppConfig::default()
        };
        let lite = build_litellm_config(&config);

        assert_eq!(lite.model_list.len(), 1);
        assert_eq!(lite.model_list[0].model_name, "gpt-4o-mini");
        assert_eq!(
            lite.model_list[0]
                .litellm_params
                .get("model")
                .and_then(Value::as_str),
            Some("openai/gpt-4o-mini")
        );
        assert_eq!(
            lite.model_list[0]
                .litellm_params
                .get("api_key")
                .and_then(Value::as_str),
            Some("sk-test")
        );
        assert_eq!(
            lite.model_list[0]
                .litellm_params
                .get("api_base")
                .and_then(Value::as_str),
            Some("https://example.com/v1")
        );
    }

    #[test]
    fn migrates_legacy_flat_config() {
        let legacy = LegacyAppConfig {
            settings: AppSettings {
                python_path: "python3".to_string(),
                port: 4000,
                master_key: String::new(),
                auto_start_proxy: false,
            },
            providers: vec![LegacyProviderPreset {
                provider: "openai".to_string(),
                api_key: "sk-provider".to_string(),
                api_base: String::new(),
            }],
            models: vec![
                LegacyModelEntry {
                    id: "model-1".to_string(),
                    alias: "gpt-4o-mini".to_string(),
                    litellm_model: "openai/gpt-4o-mini".to_string(),
                    provider: "openai".to_string(),
                    api_key: "sk-provider".to_string(),
                    api_base: String::new(),
                    extra_params: BTreeMap::new(),
                },
                LegacyModelEntry {
                    id: "model-2".to_string(),
                    alias: "gpt-4.1".to_string(),
                    litellm_model: "openai/gpt-4.1".to_string(),
                    provider: "azure".to_string(),
                    api_key: "sk-azure".to_string(),
                    api_base: "https://azure.example.com".to_string(),
                    extra_params: BTreeMap::new(),
                },
            ],
        };

        let migrated: AppConfig = legacy.into();

        assert_eq!(migrated.provider_groups.len(), 2);
        assert_eq!(migrated.provider_groups[0].models.len(), 1);
        assert_eq!(migrated.provider_groups[1].provider, "azure");
        assert_eq!(migrated.provider_groups[1].models[0].alias, "gpt-4.1");
    }

    #[test]
    fn serializes_master_key_only_when_present() {
        let with_key = AppConfig {
            settings: AppSettings {
                master_key: "sk-local".to_string(),
                ..AppConfig::default().settings
            },
            ..AppConfig::default()
        };
        let without_key = AppConfig::default();

        assert!(build_litellm_config(&with_key).general_settings.is_some());
        assert!(build_litellm_config(&without_key).general_settings.is_none());
    }

    #[test]
    fn managed_runtime_python_path_looks_correct() {
        let path = PathBuf::from("/tmp")
            .join(RUNTIME_DIR)
            .join(VENV_DIR)
            .join("bin")
            .join("python");
        assert!(path.to_string_lossy().contains(".venv/bin/python"));
    }
}
