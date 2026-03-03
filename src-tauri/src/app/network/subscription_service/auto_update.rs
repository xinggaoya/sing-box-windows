use std::time::Duration;

use tauri::{AppHandle, Emitter, Manager};
use tracing::{info, warn};

use crate::app::network::subscription_service::{download_subscription, get_current_config};
use crate::app::storage::enhanced_storage_service::{
    db_get_app_config, db_get_subscriptions, db_save_subscriptions,
};
use crate::app::storage::state_model::Subscription;

// 默认 12 小时
const DEFAULT_INTERVAL_MINUTES: u64 = 12 * 60;
const MAX_BACKOFF_MINUTES: u64 = 24 * 60;

#[derive(Debug, Clone)]
struct SubscriptionHealthPatch {
    config_path: Option<String>,
    url: String,
    fail_count: u32,
    last_attempt_ms: u64,
    last_error: Option<String>,
    last_error_type: Option<String>,
    backoff_until_ms: Option<u64>,
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn classify_error(error: &str) -> &'static str {
    let lower = error.to_ascii_lowercase();
    if lower.contains("timeout") || lower.contains("timed out") {
        return "timeout";
    }
    if lower.contains("dns") || lower.contains("resolve") {
        return "network_dns";
    }
    if lower.contains("401") || lower.contains("403") || lower.contains("unauthorized") {
        return "auth";
    }
    if lower.contains("json") || lower.contains("yaml") || lower.contains("配置") {
        return "config_parse";
    }
    if lower.contains("connect")
        || lower.contains("network")
        || lower.contains("connection")
        || lower.contains("tls")
    {
        return "network";
    }
    "unknown"
}

fn calc_backoff_minutes(base_interval_minutes: u64, fail_count: u32) -> u64 {
    let base = base_interval_minutes.max(5);
    let exp = fail_count.saturating_sub(1).min(6);
    let factor = 2_u64.pow(exp);
    (base.saturating_mul(factor)).min(MAX_BACKOFF_MINUTES)
}

fn should_run_for_subscription(sub: &Subscription, now_ms: u64) -> bool {
    let interval = sub
        .auto_update_interval_minutes
        .unwrap_or(DEFAULT_INTERVAL_MINUTES);
    if interval == 0 {
        return false;
    }

    if let Some(backoff_until_ms) = sub.last_auto_update_backoff_until {
        if now_ms < backoff_until_ms {
            return false;
        }
    }

    let last_ref = sub
        .last_auto_update_attempt
        .or(sub.last_update)
        .unwrap_or(0);
    if last_ref == 0 {
        return true;
    }

    now_ms.saturating_sub(last_ref) >= interval.max(5) * 60 * 1000
}

fn subscription_matches_patch(sub: &Subscription, patch: &SubscriptionHealthPatch) -> bool {
    let url_match = !patch.url.is_empty() && sub.url.trim() == patch.url;
    let path_match = match (&patch.config_path, &sub.config_path) {
        (Some(lhs), Some(rhs)) => lhs == rhs,
        _ => false,
    };
    path_match || url_match
}

fn apply_health_patch(sub: &mut Subscription, patch: &SubscriptionHealthPatch) {
    sub.auto_update_fail_count = Some(patch.fail_count);
    sub.last_auto_update_attempt = Some(patch.last_attempt_ms);
    sub.last_auto_update_error = patch.last_error.clone();
    sub.last_auto_update_error_type = patch.last_error_type.clone();
    sub.last_auto_update_backoff_until = patch.backoff_until_ms;
}

async fn save_health_patches(app: &AppHandle, patches: &[SubscriptionHealthPatch]) -> Result<(), String> {
    if patches.is_empty() {
        return Ok(());
    }

    let mut latest = db_get_subscriptions(app.clone())
        .await
        .map_err(|e| format!("读取订阅配置失败: {}", e))?;

    for sub in latest.iter_mut() {
        if let Some(patch) = patches
            .iter()
            .find(|patch| subscription_matches_patch(sub, patch))
        {
            apply_health_patch(sub, patch);
        }
    }

    db_save_subscriptions(latest, app.clone())
        .await
        .map_err(|e| format!("保存订阅健康状态失败: {}", e))
}

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

    let now_ms = now_millis();
    let mut health_patches: Vec<SubscriptionHealthPatch> = Vec::new();
    for sub in subs {
        let interval = sub
            .auto_update_interval_minutes
            .unwrap_or(DEFAULT_INTERVAL_MINUTES);
        if interval == 0 {
            continue;
        }
        if !should_run_for_subscription(&sub, now_ms) {
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
        let patch_base = SubscriptionHealthPatch {
            config_path: sub.config_path.clone(),
            url: sub.url.trim().to_string(),
            fail_count: sub.auto_update_fail_count.unwrap_or(0),
            last_attempt_ms: now_ms,
            last_error: None,
            last_error_type: None,
            backoff_until_ms: None,
        };

        // 关键修复：
        // 以前这里固定传 apply_runtime = true，会导致“依次刷新每个订阅时不断覆盖 active_config_path”，
        // 最终内核会使用最后一个被刷新的订阅配置（订阅数量 >= 2 时尤为明显）。
        // 现在仅当当前订阅就是用户正在使用的订阅时，才允许应用到运行时（写入 active_config_path + 触发自动拉起/重启）。
        let should_apply_runtime = match (&app_config.active_config_path, &sub.config_path) {
            (Some(active), Some(sub_path)) => active == sub_path,
            _ => false,
        };

        match download_subscription(
            sub.url.clone(),
            sub.use_original_config,
            Some(format!("{}.json", sub.name)),
            sub.config_path.clone(),
            Some(should_apply_runtime),
            window,
            Some(app_config.proxy_port),
            Some(app_config.api_port),
        )
        .await
        {
            Ok(_) => {
                info!("自动刷新订阅 {} 完成", sub.name);
                let mut patch = patch_base.clone();
                patch.fail_count = 0;
                patch.last_error = None;
                patch.last_error_type = None;
                patch.backoff_until_ms = None;
                health_patches.push(patch);
            }
            Err(e) => {
                warn!("自动刷新订阅 {} 失败: {}", sub.name, e);
                let prev_fail = patch_base.fail_count;
                let next_fail = prev_fail.saturating_add(1);
                let backoff_minutes = calc_backoff_minutes(interval, next_fail);
                let mut patch = patch_base;
                patch.fail_count = next_fail;
                patch.last_error = Some(e.clone());
                patch.last_error_type = Some(classify_error(&e).to_string());
                patch.backoff_until_ms = Some(now_ms + backoff_minutes * 60 * 1000);
                health_patches.push(patch);
            }
        };
    }

    // 仅回写自动更新健康字段，避免覆盖下载流程刚更新的流量额度等字段。
    if let Err(e) = save_health_patches(app, &health_patches).await {
        warn!("回写订阅健康状态失败: {}", e);
    }

    // 触发前端刷新当前配置
    if let Ok(cfg) = get_current_config(app.clone()).await {
        let _ = app.emit("subscription-updated", cfg);
    }

    Ok(())
}
