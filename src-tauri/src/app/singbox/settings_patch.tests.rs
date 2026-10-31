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

/// 一份带本程序标记、且写着旧版（会导致内核启动失败的）直连 detour 的配置
fn owned_config_with_legacy_http_clients() -> Value {
    json!({
        "inbounds": [
            { "type": "mixed", "tag": "mixed-in", "listen": "127.0.0.1", "listen_port": 7890 }
        ],
        "dns": { "servers": [], "rules": [] },
        "outbounds": [],
        "http_clients": [
            { "tag": "default", "detour": "direct" },
            { "tag": "direct-download", "detour": "direct" }
        ],
        "route": {
            "rule_set": [
                {
                    "type": "remote",
                    "tag": "geosite-private",
                    "format": "binary",
                    "url": "https://example.com/private.srs",
                    "http_client": "direct-download"
                }
            ],
            "rules": [],
            "final": "proxy"
        }
    })
}

fn http_client_by_tag<'a>(config: &'a Value, tag: &str) -> Option<&'a Value> {
    config
        .get("http_clients")
        .and_then(|v| v.as_array())
        .and_then(|clients| {
            clients
                .iter()
                .find(|client| client.get("tag").and_then(|v| v.as_str()) == Some(tag))
        })
}

#[test]
fn sync_http_clients_should_omit_detour_when_downloading_directly() {
    // 直连下载**不能写 detour**：detour 指向空的 direct 出站会让内核启动失败
    // （"detour to an empty direct outbound makes no sense"）。
    // 不写 detour 时 http_client 走系统 dialer，语义同样是直连。
    let intent = ConfigIntent::from_app_config(&AppConfig::default());
    let mut config = owned_config_with_legacy_http_clients();
    let ownership = Ownership::inferred(&config);
    assert!(ownership.managed, "带 mixed-in 入口的配置应被接管");

    apply_intent_to_config(&mut config, &intent, &ownership);

    let default_client = http_client_by_tag(&config, "default").expect("default 客户端应存在");
    assert!(
        default_client.get("detour").is_none(),
        "直连下载的 http_client 不应写 detour，实际: {default_client:?}"
    );
}

#[test]
fn sync_http_clients_should_repair_legacy_direct_detour() {
    // 存量配置里写的是 detour: "direct"（内核起不来的那版），
    // 补丁路径必须把它纠正成"没有 detour"。
    let intent = ConfigIntent::from_app_config(&AppConfig::default());
    let mut config = owned_config_with_legacy_http_clients();
    let ownership = Ownership::inferred(&config);

    apply_intent_to_config(&mut config, &intent, &ownership);

    // default 客户端：必须存在，且不再带 detour
    let default_client = http_client_by_tag(&config, "default").expect("default 客户端必须存在");
    assert!(
        default_client.get("detour").is_none(),
        "default 客户端不应保留 detour: \"direct\"，实际: {default_client:?}"
    );

    // direct-download 客户端：直连偏好下根本不需要它，必须被删除
    // （而不是被"保留但不写 detour"）
    assert!(
        http_client_by_tag(&config, "direct-download").is_none(),
        "直连偏好下 direct-download 客户端应被删除，实际: {:?}",
        config.get("http_clients")
    );
}

#[test]
fn sync_http_clients_should_follow_manual_download_preference() {
    // 偏好「走手动选择出站」时：default 客户端 detour 指向手动分组，
    // 并额外提供一个**直连**客户端（同样不能写 detour）给本机专用规则集。
    let app_config = AppConfig {
        singbox_download_detour: "manual".to_string(),
        ..AppConfig::default()
    };
    let intent = ConfigIntent::from_app_config(&app_config);
    let mut config = owned_config_with_legacy_http_clients();
    let ownership = Ownership::inferred(&config);

    // 先抹掉规则集上已有的 `http_client` 引用：下面的断言要证明的是
    // 「补丁路径会**主动补上**这个引用」，而不是「引用碰巧本来就在」。
    if let Some(rule_sets) = config
        .pointer_mut("/route/rule_set")
        .and_then(|v| v.as_array_mut())
    {
        for rs in rule_sets.iter_mut() {
            if let Some(obj) = rs.as_object_mut() {
                obj.remove("http_client");
            }
        }
    }

    apply_intent_to_config(&mut config, &intent, &ownership);

    let default_client = http_client_by_tag(&config, "default").expect("default 客户端应存在");
    assert_eq!(
        default_client.get("detour").and_then(|v| v.as_str()),
        Some(crate::app::singbox::common::TAG_MANUAL),
        "走手动选择出站时 default 客户端应 detour 到手动分组"
    );

    let direct_client = http_client_by_tag(&config, "direct-download").expect("直连客户端应存在");
    assert!(
        direct_client.get("detour").is_none(),
        "直连客户端不应写 detour，实际: {direct_client:?}"
    );

    let private_rule_set = config
        .pointer("/route/rule_set")
        .and_then(|v| v.as_array())
        .and_then(|sets| {
            sets.iter()
                .find(|rs| rs.get("tag").and_then(|v| v.as_str()) == Some("geosite-private"))
        })
        .expect("geosite-private 规则集应存在");
    assert_eq!(
        private_rule_set.get("http_client").and_then(|v| v.as_str()),
        Some("direct-download"),
        "本机专用规则集应固定走直连客户端"
    );
}

#[test]
fn sync_http_clients_should_drop_dangling_direct_download_reference() {
    // 从「走手动选择出站」切回「直连」后，direct-download 客户端不再存在，
    // 规则集上残留的 http_client 引用必须一起清掉
    // （引用不存在的 client 会让内核启动失败）。
    let intent = ConfigIntent::from_app_config(&AppConfig::default());
    let mut config = owned_config_with_legacy_http_clients();
    let ownership = Ownership::inferred(&config);

    apply_intent_to_config(&mut config, &intent, &ownership);

    assert!(
        http_client_by_tag(&config, "direct-download").is_none(),
        "直连偏好下不应保留 direct-download 客户端"
    );
    let private_rule_set = config
        .pointer("/route/rule_set")
        .and_then(|v| v.as_array())
        .and_then(|sets| {
            sets.iter()
                .find(|rs| rs.get("tag").and_then(|v| v.as_str()) == Some("geosite-private"))
        })
        .expect("geosite-private 规则集应存在");
    assert!(
        private_rule_set.get("http_client").is_none(),
        "直连偏好下不应残留指向 direct-download 的悬空引用"
    );
}

#[test]
fn sync_http_clients_should_not_touch_foreign_config() {
    // 外来配置（用户导入的机场原始配置）里的 http_client 一个字节都不能动
    let mut config = foreign_subscription_config();
    config["http_clients"] = json!([{ "tag": "default", "detour": "direct" }]);
    let before = config.get("http_clients").cloned();

    let intent = ConfigIntent::from_app_config(&AppConfig::default());
    apply_intent_to_config(&mut config, &intent, &Ownership::empty());

    assert_eq!(
        config.get("http_clients"),
        before.as_ref(),
        "外来配置的 http_clients 不应被改写"
    );
}

#[test]
fn generated_and_patched_http_clients_should_match() {
    // 双模型收敛断言：生成路径与补丁路径产出的 http_clients 必须逐字段一致，
    // 否则"订阅刚刷新出来的配置"与"改一次设置后的配置"下载链路会不一样。
    //
    // ⚠️ 必须从**存量（错误）配置**出发，而不是从生成结果出发：
    // 从生成结果出发的话，生成路径与补丁路径用的是同一个 `build_http_clients`，
    // 断言恒成立——哪怕把 `sync_http_clients` 整个删掉，这个测试也照样能过。
    // 真正要锁的是「**发散能修回来**」：补丁路径必须把存量配置收敛到生成结果。
    for download_detour in ["direct", "manual"] {
        let app_config = AppConfig {
            singbox_download_detour: download_detour.to_string(),
            ..AppConfig::default()
        };
        let intent = ConfigIntent::from_app_config(&app_config);

        let generated = super::super::config_generator::generate_base_config(&app_config);

        // 起点：存量配置（写错的 detour + 悬空的 http_client 引用）
        let mut patched = owned_config_with_legacy_http_clients();
        let ownership = Ownership::inferred(&patched);
        apply_intent_to_config(&mut patched, &intent, &ownership);

        assert_eq!(
            patched.get("http_clients"),
            generated.get("http_clients"),
            "补丁路径没能把存量配置的 http_clients 收敛到生成结果（download_detour={download_detour}）"
        );
    }
}

#[test]
fn sync_http_clients_should_preserve_user_defined_clients() {
    // 用户自己加的 http_client 必须原样保留——本程序只管 tag 为
    // `default` / `direct-download` 的那两个。
    let intent = ConfigIntent::from_app_config(&AppConfig::default());
    let mut config = owned_config_with_legacy_http_clients();
    config["http_clients"]
        .as_array_mut()
        .expect("http_clients 应为数组")
        .push(json!({
            "tag": "my-proxy-client",
            "detour": "proxy",
            "server": "1.2.3.4",
            "server_port": 8080
        }));
    let ownership = Ownership::inferred(&config);

    apply_intent_to_config(&mut config, &intent, &ownership);

    let user_client =
        http_client_by_tag(&config, "my-proxy-client").expect("用户自己加的 http_client 必须保留");
    assert_eq!(
        user_client.get("detour").and_then(|v| v.as_str()),
        Some("proxy"),
        "用户自己的 http_client 不应被改写"
    );
    assert_eq!(
        user_client.get("server").and_then(|v| v.as_str()),
        Some("1.2.3.4")
    );
}

#[test]
fn sync_http_clients_should_handle_missing_or_invalid_key() {
    // 两种边界：
    // 1) 配置里完全没有 `http_clients` → 补丁路径应补齐
    // 2) `http_clients` 不是数组 → 补丁路径应纠正成数组（而不是 panic）
    let intent = ConfigIntent::from_app_config(&AppConfig::default());

    let mut missing = owned_config_with_legacy_http_clients();
    missing
        .as_object_mut()
        .expect("配置应为对象")
        .remove("http_clients");
    let ownership = Ownership::inferred(&missing);
    apply_intent_to_config(&mut missing, &intent, &ownership);
    assert!(
        http_client_by_tag(&missing, "default").is_some(),
        "缺少 http_clients 时应补齐 default 客户端"
    );

    let mut invalid = owned_config_with_legacy_http_clients();
    invalid["http_clients"] = json!("not-an-array");
    let ownership = Ownership::inferred(&invalid);
    apply_intent_to_config(&mut invalid, &intent, &ownership);
    assert!(
        invalid
            .get("http_clients")
            .and_then(|v| v.as_array())
            .is_some(),
        "非数组的 http_clients 应被纠正成数组"
    );
    assert!(
        http_client_by_tag(&invalid, "default").is_some(),
        "非数组的 http_clients 应被纠正后写入 default 客户端"
    );
}

#[test]
fn sync_http_clients_should_repair_dangling_default_http_client() {
    // `route.default_http_client` 指向一个**不存在的**客户端时，内核启动失败
    // （"default http client transport is not initialized"）。
    // 配置被手工改过 / 经其它工具往返处理过都可能造出这种悬空引用。
    let intent = ConfigIntent::from_app_config(&AppConfig::default());
    let mut config = owned_config_with_legacy_http_clients();
    config["route"]["default_http_client"] = json!("direct-download");
    let ownership = Ownership::inferred(&config);

    apply_intent_to_config(&mut config, &intent, &ownership);

    assert!(
        http_client_by_tag(&config, "direct-download").is_none(),
        "直连偏好下 direct-download 客户端应被删除"
    );
    assert_eq!(
        config
            .pointer("/route/default_http_client")
            .and_then(|v| v.as_str()),
        Some("default"),
        "悬空的 route.default_http_client 应被改指到 default 客户端"
    );
}
