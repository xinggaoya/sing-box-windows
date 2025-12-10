use crate::app::constants::common::messages;
use crate::app::core::kernel_service::event::{
    cleanup_event_relay_tasks, start_websocket_relay, SHOULD_STOP_EVENTS,
};
use crate::app::core::kernel_service::guard::{disable_kernel_guard, enable_kernel_guard};
use crate::app::core::kernel_service::status::is_kernel_running;
use crate::app::core::kernel_service::utils::{
    emit_kernel_error, emit_kernel_started, emit_kernel_starting, emit_kernel_stopped,
    resolve_config_path,
};
use crate::app::core::kernel_service::PROCESS_MANAGER;
use crate::app::core::proxy_service::{
    apply_proxy_runtime_state, update_dns_strategy, ProxyRuntimeState,
};
use crate::app::core::tun_profile::TunProxyOptions;
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use serde_json::json;
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Notify;
use tracing::{error, info, warn};

lazy_static::lazy_static! {
    pub(super) static ref KERNEL_READY_NOTIFY: Arc<Notify> = Arc::new(Notify::new());
}

#[derive(Debug, Clone, Default)]
pub struct ProxyOverrides {
    pub proxy_mode: Option<String>,
    pub api_port: Option<u16>,
    pub proxy_port: Option<u16>,
    pub prefer_ipv6: Option<bool>,
    pub system_proxy_bypass: Option<String>,
    pub tun_options: Option<TunProxyOptions>,
    pub system_proxy_enabled: Option<bool>,
    pub tun_enabled: Option<bool>,
    pub keep_alive: Option<bool>,
}

#[derive(Debug, Clone)]
pub struct ResolvedProxyState {
    pub proxy: ProxyRuntimeState,
    pub api_port: u16,
    pub prefer_ipv6: bool,
}

impl ResolvedProxyState {
    fn derived_mode(&self) -> String {
        self.proxy.derived_mode()
    }
}

pub async fn resolve_proxy_runtime_state(
    app_handle: &AppHandle,
    overrides: ProxyOverrides,
) -> Result<ResolvedProxyState, String> {
    let mut app_config = db_get_app_config(app_handle.clone()).await?;

    if let Some(api_port) = overrides.api_port {
        app_config.api_port = api_port;
    }
    if let Some(proxy_port) = overrides.proxy_port {
        app_config.proxy_port = proxy_port;
    }
    if let Some(prefer_ipv6) = overrides.prefer_ipv6 {
        app_config.prefer_ipv6 = prefer_ipv6;
    }

    if let Some(proxy_mode) = overrides.proxy_mode {
        match proxy_mode.as_str() {
            "system" => {
                app_config.system_proxy_enabled = true;
                app_config.tun_enabled = false;
            }
            "tun" => {
                app_config.system_proxy_enabled = false;
                app_config.tun_enabled = true;
            }
            _ => {
                app_config.system_proxy_enabled = false;
                app_config.tun_enabled = false;
            }
        }
    }

    if let Some(enabled) = overrides.system_proxy_enabled {
        app_config.system_proxy_enabled = enabled;
    }
    if let Some(enabled) = overrides.tun_enabled {
        app_config.tun_enabled = enabled;
    }

    let tun_options = overrides.tun_options.unwrap_or_else(|| TunProxyOptions {
        ipv4_address: app_config.tun_ipv4.clone(),
        ipv6_address: app_config.tun_ipv6.clone(),
        mtu: app_config.tun_mtu,
        auto_route: app_config.tun_auto_route,
        strict_route: app_config.tun_strict_route,
        stack: app_config.tun_stack.clone(),
        enable_ipv6: app_config.tun_enable_ipv6,
        interface_name: None,
    });

    let proxy_state = ProxyRuntimeState {
        proxy_port: app_config.proxy_port,
        system_proxy_enabled: app_config.system_proxy_enabled,
        tun_enabled: app_config.tun_enabled,
        system_proxy_bypass: overrides
            .system_proxy_bypass
            .unwrap_or_else(|| app_config.system_proxy_bypass.clone()),
        tun_options,
    };

    Ok(ResolvedProxyState {
        proxy: proxy_state,
        api_port: app_config.api_port,
        prefer_ipv6: app_config.prefer_ipv6,
    })
}

pub async fn start_kernel_with_state(
    app_handle: AppHandle,
    resolved: &ResolvedProxyState,
) -> Result<serde_json::Value, String> {
    info!(
        "?? 启动内核增强版，代理模式: {}, API端口: {}, 代理端口: {}",
        resolved.derived_mode(),
        resolved.api_port,
        resolved.proxy.proxy_port
    );

    emit_kernel_starting(
        &app_handle,
        &resolved.derived_mode(),
        resolved.api_port,
        resolved.proxy.proxy_port,
    );

    crate::app::system::config_service::ensure_singbox_config(&app_handle)
        .await
        .map_err(|e| format!("准备内核配置失败: {}", e))?;
    if let Err(e) = crate::app::system::config_service::update_singbox_ports(
        app_handle.clone(),
        resolved.proxy.proxy_port,
        resolved.api_port,
    )
    .await
    {
        warn!("更新端口配置失败: {}", e);
    }

    if let Err(e) = apply_proxy_runtime_state(&app_handle, &resolved.proxy).await {
        return Ok(json!({
            "success": false,
            "message": format!("应用代理配置失败: {}", e)
        }));
    }

    if let Err(e) = update_dns_strategy(&app_handle, resolved.prefer_ipv6).await {
        warn!("更新DNS策略失败: {}", e);
    }

    if is_kernel_running().await.unwrap_or(false) {
        enable_kernel_guard(app_handle.clone(), resolved.api_port, resolved.proxy.tun_enabled).await;
        info!("内核已在运行中");
        return Ok(serde_json::json!({
            "success": true,
            "message": "内核已在运行中".to_string()
        }));
    }

    let config_path = resolve_config_path(&app_handle).await?;

    match PROCESS_MANAGER.start(&config_path, resolved.proxy.tun_enabled).await {
        Ok(_) => {
            info!("? 内核进程启动成功");

            info!("?? 启动事件中继服务，端口: {}", resolved.api_port);
            match start_websocket_relay(app_handle.clone(), Some(resolved.api_port)).await {
                Ok(_) => {
                    info!("? 事件中继启动成功");

                    enable_kernel_guard(app_handle.clone(), resolved.api_port, resolved.proxy.tun_enabled).await;

                    emit_kernel_started(
                        &app_handle,
                        &resolved.derived_mode(),
                        resolved.api_port,
                        resolved.proxy.proxy_port,
                        false,
                    );

                    Ok(serde_json::json!({
                        "success": true,
                        "message": "内核启动成功，事件中继已启动".to_string()
                    }))
                }
                Err(e) => {
                    warn!("?? 事件中继启动失败: {}, 但内核进程已启动", e);

                    enable_kernel_guard(app_handle.clone(), resolved.api_port, resolved.proxy.tun_enabled).await;

                    let _ = app_handle.emit("kernel-ready", ());

                    Ok(serde_json::json!({
                        "success": true,
                        "message": "内核启动成功，但事件中继启动失败".to_string()
                    }))
                }
            }
        }
        Err(e) => {
            error!("? 内核启动失败: {}", e);

            emit_kernel_error(&app_handle, &format!("启动失败: {}", e));

            Ok(serde_json::json!({
                "success": false,
                "message": format!("内核启动失败: {}", e)
            }))
        }
    }
}

#[tauri::command]
pub async fn kernel_start_enhanced(
    app_handle: AppHandle,
    proxy_mode: Option<String>,
    api_port: Option<u16>,
    proxy_port: Option<u16>,
    prefer_ipv6: Option<bool>,
    system_proxy_bypass: Option<String>,
    tun_options: Option<TunProxyOptions>,
    keep_alive: Option<bool>,
    system_proxy_enabled: Option<bool>,
    tun_enabled: Option<bool>,
) -> Result<serde_json::Value, String> {
    let overrides = ProxyOverrides {
        proxy_mode,
        api_port,
        proxy_port,
        prefer_ipv6,
        system_proxy_bypass,
        tun_options,
        system_proxy_enabled,
        tun_enabled,
        keep_alive,
    };

    let resolved = resolve_proxy_runtime_state(&app_handle, overrides).await?;
    start_kernel_with_state(app_handle, &resolved).await
}

#[tauri::command]
pub async fn apply_proxy_settings(
    app_handle: AppHandle,
    system_proxy_enabled: Option<bool>,
    tun_enabled: Option<bool>,
) -> Result<serde_json::Value, String> {
    let overrides = ProxyOverrides {
        system_proxy_enabled,
        tun_enabled,
        ..Default::default()
    };

    let resolved = resolve_proxy_runtime_state(&app_handle, overrides).await?;

    if let Err(e) = apply_proxy_runtime_state(&app_handle, &resolved.proxy).await {
        return Ok(json!({
            "success": false,
            "message": format!("应用代理配置失败: {}", e)
        }));
    }

    if let Err(e) = update_dns_strategy(&app_handle, resolved.prefer_ipv6).await {
        warn!("更新DNS策略失败: {}", e);
    }

    Ok(json!({
        "success": true,
        "mode": resolved.derived_mode(),
        "system_proxy_enabled": resolved.proxy.system_proxy_enabled,
        "tun_enabled": resolved.proxy.tun_enabled
    }))
}

#[tauri::command]
pub async fn kernel_stop_enhanced(app_handle: AppHandle) -> Result<serde_json::Value, String> {
    info!("?? 停止内核增强版");

    disable_kernel_guard().await;

    match stop_kernel().await {
        Ok(_) => {
            emit_kernel_stopped(&app_handle);

            Ok(serde_json::json!({
                "success": true,
                "message": "内核停止成功".to_string()
            }))
        }
        Err(e) => {
            emit_kernel_error(&app_handle, &format!("停止失败: {}", e));

            Ok(serde_json::json!({
                "success": false,
                "message": format!("内核停止失败: {}", e)
            }))
        }
    }
}

#[tauri::command]
pub async fn kernel_stop_background(app_handle: AppHandle) -> Result<serde_json::Value, String> {
    info!("?? 后台请求停止内核（快速返回）");

    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        let stop_result = tokio::time::timeout(Duration::from_secs(6), stop_kernel()).await;
        match stop_result {
            Ok(Ok(_)) => info!("? 后台停止内核完成"),
            Ok(Err(e)) => {
                error!("? 后台停止内核失败: {}", e);
                emit_kernel_error(&handle, &format!("停止失败: {}", e));
            }
            Err(_) => {
                warn!("? 停止内核超时，尝试强制清理");
                if let Err(e) = PROCESS_MANAGER.kill_existing_processes().await {
                    error!("强制清理内核进程失败: {}", e);
                }
            }
        }

        emit_kernel_stopped(&handle);
    });

    Ok(json!({
        "success": true,
        "message": "已在后台请求停止内核"
    }))
}

#[tauri::command]
pub async fn force_stop_and_exit(app_handle: AppHandle) -> Result<serde_json::Value, String> {
    info!("?? 收到强制退出请求，后台停止内核并退出应用");

    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        SHOULD_STOP_EVENTS.store(true, std::sync::atomic::Ordering::Relaxed);
        cleanup_event_relay_tasks().await;

        if let Err(e) = PROCESS_MANAGER.kill_existing_processes().await {
            error!("强制清理内核进程失败: {}", e);
        }

        emit_kernel_stopped(&handle);

        handle.exit(0);
    });

    Ok(json!({
        "success": true,
        "message": "正在后台停止内核并退出"
    }))
}

pub async fn stop_kernel() -> Result<String, String> {
    disable_kernel_guard().await;
    SHOULD_STOP_EVENTS.store(true, std::sync::atomic::Ordering::Relaxed);
    cleanup_event_relay_tasks().await;

    PROCESS_MANAGER
        .stop()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_PROCESS_STOP_FAILED, e))?;

    tokio::time::sleep(Duration::from_secs(2)).await;

    if !is_kernel_running().await.unwrap_or(true) {
        info!("? 内核停止成功");
        Ok("内核停止成功".to_string())
    } else {
        Err(messages::ERR_PROCESS_STOP_FAILED.to_string())
    }
}
