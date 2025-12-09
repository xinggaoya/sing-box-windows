use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

use crate::app::network::subscription_service::{download_subscription, get_current_config};
use crate::app::storage::enhanced_storage_service::{db_get_app_config, db_get_subscriptions};

// 默认 12 小时
const DEFAULT_INTERVAL_MINUTES: u64 = 12 * 60;

pub async fn start_subscription_auto_update(app: &AppHandle) {
    let handle = app.clone();
    tauri::async_runtime::spawn(async move {
        loop {
            if let Err(e) = run_once(&handle).await {
                warn!("自动订阅刷新失败: {}", e);
            }

            let interval = get_min_interval_minutes(&handle).await.unwrap_or(DEFAULT_INTERVAL_MINUTES);
            tokio::time::sleep(Duration::from_secs(interval * 60)).await;
        }
    });
}

async fn get_min_interval_minutes(app: &AppHandle) -> Result<u64, String> {
    let subs = db_get_subscriptions(app.clone())
        .await
        .map_err(|e| format!("读取订阅配置失败: {}", e))?;

    let mut min_interval = DEFAULT_INTERVAL_MINUTES;
    for sub in subs.iter() {
        if let Some(interval) = sub.auto_update_interval_minutes {
            if interval < min_interval {
                min_interval = interval;
            }
        }
    }
    Ok(min_interval.max(5)) // 至少 5 分钟
}

async fn run_once(app: &AppHandle) -> Result<(), String> {
    let subs = db_get_subscriptions(app.clone())
        .await
        .map_err(|e| format!("读取订阅配置失败: {}", e))?;

    let app_config = db_get_app_config(app.clone())
        .await
        .map_err(|e| format!("读取应用配置失败: {}", e))?;

    for sub in subs {
        let interval = sub
            .auto_update_interval_minutes
            .unwrap_or(DEFAULT_INTERVAL_MINUTES);
        if interval == 0 {
            continue;
        }

        info!("自动刷新订阅: {}", sub.name);
        let window = match app.get_window("main") {
            Some(w) => w,
            None => {
                warn!("未找到 main 窗口，跳过订阅自动刷新");
                continue;
            }
        };

        match download_subscription(
            sub.url.clone(),
            sub.use_original_config,
            Some(format!("{}.json", sub.name)),
            sub.config_path.clone(),
            Some(true),
            window,
            Some(app_config.proxy_port),
            Some(app_config.api_port),
        )
        .await
        {
            Ok(_) => info!("自动刷新订阅 {} 完成", sub.name),
            Err(e) => warn!("自动刷新订阅 {} 失败: {}", sub.name, e),
        };
    }

    // 触发前端刷新当前配置
    if let Ok(cfg) = get_current_config(app.clone()).await {
        let _ = app.emit("subscription-updated", cfg);
    }

    Ok(())
}
