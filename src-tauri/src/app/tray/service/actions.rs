//! 托盘触发的业务动作（重启内核 / 切系统代理 / 切 TUN）

use super::super::model::events;
use super::runtime_state::{
    derive_proxy_mode, queue_proxy_toggle_for_frontend, refresh_runtime_state_from_backend,
    with_state_write,
};
use crate::app::core::kernel_service::runtime::{apply_proxy_settings, kernel_restart_fast};
use crate::app::storage::enhanced_storage_service::db_save_app_config_internal;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tracing::{info, warn};

pub(super) async fn restart_kernel_from_tray(app: &AppHandle) -> Result<(), String> {
    let result = kernel_restart_fast(
        app.clone(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await?;

    if !result
        .get("success")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return Err(result
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("重启内核失败")
            .to_string());
    }

    refresh_runtime_state_from_backend(app, false).await
}

pub(super) async fn apply_system_proxy_toggle_from_tray(
    app: &AppHandle,
    enabled: bool,
) -> Result<(), String> {
    let result = apply_proxy_settings(app.clone(), Some(enabled), None).await?;
    if !result
        .get("success")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return Err(result
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("应用系统代理配置失败")
            .to_string());
    }

    let mut app_config =
        crate::app::storage::enhanced_storage_service::db_get_app_config_internal(app).await?;
    app_config.system_proxy_enabled = enabled;
    app_config.proxy_mode =
        derive_proxy_mode(app_config.system_proxy_enabled, app_config.tun_enabled);
    db_save_app_config_internal(app_config, app).await?;
    refresh_runtime_state_from_backend(app, false).await
}

pub(super) async fn apply_tun_toggle_from_tray(
    app: &AppHandle,
    enabled: bool,
) -> Result<(), String> {
    if !enabled {
        let apply_result = apply_proxy_settings(app.clone(), None, Some(false)).await?;
        if !apply_result
            .get("success")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            return Err(apply_result
                .get("message")
                .and_then(|value| value.as_str())
                .unwrap_or("应用 TUN 关闭配置失败")
                .to_string());
        }

        let restart_result = kernel_restart_fast(
            app.clone(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(false),
        )
        .await?;
        if !restart_result
            .get("success")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            return Err(restart_result
                .get("message")
                .and_then(|value| value.as_str())
                .unwrap_or("关闭 TUN 后重启内核失败")
                .to_string());
        }

        let mut app_config =
            crate::app::storage::enhanced_storage_service::db_get_app_config_internal(app).await?;
        app_config.tun_enabled = false;
        app_config.proxy_mode =
            derive_proxy_mode(app_config.system_proxy_enabled, app_config.tun_enabled);
        db_save_app_config_internal(app_config, app).await?;
        return refresh_runtime_state_from_backend(app, false).await;
    }

    #[cfg(target_os = "windows")]
    {
        if !crate::app::system::system_service::check_admin() {
            return queue_proxy_toggle_for_frontend(app, "tun", true);
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let sudo_status =
            crate::app::system::sudo_service::sudo_password_status(app.clone()).await?;
        if !sudo_status.supported {
            return Err("当前平台暂不支持该操作".to_string());
        }

        if !sudo_status.has_saved {
            return queue_proxy_toggle_for_frontend(app, "tun", true);
        }
    }

    let apply_result = apply_proxy_settings(app.clone(), None, Some(true)).await?;
    if !apply_result
        .get("success")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return Err(apply_result
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("应用 TUN 配置失败")
            .to_string());
    }

    let restart_result = kernel_restart_fast(
        app.clone(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(true),
    )
    .await?;
    if !restart_result
        .get("success")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        let message = restart_result
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("启用 TUN 后重启内核失败")
            .to_string();

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if message.contains(crate::app::system::sudo_service::SUDO_PASSWORD_REQUIRED)
            || message.contains(crate::app::system::sudo_service::SUDO_PASSWORD_INVALID)
        {
            return queue_proxy_toggle_for_frontend(app, "tun", true);
        }

        return Err(message);
    }

    let mut app_config =
        crate::app::storage::enhanced_storage_service::db_get_app_config_internal(app).await?;
    app_config.tun_enabled = true;
    app_config.proxy_mode =
        derive_proxy_mode(app_config.system_proxy_enabled, app_config.tun_enabled);
    db_save_app_config_internal(app_config, app).await?;
    refresh_runtime_state_from_backend(app, false).await
}

pub fn request_app_exit(app: &AppHandle) -> Result<(), String> {
    let _ = app.emit(events::ACTION_EXIT_REQUESTED, ());
    with_state_write(|state| {
        state.allow_app_exit = true;
        state.keep_alive_without_windows = false;
    });

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        match tokio::time::timeout(
            Duration::from_secs(4),
            crate::app::core::kernel_service::runtime::stop_kernel(None),
        )
        .await
        {
            Ok(Ok(message)) => info!("退出前停止内核成功: {}", message),
            Ok(Err(err)) => warn!("退出前停止内核失败，继续退出: {}", err),
            Err(_) => warn!("退出前停止内核超时，继续退出"),
        }

        app_handle.exit(0);
    });

    Ok(())
}
