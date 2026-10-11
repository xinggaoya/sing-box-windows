//! 托盘运行态读写、快照合并与后端同步
//!
//! 托盘图标、菜单文案、窗口显隐、退出判定全都读同一份 `TrayRuntimeState`；
//! 这里提供状态读写助手与「从后端 AppConfig 同步快照」的唯一实现。

use super::super::model::{events, TrayCloseBehavior, TrayToggleProxyFeaturePayload};
use super::super::state::TrayRuntimeState;
use super::menu::{refresh_tray, TRAY_RUNTIME_STATE};
use super::window::show_main_window;
use crate::app::core::kernel_service::status::is_kernel_running_ref;
use crate::app::storage::state_model::AppConfig;
use tauri::{AppHandle, Emitter, Manager};

pub(super) fn with_state_read<T>(f: impl FnOnce(&TrayRuntimeState) -> T) -> T {
    let guard = TRAY_RUNTIME_STATE
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&guard)
}

pub(super) fn with_state_write<T>(f: impl FnOnce(&mut TrayRuntimeState) -> T) -> T {
    let mut guard = TRAY_RUNTIME_STATE
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut guard)
}

pub(super) fn derive_proxy_mode(system_proxy_enabled: bool, tun_enabled: bool) -> String {
    if tun_enabled {
        "tun".to_string()
    } else if system_proxy_enabled {
        "system".to_string()
    } else {
        "manual".to_string()
    }
}

pub(super) fn apply_backend_runtime_snapshot(
    state: &mut TrayRuntimeState,
    app_config: &AppConfig,
    kernel_running: bool,
) -> bool {
    let mut changed = false;
    let close_behavior = TrayCloseBehavior::from_raw(&app_config.tray_close_behavior);

    if state.system_proxy_enabled != app_config.system_proxy_enabled {
        state.system_proxy_enabled = app_config.system_proxy_enabled;
        changed = true;
    }
    if state.tun_enabled != app_config.tun_enabled {
        state.tun_enabled = app_config.tun_enabled;
        changed = true;
    }
    if state.kernel_running != kernel_running {
        state.kernel_running = kernel_running;
        changed = true;
    }
    if state.close_behavior != close_behavior {
        state.close_behavior = close_behavior;
        changed = true;
    }

    changed
}

pub async fn refresh_runtime_state_from_backend(
    app: &AppHandle,
    force_refresh: bool,
) -> Result<(), String> {
    let app_config =
        crate::app::storage::enhanced_storage_service::db_get_app_config_internal(app).await?;
    let kernel_running = is_kernel_running_ref(app).await.unwrap_or(false);
    let changed = with_state_write(|state| {
        apply_backend_runtime_snapshot(state, &app_config, kernel_running)
    });

    if changed || force_refresh {
        refresh_tray(app)?;
    }

    let _ = app.emit(events::RUNTIME_STATE_UPDATED, ());
    Ok(())
}

pub(super) fn queue_proxy_toggle_for_frontend(
    app: &AppHandle,
    feature: &str,
    enabled: bool,
) -> Result<(), String> {
    let payload = TrayToggleProxyFeaturePayload {
        feature: feature.to_string(),
        enabled,
    };

    if app.get_webview_window("main").is_some() {
        show_main_window(app, true)?;
        app.emit(events::ACTION_SWITCH_PROXY_MODE, payload)
            .map_err(|e| format!("发送托盘代理切换事件失败: {}", e))?;
        return Ok(());
    }

    with_state_write(|state| {
        state.set_pending_proxy_toggle(payload);
    });
    show_main_window(app, true)
}
