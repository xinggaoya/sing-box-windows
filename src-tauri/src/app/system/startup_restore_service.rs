use crate::app::storage::state_model::{AppConfig, Subscription};
use crate::app::storage::{db_save_app_config_internal, get_enhanced_storage};
use tauri::AppHandle;
use tracing::info;

pub fn resolve_startup_active_config_path(
    app_config: &AppConfig,
    subscriptions: &[Subscription],
    active_subscription_index: Option<i64>,
) -> Option<String> {
    if let Some(active_config_path) = app_config.active_config_path.as_ref() {
        let normalized = active_config_path.trim();
        if !normalized.is_empty() {
            return Some(normalized.to_string());
        }
    }

    let resolved_from_index = active_subscription_index
        .and_then(|index| usize::try_from(index).ok())
        .and_then(|index| subscriptions.get(index))
        .and_then(|subscription| subscription.config_path.as_ref())
        .map(|config_path| config_path.trim().to_string())
        .filter(|config_path| !config_path.is_empty());

    if resolved_from_index.is_some() {
        return resolved_from_index;
    }

    let mut candidates = subscriptions
        .iter()
        .filter_map(|subscription| subscription.config_path.as_ref())
        .map(|config_path| config_path.trim())
        .filter(|config_path| !config_path.is_empty());

    let first = candidates.next()?;
    if candidates.next().is_none() {
        Some(first.to_string())
    } else {
        None
    }
}

pub async fn prepare_startup_restore(app_handle: &AppHandle) -> Result<Option<String>, String> {
    let storage = get_enhanced_storage(app_handle).await?;
    let mut app_config = storage.get_app_config().await.map_err(|e| e.to_string())?;
    let subscriptions = storage
        .get_subscriptions()
        .await
        .map_err(|e| e.to_string())?;
    let active_subscription_index = storage
        .get_active_subscription_index()
        .await
        .map_err(|e| e.to_string())?;

    let resolved_active_config_path =
        resolve_startup_active_config_path(&app_config, &subscriptions, active_subscription_index);

    if app_config.active_config_path != resolved_active_config_path {
        info!(
            "启动恢复阶段修正 active_config_path: {:?} -> {:?}",
            app_config.active_config_path, resolved_active_config_path
        );
        app_config.active_config_path = resolved_active_config_path.clone();
        db_save_app_config_internal(app_config, app_handle).await?;
    }

    Ok(resolved_active_config_path)
}

#[cfg(test)]
#[path = "startup_restore_service.tests.rs"]
mod tests;
