use std::{
    fs::OpenOptions,
    io::{BufRead, BufReader, Write},
    net::{SocketAddr, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    thread,
    time::Duration,
};

use anyhow::{Context, Result};
use serde::Serialize;
use tauri::{AppHandle, Emitter};

use crate::{
    config::{self, load_or_default},
    logs::{sanitize_log_line, LogEntry},
    runtime,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyStatus {
    pub state: String,
    pub pid: Option<u32>,
    pub port: u16,
    pub endpoint: String,
    pub healthy: bool,
    pub last_error: Option<String>,
}

pub struct ProxyManager {
    child: Option<Child>,
    status: ProxyStatus,
}

impl ProxyManager {
    pub fn new() -> Self {
        Self {
            child: None,
            status: ProxyStatus {
                state: "stopped".to_string(),
                pid: None,
                port: 4000,
                endpoint: "http://127.0.0.1:4000".to_string(),
                healthy: false,
                last_error: None,
            },
        }
    }

    pub fn current_status(&mut self) -> ProxyStatus {
        self.refresh_status();
        self.status.clone()
    }

    pub fn start(&mut self, app: &AppHandle) -> Result<ProxyStatus> {
        self.refresh_status();
        if self.child.is_some() {
            return Ok(self.status.clone());
        }

        let runtime_status = runtime::get_runtime_status(app)?;
        if runtime_status.state != "ready" {
            self.status.state = "error".to_string();
            self.status.last_error = Some(
                runtime_status
                    .hint
                    .unwrap_or_else(|| "LiteLLM 运行环境尚未就绪，请等待自动安装完成。".to_string()),
            );
            return Ok(self.status.clone());
        }

        let config = load_or_default(app)?;
        let yaml_path = config::write_litellm_yaml(app, &config)?;
        let log_path = config::proxy_log_path(app)?;
        config::clear_log_file(&log_path)?;

        let mut command = build_litellm_command(
            Path::new(&runtime_status.python_path),
            &yaml_path,
            config.settings.port,
        )?;
        command.stdout(Stdio::piped()).stderr(Stdio::piped());

        let mut child = command.spawn().with_context(|| {
            format!(
                "无法启动 LiteLLM，请确认 `{}` 和 `litellm[proxy]` 已安装。",
                runtime_status.python_path
            )
        })?;

        if let Some(stdout) = child.stdout.take() {
            spawn_log_pump(app.clone(), stdout, "stdout", log_path.clone());
        }
        if let Some(stderr) = child.stderr.take() {
            spawn_log_pump(app.clone(), stderr, "stderr", log_path.clone());
        }

        self.status = ProxyStatus {
            state: "starting".to_string(),
            pid: Some(child.id()),
            port: config.settings.port,
            endpoint: format!("http://127.0.0.1:{}", config.settings.port),
            healthy: false,
            last_error: None,
        };
        self.child = Some(child);
        emit_system_log(app, "LiteLLM 启动命令已发出。");

        for _ in 0..20 {
            thread::sleep(Duration::from_millis(250));
            self.refresh_status();
            if self.status.healthy {
                self.status.state = "running".to_string();
                emit_system_log(app, "LiteLLM 已通过端口健康检查。");
                return Ok(self.status.clone());
            }
            if self.child.is_none() {
                break;
            }
        }

        resolve_start_timeout_state(&mut self.status, self.child.is_some());
        Ok(self.status.clone())
    }

    pub fn stop(&mut self, app: &AppHandle) -> Result<ProxyStatus> {
        if let Some(mut child) = self.child.take() {
            emit_system_log(app, "正在停止 LiteLLM 进程。");
            let _ = child.kill();
            let _ = child.wait();
        }

        self.status.state = "stopped".to_string();
        self.status.pid = None;
        self.status.healthy = false;
        self.status.last_error = None;
        Ok(self.status.clone())
    }

    pub fn restart(&mut self, app: &AppHandle) -> Result<ProxyStatus> {
        self.stop(app)?;
        self.start(app)
    }

    fn refresh_status(&mut self) {
        if let Some(child) = self.child.as_mut() {
            match child.try_wait() {
                Ok(Some(exit_status)) => {
                    self.status.state = if exit_status.success() {
                        "stopped".to_string()
                    } else {
                        "error".to_string()
                    };
                    self.status.pid = None;
                    self.status.healthy = false;
                    self.status.last_error = if exit_status.success() {
                        None
                    } else {
                        Some(format!("LiteLLM 已退出，状态码: {:?}", exit_status.code()))
                    };
                    self.child = None;
                }
                Ok(None) => {
                    self.status.pid = Some(child.id());
                    self.status.healthy = port_is_open(self.status.port);
                    if self.status.healthy {
                        self.status.state = "running".to_string();
                    }
                }
                Err(error) => {
                    self.status.state = "error".to_string();
                    self.status.last_error = Some(error.to_string());
                }
            }
        } else {
            self.status.pid = None;
            self.status.healthy = false;
            if self.status.state != "error" {
                self.status.state = "stopped".to_string();
            }
        }
    }
}

fn emit_system_log(app: &AppHandle, message: &str) {
    let entry = LogEntry {
        stream: "system".to_string(),
        line: message.to_string(),
        timestamp: chrono_like_now(),
    };
    let _ = app.emit("proxy-log", entry);
}

fn spawn_log_pump<R: std::io::Read + Send + 'static>(
    app: AppHandle,
    reader: R,
    stream: &str,
    log_path: std::path::PathBuf,
) {
    let stream_name = stream.to_string();
    thread::spawn(move || {
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(log_path)
            .ok();
        let reader = BufReader::new(reader);
        for line in reader.lines().map_while(Result::ok) {
            let sanitized = sanitize_log_line(&line);
            if let Some(file) = file.as_mut() {
                let _ = writeln!(file, "[{}] {}", stream_name, sanitized);
            }
            let _ = app.emit(
                "proxy-log",
                LogEntry {
                    stream: stream_name.clone(),
                    line: sanitized,
                    timestamp: chrono_like_now(),
                },
            );
        }
    });
}

fn port_is_open(port: u16) -> bool {
    let addr = SocketAddr::from(([127, 0, 0, 1], port));
    TcpStream::connect_timeout(&addr, Duration::from_millis(150)).is_ok()
}

fn build_litellm_command(python_path: &Path, yaml_path: &Path, port: u16) -> Result<Command> {
    let yaml = yaml_path.to_str().context("LiteLLM 配置路径包含非法字符")?;
    let port_string = port.to_string();
    let litellm_path = litellm_command_path(python_path);

    if litellm_path.exists() {
        let mut command = Command::new(litellm_path);
        command.args(["--config", yaml, "--port", &port_string]);
        sanitize_litellm_command_env(&mut command);
        return Ok(command);
    }

    let mut command = Command::new(python_path);
    command.args(["-m", "litellm", "--config", yaml, "--port", &port_string]);
    sanitize_litellm_command_env(&mut command);
    Ok(command)
}

fn sanitize_litellm_command_env(command: &mut Command) {
    command.env_remove("DEBUG");
    command.env_remove("DETAILED_DEBUG");
}

fn litellm_command_path(python_path: &Path) -> PathBuf {
    let executable = if cfg!(windows) { "litellm.exe" } else { "litellm" };

    python_path
        .parent()
        .map(|dir| dir.join(executable))
        .unwrap_or_else(|| PathBuf::from(executable))
}

fn resolve_start_timeout_state(status: &mut ProxyStatus, child_running: bool) {
    if child_running {
        status.state = "starting".to_string();
        status.last_error = None;
        return;
    }

    status.state = "error".to_string();
    status.last_error = Some("LiteLLM 启动失败，请查看日志面板。".to_string());
}

fn chrono_like_now() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let duration = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();
    duration.as_millis().to_string()
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        build_litellm_command, litellm_command_path, port_is_open, resolve_start_timeout_state,
        ProxyManager, ProxyStatus,
    };

    #[test]
    fn starts_with_stopped_status() {
        let mut manager = ProxyManager::new();
        let status = manager.current_status();

        assert_eq!(status.state, "stopped");
        assert!(!status.healthy);
    }

    #[test]
    fn reports_closed_port_as_unhealthy() {
        assert!(!port_is_open(9));
    }

    #[test]
    fn prefers_venv_litellm_executable_on_unix() {
        let python_path = Path::new("/tmp/runtime/.venv/bin/python");

        assert_eq!(
            litellm_command_path(python_path),
            Path::new("/tmp/runtime/.venv/bin/litellm")
        );
    }

    #[test]
    fn keeps_starting_state_if_process_is_still_alive_after_timeout() {
        let mut status = ProxyStatus {
            state: "starting".to_string(),
            pid: Some(42),
            port: 4000,
            endpoint: "http://127.0.0.1:4000".to_string(),
            healthy: false,
            last_error: None,
        };

        resolve_start_timeout_state(&mut status, true);

        assert_eq!(status.state, "starting");
        assert!(status.last_error.is_none());
    }

    #[test]
    fn strips_debug_env_vars_from_litellm_command() {
        let python_path = Path::new("/tmp/runtime/.venv/bin/python");
        let yaml_path = Path::new("/tmp/runtime/litellm-config.yaml");
        let command = build_litellm_command(python_path, yaml_path, 4000).unwrap();
        let envs = command.get_envs().collect::<Vec<_>>();

        assert!(
            envs.iter()
                .any(|entry| entry.0 == "DEBUG" && entry.1.is_none())
        );
        assert!(
            envs.iter()
                .any(|entry| entry.0 == "DETAILED_DEBUG" && entry.1.is_none())
        );
    }
}
