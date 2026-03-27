use anyhow::Context;
use serde::Serialize;
use std::process::Command;

use crate::runtime;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct DiagnosticCheck {
    pub ok: bool,
    pub detail: String,
    pub hint: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EnvironmentStatus {
    pub python: DiagnosticCheck,
    pub runtime: DiagnosticCheck,
    pub overall_ok: bool,
}

fn run_command(program: &str, args: &[&str]) -> anyhow::Result<String> {
    let output = Command::new(program)
        .args(args)
        .output()
        .with_context(|| format!("无法执行命令: {} {}", program, args.join(" ")))?;

    if !output.status.success() {
        anyhow::bail!(
            "{} {} 执行失败: {}",
            program,
            args.join(" "),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

pub fn check_environment(app: &tauri::AppHandle) -> EnvironmentStatus {
    let python = match run_command("python3", &["--version"]) {
        Ok(version) => DiagnosticCheck {
            ok: true,
            detail: version,
            hint: None,
        },
        Err(error) => DiagnosticCheck {
            ok: false,
            detail: error.to_string(),
            hint: Some("请先确保系统 `python3` 可执行，应用会用它来创建专属运行环境。".to_string()),
        },
    };

    let runtime = if python.ok {
        match runtime::get_runtime_status(app) {
            Ok(status) if status.state == "ready" => DiagnosticCheck {
                ok: true,
                detail: status.detail,
                hint: status.hint,
            },
            Ok(status) => DiagnosticCheck {
                ok: false,
                detail: status.detail,
                hint: Some(
                    status
                        .hint
                        .unwrap_or_else(|| "应用会在启动时自动创建并修复运行环境。".to_string()),
                ),
            },
            Err(error) => DiagnosticCheck {
                ok: false,
                detail: error.to_string(),
                hint: Some("应用无法读取 runtime 目录，请检查用户目录写权限。".to_string()),
            },
        }
    } else {
        DiagnosticCheck {
            ok: false,
            detail: "系统 Python 不可用，跳过 runtime 检测".to_string(),
            hint: Some("先修复系统 `python3`，再让应用自动初始化运行环境。".to_string()),
        }
    };

    EnvironmentStatus {
        overall_ok: python.ok && runtime.ok,
        python,
        runtime,
    }
}

#[cfg(test)]
fn extract_version(output: &str) -> Option<String> {
    output
        .lines()
        .find_map(|line| line.strip_prefix("Version: ").map(|value| value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::extract_version;

    #[test]
    fn extracts_version_from_pip_show_output() {
        let output = "Name: litellm\nVersion: 1.63.11\nSummary: unified proxy";
        assert_eq!(extract_version(output).as_deref(), Some("1.63.11"));
    }

    #[test]
    fn returns_none_when_version_missing() {
        assert!(extract_version("Name: litellm").is_none());
    }
}
