use crate::app::core::event_relay::{
    create_connection_event_relay, create_log_event_relay, create_memory_event_relay,
    create_traffic_event_relay, start_event_relay_with_retry,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{error, info};

lazy_static::lazy_static! {
    pub(super) static ref EVENT_RELAY_TASKS: Arc<Mutex<Vec<JoinHandle<()>>>> =
        Arc::new(Mutex::new(Vec::new()));
    pub(super) static ref SHOULD_STOP_EVENTS: Arc<AtomicBool> =
        Arc::new(AtomicBool::new(false));
}

pub(super) async fn start_websocket_relay(
    app_handle: AppHandle,
    api_port: Option<u16>,
) -> Result<(), String> {
    let port = api_port.ok_or("API端口参数是必需的，请从前端传递正确的端口配置")?;

    SHOULD_STOP_EVENTS.store(false, Ordering::Relaxed);
    cleanup_event_relay_tasks().await;

    info!("?? 开始启动事件中继服务，端口: {}", port);

    // 固定短延迟，给内核 API 一点时间完成初始化，避免立即连接抖动。
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;

    let token = crate::app::core::proxy_service::get_api_token();

    let traffic_relay = create_traffic_event_relay(app_handle.clone(), port, token.clone());
    let memory_relay = create_memory_event_relay(app_handle.clone(), port, token.clone());
    let log_relay = create_log_event_relay(app_handle.clone(), port, token.clone());
    let connection_relay = create_connection_event_relay(app_handle.clone(), port, token);

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

    {
        let mut tasks = EVENT_RELAY_TASKS.lock().await;
        tasks.push(traffic_task);
        tasks.push(memory_task);
        tasks.push(log_task);
        tasks.push(connection_task);
    }

    let _ = app_handle.emit("kernel-ready", ());

    Ok(())
}

pub(super) async fn cleanup_event_relay_tasks() {
    SHOULD_STOP_EVENTS.store(true, Ordering::Relaxed);

    let mut tasks = EVENT_RELAY_TASKS.lock().await;
    for task in tasks.drain(..) {
        task.abort();
    }

    info!("已清理所有事件中继任务");
}
