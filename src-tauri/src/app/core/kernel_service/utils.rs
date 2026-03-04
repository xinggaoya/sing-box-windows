//! 内核服务通用工具模块
//!
//! 提供内核服务各模块共用的工具函数，避免代码重复。

use crate::app::constants::paths;
use crate::app::core::kernel_service::orchestrator::current_state_version;
use crate::app::core::kernel_service::state::KERNEL_STATE;
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use serde_json::json;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};

/// 解析配置文件路径
///
/// 从数据库读取 `active_config_path`，若未设置则回退到默认配置路径。
///
/// # Arguments
/// * `app_handle` - Tauri AppHandle 引用
///
/// # Returns
/// * `Ok(PathBuf)` - 解析后的配置文件路径
/// * `Err(String)` - 读取配置失败时的错误信息
pub async fn resolve_config_path(app_handle: &AppHandle) -> Result<PathBuf, String> {
    let app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;

    Ok(app_config
        .active_config_path
        .map(PathBuf::from)
        .unwrap_or_else(|| paths::get_config_dir().join("config.json")))
}

/// 解析配置文件路径（带默认值回退）
///
/// 与 `resolve_config_path` 类似，但在读取失败时使用默认配置路径而非返回错误。
/// 适用于守护进程等不能中断的场景。
pub async fn resolve_config_path_or_default(app_handle: &AppHandle) -> PathBuf {
    resolve_config_path(app_handle)
        .await
        .unwrap_or_else(|_| paths::get_config_dir().join("config.json"))
}

/// 内核状态事件数据结构
#[derive(Debug, Clone)]
pub struct KernelStatusPayload {
    pub process_running: bool,
    pub api_ready: bool,
    pub websocket_ready: bool,
}

impl KernelStatusPayload {
    /// 创建"运行中"状态的 payload
    pub fn running() -> Self {
        Self {
            process_running: true,
            api_ready: true,
            websocket_ready: true,
        }
    }

    /// 创建"已停止"状态的 payload
    pub fn stopped() -> Self {
        Self {
            process_running: false,
            api_ready: false,
            websocket_ready: false,
        }
    }

    /// 创建自定义状态的 payload
    pub fn new(process_running: bool, api_ready: bool, websocket_ready: bool) -> Self {
        Self {
            process_running,
            api_ready,
            websocket_ready,
        }
    }

    /// 转换为 JSON Value
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "process_running": self.process_running,
            "api_ready": self.api_ready,
            "websocket_ready": self.websocket_ready,
            "kernel_state": KERNEL_STATE.get_state().as_str(),
            "state_version": current_state_version()
        })
    }
}

/// 发送内核状态变更事件
///
/// 统一发送 `kernel-status-changed` 事件，确保所有状态变更通知格式一致。
///
/// # Arguments
/// * `app_handle` - Tauri AppHandle 引用
/// * `status` - 内核状态 payload
pub fn emit_kernel_status(app_handle: &AppHandle, status: &KernelStatusPayload) {
    let _ = app_handle.emit("kernel-status-changed", status.to_json());
}

/// 发送内核已启动事件
///
/// 同时发送 `kernel-started`、`kernel-status-changed` 和 `kernel-ready` 事件。
///
/// # Arguments
/// * `app_handle` - Tauri AppHandle 引用
/// * `proxy_mode` - 当前代理模式
/// * `api_port` - API 端口
/// * `proxy_port` - 代理端口
/// * `auto_restarted` - 是否为自动重启（守护进程触发）
pub fn emit_kernel_started(
    app_handle: &AppHandle,
    proxy_mode: &str,
    api_port: u16,
    proxy_port: u16,
    auto_restarted: bool,
) {
    let started_payload = json!({
        "process_running": true,
        "api_ready": true,
        "websocket_ready": true,
        "proxy_mode": proxy_mode,
        "api_port": api_port,
        "proxy_port": proxy_port,
        "auto_restarted": auto_restarted,
        "kernel_state": KERNEL_STATE.get_state().as_str(),
        "state_version": current_state_version()
    });

    let _ = app_handle.emit("kernel-started", started_payload);
    emit_kernel_status(app_handle, &KernelStatusPayload::running());
    let _ = app_handle.emit("kernel-ready", ());
}

/// 发送内核已停止事件
///
/// 同时发送 `kernel-stopped` 和 `kernel-status-changed` 事件。
pub fn emit_kernel_stopped(app_handle: &AppHandle) {
    let stopped_payload = KernelStatusPayload::stopped();
    let _ = app_handle.emit("kernel-stopped", stopped_payload.to_json());
    emit_kernel_status(app_handle, &stopped_payload);
}

/// 发送内核启动中事件
///
/// 发送 `kernel-starting` 事件，通知前端内核正在启动。
pub fn emit_kernel_starting(
    app_handle: &AppHandle,
    proxy_mode: &str,
    api_port: u16,
    proxy_port: u16,
) {
    let payload = json!({
        "proxy_mode": proxy_mode,
        "api_port": api_port,
        "proxy_port": proxy_port
    });
    let _ = app_handle.emit("kernel-starting", payload);
}

/// 发送内核错误事件
///
/// 发送 `kernel-error` 事件，通知前端发生错误。
fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

pub fn build_kernel_error_payload(
    code: &str,
    message: &str,
    details: Option<&str>,
    source: Option<&str>,
    recoverable: bool,
) -> serde_json::Value {
    let details = details.unwrap_or(message);
    let source = source.unwrap_or("kernel");

    json!({
        "code": code,
        "message": message,
        "details": details,
        "source": source,
        "recoverable": recoverable,
        "timestamp": now_millis(),
        // 兼容旧前端：仍保留 error 字段
        "error": message
    })
}

pub fn emit_kernel_error_with_context(
    app_handle: &AppHandle,
    code: &str,
    message: &str,
    details: Option<&str>,
    source: Option<&str>,
    recoverable: bool,
) {
    let payload = build_kernel_error_payload(code, message, details, source, recoverable);
    let _ = app_handle.emit("kernel-error", payload);
}

pub fn emit_kernel_error(app_handle: &AppHandle, error: &str) {
    emit_kernel_error_with_context(app_handle, "KERNEL_RUNTIME_ERROR", error, None, None, true);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kernel_status_payload_running() {
        let payload = KernelStatusPayload::running();
        assert!(payload.process_running);
        assert!(payload.api_ready);
        assert!(payload.websocket_ready);
    }

    #[test]
    fn test_kernel_status_payload_stopped() {
        let payload = KernelStatusPayload::stopped();
        assert!(!payload.process_running);
        assert!(!payload.api_ready);
        assert!(!payload.websocket_ready);
    }

    #[test]
    fn test_kernel_status_payload_to_json() {
        let payload = KernelStatusPayload::new(true, false, true);
        let json = payload.to_json();
        assert_eq!(json["process_running"], true);
        assert_eq!(json["api_ready"], false);
        assert_eq!(json["websocket_ready"], true);
    }

    #[test]
    fn test_kernel_error_payload_contains_compat_and_structured_fields() {
        let payload = build_kernel_error_payload(
            "KERNEL_START_FAILED",
            "内核启动失败",
            Some("配置校验失败"),
            Some("kernel.runtime.start"),
            true,
        );

        assert_eq!(payload["code"], "KERNEL_START_FAILED");
        assert_eq!(payload["message"], "内核启动失败");
        assert_eq!(payload["details"], "配置校验失败");
        assert_eq!(payload["source"], "kernel.runtime.start");
        assert_eq!(payload["recoverable"], true);
        // 兼容老前端字段
        assert_eq!(payload["error"], "内核启动失败");
        assert!(payload["timestamp"].as_u64().is_some());
    }
}
