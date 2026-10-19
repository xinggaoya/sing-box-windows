use super::*;
use crate::app::core::tun_profile::default_tun_route_exclude_addresses;
use crate::app::singbox::common::DNS_CN;
use crate::app::storage::state_model::AppConfig;
use serde_json::{json, Value};
fn assert_inbounds_do_not_contain_legacy_fields(config: &Value) {
    let inbounds = config
        .get("inbounds")
        .and_then(|v| v.as_array())
        .expect("inbounds 应存在");

    for inbound in inbounds {
        for legacy_field in [
            "sniff",
            "sniff_override_destination",
            "sniff_timeout",
            "domain_strategy",
            "udp_disable_domain_unmapping",
        ] {
            assert!(
                inbound.get(legacy_field).is_none(),
                "inbound 不应包含 legacy 字段 {}: {:?}",
                legacy_field,
                inbound
            );
        }
    }
}

#[test]
fn apply_app_settings_should_remove_legacy_inbound_fields_from_existing_config() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [
            {
                "type": "mixed",
                "tag": "mixed-in",
                "listen": "127.0.0.1",
                "listen_port": 7890,
                "sniff": true,
                "sniff_override_destination": true,
                "sniff_timeout": "1s",
                "domain_strategy": "prefer_ipv4",
                "udp_disable_domain_unmapping": true
            }
        ],
        "route": {
            "rule_set": [],
            "rules": [
                {
                    "action": "sniff"
                }
            ],
            "final": "direct",
            "auto_detect_interface": true
        }
    });

    apply_app_settings_to_config(&mut config, &AppConfig::default());

    assert_inbounds_do_not_contain_legacy_fields(&config);
}

#[test]
fn apply_app_settings_should_add_kernel_log_output_to_existing_config() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [],
        "route": {
            "rule_set": [],
            "rules": [],
            "final": "direct",
            "auto_detect_interface": true
        }
    });

    apply_app_settings_to_config(&mut config, &AppConfig::default());

    let log = config
        .get("log")
        .and_then(|v| v.as_object())
        .expect("应补充 log 配置");
    assert_eq!(log.get("disabled").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(log.get("level").and_then(|v| v.as_str()), Some("info"));
    assert_eq!(log.get("timestamp").and_then(|v| v.as_bool()), Some(true));

    let output = log
        .get("output")
        .and_then(|v| v.as_str())
        .expect("应补充 log.output");
    assert!(
        output.ends_with("sing-box.log"),
        "log.output 应指向 sing-box.log: {output}"
    );
}

#[test]
fn apply_app_settings_should_not_reintroduce_legacy_fields_when_tun_enabled() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [],
        "route": {
            "rule_set": [],
            "rules": [
                {
                    "action": "sniff"
                }
            ],
            "final": "direct",
            "auto_detect_interface": true
        }
    });
    let app_config = AppConfig {
        tun_enabled: true,
        tun_enable_ipv6: true,
        ..AppConfig::default()
    };

    apply_app_settings_to_config(&mut config, &app_config);

    let inbounds = config
        .get("inbounds")
        .and_then(|v| v.as_array())
        .expect("inbounds 应存在");
    assert_eq!(inbounds.len(), 2, "启用 TUN 时应生成 mixed + tun 两个入站");
    assert_inbounds_do_not_contain_legacy_fields(&config);
}

#[test]
fn apply_app_settings_should_insert_sniff_route_for_legacy_configs() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [
            {
                "type": "mixed",
                "tag": "mixed-in",
                "listen": "127.0.0.1",
                "listen_port": 7890,
                "sniff": true,
                "sniff_override_destination": true
            }
        ],
        "route": {
            "rule_set": [],
            "rules": [],
            "final": "direct",
            "auto_detect_interface": true
        }
    });

    apply_app_settings_to_config(&mut config, &AppConfig::default());

    let rules = config
        .get("route")
        .and_then(|v| v.get("rules"))
        .and_then(|v| v.as_array())
        .expect("route.rules 应存在");
    assert!(
        rules
            .iter()
            .any(|rule| rule.get("action").and_then(|v| v.as_str()) == Some("sniff")),
        "旧配置迁移后应补充 sniff 规则: {:?}",
        rules
    );
}

#[test]
fn apply_app_settings_should_use_explicit_tun_route_exclude_address_override() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [],
        "route": {
            "rule_set": [],
            "rules": [
                {
                    "action": "sniff"
                }
            ],
            "final": "direct",
            "auto_detect_interface": true
        }
    });
    let app_config = AppConfig {
        tun_enabled: true,
        tun_route_exclude_address: Some(vec!["203.0.113.0/24".to_string()]),
        ..AppConfig::default()
    };

    apply_app_settings_to_config(&mut config, &app_config);

    let tun_in = config
        .get("inbounds")
        .and_then(|value| value.as_array())
        .and_then(|inbounds| {
            inbounds.iter().find(|inbound| {
                inbound.get("tag").and_then(|value| value.as_str()) == Some("tun-in")
            })
        })
        .expect("tun-in 应存在");

    assert_eq!(
        tun_in.get("route_exclude_address"),
        Some(&json!(["203.0.113.0/24"]))
    );
}

#[test]
fn apply_app_settings_should_preserve_existing_tun_route_exclude_address_when_override_absent() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [
            {
                "type": "tun",
                "tag": "tun-in",
                "route_exclude_address": ["198.51.100.0/24"]
            }
        ],
        "route": {
            "rule_set": [],
            "rules": [
                {
                    "action": "sniff"
                }
            ],
            "final": "direct",
            "auto_detect_interface": true
        }
    });
    let app_config = AppConfig {
        tun_enabled: true,
        ..AppConfig::default()
    };

    apply_app_settings_to_config(&mut config, &app_config);

    let tun_in = config
        .get("inbounds")
        .and_then(|value| value.as_array())
        .and_then(|inbounds| {
            inbounds.iter().find(|inbound| {
                inbound.get("tag").and_then(|value| value.as_str()) == Some("tun-in")
            })
        })
        .expect("tun-in 应存在");

    assert_eq!(
        tun_in.get("route_exclude_address"),
        Some(&json!(["198.51.100.0/24"]))
    );
}

#[test]
fn apply_app_settings_should_fallback_to_canonical_tun_route_exclude_address_default() {
    let mut config = json!({
        "dns": {
            "servers": [],
            "rules": []
        },
        "experimental": {
            "clash_api": {},
            "cache_file": {}
        },
        "inbounds": [],
        "route": {
            "rule_set": [],
            "rules": [
                {
                    "action": "sniff"
                }
            ],
            "final": "direct",
            "auto_detect_interface": true
        }
    });
    let app_config = AppConfig {
        tun_enabled: true,
        ..AppConfig::default()
    };

    apply_app_settings_to_config(&mut config, &app_config);

    let tun_in = config
        .get("inbounds")
        .and_then(|value| value.as_array())
        .and_then(|inbounds| {
            inbounds.iter().find(|inbound| {
                inbound.get("tag").and_then(|value| value.as_str()) == Some("tun-in")
            })
        })
        .expect("tun-in 应存在");

    assert_eq!(
        tun_in.get("route_exclude_address"),
        Some(&json!(default_tun_route_exclude_addresses()))
    );
}

#[test]
fn apply_app_settings_should_keep_dns_bootstrap_ipv4_first_when_prefer_ipv6() {
    let mut config = json!({
        "dns": {
            "servers": [
                {
                    "tag": DNS_CN,
                    "type": "h3",
                    "server": "dns.alidns.com",
                    "server_port": 443
                }
            ],
            "rules": []
        },
        "experimental": {},
        "inbounds": [],
        "route": {
            "rule_set": [],
            "rules": [
                {
                    "action": "sniff"
                }
            ],
            "final": "direct",
            "auto_detect_interface": true
        }
    });
    let app_config = AppConfig {
        prefer_ipv6: true,
        ..AppConfig::default()
    };

    apply_app_settings_to_config(&mut config, &app_config);

    let servers = config
        .get("dns")
        .and_then(|v| v.get("servers"))
        .and_then(|v| v.as_array())
        .expect("dns.servers 应存在");
    let cn_server = servers
        .iter()
        .find(|server| server.get("tag").and_then(|v| v.as_str()) == Some(DNS_CN))
        .expect("dns_cn 应被按 tag 重建");
    assert_eq!(
        cn_server
            .get("domain_resolver")
            .and_then(|v| v.get("strategy"))
            .and_then(|v| v.as_str()),
        Some("prefer_ipv4"),
        "apply 后 DNS bootstrap 策略不应跟随全局 prefer_ipv6: {:?}",
        cn_server
    );
}

/// 一份机场原始配置：入口 / DNS / 路由全都是它自己的，与本程序无关
fn foreign_subscription_config() -> Value {
    json!({
        "inbounds": [
            {
                "type": "socks",
                "tag": "socks-in",
                "listen": "127.0.0.1",
                "listen_port": 1080
            }
        ],
        "dns": {
            "servers": [
                { "tag": "remote-dns", "type": "https", "server": "1.1.1.1" }
            ],
            "rules": [
                { "domain_suffix": ["example.com"], "server": "remote-dns" }
            ]
        },
        "outbounds": [
            { "type": "selector", "tag": "proxy", "outbounds": ["node-a"] },
            { "type": "vless", "tag": "node-a", "server": "1.2.3.4", "server_port": 443 }
        ],
        "route": {
            "rule_set": [
                {
                    "type": "remote",
                    "tag": "geosite-cn",
                    "format": "binary",
                    "url": "https://example.com/cn.srs"
                }
            ],
            "rules": [
                { "ip_cidr": ["10.0.0.0/8"], "outbound": "direct" }
            ],
            "final": "proxy"
        }
    })
}

#[test]
fn apply_intent_should_not_touch_foreign_config() {
    // 归属不属于本程序 → 入口 / dns / route.rules 必须逐字段不变。
    // 改坏用户导入的原始配置不会报错，只会表现为"能连但走错线路"。
    let mut config = foreign_subscription_config();
    let before = config.clone();

    let intent = ConfigIntent::from_app_config(&AppConfig::default());
    apply_intent_to_config(&mut config, &intent, &Ownership::empty());

    assert_eq!(
        config.get("inbounds"),
        before.get("inbounds"),
        "外来配置的入口不应被重建"
    );
    assert_eq!(
        config.get("dns"),
        before.get("dns"),
        "外来配置的 dns 不应被改写"
    );
    assert_eq!(
        config.get("route").and_then(|route| route.get("rules")),
        before.get("route").and_then(|route| route.get("rules")),
        "外来配置的 route.rules 不应被改写"
    );
    assert_eq!(
        config.pointer("/route/final"),
        before.pointer("/route/final"),
        "外来配置的路由终点不应被改写"
    );
}

#[test]
fn apply_intent_should_manage_config_owned_by_app() {
    // 归属为本程序 → 入口按设置重建，路由终点跟随设置
    let intent = ConfigIntent::from_app_config(&AppConfig::default());

    // 带本程序标记的配置
    let mut owned = json!({
        "inbounds": [
            { "type": "mixed", "tag": "mixed-in", "listen": "127.0.0.1", "listen_port": 7890 }
        ],
        "dns": { "servers": [], "rules": [] },
        "outbounds": [],
        "route": { "rule_set": [], "rules": [], "final": "proxy" }
    });
    let ownership = Ownership::inferred(&owned);
    assert!(ownership.managed, "带 mixed-in 入口的配置应被接管");

    apply_intent_to_config(&mut owned, &intent, &ownership);

    let inbounds = owned
        .get("inbounds")
        .and_then(|v| v.as_array())
        .expect("inbounds 应存在");
    assert_eq!(inbounds.len(), 1);
    assert_eq!(
        inbounds[0].get("tag").and_then(|v| v.as_str()),
        Some("mixed-in"),
        "入口应被重建为本程序的结构"
    );
    assert_eq!(
        owned.pointer("/route/final").and_then(|v| v.as_str()),
        Some(intent.default_outbound),
        "路由终点应跟随设置"
    );
}
