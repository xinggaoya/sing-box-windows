//! 配置渲染共享层：生成路径与补丁路径共用的纯函数。
//!
//! ## 为什么需要它
//!
//! 配置此前有两条路径：
//!
//! - **生成路径**（`config_generator::generate_base_config`）：从零拼一份配置
//! - **补丁路径**（`settings_patch::apply_app_settings_to_config`）：在已有配置上按 tag 打补丁
//!
//! 两条路径都要产出「入口（inbounds）」，但入口只有一处实现时，
//! 生成路径就不得不反过来调用补丁函数——于是"生成"依赖"补丁"，
//! 补丁里的清理逻辑（删旧字段、补 api 服务）也会被无差别地跑一遍。
//!
//! 本模块把两条路径**真正共有**的部分抽成纯函数：
//! 入口构建、假 DNS server / 规则同步等。两条路径各自动用自己的上下文调用，
//! 语义一致但职责清晰。

use super::common::{
    DNS_FAKEIP, FAKE_DNS_FILTER_GLOBAL_NON_CN, RS_GEOSITE_GEOLOCATION_NOT_CN, TAG_DIRECT,
};
use super::config_intent::ConfigIntent;
use super::config_schema::DnsServerConfig;
use crate::app::core::tun_profile::normalize_persisted_tun_route_exclude_address;
use serde_json::{json, Map, Value};

/// 构建入口（inbounds）列表。
///
/// - `existing_route_exclude`：已有配置里 `tun-in` 的 `route_exclude_address`。
///   生成路径传 `None`（新配置没有历史值）；补丁路径传当前配置里的值，
///   以便用户没有显式设置时保持"沿用原有网段"的语义。
pub fn build_inbounds(
    intent: &ConfigIntent,
    existing_route_exclude: Option<Vec<String>>,
) -> Vec<Value> {
    let mut inbounds = Vec::new();

    // mixed 是桌面端最通用的入口（HTTP + SOCKS），便于系统代理/浏览器直接使用。
    inbounds.push(json!({
        "type": "mixed",
        "tag": "mixed-in",
        "listen": intent.proxy_listen,
        "listen_port": intent.proxy_port,
        "set_system_proxy": intent.system_proxy_enabled
    }));

    // TUN 模式依赖 sing-box 配置里显式存在 tun inbound，所以这里根据设置开关动态添加/移除。
    if intent.tun.enabled {
        let route_exclude_address = intent
            .tun
            .route_exclude_address
            .clone()
            .or(existing_route_exclude)
            .unwrap_or_else(crate::app::core::tun_profile::default_tun_route_exclude_addresses);

        inbounds.push(json!({
            "type": "tun",
            "tag": "tun-in",
            "address": intent.tun.addresses(),
            "auto_route": intent.tun.auto_route,
            "strict_route": intent.tun.strict_route,
            "stack": intent.tun.stack,
            "mtu": intent.tun.mtu,
            // 1.14 新增：对系统 DNS 的接管方式（hijack 默认改写系统 DNS，disabled 保持不接管）
            "dns_mode": intent.tun.dns_mode,
            "route_exclude_address": route_exclude_address
        }));
    }

    inbounds
}

/// 从已有配置中提取 `tun-in` 的 `route_exclude_address`（已归一化）
pub fn extract_existing_tun_route_exclude_address(
    config_obj: &Map<String, Value>,
) -> Option<Vec<String>> {
    config_obj
        .get("inbounds")
        .and_then(|value| value.as_array())
        .and_then(|inbounds| {
            inbounds.iter().find(|inbound| {
                inbound.get("tag").and_then(|value| value.as_str()) == Some("tun-in")
            })
        })
        .and_then(|tun_in| tun_in.get("route_exclude_address"))
        .and_then(|value| value.as_array())
        .map(|route_exclude_address| {
            route_exclude_address
                .iter()
                .filter_map(|value| value.as_str().map(|cidr| cidr.to_string()))
                .collect::<Vec<_>>()
        })
        .and_then(|route_exclude_address| {
            normalize_persisted_tun_route_exclude_address(Some(route_exclude_address))
        })
}

/// 确保 `route.rules` 里存在 sniff 动作，返回其下标
pub fn ensure_sniff_route_rule(rules: &mut Vec<Value>) -> usize {
    if let Some(index) = rules
        .iter()
        .position(|rule| rule.get("action").and_then(|v| v.as_str()) == Some("sniff"))
    {
        index
    } else {
        rules.insert(0, json!({ "action": "sniff" }));
        0
    }
}

/// 构造假 DNS（FakeIP）server。
///
/// 生成路径直接把它放进 `dns.servers`；补丁路径先移除旧的再放新的——
/// 两者共用这一个定义，避免地址段 / 类型字段在两处漂移。
pub(crate) fn build_fakeip_dns_server(intent: &ConfigIntent) -> DnsServerConfig {
    DnsServerConfig {
        tag: DNS_FAKEIP.to_string(),
        server_type: Some("fakeip".to_string()),
        server: None,
        server_port: None,
        path: None,
        interface: None,
        inet4_range: Some(intent.fake_dns.ipv4_range.clone()),
        inet6_range: Some(intent.fake_dns.ipv6_range.clone()),
        domain_resolver: None,
        detour: None,
    }
}

/// 假 DNS server 的 JSON 形态（补丁路径直接在 `serde_json::Value` 上操作时使用）
pub fn fakeip_dns_server_value(intent: &ConfigIntent) -> Value {
    serde_json::to_value(build_fakeip_dns_server(intent)).expect("假 DNS server 序列化失败")
}

/// 同步 `dns.servers` 中的假 DNS server：先移除旧的，再按开关重建
pub fn sync_fake_dns_server(servers: &mut Vec<Value>, intent: &ConfigIntent) {
    servers.retain(|server| server.get("tag").and_then(|v| v.as_str()) != Some(DNS_FAKEIP));
    if intent.fake_dns_enabled() {
        servers.push(fakeip_dns_server_value(intent));
    }
}

/// 提取已有配置里假 DNS server 的地址段对（用于清理历史遗留的直连规则）
pub fn extract_fake_dns_server_range_pair(server: &Value) -> Option<(String, String)> {
    if server.get("tag").and_then(|v| v.as_str()) != Some(DNS_FAKEIP) {
        return None;
    }

    let inet4_range = server.get("inet4_range").and_then(|v| v.as_str())?.trim();
    let inet6_range = server.get("inet6_range").and_then(|v| v.as_str())?.trim();
    if inet4_range.is_empty() || inet6_range.is_empty() {
        return None;
    }

    Some((inet4_range.to_string(), inet6_range.to_string()))
}

/// 同步 `dns.rules` 中的假 DNS 规则。
///
/// ## 为什么用"路由到 fakeip server"而不是 1.14 的 `action: evaluate`
///
/// sing-box 官方 fakeip 的标准写法是把 A/AAAA 查询**路由**到 fakeip server；
/// `action: evaluate` + `tag` 那种写法只是给响应打标记，本身不做路由——
/// 没有配套的 `match_response` 规则时它不会产生任何效果。
///
/// 历史实现里生成路径写的是 `evaluate` 写法、补丁路径写的是本函数这种写法，
/// 而补丁路径（真正落到用户配置上的那条）会覆盖生成路径的结果——
/// 也就是说**运行时实际生效的始终是本函数这种写法**。
/// 双模型收敛时以"实际生效的语义"为准，而不是以某条分支的注释为准。
pub fn sync_fake_dns_rules(rules: &mut Vec<Value>, intent: &ConfigIntent) {
    // 先清理所有历史 fakeip 规则，避免重复叠加
    rules.retain(|rule| rule.get("server").and_then(|v| v.as_str()) != Some(DNS_FAKEIP));

    if !intent.fake_dns_enabled() {
        return;
    }

    if intent.fake_dns_filter_mode() == FAKE_DNS_FILTER_GLOBAL_NON_CN {
        // 全局非 CN：其他 A/AAAA 查询统一走 fakeip
        rules.push(json!({
            "query_type": ["A", "AAAA"],
            "server": DNS_FAKEIP
        }));
    } else {
        // 仅代理流量：仅对非 CN 域名走 fakeip，国内域名仍按原逻辑直连解析。
        let rule = json!({
            "query_type": ["A", "AAAA"],
            "rule_set": RS_GEOSITE_GEOLOCATION_NOT_CN,
            "server": DNS_FAKEIP
        });

        // 插到"非 CN"规则之前，保证非 CN 域名优先生成 fakeip 地址
        let insert_idx = rules
            .iter()
            .position(|item| {
                item.get("rule_set").and_then(|v| v.as_str()) == Some(RS_GEOSITE_GEOLOCATION_NOT_CN)
            })
            .unwrap_or(rules.len());
        rules.insert(insert_idx, rule);
    }
}

/// 同步 `route.rules` 中的假 DNS 直连网段规则。
///
/// `cleanup_pairs` 是需要一并清理的历史地址段对（用户改过地址段后，
/// 旧网段的直连规则会变成"打不开的网段直连"，必须清掉）。
pub fn sync_fake_dns_route_rules(
    rules: &mut Vec<Value>,
    intent: &ConfigIntent,
    cleanup_pairs: &[(String, String)],
) {
    rules.retain(|rule| !is_fake_dns_route_rule(rule, cleanup_pairs));

    if !intent.fake_dns_enabled() {
        return;
    }

    let (ipv4_range, ipv6_range) = (
        intent.fake_dns.ipv4_range.clone(),
        intent.fake_dns.ipv6_range.clone(),
    );
    rules.push(json!({
        "ip_cidr": [ipv4_range, ipv6_range],
        "outbound": TAG_DIRECT
    }));
}

/// 判断一条 route rule 是否是"假 DNS 地址段直连"规则
fn is_fake_dns_route_rule(rule: &Value, cleanup_pairs: &[(String, String)]) -> bool {
    let Some(ip_cidr) = rule.get("ip_cidr").and_then(|v| v.as_array()) else {
        return false;
    };
    let outbound = rule.get("outbound").and_then(|v| v.as_str()).unwrap_or("");
    if outbound != TAG_DIRECT {
        return false;
    }

    if ip_cidr.len() != 2 {
        return false;
    }

    cleanup_pairs.iter().any(|(inet4, inet6)| {
        [inet4.as_str(), inet6.as_str()]
            .iter()
            .all(|cidr| ip_cidr.iter().any(|v| v.as_str() == Some(*cidr)))
    })
}
