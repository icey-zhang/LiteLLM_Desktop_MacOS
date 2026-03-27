use std::{
    fs,
    io::{BufRead, BufReader},
    path::Path,
    process::{Command, Stdio},
    thread,
};

use anyhow::{Context, Result};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::{
    config,
    logs::{sanitize_log_line, LogEntry},
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeStatus {
    pub state: String,
    pub detail: String,
    pub python_path: String,
    pub runtime_dir: String,
    pub version: Option<String>,
    pub hint: Option<String>,
}

pub fn get_runtime_status(app: &AppHandle) -> Result<RuntimeStatus> {
    let runtime_dir = config::runtime_venv_dir(app)?;
    let python_path = config::runtime_python_path(app)?;

    if !runtime_dir.exists() {
        return Ok(RuntimeStatus {
            state: "unknown".to_string(),
            detail: "尚未创建 LiteLLM 专属运行环境。".to_string(),
            python_path: python_path.to_string_lossy().into_owned(),
            runtime_dir: runtime_dir.to_string_lossy().into_owned(),
            version: None,
            hint: Some("应用启动时会自动创建并安装 litellm[proxy]。".to_string()),
        });
    }

    if !python_path.exists() {
        return Ok(RuntimeStatus {
            state: "error".to_string(),
            detail: "运行环境目录存在，但 Python 可执行文件缺失。".to_string(),
            python_path: python_path.to_string_lossy().into_owned(),
            runtime_dir: runtime_dir.to_string_lossy().into_owned(),
            version: None,
            hint: Some("建议重新修复运行环境。".to_string()),
        });
    }

    // Use a simple version check that doesn't stream for status check
    let output = Command::new(python_path.as_path())
        .args(["-m", "pip", "show", "litellm"])
        .output()?;

    if output.status.success() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        Ok(RuntimeStatus {
            state: "ready".to_string(),
            detail: "运行环境已就绪。".to_string(),
            python_path: python_path.to_string_lossy().into_owned(),
            runtime_dir: runtime_dir.to_string_lossy().into_owned(),
            version: extract_version(&stdout),
            hint: None,
        })
    } else {
        Ok(RuntimeStatus {
            state: "error".to_string(),
            detail: "运行环境不完整。".to_string(),
            python_path: python_path.to_string_lossy().into_owned(),
            runtime_dir: runtime_dir.to_string_lossy().into_owned(),
            version: None,
            hint: Some("应用会尝试自动重建该环境。".to_string()),
        })
    }
}

pub fn bootstrap_runtime(app: &AppHandle) -> Result<RuntimeStatus> {
    let current = get_runtime_status(app)?;
    if current.state == "ready" {
        emit_runtime_status(app, current.clone());
        return Ok(current);
    }

    let repairing = current.state == "error";
    let phase = if repairing { "repairing" } else { "installing" };
    let runtime_root = config::runtime_dir(app)?;
    let runtime_venv = config::runtime_venv_dir(app)?;
    let runtime_python = config::runtime_python_path(app)?;

    emit_system_log(app, "开始初始化 LiteLLM 运行环境...");

    emit_runtime_status(
        app,
        RuntimeStatus {
            state: phase.to_string(),
            detail: if repairing {
                "检测到运行环境损坏，正在重建。".to_string()
            } else {
                "正在创建 LiteLLM 运行环境。".to_string()
            },
            python_path: runtime_python.to_string_lossy().into_owned(),
            runtime_dir: runtime_venv.to_string_lossy().into_owned(),
            version: None,
            hint: None,
        },
    );

    if repairing && runtime_venv.exists() {
        fs::remove_dir_all(&runtime_venv).context("无法移除损坏的运行环境目录")?;
    }

    fs::create_dir_all(&runtime_root).context("无法创建 runtime 根目录")?;
    
    // Use the new streaming command runner
    run_command_streaming(app, Path::new("python3"), &["-m", "venv", path_str(&runtime_venv)?])?;

    emit_runtime_status(
        app,
        RuntimeStatus {
            state: phase.to_string(),
            detail: "虚拟环境已创建，正在升级 pip。".to_string(),
            python_path: runtime_python.to_string_lossy().into_owned(),
            runtime_dir: runtime_venv.to_string_lossy().into_owned(),
            version: None,
            hint: None,
        },
    );
    run_command_streaming(app, runtime_python.as_path(), &["-m", "pip", "install", "-U", "pip"])?;

    emit_runtime_status(
        app,
        RuntimeStatus {
            state: phase.to_string(),
            detail: "正在安装 litellm[proxy]。".to_string(),
            python_path: runtime_python.to_string_lossy().into_owned(),
            runtime_dir: runtime_venv.to_string_lossy().into_owned(),
            version: None,
            hint: None,
        },
    );
    run_command_streaming(
        app,
        runtime_python.as_path(),
        &["-m", "pip", "install", "litellm[proxy]"],
    )?;

    let final_status = get_runtime_status(app)?;
    emit_runtime_status(app, final_status.clone());
    emit_system_log(app, "LiteLLM 运行环境安装完成。");

    let config = config::load_or_default(app)?;
    let _ = config::save_app_config(app, &config)?;

    Ok(final_status)
}

fn emit_runtime_status(app: &AppHandle, status: RuntimeStatus) {
    let _ = app.emit("runtime-status", status);
}

fn emit_system_log(app: &AppHandle, message: &str) {
    let _ = app.emit(
        "proxy-log",
        LogEntry {
            stream: "system".to_string(),
            line: message.to_string(),
            timestamp: chrono_like_now(),
        },
    );
}

fn run_command_streaming(app: &AppHandle, program: &Path, args: &[&str]) -> Result<()> {
    let mut child = Command::new(program)
        .args(args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .with_context(|| format!("无法执行命令: {} {}", program.display(), args.join(" ")))?;

    let stdout = child.stdout.take().context("无法获取 stdout")?;
    let stderr = child.stderr.take().context("无法获取 stderr")?;

    let app_clone = app.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stdout);
        for line in reader.lines().map_while(Result::ok) {
            let _ = app_clone.emit(
                "proxy-log",
                LogEntry {
                    stream: "stdout".to_string(),
                    line: sanitize_log_line(&line),
                    timestamp: chrono_like_now(),
                },
            );
        }
    });

    let app_clone = app.clone();
    thread::spawn(move || {
        let reader = BufReader::new(stderr);
        for line in reader.lines().map_while(Result::ok) {
            let _ = app_clone.emit(
                "proxy-log",
                LogEntry {
                    stream: "stderr".to_string(),
                    line: sanitize_log_line(&line),
                    timestamp: chrono_like_now(),
                },
            );
        }
    });

    let status = child.wait()?;
    if !status.success() {
        anyhow::bail!(
            "{} {} 执行失败，退出码: {:?}",
            program.display(),
            args.join(" "),
            status.code()
        );
    }

    Ok(())
}

fn path_str(path: &Path) -> Result<&str> {
    path.to_str().context("路径包含非法字符")
}

fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_millis().to_string()
}

fn extract_version(output: &str) -> Option<String> {
    output
        .lines()
        .find_map(|line| line.strip_prefix("Version: ").map(|value| value.to_string()))
}

#[cfg(test)]
mod tests {
    use super::extract_version;

    #[test]
    fn extracts_version_from_pip_output() {
        let output = "Name: litellm\nVersion: 1.75.0\nSummary: proxy";
        assert_eq!(extract_version(output).as_deref(), Some("1.75.0"));
    }

    #[test]
    fn returns_none_when_version_is_missing() {
        assert!(extract_version("Name: litellm").is_none());
    }
}
