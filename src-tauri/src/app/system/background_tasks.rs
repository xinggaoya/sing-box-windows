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

/// 等待存储服务就绪的上限。数据库初始化位于启动链的同步串行段，正常在毫秒级完成；
/// 超过该阈值说明初始化已经失败（或永久卡住），此时必须放弃并报错，
/// 否则后台任务会以 1Hz 无限空转并持续刷日志。
const STORAGE_WAIT_TIMEOUT: Duration = Duration::from_secs(30);
const STORAGE_WAIT_INTERVAL: Duration = Duration::from_millis(500);
/// 每 N 次轮询打一条 warn（约 2 秒一条），避免高频轮询刷屏。
const STORAGE_WAIT_LOG_EVERY: u32 = 4;

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

/// 等待存储服务就绪，超时则返回错误而不是永久空转。
async fn wait_for_storage(app: &AppHandle) -> Result<Arc<EnhancedStorageService>, String> {
    let storage_cell = app.state::<Arc<OnceCell<Arc<EnhancedStorageService>>>>();
    let deadline = tokio::time::Instant::now() + STORAGE_WAIT_TIMEOUT;
    let mut attempt: u32 = 0;

    loop {
        if let Some(storage) = storage_cell.get() {
            return Ok(storage.clone());
        }

        attempt += 1;
        if tokio::time::Instant::now() >= deadline {
            return Err(format!(
                "等待存储服务就绪超时（{} 秒 / {} 次尝试）；数据库初始化可能已失败，\
                 后台更新检查与内核健康检查将不再运行",
                STORAGE_WAIT_TIMEOUT.as_secs(),
                attempt
            ));
        }

        if attempt % STORAGE_WAIT_LOG_EVERY == 1 {
            warn!("存储服务尚未就绪，稍后重试（第 {} 次）", attempt);
        }
        tokio::time::sleep(STORAGE_WAIT_INTERVAL).await;
    }
}

async fn start_update_loop(app: AppHandle) -> Result<(), String> {
    let version = app.package_info().version.to_string();
    let storage = wait_for_storage(&app).await?;

    loop {
        match storage.get_update_config().await {
            Ok(config) => {
                if !config.auto_check {
                    warn!("自动更新检查已关闭，跳过本轮后台检查");
                } else {
                    match check_update(
                        version.clone(),
                        Some(config.accept_prerelease),
                        config.update_channel.clone(),
                    )
                    .await
                    {
                        Ok(info) => {
                            handle_update_result(&app, &config.skip_version, info).await
                        }
                        Err(e) => warn!("后台检查更新失败: {}", e),
                    }
                }
            }
            Err(e) => warn!("读取更新配置失败: {}", e),
        }

        tokio::time::sleep(UPDATE_CHECK_INTERVAL).await;
    }
}

async fn handle_update_result(app: &AppHandle, skip_version: &Option<String>, info: UpdateInfo) {
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
            info!("后台检测到新版本 {}，已推送事件", info.latest_version);
        }
    } else {
        info!("后台检查：当前已是最新版本");
    }
}

async fn start_kernel_health_loop(app: AppHandle) -> Result<(), String> {
    // 与更新检查共享同一个存储就绪门禁，避免两套行为不一致。
    wait_for_storage(&app).await?;

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
