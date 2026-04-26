use super::*;
use crate::app::core::tun_profile::default_tun_route_exclude_addresses;
use crate::app::storage::state_model::AppConfig;
use serde_json::Value;

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

fn assert_route_rules_keep_sniff_action(config: &Value) {
    let rules = config
        .get("route")
        .and_then(|v| v.get("rules"))
        .and_then(|v| v.as_array())
        .expect("route.rules 应存在");

    assert!(
        rules
            .iter()
            .any(|rule| rule.get("action").and_then(|v| v.as_str()) == Some("sniff")),
        "route.rules 应保留 sniff action: {:?}",
        rules
    );
}

#[test]
fn generated_dns_servers_should_use_new_format() {
    let config = generate_base_config(&AppConfig::default());
    let servers = config
        .get("dns")
        .and_then(|v| v.get("servers"))
        .and_then(|v| v.as_array())
        .expect("dns.servers 应存在");

    for server in servers {
        assert!(
            server.get("type").and_then(|v| v.as_str()).is_some(),
            "dns server 应包含 type 字段: {:?}",
            server
        );
        assert!(
            server.get("address").is_none(),
            "dns server 不应再输出 legacy address 字段: {:?}",
            server
        );
        assert!(
            server.get("address_resolver").is_none(),
            "dns server 不应再输出 legacy address_resolver 字段: {:?}",
            server
        );
        assert!(
            server.get("strategy").is_none(),
            "dns server 不应包含 strategy 字段（该字段属于 dns 根配置而非 server）: {:?}",
            server
        );
        assert!(
            server.get("domain_strategy").is_none(),
            "dns server 不应包含已弃用的 domain_strategy 字段: {:?}",
            server
        );
        assert!(
            server.get("detour").and_then(|v| v.as_str()) != Some("direct"),
            "dns server 不应显式设置 detour=direct: {:?}",
            server
        );
    }

    let route_default_resolver = config
        .get("route")
        .and_then(|v| v.get("default_domain_resolver"))
        .expect("route.default_domain_resolver 应存在");
    assert_eq!(
        route_default_resolver
            .get("server")
            .and_then(|v| v.as_str()),
        Some(DNS_RESOLVER)
    );
    assert!(route_default_resolver.get("strategy").is_some());
}

#[test]
fn generated_log_should_write_to_kernel_work_dir_file() {
    let config = generate_base_config(&AppConfig::default());
    let log = config
        .get("log")
        .and_then(|v| v.as_object())
        .expect("log 配置应存在");

    assert_eq!(log.get("disabled").and_then(|v| v.as_bool()), Some(false));
    assert_eq!(log.get("level").and_then(|v| v.as_str()), Some("info"));
    assert_eq!(log.get("timestamp").and_then(|v| v.as_bool()), Some(true));

    let output = log
        .get("output")
        .and_then(|v| v.as_str())
        .expect("log.output 应存在");
    assert!(
        output.ends_with("sing-box.log"),
        "log.output 应指向 sing-box.log: {output}"
    );
}

#[test]
fn ads_dns_rule_should_use_reject_action() {
    let app_config = AppConfig {
        singbox_block_ads: true,
        ..AppConfig::default()
    };

    let config = generate_base_config(&app_config);
    let rules = config
        .get("dns")
        .and_then(|v| v.get("rules"))
        .and_then(|v| v.as_array())
        .expect("dns.rules 应存在");

    let ads_rule = rules
        .iter()
        .find(|rule| rule.get("rule_set").and_then(|v| v.as_str()) == Some(RS_GEOSITE_ADS))
        .expect("启用广告拦截时应包含 geosite ads DNS 规则");

    assert_eq!(
        ads_rule.get("action").and_then(|v| v.as_str()),
        Some("reject")
    );
    assert!(ads_rule.get("server").is_none());
}

#[test]
fn fake_dns_should_append_fakeip_server_and_enable_reverse_mapping() {
    let app_config = AppConfig {
        singbox_fake_dns_enabled: true,
        ..AppConfig::default()
    };

    let config = generate_base_config(&app_config);
    let servers = config
        .get("dns")
        .and_then(|v| v.get("servers"))
        .and_then(|v| v.as_array())
        .expect("dns.servers 应存在");

    let fakeip_server = servers
        .iter()
        .find(|server| server.get("tag").and_then(|v| v.as_str()) == Some(DNS_FAKEIP))
        .expect("启用 fake dns 后应包含 fakeip dns server");

    assert_eq!(
        fakeip_server.get("type").and_then(|v| v.as_str()),
        Some("fakeip")
    );
    assert_eq!(
        fakeip_server.get("inet4_range").and_then(|v| v.as_str()),
        Some("198.18.0.0/15")
    );
    assert_eq!(
        fakeip_server.get("inet6_range").and_then(|v| v.as_str()),
        Some("fc00::/18")
    );

    assert_eq!(
        config
            .get("dns")
            .and_then(|v| v.get("reverse_mapping"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );
    assert_eq!(
        config
            .get("experimental")
            .and_then(|v| v.get("cache_file"))
            .and_then(|v| v.get("store_rdrc"))
            .and_then(|v| v.as_bool()),
        Some(true)
    );
}

#[test]
fn fake_dns_global_non_cn_should_add_catch_all_query_rule() {
    let app_config = AppConfig {
        singbox_fake_dns_enabled: true,
        singbox_fake_dns_filter_mode: "global_non_cn".to_string(),
        ..AppConfig::default()
    };

    let config = generate_base_config(&app_config);
    let rules = config
        .get("dns")
        .and_then(|v| v.get("rules"))
        .and_then(|v| v.as_array())
        .expect("dns.rules 应存在");

    let catch_all = rules.iter().find(|rule| {
        rule.get("server").and_then(|v| v.as_str()) == Some(DNS_FAKEIP)
            && rule.get("rule_set").is_none()
            && rule.get("query_type").is_some()
    });
    assert!(
        catch_all.is_some(),
        "global_non_cn 模式应生成 A/AAAA catch-all fakeip 规则"
    );
}

#[test]
fn generated_inbounds_should_not_use_legacy_fields() {
    let config = generate_base_config(&AppConfig::default());

    assert_inbounds_do_not_contain_legacy_fields(&config);
    assert_route_rules_keep_sniff_action(&config);
}

#[test]
fn generated_tun_inbounds_should_not_use_legacy_fields() {
    let app_config = AppConfig {
        tun_enabled: true,
        tun_enable_ipv6: true,
        ..AppConfig::default()
    };

    let config = generate_base_config(&app_config);
    let inbounds = config
        .get("inbounds")
        .and_then(|v| v.as_array())
        .expect("inbounds 应存在");

    assert_eq!(inbounds.len(), 2, "启用 TUN 时应生成 mixed + tun 两个入站");
    assert_inbounds_do_not_contain_legacy_fields(&config);
    assert_route_rules_keep_sniff_action(&config);
}

#[test]
fn generated_tun_inbound_should_use_canonical_route_exclude_address_default() {
    let app_config = AppConfig {
        tun_enabled: true,
        ..AppConfig::default()
    };

    let config = generate_base_config(&app_config);
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
        Some(&serde_json::json!(default_tun_route_exclude_addresses()))
    );
}

#[test]
fn generated_tun_inbound_should_use_explicit_route_exclude_address_override() {
    let app_config = AppConfig {
        tun_enabled: true,
        tun_route_exclude_address: Some(vec!["203.0.113.0/24".to_string()]),
        ..AppConfig::default()
    };

    let config = generate_base_config(&app_config);
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
        Some(&serde_json::json!(["203.0.113.0/24"]))
    );
}
