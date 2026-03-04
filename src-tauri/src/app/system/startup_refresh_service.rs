use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

use crate::app::network::subscription_service::{add_manual_subscription, download_subscription};
use crate::app::storage::enhanced_storage_service::{
    db_get_app_config, db_get_subscriptions, get_enhanced_storage,
};
use crate::app::storage::state_model::{AppConfig, Subscription};

const LAST_LAUNCHED_APP_VERSION_KEY: &str = "last_launched_app_version";
const RETRY_DELAYS_SECONDS: &[u64] = &[30, 120, 600];
const IMMEDIATE_TIMEOUT_SECONDS: u64 = 10;
const RETRY_TIMEOUT_SECONDS: u64 = 30;
const FAILURE_EVENT: &str = "upgrade-subscription-refresh-failed";

#[derive(Debug, Clone, serde::Serialize)]
struct UpgradeRefreshFailedPayload {
    message: String,
    version: String,
    active_config_path: String,
    attempts: usize,
    last_error: String,
}

pub async fn start_upgrade_subscription_refresh(app: &AppHandle) {
    if let Err(e) = run_upgrade_subscription_refresh(app).await {
        warn!("启动升级后订阅刷新流程失败: {}", e);
    }
}

async fn run_upgrade_subscription_refresh(app: &AppHandle) -> Result<(), String> {
    let current_version = app.package_info().version.to_string();
    let last_launched_version = load_last_launched_version(app).await?;

    if last_launched_version.as_deref() == Some(current_version.as_str()) {
        info!("应用版本未变化，跳过升级后订阅刷新: {}", current_version);
        return Ok(());
    }

    let Some(active_config_path) = resolve_active_subscription_config_path(app).await? else {
        // 没有可刷新的活动订阅时，记录版本避免无意义重复执行
        save_last_launched_version(app, &current_version).await?;
        info!("当前无可刷新的活动订阅，已记录版本: {}", current_version);
        return Ok(());
    };

    info!(
        "检测到应用版本变化，将尝试刷新当前活动订阅: {:?} ({} -> {})",
        active_config_path,
        last_launched_version.unwrap_or_else(|| "unknown".to_string()),
        current_version
    );

    let immediate_result = tokio::time::timeout(
        Duration::from_secs(IMMEDIATE_TIMEOUT_SECONDS),
        refresh_subscription_by_config_path(app, &active_config_path, false),
    )
    .await;

    match immediate_result {
        Ok(Ok(())) => {
            info!("升级后首次订阅刷新成功: {}", active_config_path);
            save_last_launched_version(app, &current_version).await?;
            return Ok(());
        }
        Ok(Err(e)) => {
            warn!("升级后首次订阅刷新失败，将进入后台重试: {}", e);
            spawn_retry_task(app.clone(), current_version, active_config_path, e);
        }
        Err(_) => {
            let timeout_error = format!("升级后首次订阅刷新超时（{}s）", IMMEDIATE_TIMEOUT_SECONDS);
            warn!("{}", timeout_error);
            spawn_retry_task(
                app.clone(),
                current_version,
                active_config_path,
                timeout_error,
            );
        }
    }

    Ok(())
}

fn spawn_retry_task(
    app: AppHandle,
    current_version: String,
    active_config_path: String,
    initial_error: String,
) {
    tauri::async_runtime::spawn(async move {
        let mut last_error = initial_error;

        for (idx, delay_secs) in RETRY_DELAYS_SECONDS.iter().enumerate() {
            tokio::time::sleep(Duration::from_secs(*delay_secs)).await;
            info!(
                "升级后订阅后台重试第 {} 次（{}s 后触发）: {}",
                idx + 1,
                delay_secs,
                active_config_path
            );

            let retry_result = tokio::time::timeout(
                Duration::from_secs(RETRY_TIMEOUT_SECONDS),
                refresh_subscription_by_config_path(&app, &active_config_path, true),
            )
            .await;

            match retry_result {
                Ok(Ok(())) => {
                    info!(
                        "升级后订阅后台重试成功（第 {} 次）: {}",
                        idx + 1,
                        active_config_path
                    );
                    if let Err(e) = save_last_launched_version(&app, &current_version).await {
                        warn!("写入 last_launched_app_version 失败: {}", e);
                    }
                    return;
                }
                Ok(Err(e)) => {
                    last_error = e;
                    warn!(
                        "升级后订阅后台重试失败（第 {} 次）: {}",
                        idx + 1,
                        last_error
                    );
                }
                Err(_) => {
                    last_error = format!("后台重试超时（{}s）", RETRY_TIMEOUT_SECONDS);
                    warn!(
                        "升级后订阅后台重试超时（第 {} 次）: {}",
                        idx + 1,
                        active_config_path
                    );
                }
            }
        }

        let message =
            "应用升级后已多次尝试刷新当前订阅但仍失败，请手动在订阅页执行“立即更新配置”。";
        let payload = UpgradeRefreshFailedPayload {
            message: message.to_string(),
            version: current_version,
            active_config_path,
            attempts: RETRY_DELAYS_SECONDS.len() + 1,
            last_error,
        };

        if let Err(e) = app.emit(FAILURE_EVENT, &payload) {
            warn!("发送升级后订阅刷新失败事件失败: {}", e);
        }
    });
}

async fn resolve_active_subscription_config_path(
    app: &AppHandle,
) -> Result<Option<String>, String> {
    let app_config = db_get_app_config(app.clone()).await?;
    let Some(active_path) = app_config.active_config_path else {
        return Ok(None);
    };

    let subscriptions = db_get_subscriptions(app.clone()).await?;
    let exists = subscriptions
        .iter()
        .any(|sub| sub.config_path.as_deref() == Some(active_path.as_str()));
    if exists {
        Ok(Some(active_path))
    } else {
        Ok(None)
    }
}

async fn refresh_subscription_by_config_path(
    app: &AppHandle,
    config_path: &str,
    apply_runtime: bool,
) -> Result<(), String> {
    let app_config = db_get_app_config(app.clone()).await?;
    let subscriptions = db_get_subscriptions(app.clone()).await?;
    let Some(subscription) = subscriptions
        .into_iter()
        .find(|sub| sub.config_path.as_deref() == Some(config_path))
    else {
        return Err(format!(
            "活动配置未在订阅列表中找到，跳过自动刷新: {}",
            config_path
        ));
    };

    refresh_subscription(app, &app_config, &subscription, apply_runtime).await
}

async fn refresh_subscription(
    app: &AppHandle,
    app_config: &AppConfig,
    sub: &Subscription,
    apply_runtime: bool,
) -> Result<(), String> {
    let window = app
        .get_window("main")
        .ok_or_else(|| "无法获取主窗口，无法执行订阅刷新".to_string())?;

    let file_name = Some(format!("{}.json", sub.name));
    let config_path = sub.config_path.clone();
    let proxy_port = Some(app_config.proxy_port);
    let api_port = Some(app_config.api_port);

    if sub.is_manual {
        let content = sub
            .manual_content
            .clone()
            .ok_or_else(|| format!("手动订阅内容为空: {}", sub.name))?;
        add_manual_subscription(
            content,
            sub.use_original_config,
            file_name,
            config_path,
            Some(apply_runtime),
            window,
            proxy_port,
            api_port,
        )
        .await
        .map(|_| ())
    } else {
        let url = sub.url.trim().to_string();
        if url.is_empty() {
            return Err(format!("订阅 URL 为空: {}", sub.name));
        }
        download_subscription(
            url,
            sub.use_original_config,
            file_name,
            config_path,
            Some(apply_runtime),
            window,
            proxy_port,
            api_port,
        )
        .await
        .map(|_| ())
    }
}

async fn load_last_launched_version(app: &AppHandle) -> Result<Option<String>, String> {
    let storage = get_enhanced_storage(app).await?;
    storage
        .get_config::<String>(LAST_LAUNCHED_APP_VERSION_KEY)
        .await
        .map_err(|e| format!("读取启动版本标记失败: {}", e))
}

async fn save_last_launched_version(app: &AppHandle, version: &str) -> Result<(), String> {
    let storage = get_enhanced_storage(app).await?;
    storage
        .save_config(LAST_LAUNCHED_APP_VERSION_KEY, &version.to_string())
        .await
        .map_err(|e| format!("保存启动版本标记失败: {}", e))
}
