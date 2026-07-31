//! 内核服务通用工具模块
//!
//! 提供内核服务各模块共用的工具函数，避免代码重复。

use crate::app::constants::paths;
use crate::app::core::kernel_service::orchestrator::current_state_version;
use crate::app::core::kernel_service::state::{
    KernelReadinessSnapshot, StartupDiagnosis, StartupDiagnosisKind, StartupStage, KERNEL_STATE,
};
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
    pub readiness: KernelReadinessSnapshot,
    pub startup_diagnosis: Option<StartupDiagnosis>,
}

impl KernelStatusPayload {
    /// 创建"运行中"状态的 payload
    pub fn running() -> Self {
        Self {
            process_running: true,
            api_ready: true,
            websocket_ready: true,
            readiness: KernelReadinessSnapshot {
                config_validated: Some(true),
                process_spawned: Some(true),
                process_alive: true,
                api_ready: true,
                relay_ready: true,
            },
            startup_diagnosis: None,
        }
    }

    /// 创建"已停止"状态的 payload
    pub fn stopped() -> Self {
        Self {
            process_running: false,
            api_ready: false,
            websocket_ready: false,
            readiness: KernelReadinessSnapshot::default(),
            startup_diagnosis: KERNEL_STATE.get_startup_diagnosis(),
        }
    }

    /// 创建自定义状态的 payload
    pub fn new(
        process_running: bool,
        api_ready: bool,
        websocket_ready: bool,
        readiness: KernelReadinessSnapshot,
        startup_diagnosis: Option<StartupDiagnosis>,
    ) -> Self {
        Self {
            process_running,
            api_ready,
            websocket_ready,
            readiness,
            startup_diagnosis,
        }
    }

    pub fn from_state() -> Self {
        let readiness = KERNEL_STATE.get_readiness();
        Self {
            process_running: readiness.process_alive,
            api_ready: readiness.api_ready,
            websocket_ready: readiness.relay_ready,
            readiness,
            startup_diagnosis: KERNEL_STATE.get_startup_diagnosis(),
        }
    }

    /// 转换为 JSON Value
    pub fn to_json(&self) -> serde_json::Value {
        json!({
            "process_running": self.process_running,
            "api_ready": self.api_ready,
            "websocket_ready": self.websocket_ready,
            "readiness": self.readiness.clone(),
            "startup_diagnosis": self.startup_diagnosis.clone(),
            "error": self
                .startup_diagnosis
                .as_ref()
                .map(|diagnosis| diagnosis.message.clone()),
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
/// * `relay_ready` - 事件中继（WebSocket）是否就绪；为 false 时不覆盖已记录的 relay_ready，
///   payload 的 `websocket_ready` 也会同步反映，避免"中继失败却上报就绪"的不一致。
pub fn emit_kernel_started(
    app_handle: &AppHandle,
    proxy_mode: &str,
    api_port: u16,
    proxy_port: u16,
    auto_restarted: bool,
    relay_ready: bool,
) {
    KERNEL_STATE.update_readiness(|readiness| {
        readiness.config_validated = Some(true);
        readiness.process_spawned = Some(true);
        readiness.process_alive = true;
        readiness.api_ready = true;
        readiness.relay_ready = relay_ready;
    });
    KERNEL_STATE.clear_startup_diagnosis();

    let started_payload = json!({
        "process_running": true,
        "api_ready": true,
        "websocket_ready": relay_ready,
        "readiness": KERNEL_STATE.get_readiness(),
        "startup_diagnosis": KERNEL_STATE.get_startup_diagnosis(),
        "proxy_mode": proxy_mode,
        "api_port": api_port,
        "proxy_port": proxy_port,
        "auto_restarted": auto_restarted,
        "kernel_state": KERNEL_STATE.get_state().as_str(),
        "state_version": current_state_version()
    });

    let _ = app_handle.emit("kernel-started", started_payload);
    emit_kernel_status(app_handle, &KernelStatusPayload::from_state());
    let _ = app_handle.emit("kernel-ready", ());
}

/// 发送内核已停止事件
///
/// 同时发送 `kernel-stopped` 和 `kernel-status-changed` 事件。
pub fn emit_kernel_stopped(app_handle: &AppHandle) {
    KERNEL_STATE.update_readiness(|readiness| {
        readiness.process_alive = false;
        readiness.api_ready = false;
        readiness.relay_ready = false;
    });
    let stopped_payload = KernelStatusPayload::from_state();
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
    KERNEL_STATE.update_readiness(|readiness| {
        readiness.process_alive = false;
        readiness.api_ready = false;
        readiness.relay_ready = false;
    });
    let payload = json!({
        "proxy_mode": proxy_mode,
        "api_port": api_port,
        "proxy_port": proxy_port
    });
    let _ = app_handle.emit("kernel-starting", payload);
    emit_kernel_status(app_handle, &KernelStatusPayload::from_state());
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

fn infer_stage_from_source(source: &str) -> StartupStage {
    if source.contains("check_config") || source.contains("preflight") {
        StartupStage::Preflight
    } else if source.contains("auto_manage") {
        StartupStage::AutoManage
    } else if source.contains("guard") {
        StartupStage::Guard
    } else if source.contains("startup_stability")
        || source.contains("startup")
        || source.contains("readiness")
    {
        StartupStage::Readiness
    } else {
        StartupStage::Spawn
    }
}

fn infer_kind_from_code(code: &str, details: &str) -> StartupDiagnosisKind {
    match code {
        "KERNEL_CONFIG_INVALID" => StartupDiagnosisKind::ConfigInvalid,
        "KERNEL_CONFIG_MISSING" => StartupDiagnosisKind::ConfigMissing,
        "KERNEL_BINARY_MISSING" => StartupDiagnosisKind::BinaryMissing,
        "KERNEL_SUDO_REQUIRED" => StartupDiagnosisKind::SudoRequired,
        "KERNEL_SUDO_INVALID" => StartupDiagnosisKind::SudoInvalid,
        "KERNEL_PERMISSION_DENIED" => StartupDiagnosisKind::PermissionDenied,
        "KERNEL_PORT_CONFLICT" | "KERNEL_CONFLICT_DETECTED" => StartupDiagnosisKind::PortConflict,
        "KERNEL_CONFLICT_FORCE_STOP_FAILED" => StartupDiagnosisKind::ConflictCleanupFailed,
        "KERNEL_PROCESS_EXITED_EARLY" => StartupDiagnosisKind::ProcessExitedEarly,
        "KERNEL_API_HTTP_ERROR" => StartupDiagnosisKind::ApiHttpError,
        "KERNEL_API_TIMEOUT" => StartupDiagnosisKind::ApiTimeout,
        "KERNEL_GUARD_RESTART_FAILED" | "KERNEL_GUARD_SELF_HEAL_FAILED" => {
            StartupDiagnosisKind::GuardRestartFailed
        }
        "KERNEL_START_UNSTABLE" => {
            if details.contains("API状态码") {
                StartupDiagnosisKind::ApiHttpError
            } else if details.contains("立即退出") {
                StartupDiagnosisKind::ProcessExitedEarly
            } else {
                StartupDiagnosisKind::ApiTimeout
            }
        }
        "KERNEL_START_FAILED" => {
            if details.contains("SUDO_PASSWORD_REQUIRED") {
                StartupDiagnosisKind::SudoRequired
            } else if details.contains("SUDO_PASSWORD_INVALID") {
                StartupDiagnosisKind::SudoInvalid
            } else if details.contains("内核文件不存在") {
                StartupDiagnosisKind::BinaryMissing
            } else if details.contains("配置文件不存在") {
                StartupDiagnosisKind::ConfigMissing
            } else if details.contains("配置校验失败") {
                StartupDiagnosisKind::ConfigInvalid
            } else if details.contains("权限") {
                StartupDiagnosisKind::PermissionDenied
            } else {
                StartupDiagnosisKind::Unknown
            }
        }
        "KERNEL_AUTO_MANAGE_FAILED" | "KERNEL_STOP_FAILED" | "KERNEL_RUNTIME_ERROR" => {
            StartupDiagnosisKind::Unknown
        }
        _ => StartupDiagnosisKind::Unknown,
    }
}

fn extract_http_status(details: &str) -> Option<u16> {
    if let Some(index) = details.find("API状态码 ") {
        let remainder = &details[index + "API状态码 ".len()..];
        let digits = remainder
            .chars()
            .take_while(|char| char.is_ascii_digit())
            .collect::<String>();
        return digits.parse::<u16>().ok();
    }
    None
}

fn build_suggested_actions(kind: StartupDiagnosisKind) -> Option<Vec<String>> {
    let actions = match kind {
        StartupDiagnosisKind::ConfigInvalid => vec![
            "在订阅页刷新当前订阅配置".to_string(),
            "关闭按原始配置运行后重新生成".to_string(),
        ],
        StartupDiagnosisKind::ConfigMissing => vec!["确认当前生效配置文件仍存在".to_string()],
        StartupDiagnosisKind::BinaryMissing => vec!["先在设置页安装或导入内核".to_string()],
        StartupDiagnosisKind::PortConflict | StartupDiagnosisKind::ConflictCleanupFailed => {
            vec!["修改端口或结束占用当前端口的进程".to_string()]
        }
        StartupDiagnosisKind::SudoRequired => vec!["输入系统密码后重试".to_string()],
        StartupDiagnosisKind::SudoInvalid => vec!["重新保存正确的系统密码".to_string()],
        StartupDiagnosisKind::PermissionDenied => vec!["检查权限后重试".to_string()],
        _ => Vec::new(),
    };

    if actions.is_empty() {
        None
    } else {
        Some(actions)
    }
}

pub fn build_startup_diagnosis(
    code: &str,
    message: &str,
    details: Option<&str>,
    source: Option<&str>,
    recoverable: bool,
) -> StartupDiagnosis {
    let detail = details.unwrap_or(message).to_string();
    let source = source.unwrap_or("kernel").to_string();
    let kind = infer_kind_from_code(code, &detail);
    StartupDiagnosis {
        attempt_id: KERNEL_STATE.ensure_attempt("kernel-start"),
        stage: infer_stage_from_source(&source),
        code: code.to_string(),
        kind,
        message: message.to_string(),
        detail: detail.clone(),
        source,
        recoverable,
        config_path: None,
        http_status: extract_http_status(&detail),
        suggested_actions: build_suggested_actions(kind),
        timestamp_ms: now_millis(),
    }
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
    let startup_diagnosis =
        build_startup_diagnosis(code, message, Some(details), Some(source), recoverable);

    json!({
        "code": code,
        "message": message,
        "details": details,
        "source": source,
        "recoverable": recoverable,
        "timestamp": now_millis(),
        "startup_diagnosis": startup_diagnosis,
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
    if let Ok(startup_diagnosis) =
        serde_json::from_value::<StartupDiagnosis>(payload["startup_diagnosis"].clone())
    {
        KERNEL_STATE.record_startup_diagnosis(startup_diagnosis);
    }
    let _ = app_handle.emit("kernel-error", payload);
    emit_kernel_status(app_handle, &KernelStatusPayload::from_state());
}

pub fn emit_kernel_error(app_handle: &AppHandle, error: &str) {
    emit_kernel_error_with_context(app_handle, "KERNEL_RUNTIME_ERROR", error, None, None, true);
}

#[cfg(test)]
#[path = "utils.tests.rs"]
mod tests;
