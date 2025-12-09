use std::sync::Arc;
use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tokio::sync::OnceCell;
use tracing::{error, info, warn};

use crate::app::core::kernel_service::status::kernel_check_health;
use crate::app::storage::enhanced_storage_service::EnhancedStorageService;
use crate::app::system::update_service::{check_update, UpdateInfo};

const UPDATE_CHECK_INTERVAL: Duration = Duration::from_secs(4 * 60 * 60); // 4h
const KERNEL_HEALTH_INTERVAL: Duration = Duration::from_secs(10 * 60); // 10min

pub async fn start_background_tasks(app: &AppHandle) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start_update_loop(app_handle.clone()).await {
            error!("后台更新检查任务结束，原因: {}", e);
        }
    });

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(e) = start_kernel_health_loop(app_handle.clone()).await {
            error!("后台内核健康检查任务结束，原因: {}", e);
        }
    });
}

async fn wait_for_storage(app: &AppHandle) -> Option<Arc<EnhancedStorageService>> {
    let storage_cell = app.state::<Arc<OnceCell<Arc<EnhancedStorageService>>>>();
    loop {
        if let Some(storage) = storage_cell.get() {
            return Some(storage.clone());
        }
        warn!("存储服务尚未就绪，稍后重试...");
        tokio::time::sleep(Duration::from_secs(1)).await;
    }
}

async fn start_update_loop(app: AppHandle) -> Result<(), String> {
    let version = app.package_info().version.to_string();
    let storage = wait_for_storage(&app).await;

    loop {
        if let Some(storage) = &storage {
            match storage.get_update_config().await {
                Ok(config) => {
                    if !config.auto_check {
                        warn!("自动更新检查已关闭，跳过本轮后台检查");
                    } else {
                        match check_update(version.clone(), Some(config.accept_prerelease)).await {
                            Ok(info) => handle_update_result(&app, &config.skip_version, info).await,
                            Err(e) => warn!("后台检查更新失败: {}", e),
                        }
                    }
                }
                Err(e) => warn!("读取更新配置失败: {}", e),
            }
        }

        tokio::time::sleep(UPDATE_CHECK_INTERVAL).await;
    }
}

async fn handle_update_result(
    app: &AppHandle,
    skip_version: &Option<String>,
    info: UpdateInfo,
) {
    if info.has_update {
        if skip_version
            .as_ref()
            .map(|v| v == &info.latest_version)
            .unwrap_or(false)
        {
            info!("检测到可更新版本 {}，但用户已选择跳过", info.latest_version);
            return;
        }

        if let Err(e) = app.emit("update-available", &info) {
            error!("发送 update-available 事件失败: {}", e);
        } else {
            info!(
                "后台检测到新版本 {}，已推送事件",
                info.latest_version
            );
        }
    } else {
        info!("后台检查：当前已是最新版本");
    }
}

async fn start_kernel_health_loop(app: AppHandle) -> Result<(), String> {
    loop {
        match kernel_check_health(None).await {
            Ok(payload) => {
                if let Err(e) = app.emit("kernel-health", &payload) {
                    error!("发送 kernel-health 事件失败: {}", e);
                }
            }
            Err(e) => warn!("后台内核健康检查失败: {}", e),
        }

        tokio::time::sleep(KERNEL_HEALTH_INTERVAL).await;
    }
}
