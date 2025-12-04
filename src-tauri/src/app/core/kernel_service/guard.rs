use crate::app::constants::paths;
use crate::app::core::kernel_service::event::start_websocket_relay;
use crate::app::core::kernel_service::status::is_kernel_running;
use crate::app::core::kernel_service::PROCESS_MANAGER;
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use serde_json::json;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{info, warn};

static KEEP_ALIVE_ENABLED: AtomicBool = AtomicBool::new(false);
static GUARDED_API_PORT: AtomicU16 = AtomicU16::new(0);

lazy_static::lazy_static! {
    pub(super) static ref KERNEL_GUARD_HANDLE: Mutex<Option<JoinHandle<()>>> =
        Mutex::new(None);
}

pub(super) async fn enable_kernel_guard(app_handle: AppHandle, api_port: u16) {
    GUARDED_API_PORT.store(api_port, Ordering::Relaxed);
    if KEEP_ALIVE_ENABLED.swap(true, Ordering::Relaxed) {
        return;
    }

    let mut handle_slot = KERNEL_GUARD_HANDLE.lock().await;
    let guard_handle = tokio::spawn(async move {
        info!("内核守护已启动");
        loop {
            if !KEEP_ALIVE_ENABLED.load(Ordering::Relaxed) {
                break;
            }

            tokio::time::sleep(Duration::from_secs(8)).await;

            if !KEEP_ALIVE_ENABLED.load(Ordering::Relaxed) {
                break;
            }

            match is_kernel_running().await {
                Ok(true) => continue,
                _ => {
                    info!("守护检测到内核停止，尝试自动重启...");

                    let _ = app_handle.emit(
                        "kernel-stopped",
                        json!({
                            "process_running": false,
                            "api_ready": false,
                            "websocket_ready": false
                        }),
                    );
                    let _ = app_handle.emit(
                        "kernel-status-changed",
                        json!({
                            "process_running": false,
                            "api_ready": false,
                            "websocket_ready": false
                        }),
                    );

                    let app_config = db_get_app_config(app_handle.clone())
                        .await
                        .unwrap_or_default();
                    let config_path = if let Some(path_str) = app_config.active_config_path {
                        std::path::PathBuf::from(path_str)
                    } else {
                        paths::get_config_dir().join("config.json")
                    };

                    let kernel_path = paths::get_kernel_path();
                    if !kernel_path.exists() {
                        warn!("守护跳过重启：内核文件不存在 {:?}", kernel_path);
                        KEEP_ALIVE_ENABLED.store(false, Ordering::Relaxed);
                        GUARDED_API_PORT.store(0, Ordering::Relaxed);
                        break;
                    }
                    if !config_path.exists() {
                        warn!("守护跳过重启：配置不存在 {:?}", config_path);
                        KEEP_ALIVE_ENABLED.store(false, Ordering::Relaxed);
                        GUARDED_API_PORT.store(0, Ordering::Relaxed);
                        break;
                    }

                    if let Err(err) = PROCESS_MANAGER.start(&config_path).await {
                        warn!("守护重启内核失败: {}", err);
                        continue;
                    }

                    let port_value = GUARDED_API_PORT.load(Ordering::Relaxed);
                    if port_value > 0 {
                        if let Err(e) =
                            start_websocket_relay(app_handle.clone(), Some(port_value)).await
                        {
                            warn!("守护启动事件中继失败: {}", e);
                        }
                    }

                    let _ = app_handle.emit(
                        "kernel-started",
                        json!({
                            "process_running": true,
                            "api_ready": true,
                            "auto_restarted": true
                        }),
                    );
                    let _ = app_handle.emit(
                        "kernel-status-changed",
                        json!({
                            "process_running": true,
                            "api_ready": true,
                            "websocket_ready": true
                        }),
                    );
                    let _ = app_handle.emit("kernel-ready", ());
                }
            }
        }

        info!("内核守护任务结束");
    });

    *handle_slot = Some(guard_handle);
}

pub(super) async fn disable_kernel_guard() {
    if !KEEP_ALIVE_ENABLED.swap(false, Ordering::Relaxed) {
        return;
    }

    GUARDED_API_PORT.store(0, Ordering::Relaxed);
    let mut handle_slot = KERNEL_GUARD_HANDLE.lock().await;
    if let Some(handle) = handle_slot.take() {
        handle.abort();
    }
}
