//! 路由 / DNS 规则共享语义层的回归测试。
//!
//! 这些测试守护的是**实测发现的真实缺陷**，每条都对应一个可复现的现象：
//!
//! 1. `geoip-cn` 对域名流量不生效 → 国内站点误走代理
//!    （sing-box 1.11+ 必须先有 `action: "resolve"` 才能用 IP 规则匹配域名目标）
//! 2. `dns.final` 指向境外解析器 → 代理不可用时全部域名解析失败
//! 3. `dns.rules` 里出现 `geoip-cn` → 触发 1.14 弃用告警，且语义上本就无法匹配
//! 4. `optimistic: true` 隐含 3 天陈旧窗口 → 后台跑久了网络被钉死
//! 5. `urltest` 的 `interrupt_exist_connections: true` → 节点抖动时长连接被成批掐断

use super::*;
use crate::app::singbox::common::{
    DNS_CN, DNS_PROXY, RS_GEOIP_CN, RS_GEOSITE_ADS, RS_GEOSITE_CN, RS_GEOSITE_GEOLOCATION_NOT_CN,
    TAG_DIRECT,
};
use crate::app::storage::state_model::AppConfig;
use serde_json::Value;

fn default_intent() -> ConfigIntent {
    ConfigIntent::from_app_config(&AppConfig::default())
}

/// 在 route.rules 里找出第一条带指定 rule_set 的规则。
fn find_rule_by_rule_set<'a>(rules: &'a [Value], tag: &str) -> Option<&'a Value> {
    rules.iter().find(|rule| {
        rule.get("rule_set")
            .and_then(|v| v.as_str())
            .map(|value| value == tag)
            .unwrap_or(false)
            || rule
                .get("rule_set")
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .any(|item| item.as_str().map(|s| s == tag).unwrap_or(false))
                })
                .unwrap_or(false)
    })
}

fn rule_index_by_rule_set(rules: &[Value], tag: &str) -> Option<usize> {
    rules.iter().position(|rule| {
        rule.get("rule_set")
            .and_then(|v| v.as_str())
            .map(|value| value == tag)
            .unwrap_or(false)
            || rule
                .get("rule_set")
                .and_then(|v| v.as_array())
                .map(|items| {
                    items
                        .iter()
                        .any(|item| item.as_str().map(|s| s == tag).unwrap_or(false))
                })
                .unwrap_or(false)
    })
}

/// 复现"国内站点误走代理"的核心条件：
/// `geoip-cn` 规则必须在 `resolve` 动作**之后**，否则 IP 规则对域名目标永远不命中。
#[test]
fn geoip_cn_rule_must_come_after_resolve_action() {
    let rules = build_route_rules(&default_intent());

    let resolve_index = rules
        .iter()
        .position(|rule| rule.get("action").and_then(|v| v.as_str()) == Some("resolve"))
        .expect("route.rules 必须包含 resolve 动作");
    let geoip_index =
        rule_index_by_rule_set(&rules, RS_GEOIP_CN).expect("route.rules 必须包含 geoip-cn 规则");

    assert!(
        geoip_index > resolve_index,
        "geoip-cn 规则必须排在 resolve 动作之后（否则 IP 规则对域名目标不生效）：resolve={} geoip={}",
        resolve_index,
        geoip_index
    );
}

/// `resolve` 动作必须排在 `clash_mode` 覆盖规则之后：
/// 全局/直连模式不应该先触发一次没必要的 DNS 解析。
#[test]
fn resolve_action_must_come_after_clash_mode_rules() {
    let rules = build_route_rules(&default_intent());

    let resolve_index = rules
        .iter()
        .position(|rule| rule.get("action").and_then(|v| v.as_str()) == Some("resolve"))
        .expect("route.rules 必须包含 resolve 动作");
    let global_index = rules
        .iter()
        .position(|rule| rule.get("clash_mode").and_then(|v| v.as_str()) == Some("global"))
        .expect("route.rules 必须包含 clash_mode=global 规则");

    assert!(
        resolve_index > global_index,
        "resolve 动作必须排在 clash_mode 规则之后：global={} resolve={}",
        global_index,
        resolve_index
    );
}

/// 国内域名规则必须排在 `resolve` 之前：
/// 命中的域名直接定去向，不必多一次 DNS 解析。
#[test]
fn geosite_cn_rule_should_come_before_resolve_action() {
    let rules = build_route_rules(&default_intent());

    let resolve_index = rules
        .iter()
        .position(|rule| rule.get("action").and_then(|v| v.as_str()) == Some("resolve"))
        .expect("route.rules 必须包含 resolve 动作");
    let cn_index = rule_index_by_rule_set(&rules, RS_GEOSITE_CN)
        .expect("route.rules 必须包含 geosite-cn 规则");

    assert!(
        cn_index < resolve_index,
        "geosite-cn 规则应排在 resolve 之前：cn={} resolve={}",
        cn_index,
        resolve_index
    );
}

/// 境外域名规则必须排在 `resolve` 之前：
/// 这样境外域名以"域名"形态直接交给代理（由远端解析），
/// 不产生"本地解析 → 再走代理"的额外依赖。
#[test]
fn geolocation_not_cn_rule_should_come_before_resolve_action() {
    let rules = build_route_rules(&default_intent());

    let resolve_index = rules
        .iter()
        .position(|rule| rule.get("action").and_then(|v| v.as_str()) == Some("resolve"))
        .expect("route.rules 必须包含 resolve 动作");
    let not_cn_index = rule_index_by_rule_set(&rules, RS_GEOSITE_GEOLOCATION_NOT_CN)
        .expect("route.rules 必须包含 geosite-geolocation-!cn 规则");

    assert!(
        not_cn_index < resolve_index,
        "境外域名规则应排在 resolve 之前：not_cn={} resolve={}",
        not_cn_index,
        resolve_index
    );
}

/// 所有路由规则都应使用 1.11+ 官方形态 `action: "route"` + `outbound`。
#[test]
fn route_rules_should_use_explicit_route_action() {
    let rules = build_route_rules(&default_intent());

    for rule in &rules {
        let Some(outbound) = rule.get("outbound").and_then(|v| v.as_str()) else {
            continue;
        };
        assert_eq!(
            rule.get("action").and_then(|v| v.as_str()),
            Some("route"),
            "带 outbound 的规则应显式声明 action=route: {:?}",
            rule
        );
        assert!(!outbound.is_empty(), "outbound 不应为空: {:?}", rule);
    }
}

/// `dns.rules` 里不应再出现 `geoip-cn`：
/// 1. IP 规则集在 DNS 阶段无法匹配"尚未解析的域名"；
/// 2. 它属于 sing-box 1.14 起废弃的 "Legacy Address Filter Fields"（1.16 移除）。
#[test]
fn dns_rules_should_not_contain_geoip_cn() {
    let rules = build_dns_rules(&default_intent());

    assert!(
        rule_index_by_rule_set(&rules, RS_GEOIP_CN).is_none(),
        "dns.rules 不应包含 geoip-cn（1.14 起废弃的 Legacy Address Filter Fields）：{:?}",
        rules
    );
}

/// `dns.rules` 里的域名分流必须拆成两条：
/// `geosite-cn` → 国内解析器；`geosite-geolocation-!cn` → 境外解析器。
#[test]
fn dns_rules_should_split_cn_and_non_cn_rule_sets() {
    let rules = build_dns_rules(&default_intent());

    let cn = find_rule_by_rule_set(&rules, RS_GEOSITE_CN).expect("应有 geosite-cn 规则");
    assert_eq!(
        cn.get("server").and_then(|v| v.as_str()),
        Some(DNS_CN),
        "国内域名应走国内解析器：{:?}",
        cn
    );

    let not_cn = find_rule_by_rule_set(&rules, RS_GEOSITE_GEOLOCATION_NOT_CN)
        .expect("应有 geosite-geolocation-!cn 规则");
    assert_eq!(
        not_cn.get("server").and_then(|v| v.as_str()),
        Some(DNS_PROXY),
        "境外域名应走境外解析器：{:?}",
        not_cn
    );
}

/// DNS 规则必须使用 1.11+ 官方形态 `action: "route"` + `server`。
#[test]
fn dns_rules_should_use_explicit_route_action() {
    let rules = build_dns_rules(&default_intent());

    for rule in &rules {
        let Some(server) = rule.get("server").and_then(|v| v.as_str()) else {
            continue;
        };
        assert_eq!(
            rule.get("action").and_then(|v| v.as_str()),
            Some("route"),
            "带 server 的 DNS 规则应显式声明 action=route: {:?}",
            rule
        );
        assert!(!server.is_empty(), "server 不应为空: {:?}", rule);
    }
}

/// 兜底解析器默认必须是**国内解析器**：
/// 代理链路不可用时，未命中规则的域名仍要能解析（否则全部网站打不开）。
#[test]
fn dns_final_should_default_to_cn_resolver() {
    assert_eq!(dns_final_server(&default_intent()), DNS_CN);
}

/// 用户显式选择 `proxy` 策略时，兜底解析器切到境外解析器。
#[test]
fn dns_final_should_follow_user_strategy() {
    let config = AppConfig {
        singbox_dns_final_strategy: "proxy".to_string(),
        ..AppConfig::default()
    };
    let intent = ConfigIntent::from_app_config(&config);

    assert_eq!(dns_final_server(&intent), DNS_PROXY);
}

/// 广告拦截规则必须排在 `clash_mode` 覆盖规则之后：
/// 模式切换（全局/直连）优先于广告拦截，行为才与 UI 一致。
#[test]
fn ads_dns_rule_should_come_after_clash_mode_rules() {
    let rules = build_dns_rules(&default_intent());

    let ads_index = rule_index_by_rule_set(&rules, RS_GEOSITE_ADS)
        .expect("应有广告拦截规则（block_ads 默认开）");
    let global_index = rules
        .iter()
        .position(|rule| rule.get("clash_mode").and_then(|v| v.as_str()) == Some("global"))
        .expect("应有 clash_mode=global 规则");

    assert!(
        ads_index > global_index,
        "广告拦截应排在 clash_mode 规则之后：ads={} global={}",
        ads_index,
        global_index
    );
}

/// 补丁路径：把旧配置对齐后，必须补上 `resolve` 动作并把 `geoip-cn` 拆到它之后。
#[test]
fn sync_route_rules_should_insert_resolve_and_split_cn_rules() {
    let intent = default_intent();
    // 复现旧版生成的路由规则形态：geosite-cn 与 geoip-cn 合并在一条规则里，且没有 resolve
    let mut rules = vec![
        serde_json::json!({ "action": "sniff" }),
        serde_json::json!({ "outbound": TAG_DIRECT, "rule_set": [RS_GEOSITE_CN, RS_GEOIP_CN] }),
        serde_json::json!({ "outbound": TAG_DIRECT, "rule_set": RS_GEOSITE_GEOLOCATION_NOT_CN }),
    ];

    sync_route_rules(&mut rules, &intent);

    let resolve_index = rules
        .iter()
        .position(|rule| rule.get("action").and_then(|v| v.as_str()) == Some("resolve"))
        .expect("补丁后必须存在 resolve 动作");
    let geoip_index =
        rule_index_by_rule_set(&rules, RS_GEOIP_CN).expect("补丁后必须存在独立的 geoip-cn 规则");
    assert!(
        geoip_index > resolve_index,
        "补丁后 geoip-cn 必须排在 resolve 之后：resolve={} geoip={}",
        resolve_index,
        geoip_index
    );

    // 旧的合并规则不能再残留 geoip-cn
    let merged_has_geoip = find_rule_by_rule_set(&rules, RS_GEOSITE_CN)
        .and_then(|rule| rule.get("rule_set"))
        .and_then(|v| v.as_array())
        .map(|items| items.iter().any(|item| item.as_str() == Some(RS_GEOIP_CN)))
        .unwrap_or(false);
    assert!(
        !merged_has_geoip,
        "geosite-cn 规则不应再与 geoip-cn 合并：{:?}",
        rules
    );
}

/// 补丁路径必须保留用户自己加的规则。
#[test]
fn sync_route_rules_should_keep_user_rules() {
    let intent = default_intent();
    let user_rule = serde_json::json!({
        "domain_suffix": ["my-own-site.com"],
        "outbound": TAG_DIRECT
    });
    let mut rules = vec![
        serde_json::json!({ "action": "sniff" }),
        user_rule.clone(),
        serde_json::json!({ "outbound": TAG_DIRECT, "rule_set": [RS_GEOSITE_CN, RS_GEOIP_CN] }),
    ];

    sync_route_rules(&mut rules, &intent);

    assert!(
        rules.contains(&user_rule),
        "用户自定义规则必须保留：{:?}",
        rules
    );
}

/// 补丁路径：DNS 规则对齐后不应残留 `geoip-cn`。
#[test]
fn sync_dns_rules_should_drop_geoip_cn() {
    let intent = default_intent();
    let mut rules = vec![
        serde_json::json!({ "clash_mode": "direct", "server": DNS_CN }),
        serde_json::json!({ "rule_set": [RS_GEOSITE_CN, RS_GEOIP_CN], "server": DNS_CN }),
        serde_json::json!({ "rule_set": RS_GEOSITE_GEOLOCATION_NOT_CN, "server": DNS_PROXY }),
    ];

    sync_dns_rules(&mut rules, &intent);

    assert!(
        rule_index_by_rule_set(&rules, RS_GEOIP_CN).is_none(),
        "补丁后 dns.rules 不应残留 geoip-cn：{:?}",
        rules
    );
    assert!(
        rule_index_by_rule_set(&rules, RS_GEOSITE_CN).is_some(),
        "补丁后 dns.rules 应保留 geosite-cn：{:?}",
        rules
    );
}

/// 配置里**一条托管规则都没有**时（用户删过 CN 规则），
/// 补齐的托管规则应**追加到末尾**，而不是插到用户规则前面。
#[test]
fn sync_route_rules_should_append_when_no_managed_rule_exists() {
    let intent = default_intent();
    let user_rule = serde_json::json!({
        "domain_suffix": ["my-own-site.com"],
        "outbound": TAG_DIRECT
    });
    let mut rules = vec![user_rule.clone()];

    sync_route_rules(&mut rules, &intent);

    assert_eq!(
        rules.first(),
        Some(&user_rule),
        "用户规则应保持在最前面（托管规则应追加到末尾）：{:?}",
        rules
    );
    assert!(rules.len() > 1, "应补齐托管规则：{:?}", rules);
    let resolve_index = rules
        .iter()
        .position(|rule| rule.get("action").and_then(|v| v.as_str()) == Some("resolve"))
        .expect("应补上 resolve 动作");
    assert!(
        resolve_index > 0,
        "resolve 动作应排在用户规则之后：{:?}",
        rules
    );
}
