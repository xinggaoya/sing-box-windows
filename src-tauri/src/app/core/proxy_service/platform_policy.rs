//! TUN 策略：区分「用户意图」与「运行态」。
//!
//! - **用户意图**（`config.tun_enabled`）：用户显式设置，**任何情况下都不因平台判定而改写**；
//! - **运行态**（`runtime_tun_enabled`）：此刻是否真的启用 TUN。
//!
//! TUN-02 修复的正是这两者的混淆：旧实现在读配置时把意图改写掉并落库。

use crate::app::storage::state_model::AppConfig;

/// TUN 策略判定结果。
#[derive(Debug, Clone, PartialEq)]
pub struct TunPolicyOutcome {
    /// 供调用方使用的配置 —— **用户意图原样保留**。
    pub config: AppConfig,
    /// 运行态是否真的启用 TUN。
    pub runtime_tun_enabled: bool,
    /// 是否需要用户授权（当前环境无法以所需权限运行 TUN 内核）。
    pub elevation_required: bool,
}

/// 应用 TUN 策略。
///
/// - `config`：用户显式设置；
/// - `can_run_tun_elevated`：当前环境能否以 TUN 所需的权限启动内核。
///
/// 无法满足权限时**返回需要授权**而不是静默降级：静默降级会让用户以为 TUN 已开启，
/// 实际却没有接管流量（TUN-02 的根因）。
pub fn apply_tun_policy(config: AppConfig, can_run_tun_elevated: bool) -> TunPolicyOutcome {
    let tun_intent = config.tun_enabled;

    if !tun_intent {
        return TunPolicyOutcome {
            config,
            runtime_tun_enabled: false,
            elevation_required: false,
        };
    }

    if can_run_tun_elevated {
        return TunPolicyOutcome {
            config,
            runtime_tun_enabled: true,
            elevation_required: false,
        };
    }

    TunPolicyOutcome {
        config,
        runtime_tun_enabled: false,
        elevation_required: true,
    }
}

#[cfg(test)]
#[path = "platform_policy.tests.rs"]
mod tests;
