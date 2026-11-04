//! TUN 策略（用户意图 vs 运行态）的单元测试。
//!
//! 背景（TUN-02）：旧实现在**读配置**时把 `tun_enabled` 改回 false **并落库**。
//! 后果：用户显式开启的 TUN 被后台悄悄抹掉 → 开关反复弹回 → 每次都要重新
//! 走一遍「提权 + 重启」。而且读函数里写库，让配置层与策略层职责混在一起。
//!
//! 新契约：
//! - **用户意图**（`config.tun_enabled`）任何情况下都不被平台判定改写；
//! - **运行态**（`runtime_tun_enabled`）反映此刻是否真的启用 TUN；
//! - 无法满足权限时上报 `elevation_required`，由前端引导一次性授权，
//!   而不是静默降级。

use super::apply_tun_policy;
use crate::app::storage::state_model::AppConfig;

fn config_with_tun(enabled: bool) -> AppConfig {
    AppConfig {
        tun_enabled: enabled,
        ..AppConfig::default()
    }
}

#[test]
fn tun_disabled_needs_no_elevation() {
    let outcome = apply_tun_policy(config_with_tun(false), false);
    assert!(
        !outcome.runtime_tun_enabled,
        "未开启 TUN 时运行态不应启用 TUN"
    );
    assert!(!outcome.elevation_required, "用户没开 TUN 时不应要求授权");
}

#[test]
fn tun_enabled_with_privilege_runs_as_tun() {
    let outcome = apply_tun_policy(config_with_tun(true), true);
    assert!(outcome.runtime_tun_enabled, "有权限时运行态应启用 TUN");
    assert!(!outcome.elevation_required, "有权限时不应要求授权");
}

#[test]
fn tun_enabled_without_privilege_reports_elevation_required() {
    let outcome = apply_tun_policy(config_with_tun(true), false);
    assert!(
        !outcome.runtime_tun_enabled,
        "无权限时运行态不能启用 TUN（否则内核会启动失败）"
    );
    assert!(
        outcome.elevation_required,
        "无权限时必须上报需要授权，而不是静默降级"
    );
}

#[test]
fn elevation_required_only_appears_when_tun_is_requested() {
    // 关键回归：不能因为「当前进程没提权」就在用户没要求 TUN 时也弹授权
    assert!(!apply_tun_policy(config_with_tun(false), false).elevation_required);
    assert!(!apply_tun_policy(config_with_tun(false), true).elevation_required);
}

#[test]
fn user_tun_intent_is_never_rewritten_by_platform_policy() {
    // TUN-02 核心回归：平台策略**不得**改写用户的 TUN 意图。
    // 无论能否提权，config.tun_enabled 都必须保持用户设置的值。
    for can_elevate in [true, false] {
        let outcome = apply_tun_policy(config_with_tun(true), can_elevate);
        assert!(
            outcome.config.tun_enabled,
            "平台策略改写了用户的 TUN 意图（can_elevate={}）",
            can_elevate
        );
    }
}

#[test]
fn disabled_tun_intent_is_also_preserved() {
    let outcome = apply_tun_policy(config_with_tun(false), true);
    assert!(!outcome.config.tun_enabled, "未开启时意图应保持关闭");
}

#[test]
fn policy_preserves_other_config_fields() {
    let mut original = config_with_tun(true);
    original.proxy_port = 23456;
    original.api_port = 23457;
    original.system_proxy_enabled = true;

    let outcome = apply_tun_policy(original.clone(), false);
    assert_eq!(outcome.config.proxy_port, original.proxy_port);
    assert_eq!(outcome.config.api_port, original.api_port);
    assert_eq!(
        outcome.config.system_proxy_enabled,
        original.system_proxy_enabled
    );
}
