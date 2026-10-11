use crate::app::constants::paths;
use crate::app::core::kernel_service::health;
use crate::app::core::kernel_service::PROCESS_MANAGER;
use crate::platform;
use crate::utils::http_client;
use std::time::Duration;
use tracing::info;

/// 内部实现：接收引用（`AppHandle` 不实现 `Copy`，多调用点场景必须传引用）。
pub async fn is_kernel_running_ref(app_handle: &tauri::AppHandle) -> Result<bool, String> {
    // 统一走 kernel_service::health —— 端口 > 托管句柄 > 平台扫描。
    //
    // 原实现只用「进程句柄 / 平台扫描」，与前端所看的 kernel_get_status_enhanced_v2
    // （纯 TCP 端口探测）结论可能相反：内核端口已经在服务但句柄丢失时，
    // 前端显示「运行中」而这里返回 false，守护逻辑也会据此做出错误判断。
    //
    // 需要 `app_handle` 读取权威端口配置（db）；读失败时退化为「只看进程」。
    let (api_port, proxy_port) =
        match crate::app::storage::enhanced_storage_service::db_get_app_config_ref(app_handle).await
        {
            Ok(config) => (config.api_port, config.proxy_port),
            Err(error) => {
                tracing::warn!(
                    "is_kernel_running: 读取端口配置失败，退化为仅进程检测: {}",
                    error
                );
                let managed = PROCESS_MANAGER.is_running().await;
                let scanned = platform::is_process_running(platform::get_kernel_executable_name())
                    .await
                    .unwrap_or(false);
                return Ok(managed || scanned);
            }
        };

    let managed = PROCESS_MANAGER.is_running().await;
    let scanned = platform::is_process_running(platform::get_kernel_executable_name())
        .await
        .unwrap_or(false);

    let health =
        health::probe_kernel(api_port, proxy_port, health::TcpPortProbe, managed, scanned).await;

    if health.is_running() {
        info!("内核运行状态检查: true (来源: {})", health.source.as_str());
    } else {
        info!("内核运行状态检查: false (端口不通且未发现进程)");
    }
    Ok(health.is_running())
}

/// Tauri 命令包装（Tauri 命令参数必须按值接收 `AppHandle`）。
#[tauri::command]
pub async fn is_kernel_running(app_handle: tauri::AppHandle) -> Result<bool, String> {
    is_kernel_running_ref(&app_handle).await
}

#[tauri::command]
pub async fn get_system_uptime() -> Result<u64, String> {
    platform::get_system_uptime_ms().await
}

pub async fn kernel_check_health_ref(
    app_handle: &tauri::AppHandle,
    api_port: Option<u16>,
) -> Result<serde_json::Value, String> {
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

    let process_running = is_kernel_running_ref(app_handle).await.unwrap_or(false);
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

/// Tauri 命令包装（后台健康巡检注册的是这个）。
#[tauri::command]
pub async fn kernel_check_health(
    app_handle: tauri::AppHandle,
    api_port: Option<u16>,
) -> Result<serde_json::Value, String> {
    kernel_check_health_ref(&app_handle, api_port).await
}
