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

const MIN_PYTHON_MAJOR: u32 = 3;
const MIN_PYTHON_MINOR: u32 = 10;

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

    let python_version_output = Command::new(python_path.as_path())
        .arg("--version")
        .output()?;
    let python_version_text = String::from_utf8_lossy(
        if python_version_output.stdout.is_empty() {
            &python_version_output.stderr
        } else {
            &python_version_output.stdout
        },
    )
    .trim()
    .to_string();
    if let Some(version) = parse_python_version(&python_version_text) {
        if !is_supported_python_version(version) {
            return Ok(RuntimeStatus {
                state: "error".to_string(),
                detail: format!("运行环境 Python 版本过低: {}", python_version_text),
                python_path: python_path.to_string_lossy().into_owned(),
                runtime_dir: runtime_dir.to_string_lossy().into_owned(),
                version: None,
                hint: Some("需要 Python 3.10+，应用会尝试重建运行环境。".to_string()),
            });
        }
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

    let system_python = resolve_system_python()?;
    run_command_streaming(
        app,
        Path::new(&system_python),
        &["-m", "venv", path_str(&runtime_venv)?],
    )?;

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

fn parse_python_version(output: &str) -> Option<(u32, u32)> {
    let version = output.strip_prefix("Python ")?;
    let mut parts = version.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    Some((major, minor))
}

fn is_supported_python_version(version: (u32, u32)) -> bool {
    version.0 > MIN_PYTHON_MAJOR
        || (version.0 == MIN_PYTHON_MAJOR && version.1 >= MIN_PYTHON_MINOR)
}

fn pick_python_command(candidates: &[(&str, Option<(u32, u32)>)]) -> Option<String> {
    candidates.iter().find_map(|(command, version)| {
        version
            .filter(|version| is_supported_python_version(*version))
            .map(|_| (*command).to_string())
    })
}

fn resolve_system_python() -> Result<String> {
    let candidates = [
        "/opt/homebrew/bin/python3.13",
        "/opt/homebrew/bin/python3.12",
        "/opt/homebrew/bin/python3.11",
        "/opt/homebrew/bin/python3.10",
        "/opt/homebrew/bin/python3",
        "/usr/local/bin/python3.13",
        "/usr/local/bin/python3.12",
        "/usr/local/bin/python3.11",
        "/usr/local/bin/python3.10",
        "/usr/local/bin/python3",
        "python3.13",
        "python3.12",
        "python3.11",
        "python3.10",
        "python3",
    ];

    let probed = candidates
        .iter()
        .map(|candidate| {
            let output = Command::new(candidate).arg("--version").output().ok();
            let version = output.as_ref().and_then(|output| {
                if !output.status.success() {
                    return None;
                }
                let text = if output.stdout.is_empty() {
                    String::from_utf8_lossy(&output.stderr).into_owned()
                } else {
                    String::from_utf8_lossy(&output.stdout).into_owned()
                };
                parse_python_version(text.trim())
            });
            (*candidate, version)
        })
        .collect::<Vec<_>>();

    pick_python_command(&probed).context("未找到可用的 Python 3.10+ 解释器")
}

#[cfg(test)]
mod tests {
    use super::{
        extract_version, is_supported_python_version, parse_python_version, pick_python_command,
    };

    #[test]
    fn extracts_version_from_pip_output() {
        let output = "Name: litellm\nVersion: 1.75.0\nSummary: proxy";
        assert_eq!(extract_version(output).as_deref(), Some("1.75.0"));
    }

    #[test]
    fn returns_none_when_version_is_missing() {
        assert!(extract_version("Name: litellm").is_none());
    }

    #[test]
    fn parses_python_major_minor_version() {
        assert_eq!(parse_python_version("Python 3.13.11"), Some((3, 13)));
        assert_eq!(parse_python_version("Python 3.9.6"), Some((3, 9)));
    }

    #[test]
    fn rejects_python_39_for_runtime() {
        assert!(!is_supported_python_version((3, 9)));
        assert!(is_supported_python_version((3, 10)));
        assert!(is_supported_python_version((3, 13)));
    }

    #[test]
    fn picks_first_supported_python_candidate() {
        let selected = pick_python_command(&[
            ("python3", Some((3, 9))),
            ("/opt/homebrew/bin/python3.13", Some((3, 13))),
            ("/usr/bin/python3", Some((3, 9))),
        ]);

        assert_eq!(selected.as_deref(), Some("/opt/homebrew/bin/python3.13"));
    }
}
