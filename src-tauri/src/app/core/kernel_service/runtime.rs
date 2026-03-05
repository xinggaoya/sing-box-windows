use crate::app::constants::common::messages;
use crate::app::core::kernel_service::event::{
    cleanup_event_relay_tasks, start_websocket_relay, SHOULD_STOP_EVENTS,
};
use crate::app::core::kernel_service::guard::{disable_kernel_guard, enable_kernel_guard};
use crate::app::core::kernel_service::orchestrator::execute_kernel_operation;
use crate::app::core::kernel_service::state::{KernelState, KERNEL_STATE};
use crate::app::core::kernel_service::status::is_kernel_running;
use crate::app::core::kernel_service::utils::{
    emit_kernel_error_with_context, emit_kernel_started, emit_kernel_starting, emit_kernel_stopped,
    resolve_config_path,
};
use crate::app::core::kernel_service::PROCESS_MANAGER;
use crate::app::core::proxy_service::{
    apply_proxy_runtime_state, update_dns_strategy, ProxyRuntimeState,
};
use crate::app::core::tun_profile::TunProxyOptions;
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use crate::utils::http_client;
use futures::FutureExt;
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

async fn verify_kernel_startup_stability(api_port: u16) -> Result<(), String> {
    // 稳定性窗口：尽早识别“启动成功后立刻崩溃”的假成功场景。
    const MAX_CHECKS: u8 = 4;
    const RETRY_INTERVAL_MS: u64 = 450;
    const API_TIMEOUT_MS: u64 = 500;

    let client = http_client::get_client();
    let api_url = format!("http://127.0.0.1:{}/version", api_port);
    let mut last_error = String::new();

    for attempt in 1..=MAX_CHECKS {
        if !is_kernel_running().await.unwrap_or(false) {
            return Err("内核进程启动后立即退出".to_string());
        }

        match client
            .get(&api_url)
            .timeout(Duration::from_millis(API_TIMEOUT_MS))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => return Ok(()),
            Ok(response) => {
                last_error = format!(
                    "稳定性检查第{}次失败：API状态码 {}",
                    attempt,
                    response.status()
                );
            }
            Err(e) => {
                last_error = format!("稳定性检查第{}次失败：API连接异常 {}", attempt, e);
            }
        }

        tokio::time::sleep(Duration::from_millis(RETRY_INTERVAL_MS)).await;
    }

    if last_error.is_empty() {
        last_error = "稳定性窗口内 API 未就绪".to_string();
    }

    Err(last_error)
}

async fn try_cleanup_conflicting_kernel(app_handle: &AppHandle) -> Result<(), String> {
    let kernel_name = crate::platform::get_kernel_executable_name();
    let details = format!(
        "检测到非托管内核进程 {} 正在运行，尝试强制停止后继续启动",
        kernel_name
    );

    warn!("{}", details);
    emit_kernel_error_with_context(
        app_handle,
        "KERNEL_CONFLICT_DETECTED",
        "检测到旧内核正在运行，正在尝试强制停止后继续",
        Some(&details),
        Some("kernel.runtime.conflict"),
        true,
    );

    PROCESS_MANAGER
        .force_kill_kernel_processes_by_name()
        .await
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
    KERNEL_STATE.set_state(KernelState::Starting);

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
        .map_err(|e| {
            KERNEL_STATE.mark_failed();
            format!("准备内核配置失败: {}", e)
        })?;
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
        KERNEL_STATE.mark_failed();
        return Ok(json!({
            "success": false,
            "message": format!("应用代理配置失败: {}", e)
        }));
    }

    if let Err(e) = update_dns_strategy(&app_handle, resolved.prefer_ipv6).await {
        warn!("更新DNS策略失败: {}", e);
    }

    if PROCESS_MANAGER.is_running().await {
        KERNEL_STATE.mark_running(resolved.api_port);
        enable_kernel_guard(
            app_handle.clone(),
            resolved.api_port,
            resolved.proxy.tun_enabled,
        )
        .await;
        info!("内核已在运行中");
        return Ok(serde_json::json!({
            "success": true,
            "message": "内核已在运行中".to_string()
        }));
    }

    if is_kernel_running().await.unwrap_or(false) {
        if let Err(err) = try_cleanup_conflicting_kernel(&app_handle).await {
            KERNEL_STATE.mark_failed();
            let kernel_name = crate::platform::get_kernel_executable_name();
            let user_message = format!(
                "检测到旧内核进程且强制停止失败，请手动结束 {} 进程后重试（必要时以管理员权限运行）",
                kernel_name
            );
            emit_kernel_error_with_context(
                &app_handle,
                "KERNEL_CONFLICT_FORCE_STOP_FAILED",
                &user_message,
                Some(&err),
                Some("kernel.runtime.conflict"),
                false,
            );
            return Ok(json!({
                "success": false,
                "message": format!("内核启动失败: {}", user_message)
            }));
        }

        // 再次复核，避免平台命令执行成功但仍有残留进程占用端口。
        if is_kernel_running().await.unwrap_or(false) {
            KERNEL_STATE.mark_failed();
            let kernel_name = crate::platform::get_kernel_executable_name();
            let details = format!("强制清理后仍检测到 {} 进程在运行", kernel_name);
            let user_message = format!(
                "检测到旧内核进程未完全退出，请手动结束 {} 进程后重试",
                kernel_name
            );
            emit_kernel_error_with_context(
                &app_handle,
                "KERNEL_CONFLICT_FORCE_STOP_FAILED",
                &user_message,
                Some(&details),
                Some("kernel.runtime.conflict"),
                false,
            );
            return Ok(json!({
                "success": false,
                "message": format!("内核启动失败: {}", user_message)
            }));
        }

        info!("旧内核残留进程清理完成，继续启动新内核");
    }

    let config_path = resolve_config_path(&app_handle).await.inspect_err(|_e| {
        KERNEL_STATE.mark_failed();
    })?;

    match PROCESS_MANAGER
        .start(&app_handle, &config_path, resolved.proxy.tun_enabled)
        .await
    {
        Ok(_) => {
            info!("? 内核进程启动成功，开始稳定性校验");

            if let Err(e) = verify_kernel_startup_stability(resolved.api_port).await {
                error!("? 内核稳定性校验失败: {}", e);
                KERNEL_STATE.mark_failed();
                if let Err(stop_err) = PROCESS_MANAGER.stop().await {
                    warn!("稳定性校验失败后的进程清理失败: {}", stop_err);
                }
                emit_kernel_error_with_context(
                    &app_handle,
                    "KERNEL_START_UNSTABLE",
                    "内核启动后快速退出，请检查配置或端口占用",
                    Some(&e),
                    Some("kernel.runtime.startup_stability"),
                    true,
                );
                return Ok(serde_json::json!({
                    "success": false,
                    "message": format!("内核启动失败: {}", e)
                }));
            }

            KERNEL_STATE.mark_running(resolved.api_port);

            info!("?? 启动事件中继服务，端口: {}", resolved.api_port);
            match start_websocket_relay(app_handle.clone(), Some(resolved.api_port)).await {
                Ok(_) => {
                    info!("? 事件中继启动成功");

                    enable_kernel_guard(
                        app_handle.clone(),
                        resolved.api_port,
                        resolved.proxy.tun_enabled,
                    )
                    .await;

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

                    enable_kernel_guard(
                        app_handle.clone(),
                        resolved.api_port,
                        resolved.proxy.tun_enabled,
                    )
                    .await;

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
            KERNEL_STATE.mark_failed();

            let detail = e.to_string();
            let code = if detail.contains("配置校验失败")
                || detail.contains("legacy DNS servers")
                || detail.contains("配置文件")
            {
                "KERNEL_CONFIG_INVALID"
            } else {
                "KERNEL_START_FAILED"
            };

            emit_kernel_error_with_context(
                &app_handle,
                code,
                &format!("启动失败: {}", detail),
                Some(&detail),
                Some("kernel.runtime.start"),
                true,
            );

            Ok(serde_json::json!({
                "success": false,
                "message": format!("内核启动失败: {}", e)
            }))
        }
    }
}

async fn stop_kernel_command_impl(app_handle: AppHandle) -> Result<serde_json::Value, String> {
    info!("?? 停止内核（编排器模式）");

    match stop_kernel().await {
        Ok(_) => {
            emit_kernel_stopped(&app_handle);
            Ok(serde_json::json!({
                "success": true,
                "message": "内核停止成功".to_string()
            }))
        }
        Err(e) => {
            let detail = e.to_string();
            emit_kernel_error_with_context(
                &app_handle,
                "KERNEL_STOP_FAILED",
                &format!("停止失败: {}", detail),
                Some(&detail),
                Some("kernel.runtime.stop"),
                true,
            );
            Ok(serde_json::json!({
                "success": false,
                "message": format!("内核停止失败: {}", e)
            }))
        }
    }
}

async fn restart_kernel_internal(
    app_handle: AppHandle,
    overrides: ProxyOverrides,
) -> Result<serde_json::Value, String> {
    info!("?? 收到快速重启请求（编排器模式）");

    let resolved = resolve_proxy_runtime_state(&app_handle, overrides).await?;

    // 先尝试停止，超时时强杀
    let stop_result = tokio::time::timeout(Duration::from_secs(4), stop_kernel()).await;
    match stop_result {
        Ok(Ok(_)) => info!("? 快速重启：停止阶段完成"),
        Ok(Err(e)) => {
            warn!("? 快速重启：停止失败，继续强杀: {}", e);
            if let Err(e) = PROCESS_MANAGER.force_kill_kernel_processes_by_name().await {
                error!("强制清理内核进程失败: {}", e);
            }
        }
        Err(_) => {
            warn!("? 快速重启：停止超时，强制清理");
            if let Err(e) = PROCESS_MANAGER.force_kill_kernel_processes_by_name().await {
                error!("强制清理内核进程失败: {}", e);
            }
        }
    }

    start_kernel_with_state(app_handle.clone(), &resolved).await
}

pub async fn orchestrated_start_kernel(
    app_handle: AppHandle,
    overrides: ProxyOverrides,
) -> Result<serde_json::Value, String> {
    let event_handle = app_handle.clone();
    execute_kernel_operation(
        event_handle,
        "kernel.start",
        async move {
            let resolved = resolve_proxy_runtime_state(&app_handle, overrides).await?;
            start_kernel_with_state(app_handle, &resolved).await
        }
        .boxed(),
    )
    .await
}

pub async fn orchestrated_stop_kernel(app_handle: AppHandle) -> Result<serde_json::Value, String> {
    let event_handle = app_handle.clone();
    execute_kernel_operation(
        event_handle,
        "kernel.stop",
        async move { stop_kernel_command_impl(app_handle).await }.boxed(),
    )
    .await
}

pub async fn orchestrated_restart_kernel(
    app_handle: AppHandle,
    overrides: ProxyOverrides,
) -> Result<serde_json::Value, String> {
    let event_handle = app_handle.clone();
    execute_kernel_operation(
        event_handle,
        "kernel.restart",
        async move { restart_kernel_internal(app_handle, overrides).await }.boxed(),
    )
    .await
}

#[tauri::command]
#[allow(clippy::too_many_arguments)] // 保持 Tauri 调用签名，参数拆分由前端传入
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

    orchestrated_start_kernel(app_handle, overrides).await
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
    orchestrated_stop_kernel(app_handle).await
}

/// 快速重启：串行执行停止与启动，保证生命周期命令有序
#[tauri::command]
#[allow(clippy::too_many_arguments)] // 保持 Tauri 调用签名，参数拆分由前端传入
pub async fn kernel_restart_fast(
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

    orchestrated_restart_kernel(app_handle, overrides).await
}

// 退出+停核逻辑不再保留单独 API，前端统一使用快速重启或停止

pub async fn stop_kernel() -> Result<String, String> {
    KERNEL_STATE.set_state(KernelState::Stopping);
    disable_kernel_guard().await;
    SHOULD_STOP_EVENTS.store(true, std::sync::atomic::Ordering::Relaxed);
    cleanup_event_relay_tasks().await;

    if let Err(e) = PROCESS_MANAGER.stop().await {
        KERNEL_STATE.mark_failed();
        return Err(format!("{}: {}", messages::ERR_PROCESS_STOP_FAILED, e));
    }

    // 快速轮询确认，避免固定长等待
    for i in 1..=2 {
        if !is_kernel_running().await.unwrap_or(true) {
            info!("? 内核停止成功（第{}次检查）", i);
            KERNEL_STATE.mark_stopped();
            return Ok("内核停止成功".to_string());
        }
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    KERNEL_STATE.mark_failed();
    Err(messages::ERR_PROCESS_STOP_FAILED.to_string())
}
