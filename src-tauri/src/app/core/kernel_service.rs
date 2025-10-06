use crate::app::constants::{common::messages, paths, process};
use crate::app::core::event_relay::{
    create_connection_event_relay, create_log_event_relay, create_memory_event_relay,
    create_traffic_event_relay, start_event_relay_with_retry,
};
use crate::process::manager::ProcessManager;
use crate::utils::http_client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

// 全局进程管理器
lazy_static::lazy_static! {
    pub(crate) static ref PROCESS_MANAGER: Arc<ProcessManager> = Arc::new(ProcessManager::new());
}

// 事件中继任务管理器
lazy_static::lazy_static! {
    static ref EVENT_RELAY_TASKS: Arc<tokio::sync::Mutex<Vec<JoinHandle<()>>>> = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    static ref SHOULD_STOP_EVENTS: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

// 内核启动通知器
lazy_static::lazy_static! {
    static ref KERNEL_READY_NOTIFY: Arc<Notify> = Arc::new(Notify::new());
}

// 检查内核版本
#[tauri::command]
pub async fn check_kernel_version() -> Result<String, String> {
    let kernel_path = paths::get_kernel_path();

    if !kernel_path.exists() {
        return Err(messages::ERR_KERNEL_NOT_FOUND.to_string());
    }

    let output = tokio::process::Command::new(kernel_path)
        .arg("version")
        .creation_flags(process::CREATE_NO_WINDOW)
        .output()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_VERSION_CHECK_FAILED, e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{}: {}", messages::ERR_GET_VERSION_FAILED, error));
    }

    let version_info = String::from_utf8_lossy(&output.stdout);
    Ok(version_info.to_string())
}

// 检查配置是否正常
#[tauri::command]
pub async fn check_config_validity(config_path: String) -> Result<(), String> {
    let kernel_path = paths::get_kernel_path();

    if !kernel_path.exists() {
        return Err(messages::ERR_KERNEL_NOT_FOUND.to_string());
    }

    // 确保配置文件路径存在
    let path = if config_path.is_empty() {
        paths::get_config_path().to_string_lossy().to_string()
    } else {
        config_path
    };

    // 检查配置文件是否存在
    if !std::path::Path::new(&path).exists() {
        return Err(format!("配置文件不存在: {}", path));
    }

    let output = tokio::process::Command::new(kernel_path)
        .arg("check")
        .arg("--config")
        .arg(path)
        .creation_flags(process::CREATE_NO_WINDOW)
        .output()
        .await
        .map_err(|e| format!("执行配置检查命令失败: {}", e))?;

    // 检查命令是否成功执行
    if !output.status.success() {
        // 如果有错误输出，返回错误信息
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("配置检查失败: {}", error));
    }

    Ok(())
}

// 下载最新内核版本
#[tauri::command]
pub async fn download_latest_kernel() -> Result<(), String> {
    info!("开始检查内核更新...");
    
    // 这里应该实现实际的下载逻辑
    // 暂时返回成功，表示检查完成
    Ok(())
}

// 启动内核
#[tauri::command]
pub async fn start_kernel() -> Result<String, String> {
    let kernel_path = paths::get_kernel_path();
    let config_path = paths::get_config_path();

    if !kernel_path.exists() {
        return Err(messages::ERR_KERNEL_NOT_FOUND.to_string());
    }

    if !config_path.exists() {
        return Err("配置文件不存在".to_string());
    }

    // 检查内核是否已经在运行
    if is_kernel_running().await.unwrap_or(false) {
        warn!("内核已在运行中");
        return Ok("内核已在运行中".to_string());
    }

    // 启动内核进程
    let _handle = PROCESS_MANAGER
        .start()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_PROCESS_START_FAILED, e))?;

    // 等待内核启动
    tokio::time::sleep(Duration::from_secs(3)).await;

    if is_kernel_running().await.unwrap_or(false) {
        info!("✅ 内核启动成功");
        
        // 通知内核就绪
        KERNEL_READY_NOTIFY.notify_waiters();
        
        Ok("内核启动成功".to_string())
    } else {
        Err(messages::ERR_PROCESS_START_FAILED.to_string())
    }
}

// 停止内核
#[tauri::command]
pub async fn stop_kernel() -> Result<String, String> {
    // 停止事件中继
    SHOULD_STOP_EVENTS.store(true, Ordering::Relaxed);
    cleanup_event_relay_tasks().await;

    // 停止内核进程
    PROCESS_MANAGER
        .stop()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_PROCESS_STOP_FAILED, e))?;

    // 等待进程完全停止
    tokio::time::sleep(Duration::from_secs(2)).await;

    if !is_kernel_running().await.unwrap_or(true) {
        info!("✅ 内核停止成功");
        Ok("内核停止成功".to_string())
    } else {
        Err(messages::ERR_PROCESS_STOP_FAILED.to_string())
    }
}

// 重启内核
#[tauri::command]
pub async fn restart_kernel() -> Result<String, String> {
    stop_kernel().await?;
    tokio::time::sleep(Duration::from_secs(3)).await;
    start_kernel().await
}

/// 启动事件中继服务
#[tauri::command]
pub async fn start_websocket_relay(
    app_handle: AppHandle,
    api_port: Option<u16>,
) -> Result<(), String> {
    // 要求前端必须传递API端口，不使用硬编码默认值
    let port = api_port.ok_or("API端口参数是必需的，请从前端传递正确的端口配置")?;

    // 重置停止标志
    SHOULD_STOP_EVENTS.store(false, Ordering::Relaxed);

    // 清理旧任务
    cleanup_event_relay_tasks().await;

    info!("🔌 开始启动事件中继服务，端口: {}", port);

    // 等待一段时间确保内核的 WebSocket 服务完全就绪
    tokio::time::sleep(Duration::from_millis(2000)).await;

    // 获取API token
    let token = crate::app::core::proxy_service::get_api_token();

    // 启动各种数据流的事件中继
    let traffic_relay = create_traffic_event_relay(app_handle.clone(), port, token.clone());
    let memory_relay = create_memory_event_relay(app_handle.clone(), port, token.clone());
    let log_relay = create_log_event_relay(app_handle.clone(), port, token.clone());
    let connection_relay = create_connection_event_relay(app_handle.clone(), port, token);

    // 启动事件中继任务
    let traffic_task = tokio::task::spawn(async move {
        if let Err(e) = start_event_relay_with_retry(traffic_relay, "traffic").await {
            error!("流量事件中继启动失败: {}", e);
        }
    });

    let memory_task = tokio::task::spawn(async move {
        if let Err(e) = start_event_relay_with_retry(memory_relay, "memory").await {
            error!("内存事件中继启动失败: {}", e);
        }
    });

    let log_task = tokio::task::spawn(async move {
        if let Err(e) = start_event_relay_with_retry(log_relay, "logs").await {
            error!("日志事件中继启动失败: {}", e);
        }
    });

    let connection_task = tokio::task::spawn(async move {
        if let Err(e) = start_event_relay_with_retry(connection_relay, "connections").await {
            error!("连接事件中继启动失败: {}", e);
        }
    });

    // 将任务添加到管理器
    {
        let mut tasks = EVENT_RELAY_TASKS.lock().await;
        tasks.push(traffic_task);
        tasks.push(memory_task);
        tasks.push(log_task);
        tasks.push(connection_task);
    }

    // 发送内核就绪事件
    let _ = app_handle.emit("kernel-ready", ());

    Ok(())
}

// 检查内核是否正在运行
#[tauri::command]
pub async fn is_kernel_running() -> Result<bool, String> {
    // 通过tasklist命令检查sing-box.exe是否在运行
    let output = tokio::process::Command::new("tasklist")
        .args(&["/FI", "IMAGENAME eq sing-box.exe", "/FO", "CSV", "/NH"])
        .creation_flags(crate::app::constants::process::CREATE_NO_WINDOW)
        .output()
        .await
        .map_err(|e| format!("检查内核进程失败: {}", e))?;

    // 检查输出中是否包含sing-box.exe
    let stdout = String::from_utf8_lossy(&output.stdout);
    let is_running = stdout.contains("sing-box.exe");

    info!("内核运行状态检查: {}", is_running);
    Ok(is_running)
}

// 检查内核完整状态（进程 + API）
#[tauri::command]
pub async fn check_kernel_status(api_port: Option<u16>) -> Result<serde_json::Value, String> {
    // 要求前端必须传递API端口，不使用硬编码默认值
    let port = api_port.ok_or("API端口参数是必需的，请从前端传递正确的端口配置")?;

    let process_running = is_kernel_running().await.unwrap_or(false);

    let mut status = serde_json::json!({
        "process_running": process_running,
        "api_ready": false,
        "websocket_ready": false
    });

    if process_running {
        // 检查API是否可用
        let client = http_client::get_client();
        let api_url = format!("http://127.0.0.1:{}/version?token=", port);

        let api_ready = match client
            .get(&api_url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => true,
            _ => false,
        };

        status["api_ready"] = serde_json::Value::Bool(api_ready);

        // 如果API可用，检查WebSocket
        if api_ready {
            let token = crate::app::core::proxy_service::get_api_token();
            let ws_ready = check_websocket_endpoints_ready(port, &token).await;
            status["websocket_ready"] = serde_json::Value::Bool(ws_ready);
        }
    }

    info!("内核完整状态: {}", status);
    Ok(status)
}

/// 清理事件中继任务
async fn cleanup_event_relay_tasks() {
    // 设置停止标志
    SHOULD_STOP_EVENTS.store(true, Ordering::Relaxed);

    // 等待所有任务完成
    let mut tasks = EVENT_RELAY_TASKS.lock().await;
    
    for task in tasks.drain(..) {
        task.abort();
    }
    
    info!("已清理所有事件中继任务");
}

/// 检查WebSocket端点是否就绪
async fn check_websocket_endpoints_ready(api_port: u16, token: &str) -> bool {
    use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
    use url::Url;

    let endpoints = ["traffic", "memory", "logs", "connections"];
    
    for endpoint in &endpoints {
        let url_str = format!("ws://127.0.0.1:{}/{}?token={}", api_port, endpoint, token);
        
        match Url::parse(&url_str) {
            Ok(url) => {
                match tokio::time::timeout(
                    Duration::from_secs(3),
                    connect_async(url)
                ).await {
                    Ok(Ok((ws_stream, _))) => {
                        // 连接成功，立即关闭
                        drop(ws_stream);
                        info!("✅ {} 端点就绪", endpoint);
                    }
                    _ => {
                        warn!("❌ {} 端点未就绪", endpoint);
                        return false;
                    }
                }
            }
            Err(_) => {
                warn!("❌ {} 端点URL解析失败", endpoint);
                return false;
            }
        }
    }
    
    true
}