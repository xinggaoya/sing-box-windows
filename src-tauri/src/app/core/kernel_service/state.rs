//! 内核运行时配置与状态类型定义
//!
//! 提供统一的配置类型，替代分散的 ProxyOverrides 和 AutoManageOptions。

use crate::app::core::tun_profile::TunProxyOptions;
use crate::app::storage::state_model::AppConfig;
use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU16, AtomicU8, Ordering};
use std::sync::Arc;

/// 内核运行状态枚举
/// 
/// 使用状态机模式管理内核生命周期，确保状态转换的一致性。
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum KernelState {
    /// 内核已停止
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

impl Default for KernelState {
    fn default() -> Self {
        KernelState::Stopped
    }
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
        matches!(self, KernelState::Stopped | KernelState::Failed | KernelState::Crashed)
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
}

impl KernelStateManager {
    pub fn new() -> Self {
        Self {
            state: AtomicU8::new(KernelState::Stopped as u8),
            api_port: AtomicU16::new(0),
        }
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
    }

    /// 标记为已停止
    pub fn mark_stopped(&self) {
        self.api_port.store(0, Ordering::SeqCst);
        self.set_state(KernelState::Stopped);
    }

    /// 标记为失败
    pub fn mark_failed(&self) {
        self.set_state(KernelState::Failed);
    }

    /// 标记为崩溃（守护进程检测）
    pub fn mark_crashed(&self) {
        self.set_state(KernelState::Crashed);
    }

    /// 获取 API 端口
    pub fn get_api_port(&self) -> u16 {
        self.api_port.load(Ordering::SeqCst)
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
mod tests {
    use super::*;

    #[test]
    fn test_kernel_state_transitions() {
        let manager = KernelStateManager::new();
        
        assert_eq!(manager.get_state(), KernelState::Stopped);
        assert!(manager.get_state().can_start());
        
        assert!(manager.try_transition_to_starting());
        assert_eq!(manager.get_state(), KernelState::Starting);
        assert!(!manager.get_state().can_start());
        
        manager.mark_running(12081);
        assert_eq!(manager.get_state(), KernelState::Running);
        assert!(manager.get_state().is_running());
        
        assert!(manager.try_transition_to_stopping());
        assert_eq!(manager.get_state(), KernelState::Stopping);
        
        manager.mark_stopped();
        assert_eq!(manager.get_state(), KernelState::Stopped);
    }

    #[test]
    fn test_kernel_runtime_config_merge() {
        let mut base = KernelRuntimeConfig {
            api_port: Some(12081),
            proxy_port: Some(12080),
            ..Default::default()
        };
        
        let overrides = KernelRuntimeConfig {
            api_port: Some(9090),
            prefer_ipv6: Some(true),
            ..Default::default()
        };
        
        base.merge(&overrides);
        
        assert_eq!(base.api_port, Some(9090));
        assert_eq!(base.proxy_port, Some(12080)); // 未被覆盖
        assert_eq!(base.prefer_ipv6, Some(true));
    }
}
