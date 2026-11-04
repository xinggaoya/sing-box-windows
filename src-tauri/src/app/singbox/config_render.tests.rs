//! TUN inbound 渲染收敛测试（TUN-12）。
//!
//! 背景：TUN inbound 此前有**两份并行实现**：
//! - [`crate::app::singbox::config_render::build_inbounds`]（生成路径 / 补丁路径）
//! - [`crate::app::core::tun_profile::TunProfile::to_inbounds`]（运行时切换路径）
//!
//! 两份产出的字段不一致（生成路径没有 `interface_name`），意味着
//! 「订阅更新后生成的配置」与「用户手动切换 TUN 后写入的配置」不是同一份东西 ——
//! Windows 上会因此使用不同的 Wintun 适配器名。
//!
//! 本测试锁定：**两条路径产出的 TUN inbound 必须逐字段相同**。

use super::build_inbounds;
use crate::app::core::tun_profile::{TunProfile, TunProxyOptions};
use crate::app::singbox::config_intent::ConfigIntent;
use crate::app::storage::state_model::AppConfig;
use serde_json::Value;

/// 构造一个启用 TUN 的应用配置。
fn tun_app_config() -> AppConfig {
    AppConfig {
        tun_enabled: true,
        tun_auto_route: true,
        tun_strict_route: true,
        tun_mtu: 9000,
        tun_stack: "mixed".to_string(),
        ..AppConfig::default()
    }
}

/// 构造一个启用 TUN 的配置意图。
fn tun_config_intent() -> ConfigIntent {
    ConfigIntent::from_app_config(&tun_app_config())
}

/// 运行时切换路径使用的选项构造（与 `resolve_proxy_runtime_state` 一致）。
fn runtime_options(app_config: &AppConfig) -> TunProxyOptions {
    TunProxyOptions {
        ipv4_address: app_config.tun_ipv4.clone(),
        ipv6_address: app_config.tun_ipv6.clone(),
        mtu: app_config.tun_mtu,
        auto_route: app_config.tun_auto_route,
        strict_route: app_config.tun_strict_route,
        stack: app_config.tun_stack.clone(),
        enable_ipv6: app_config.tun_enable_ipv6,
        route_exclude_address: app_config.tun_route_exclude_address.clone(),
        interface_name: None,
        dns_mode: app_config.tun_dns_mode.clone(),
    }
}

fn tun_inbound_of(inbounds: &[Value]) -> &Value {
    inbounds
        .iter()
        .find(|v| v.get("type").and_then(|t| t.as_str()) == Some("tun"))
        .expect("inbounds 中应存在 tun inbound")
}

#[test]
fn generate_path_and_runtime_path_produce_identical_tun_inbound() {
    // 核心回归：两条路径的 TUN inbound 必须完全一致
    let intent = tun_config_intent();

    let generated = build_inbounds(&intent, None);
    let from_generate = tun_inbound_of(&generated).clone();

    let app_config = tun_app_config();

    let profile = TunProfile::from_options(&runtime_options(&app_config), None);
    let runtime_inbounds = profile.to_inbounds(23456);
    let from_runtime: Value =
        serde_json::to_value(runtime_inbounds.iter().find(|i| i.r#type == "tun").unwrap())
            .expect("序列化 tun inbound");

    assert_eq!(
        from_generate, from_runtime,
        "生成路径与运行时切换路径的 TUN inbound 必须逐字段一致"
    );
}

#[test]
fn tun_inbound_carries_interface_name_on_windows_and_linux() {
    // Windows 上 interface_name 决定 Wintun 适配器名 —— 生成路径此前缺失该字段，
    // 会让「订阅更新后的配置」与「切换 TUN 后的配置」使用不同适配器。
    let intent = tun_config_intent();
    let generated = build_inbounds(&intent, None);
    let tun_inbound = tun_inbound_of(&generated);

    #[cfg(not(target_os = "macos"))]
    assert_eq!(
        tun_inbound.get("interface_name").and_then(|v| v.as_str()),
        Some("singbox_tun"),
        "非 macOS 平台应固定 TUN 适配器名，避免配置更新后适配器名漂移"
    );

    #[cfg(target_os = "macos")]
    assert!(
        tun_inbound.get("interface_name").is_none(),
        "macOS 由系统分配 utun 设备名，不应固定名称"
    );
}

#[test]
fn tun_inbound_keeps_user_configured_route_exclude_address() {
    let app_config = AppConfig {
        tun_enabled: true,
        tun_route_exclude_address: Some(vec!["192.168.0.0/16".to_string()]),
        ..AppConfig::default()
    };
    let intent = ConfigIntent::from_app_config(&app_config);

    let generated = build_inbounds(&intent, None);
    let tun_inbound = tun_inbound_of(&generated);

    assert_eq!(
        tun_inbound
            .get("route_exclude_address")
            .and_then(|v| v.as_array())
            .map(|arr| arr
                .iter()
                .filter_map(|v| v.as_str().map(String::from))
                .collect::<Vec<_>>()),
        Some(vec!["192.168.0.0/16".to_string()]),
        "用户显式设置的路由排除网段必须保留"
    );
}

#[test]
fn disabled_tun_produces_no_tun_inbound() {
    let intent = ConfigIntent::from_app_config(&AppConfig::default());
    let generated = build_inbounds(&intent, None);

    assert!(
        generated
            .iter()
            .all(|v| v.get("type").and_then(|t| t.as_str()) != Some("tun")),
        "未启用 TUN 时不应生成 tun inbound"
    );
}
