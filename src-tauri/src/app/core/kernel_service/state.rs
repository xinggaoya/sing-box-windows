//! 内核运行时配置与状态类型定义
//!
//! 提供统一的配置类型，替代分散的 ProxyOverrides 和 AutoManageOptions。

use crate::app::core::tun_profile::TunProxyOptions;
use crate::app::storage::state_model::AppConfig;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU16, AtomicU32, AtomicU8, Ordering};
use std::sync::{Arc, RwLock};
use std::time::{SystemTime, UNIX_EPOCH};

/// 内核运行状态枚举
///
/// 使用状态机模式管理内核生命周期，确保状态转换的一致性。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum KernelState {
    /// 内核已停止
    #[default]
    Stopped = 0,
    /// 内核正在启动
    Starting = 1,
    /// 内核运行中
    Running = 2,
    /// 内核正在停止
    Stopping = 3,
    /// 内核启动失败
    Failed = 4,
    /// 内核意外崩溃（由守护进程检测）
    Crashed = 5,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum StartupStage {
    #[default]
    Preflight,
    Spawn,
    Readiness,
    Guard,
    AutoManage,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StartupDiagnosisKind {
    ConfigInvalid,
    ConfigMissing,
    BinaryMissing,
    PermissionDenied,
    SudoRequired,
    SudoInvalid,
    PortConflict,
    ProcessExitedEarly,
    ApiHttpError,
    ApiTimeout,
    ConflictCleanupFailed,
    GuardRestartFailed,
    Unknown,
}

impl StartupDiagnosisKind {
    pub fn priority(&self) -> u8 {
        match self {
            StartupDiagnosisKind::ConfigMissing => 100,
            StartupDiagnosisKind::ConfigInvalid => 90,
            StartupDiagnosisKind::BinaryMissing => 80,
            StartupDiagnosisKind::SudoRequired
            | StartupDiagnosisKind::SudoInvalid
            | StartupDiagnosisKind::PermissionDenied => 70,
            StartupDiagnosisKind::PortConflict
            | StartupDiagnosisKind::ConflictCleanupFailed => 60,
            StartupDiagnosisKind::ProcessExitedEarly => 50,
            StartupDiagnosisKind::ApiHttpError | StartupDiagnosisKind::ApiTimeout => 40,
            StartupDiagnosisKind::GuardRestartFailed => 30,
            StartupDiagnosisKind::Unknown => 10,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct StartupDiagnosis {
    pub attempt_id: String,
    pub stage: StartupStage,
    pub code: String,
    pub kind: StartupDiagnosisKind,
    pub message: String,
    pub detail: String,
    pub source: String,
    pub recoverable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_status: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub suggested_actions: Option<Vec<String>>,
    pub timestamp_ms: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct KernelReadinessSnapshot {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_validated: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub process_spawned: Option<bool>,
    pub process_alive: bool,
    pub api_ready: bool,
    pub relay_ready: bool,
}

/// 守护/自愈重启的可观测性统计，随内核状态事件一起上报，便于诊断内核抖动。
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, Default)]
pub struct RestartStats {
    /// 累计自愈/守护重启次数（进程启动后不清零，用于长期观测）。
    pub restart_count: u32,
    /// 最近一次重启原因（如 "process-crashed"、"tun-connectivity"、"system-connectivity"）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_restart_reason: Option<String>,
    /// 最近一次重启的时间戳（毫秒）。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_restart_at: Option<u64>,
}

impl From<u8> for KernelState {
    fn from(v: u8) -> Self {
        match v {
            0 => KernelState::Stopped,
            1 => KernelState::Starting,
            2 => KernelState::Running,
            3 => KernelState::Stopping,
            4 => KernelState::Failed,
            5 => KernelState::Crashed,
            _ => KernelState::Stopped,
        }
    }
}

impl KernelState {
    /// 是否处于可启动状态
    pub fn can_start(&self) -> bool {
        matches!(
            self,
            KernelState::Stopped | KernelState::Failed | KernelState::Crashed
        )
    }

    /// 是否处于可停止状态
    pub fn can_stop(&self) -> bool {
        matches!(self, KernelState::Running | KernelState::Starting)
    }

    /// 是否正在运行
    pub fn is_running(&self) -> bool {
        matches!(self, KernelState::Running)
    }

    /// 是否处于过渡状态
    pub fn is_transitioning(&self) -> bool {
        matches!(self, KernelState::Starting | KernelState::Stopping)
    }

    /// 转字符串用于日志
    pub fn as_str(&self) -> &'static str {
        match self {
            KernelState::Stopped => "stopped",
            KernelState::Starting => "starting",
            KernelState::Running => "running",
            KernelState::Stopping => "stopping",
            KernelState::Failed => "failed",
            KernelState::Crashed => "crashed",
        }
    }
}

/// 全局内核状态管理器
///
/// 线程安全的状态追踪，供所有模块共享访问。
/// 使用无锁原子类型确保高性能和无死锁风险。
pub struct KernelStateManager {
    state: AtomicU8,
    api_port: AtomicU16,
    startup_diagnosis: RwLock<Option<StartupDiagnosis>>,
    readiness: RwLock<KernelReadinessSnapshot>,
    current_attempt_id: RwLock<Option<String>>,
    restart_count: AtomicU32,
    last_restart_reason: RwLock<Option<String>>,
    last_restart_at: RwLock<Option<u64>>,
}

impl KernelStateManager {
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(KernelState::Stopped as u8),
            api_port: AtomicU16::new(0),
            startup_diagnosis: RwLock::new(None),
            readiness: RwLock::new(KernelReadinessSnapshot::default()),
            current_attempt_id: RwLock::new(None),
            restart_count: AtomicU32::new(0),
            last_restart_reason: RwLock::new(None),
            last_restart_at: RwLock::new(None),
        }
    }

    fn now_millis() -> u64 {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_millis() as u64)
            .unwrap_or(0)
    }

    pub fn begin_attempt(&self, prefix: &str) -> String {
        let attempt_id = format!("{}-{}", prefix, Self::now_millis());
        if let Ok(mut guard) = self.current_attempt_id.write() {
            *guard = Some(attempt_id.clone());
        }
        if let Ok(mut diagnosis) = self.startup_diagnosis.write() {
            *diagnosis = None;
        }
        if let Ok(mut readiness) = self.readiness.write() {
            *readiness = KernelReadinessSnapshot::default();
        }
        attempt_id
    }

    pub fn ensure_attempt(&self, prefix: &str) -> String {
        if let Ok(guard) = self.current_attempt_id.read() {
            if let Some(existing) = guard.clone() {
                return existing;
            }
        }
        self.begin_attempt(prefix)
    }

    /// 获取当前状态
    pub fn get_state(&self) -> KernelState {
        KernelState::from(self.state.load(Ordering::SeqCst))
    }

    /// 设置状态
    pub fn set_state(&self, state: KernelState) {
        self.state.store(state as u8, Ordering::SeqCst);
    }

    /// 尝试过渡到启动状态，仅在可启动时返回 true
    pub fn try_transition_to_starting(&self) -> bool {
        let current = self.get_state();
        if current.can_start() {
            self.set_state(KernelState::Starting);
            true
        } else {
            false
        }
    }

    /// 尝试过渡到停止状态，仅在可停止时返回 true
    pub fn try_transition_to_stopping(&self) -> bool {
        let current = self.get_state();
        if current.can_stop() {
            self.set_state(KernelState::Stopping);
            true
        } else {
            false
        }
    }

    /// 标记为运行中
    pub fn mark_running(&self, api_port: u16) {
        self.api_port.store(api_port, Ordering::SeqCst);
        self.set_state(KernelState::Running);
        self.clear_startup_diagnosis();
        self.update_readiness(|readiness| {
            readiness.config_validated = Some(true);
            readiness.process_spawned = Some(true);
            readiness.process_alive = true;
            readiness.api_ready = true;
        });
    }

    /// 标记为已停止
    pub fn mark_stopped(&self) {
        self.api_port.store(0, Ordering::SeqCst);
        self.set_state(KernelState::Stopped);
        self.update_readiness(|readiness| {
            readiness.process_alive = false;
            readiness.api_ready = false;
            readiness.relay_ready = false;
        });
    }

    /// 标记为失败
    pub fn mark_failed(&self) {
        self.set_state(KernelState::Failed);
        self.update_readiness(|readiness| {
            readiness.process_alive = false;
            readiness.api_ready = false;
            readiness.relay_ready = false;
        });
    }

    /// 标记为崩溃（守护进程检测）
    pub fn mark_crashed(&self) {
        self.set_state(KernelState::Crashed);
    }

    /// 获取 API 端口
    pub fn get_api_port(&self) -> u16 {
        self.api_port.load(Ordering::SeqCst)
    }

    pub fn get_readiness(&self) -> KernelReadinessSnapshot {
        self.readiness.read().map(|g| g.clone()).unwrap_or_default()
    }

    pub fn set_readiness(&self, readiness: KernelReadinessSnapshot) {
        if let Ok(mut guard) = self.readiness.write() {
            *guard = readiness;
        }
    }

    pub fn update_readiness<F>(&self, updater: F)
    where
        F: FnOnce(&mut KernelReadinessSnapshot),
    {
        if let Ok(mut guard) = self.readiness.write() {
            updater(&mut guard);
        }
    }

    pub fn get_startup_diagnosis(&self) -> Option<StartupDiagnosis> {
        self.startup_diagnosis.read().ok().and_then(|g| g.clone())
    }

    pub fn clear_startup_diagnosis(&self) {
        if let Ok(mut guard) = self.startup_diagnosis.write() {
            *guard = None;
        }
        if let Ok(mut guard) = self.current_attempt_id.write() {
            *guard = None;
        }
    }

    pub fn record_startup_diagnosis(&self, diagnosis: StartupDiagnosis) {
        if let Ok(mut guard) = self.startup_diagnosis.write() {
            match guard.as_ref() {
                None => {
                    *guard = Some(diagnosis)
                }
                Some(existing) if existing.attempt_id != diagnosis.attempt_id => {
                    *guard = Some(diagnosis)
                }
                Some(existing) => {
                    let should_replace = diagnosis.kind.priority() > existing.kind.priority()
                        || (diagnosis.kind == existing.kind
                            && diagnosis.detail.len() >= existing.detail.len());
                    if should_replace {
                        *guard = Some(diagnosis);
                    }
                }
            }
        }
    }

    /// 记录一次守护/自愈重启，累加计数并写入原因与时间戳。
    pub fn record_restart(&self, reason: impl Into<String>) {
        self.restart_count.fetch_add(1, Ordering::SeqCst);
        let reason = reason.into();
        if let Ok(mut guard) = self.last_restart_reason.write() {
            *guard = Some(reason);
        }
        if let Ok(mut guard) = self.last_restart_at.write() {
            *guard = Some(Self::now_millis());
        }
    }

    /// 读取重启统计快照（供状态接口上报前端）。
    pub fn get_restart_stats(&self) -> RestartStats {
        RestartStats {
            restart_count: self.restart_count.load(Ordering::SeqCst),
            last_restart_reason: self
                .last_restart_reason
                .read()
                .ok()
                .and_then(|g| g.clone()),
            last_restart_at: self.last_restart_at.read().ok().and_then(|g| *g),
        }
    }
}

impl Default for KernelStateManager {
    fn default() -> Self {
        Self::new()
    }
}

// 全局状态管理器实例
lazy_static::lazy_static! {
    pub static ref KERNEL_STATE: Arc<KernelStateManager> = Arc::new(KernelStateManager::new());
}

/// 统一的内核运行时配置
///
/// 替代分散的 ProxyOverrides 和 AutoManageOptions。
/// 所有字段为 Option，便于覆盖式合并。
#[derive(Debug, Clone, Default)]
pub struct KernelRuntimeConfig {
    /// 代理模式字符串 ("system" | "tun" | "manual")
    pub proxy_mode: Option<String>,
    /// API 端口
    pub api_port: Option<u16>,
    /// 代理端口
    pub proxy_port: Option<u16>,
    /// 是否优先 IPv6
    pub prefer_ipv6: Option<bool>,
    /// 系统代理绕过列表
    pub system_proxy_bypass: Option<String>,
    /// TUN 配置选项
    pub tun_options: Option<TunProxyOptions>,
    /// 是否启用系统代理
    pub system_proxy_enabled: Option<bool>,
    /// 是否启用 TUN
    pub tun_enabled: Option<bool>,
    /// 是否开启守护（keep-alive）
    pub keep_alive: Option<bool>,
    /// 是否强制重启（仅用于 auto_manage）
    pub force_restart: bool,
}

impl KernelRuntimeConfig {
    /// 从 AppConfig 构建完整配置
    pub fn from_app_config(config: &AppConfig) -> Self {
        KernelRuntimeConfig {
            proxy_mode: Some(config.proxy_mode.clone()),
            api_port: Some(config.api_port),
            proxy_port: Some(config.proxy_port),
            prefer_ipv6: Some(config.prefer_ipv6),
            system_proxy_bypass: Some(config.system_proxy_bypass.clone()),
            tun_options: Some(TunProxyOptions {
                ipv4_address: config.tun_ipv4.clone(),
                ipv6_address: config.tun_ipv6.clone(),
                mtu: config.tun_mtu,
                auto_route: config.tun_auto_route,
                strict_route: config.tun_strict_route,
                stack: config.tun_stack.clone(),
                enable_ipv6: config.tun_enable_ipv6,
                route_exclude_address: config.tun_route_exclude_address.clone(),
                interface_name: None,
            }),
            system_proxy_enabled: Some(config.system_proxy_enabled),
            tun_enabled: Some(config.tun_enabled),
            keep_alive: Some(config.auto_start_kernel),
            force_restart: false,
        }
    }

    /// 将部分覆盖合并到当前配置
    pub fn merge(&mut self, overrides: &KernelRuntimeConfig) {
        if overrides.proxy_mode.is_some() {
            self.proxy_mode = overrides.proxy_mode.clone();
        }
        if overrides.api_port.is_some() {
            self.api_port = overrides.api_port;
        }
        if overrides.proxy_port.is_some() {
            self.proxy_port = overrides.proxy_port;
        }
        if overrides.prefer_ipv6.is_some() {
            self.prefer_ipv6 = overrides.prefer_ipv6;
        }
        if overrides.system_proxy_bypass.is_some() {
            self.system_proxy_bypass = overrides.system_proxy_bypass.clone();
        }
        if overrides.tun_options.is_some() {
            self.tun_options = overrides.tun_options.clone();
        }
        if overrides.system_proxy_enabled.is_some() {
            self.system_proxy_enabled = overrides.system_proxy_enabled;
        }
        if overrides.tun_enabled.is_some() {
            self.tun_enabled = overrides.tun_enabled;
        }
        if overrides.keep_alive.is_some() {
            self.keep_alive = overrides.keep_alive;
        }
        if overrides.force_restart {
            self.force_restart = true;
        }
    }
}

#[cfg(test)]
#[path = "state.tests.rs"]
mod tests;
