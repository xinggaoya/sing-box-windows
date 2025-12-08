use crate::app::constants::paths;
use crate::app::core::kernel_service::event::start_websocket_relay;
use crate::app::core::kernel_service::status::is_kernel_running;
use crate::app::core::kernel_service::utils::{
    emit_kernel_started, emit_kernel_stopped, resolve_config_path_or_default,
};
use crate::app::core::kernel_service::PROCESS_MANAGER;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::time::Duration;
use tauri::AppHandle;
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

                    emit_kernel_stopped(&app_handle);

                    let config_path = resolve_config_path_or_default(&app_handle).await;

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

                    // Guard restart uses port 0 since we don't have full state
                    emit_kernel_started(&app_handle, "auto", port_value, 0, true);
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
