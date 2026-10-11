//! 内核状态查询与网络质量测试
//!
//! 存活判定统一走 `kernel_service::health`（端口 > 托管句柄 > 平台扫描），
//! 不在本模块自行判断「进程是否在跑」。

use crate::app::storage::enhanced_storage_service::db_get_app_config;
use tauri::AppHandle;

use super::grpc_bridge::make_handle_async;

/// 网络质量测试（unary：TCP RTT + 上下行带宽 + 延迟）
#[tauri::command]
pub async fn network_quality_test(
    app_handle: AppHandle,
) -> Result<crate::app::singbox_api::NetworkQualityResult, String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .network_quality_test()
        .await
        .map_err(|e| format!("网络质量测试失败: {}", e))
}

/// 获取内核完整状态(sing-box 1.14+ 走 gRPC unary 探测,不用 streaming,
/// 也不用 1.13 时代 `experimental.clash_api` 的 HTTP /version 端点)。
///
/// 核心逻辑:**纯 TCP 端口探测 + gRPC GetVersion**,完全绕开 ProcessManager 内部状态
/// (用户实测:sing-box 实际在跑 + streaming 数据正常,但 is_kernel_running 因 ProcessManager
/// 状态问题返回 false,导致前端一直显示"已停止")。
///
/// - gRPC 端口(默认 12081)能连 = sing-box 在跑 = process_running=true
/// - gRPC GetVersion 成功 = kernel_state=running
/// - gRPC 端口都不通 = 真的没启动 = process_running=false
#[tauri::command]
pub async fn kernel_get_status_enhanced_v2(
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    use crate::app::core::kernel_service::health;
    use crate::app::core::kernel_service::state::KERNEL_STATE;
    use std::time::Duration;

    // 0) binary 版本戳 —— 用户可凭此确认 Rust 后端是否真的更新
    tracing::info!("[STATUS_V2_BUILD] 2026-09-01-v4-binary-version-stamp");

    // 1) 从 db 读 api_port / proxy_port
    //    端口配置工作流:UI 改端口 → invoke `update_singbox_ports` 写 db + `kernel_restart_*` 触发重启,
    //    内核用 db 新端口启动。所以 db 永远是权威源,不需要前端注入端口参数。
    let (api_port, proxy_port) = match db_get_app_config(app_handle.clone()).await {
        Ok(c) => (c.api_port, c.proxy_port),
        Err(e) => {
            tracing::warn!("status_v2: db_get_app_config failed: {}", e);
            return Ok(serde_json::json!({
                "process_running": false, "api_ready": false, "websocket_ready": false,
                "kernel_state": "stopped",
                "error": format!("获取 app config 失败: {}", e),
            }));
        }
    };

    // 2) 统一健康探测（端口 > 托管句柄 > 平台扫描）
    //    见 kernel_service::health：此前本命令用「纯 TCP 端口」而托盘/守护用
    //    「进程句柄」，两套结论会矛盾（前端显示已停止但内核其实在跑）。
    let managed_running = crate::app::core::kernel_service::PROCESS_MANAGER
        .is_running()
        .await;
    let platform_scanned =
        crate::platform::is_process_running(crate::platform::get_kernel_executable_name())
            .await
            .unwrap_or(false);

    let health = health::probe_kernel(
        api_port,
        proxy_port,
        health::TcpPortProbe,
        managed_running,
        platform_scanned,
    )
    .await;

    let process_running = health.is_running();
    let api_port_open = health.api_ready;
    let proxy_port_open = health.proxy_port_open;
    tracing::info!(
        "[STATUS_V2_BUILD] api_port={} open={} | proxy_port={} open={} | managed={} | scanned={} | process_running={} | source={}",
        api_port,
        api_port_open,
        proxy_port,
        proxy_port_open,
        managed_running,
        platform_scanned,
        process_running,
        health.source.as_str()
    );

    // 3) gRPC GetVersion 探测(决定 api_ready 和 kernel_state)
    let (api_ready, kernel_state_str, error_msg, version) = if process_running {
        let handle = match make_handle_async(app_handle.clone()).await {
            Err(e) => {
                tracing::warn!("status_v2: make_handle_async failed: {}", e);
                return Ok(serde_json::json!({
                    "process_running": true, // TCP 端口通,进程在跑
                    "api_ready": false,
                    "websocket_ready": false,
                    "kernel_state": "starting",
                    "error": format!("创建 gRPC 句柄失败: {}", e),
                }));
            }
            Ok(v) => v.0,
        };
        let probe = tokio::time::timeout(Duration::from_secs(2), handle.get_version()).await;
        match probe {
            Ok(Ok((v, _api))) => {
                tracing::info!("[STATUS_V2_BUILD] gRPC GetVersion OK -> {}", v);
                (true, "running".to_string(), None, Some(v))
            }
            Ok(Err(e)) => {
                tracing::warn!("[STATUS_V2_BUILD] gRPC GetVersion Err: {} (端口通但 gRPC 失败 — sing-box 启动早期?)", e);
                (
                    false,
                    "starting".to_string(),
                    Some(format!("gRPC 探测失败: {}", e)),
                    None,
                )
            }
            Err(_) => {
                tracing::warn!(
                    "[STATUS_V2_BUILD] gRPC GetVersion timeout (2s) — 端口通但 gRPC 不响应"
                );
                (
                    false,
                    "starting".to_string(),
                    Some("gRPC 探测超时(2s)".to_string()),
                    None,
                )
            }
        }
    } else {
        tracing::warn!("[STATUS_V2_BUILD] TCP 端口都不通 → sing-box 真的没启动");
        (false, "stopped".to_string(), None, None)
    };

    // 4) 同步 readiness
    let mut readiness = KERNEL_STATE.get_readiness();
    readiness.process_alive = process_running;
    readiness.api_ready = api_ready;

    Ok(serde_json::json!({
        "process_running": process_running,
        "api_ready": api_ready,
        "websocket_ready": api_ready,
        "version": version,
        "error": error_msg,
        "kernel_state": kernel_state_str,
        "readiness": {
            "process_alive": readiness.process_alive,
            "api_ready": readiness.api_ready,
            "relay_ready": readiness.relay_ready,
        }
    }))
}
