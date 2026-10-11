use crate::app::constants::paths;
use crate::app::core::kernel_service::PROCESS_MANAGER;
use crate::platform;
use crate::utils::http_client;
use std::time::Duration;
use tracing::info;

#[tauri::command]
pub async fn is_kernel_running() -> Result<bool, String> {
    // 首先检查 ProcessManager 中的进程句柄
    if PROCESS_MANAGER.is_running().await {
        return Ok(true);
    }

    // 使用平台抽象层检测外部启动的内核进程
    let kernel_name = platform::get_kernel_executable_name();
    match platform::is_process_running(kernel_name).await {
        Ok(running) => {
            if running {
                info!("通过平台抽象层检测到内核进程");
            } else {
                info!("内核运行状态检查: false (未找到相关进程)");
            }
            Ok(running)
        }
        Err(e) => {
            info!("平台进程检测失败: {}, 返回 false", e);
            Ok(false)
        }
    }
}

#[tauri::command]
pub async fn get_system_uptime() -> Result<u64, String> {
    platform::get_system_uptime_ms().await
}

#[tauri::command]
pub async fn kernel_check_health(api_port: Option<u16>) -> Result<serde_json::Value, String> {
    let mut issues = Vec::new();
    let mut healthy = true;

    let kernel_path = paths::get_kernel_path();
    if !kernel_path.exists() {
        issues.push("内核文件不存在".to_string());
        healthy = false;
    }

    let config_path = paths::get_config_dir().join("config.json");
    if !config_path.exists() {
        issues.push("配置文件不存在".to_string());
        healthy = false;
    }

    let process_running = is_kernel_running().await.unwrap_or(false);
    if process_running {
        let port = api_port.unwrap_or(12081);
        let client = http_client::get_client();
        let api_url = format!("http://127.0.0.1:{}/version", port);

        let api_ready = matches!(
            client
                .get(&api_url)
                .timeout(Duration::from_secs(2))
                .send()
                .await,
            Ok(response) if response.status().is_success()
        );

        if !api_ready {
            issues.push(format!("内核进程运行但API不可用（端口: {}）", port));
            healthy = false;
        }
    }

    Ok(serde_json::json!({
        "healthy": healthy,
        "issues": issues
    }))
}
