#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod config;
mod environment;
mod logs;
mod proxy_manager;
mod runtime;
mod test_request;

use std::sync::Mutex;

use config::AppConfig;
use tauri::{AppHandle, State};

struct AppState {
    proxy_manager: Mutex<proxy_manager::ProxyManager>,
}

#[tauri::command]
fn load_app_config(app: AppHandle) -> Result<AppConfig, String> {
    config::load_or_default(&app).map_err(|error| error.to_string())
}

#[tauri::command]
fn save_app_config(app: AppHandle, config: AppConfig) -> Result<AppConfig, String> {
    config::save_app_config(&app, &config).map_err(|error| error.to_string())
}

#[tauri::command]
fn check_environment(app: AppHandle) -> environment::EnvironmentStatus {
    environment::check_environment(&app)
}

#[tauri::command]
fn get_runtime_status(app: AppHandle) -> Result<runtime::RuntimeStatus, String> {
    runtime::get_runtime_status(&app).map_err(|error| error.to_string())
}

#[tauri::command]
fn bootstrap_runtime(app: AppHandle) -> Result<runtime::RuntimeStatus, String> {
    runtime::bootstrap_runtime(&app).map_err(|error| error.to_string())
}

#[tauri::command]
fn get_proxy_status(
    _app: AppHandle,
    state: State<'_, AppState>,
) -> Result<proxy_manager::ProxyStatus, String> {
    let mut manager = state
        .proxy_manager
        .lock()
        .map_err(|_| "无法获取代理状态锁".to_string())?;
    Ok(manager.current_status())
}

#[tauri::command]
fn start_proxy(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<proxy_manager::ProxyStatus, String> {
    let mut manager = state
        .proxy_manager
        .lock()
        .map_err(|_| "无法获取代理状态锁".to_string())?;
    manager.start(&app).map_err(|error| error.to_string())
}

#[tauri::command]
fn stop_proxy(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<proxy_manager::ProxyStatus, String> {
    let mut manager = state
        .proxy_manager
        .lock()
        .map_err(|_| "无法获取代理状态锁".to_string())?;
    manager.stop(&app).map_err(|error| error.to_string())
}

#[tauri::command]
fn restart_proxy(
    app: AppHandle,
    state: State<'_, AppState>,
) -> Result<proxy_manager::ProxyStatus, String> {
    let mut manager = state
        .proxy_manager
        .lock()
        .map_err(|_| "无法获取代理状态锁".to_string())?;
    manager.restart(&app).map_err(|error| error.to_string())
}

#[tauri::command]
async fn test_proxy_request(
    payload: test_request::TestRequestPayload,
) -> test_request::TestRequestResult {
    test_request::run_test_request(payload).await
}

fn main() {
    tauri::Builder::default()
        .manage(AppState {
            proxy_manager: Mutex::new(proxy_manager::ProxyManager::new()),
        })
        .invoke_handler(tauri::generate_handler![
            load_app_config,
            save_app_config,
            check_environment,
            get_runtime_status,
            bootstrap_runtime,
            get_proxy_status,
            start_proxy,
            stop_proxy,
            restart_proxy,
            test_proxy_request
        ])
        .run(tauri::generate_context!())
        .expect("tauri application error");
}
