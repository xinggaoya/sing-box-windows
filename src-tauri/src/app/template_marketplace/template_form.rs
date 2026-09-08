//! 模板可视化表单：把官方骨架（generate_base_config）的关键配置抽象成结构化表单，
//! 用户无需手写 JSON 即可制作模板。提供 表单 → 骨架 JSON 与 骨架 JSON → 表单 双向转换。
//!
//! 设计说明：
//! - 表单里"留空"的字段生成时回落到当前应用设置（AppConfig），模板只固化作者明确选择的部分。
//! - 自定义分流规则复用 `CustomRule` 模型与 `inject_custom_rules` 注入逻辑（优先于内置 CN/私网分流）。
//! - `parse_form_from_template` 只识别"表单可表达"的骨架：解析失败时调用方应回退 JSON 编辑模式。

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::HashSet;
use ts_rs::TS;

use crate::app::singbox::common::{
    normalize_default_outbound, DNS_CN, DNS_FAKEIP, DNS_MDNS, DNS_PROXY, DNS_RESOLVER,
    PRIVATE_IP_CIDRS, RS_GEOIP_CN, RS_GEOIP_PRIVATE, RS_GEOSITE_ADS, RS_GEOSITE_CN,
    RS_GEOSITE_GEOLOCATION_NOT_CN, RS_GEOSITE_GOOGLE, RS_GEOSITE_NETFLIX, RS_GEOSITE_OPENAI,
    RS_GEOSITE_PRIVATE, RS_GEOSITE_TELEGRAM, RS_GEOSITE_YOUTUBE, TAG_AUTO, TAG_DIRECT, TAG_GOOGLE,
    TAG_MANUAL, TAG_NETFLIX, TAG_OPENAI, TAG_TELEGRAM, TAG_YOUTUBE,
};
use crate::app::singbox::config_generator::{generate_base_config, inject_custom_rules};
use crate::app::storage::custom_rule::{CustomRule, CustomRuleAction, CustomRuleMatchType};
use crate::app::storage::state_model::AppConfig;

/// 业务分流组 tag 与对应规则集 tag（顺序一一对应）。
const APP_GROUP_TAGS: [&str; 5] = [
    TAG_TELEGRAM,
    TAG_YOUTUBE,
    TAG_NETFLIX,
    TAG_OPENAI,
    TAG_GOOGLE,
];
const APP_GROUP_RULE_SETS: [&str; 5] = [
    RS_GEOSITE_TELEGRAM,
    RS_GEOSITE_YOUTUBE,
    RS_GEOSITE_NETFLIX,
    RS_GEOSITE_OPENAI,
    RS_GEOSITE_GOOGLE,
];

/// 骨架内置的标准规则集 tag（出现在这些 tag 上的规则不属于自定义规则）。
const STANDARD_RULE_SETS: [&str; 11] = [
    RS_GEOSITE_ADS,
    RS_GEOSITE_PRIVATE,
    RS_GEOSITE_CN,
    RS_GEOSITE_GEOLOCATION_NOT_CN,
    RS_GEOSITE_TELEGRAM,
    RS_GEOSITE_YOUTUBE,
    RS_GEOSITE_NETFLIX,
    RS_GEOSITE_OPENAI,
    RS_GEOSITE_GOOGLE,
    RS_GEOIP_CN,
    RS_GEOIP_PRIVATE,
];

/// 可视化模板表单。留空字段生成时跟随当前应用设置。
#[derive(Debug, Clone, Serialize, Deserialize, TS, Default)]
#[ts(export, export_to = "../src/types/generated/TemplateFormOptions.ts")]
pub struct TemplateFormOptions {
    /// 非国内流量默认出站：manual（手动切换）/ auto（自动选择）
    pub default_outbound: String,
    /// 自动选择组测速地址；None 跟随应用设置
    #[ts(optional)]
    pub urltest_url: Option<String>,
    /// 广告拦截（DNS + 路由双层 reject）
    #[serde(default)]
    pub block_ads: bool,
    /// DNS 劫持（hijack-dns）
    #[serde(default)]
    pub dns_hijack: bool,
    /// Fake DNS（fakeip）
    #[serde(default)]
    pub fake_dns_enabled: bool,
    /// mDNS（*.local 本地多播解析）
    #[serde(default)]
    pub dns_use_mdns: bool,
    /// 启用的业务分流组 tag：Telegram/YouTube/Netflix/OpenAI/Google
    #[serde(default)]
    pub app_groups: Vec<String>,
    /// 代理 DNS（DoH）；None 跟随应用设置
    #[ts(optional)]
    pub dns_proxy: Option<String>,
    /// 国内 DNS；None 跟随应用设置
    #[ts(optional)]
    pub dns_cn: Option<String>,
    /// 默认 DNS 解析；None 跟随应用设置
    #[ts(optional)]
    pub dns_resolver: Option<String>,
    /// 自定义分流规则（优先级高于内置 CN/私网分流）
    #[serde(default)]
    pub custom_rules: Vec<CustomRule>,
}

fn normalize_outbound_choice(value: &str) -> &'static str {
    if value.trim().eq_ignore_ascii_case("auto") {
        "auto"
    } else {
        "manual"
    }
}

fn non_empty(value: &Option<String>) -> Option<String> {
    value
        .as_deref()
        .map(str::trim)
        .filter(|s| !s.is_empty())
        .map(str::to_string)
}

/// 由表单生成模板骨架（不含订阅节点；节点在订阅生成时注入 `{{NODES}}` 占位符或追加）。
pub fn generate_config_from_form(
    app_config: &AppConfig,
    form: &TemplateFormOptions,
) -> Result<Value, String> {
    // 1) 表单覆盖项落到 AppConfig（生成器只认 AppConfig 参数）
    let mut cfg = app_config.clone();
    cfg.singbox_default_proxy_outbound =
        normalize_outbound_choice(&form.default_outbound).to_string();
    if let Some(url) = non_empty(&form.urltest_url) {
        cfg.singbox_urltest_url = url;
    }
    cfg.singbox_block_ads = form.block_ads;
    cfg.singbox_dns_hijack = form.dns_hijack;
    cfg.singbox_fake_dns_enabled = form.fake_dns_enabled;
    cfg.singbox_dns_use_mdns = form.dns_use_mdns;
    if let Some(v) = non_empty(&form.dns_proxy) {
        cfg.singbox_dns_proxy = v;
    }
    if let Some(v) = non_empty(&form.dns_cn) {
        cfg.singbox_dns_cn = v;
    }
    if let Some(v) = non_empty(&form.dns_resolver) {
        cfg.singbox_dns_resolver = v;
    }
    cfg.singbox_enable_app_groups = form
        .app_groups
        .iter()
        .any(|g| APP_GROUP_TAGS.contains(&g.as_str()));

    // 2) 官方骨架
    let mut config = generate_base_config(&cfg);

    // 3) 裁剪未启用的业务分流组（出站 selector + 对应路由规则）
    retain_app_groups(&mut config, &form.app_groups);

    // 4) 注入自定义规则（优先于内置 CN/私网分流）
    let default_tag = normalize_default_outbound(&cfg);
    inject_custom_rules(&mut config, &form.custom_rules, default_tag);

    Ok(config)
}

/// 从 outbounds/route.rules 中移除未启用的业务分流组。
/// 节点注入阶段 `ensure_app_group_selectors` 对缺失组是跳过的，裁剪结果在订阅生成后仍保留。
fn retain_app_groups(config: &mut Value, enabled: &[String]) {
    let enabled: HashSet<&str> = enabled.iter().map(|s| s.as_str()).collect();

    let removed_tags: Vec<&str> = APP_GROUP_TAGS
        .iter()
        .copied()
        .filter(|t| !enabled.contains(t))
        .collect();
    if removed_tags.is_empty() {
        return;
    }
    let removed_rule_sets: Vec<&str> = removed_tags
        .iter()
        .map(|t| APP_GROUP_RULE_SETS[APP_GROUP_TAGS.iter().position(|g| g == t).unwrap()])
        .collect();

    if let Some(outbounds) = config.get_mut("outbounds").and_then(|v| v.as_array_mut()) {
        outbounds.retain(|ob| {
            !matches!(ob.get("tag").and_then(|v| v.as_str()), Some(t) if removed_tags.contains(&t))
        });
    }
    if let Some(rules) = config
        .get_mut("route")
        .and_then(|r| r.get_mut("rules"))
        .and_then(|v| v.as_array_mut())
    {
        rules.retain(|rule| {
            !matches!(rule.get("rule_set").and_then(|v| v.as_str()), Some(rs) if removed_rule_sets.contains(&rs))
        });
    }
}

/// 尝试把模板骨架解析回表单。骨架不是"表单可表达"形状时返回 Err（调用方回退 JSON 模式）。
pub fn parse_form_from_template(content: &str) -> Result<TemplateFormOptions, String> {
    let value: Value =
        serde_json::from_str(content.trim()).map_err(|e| format!("模板不是合法 JSON: {e}"))?;
    let obj = value.as_object().ok_or("模板内容必须是 JSON 对象")?;

    let mut form = TemplateFormOptions::default();

    // 默认出站（route.final）
    let final_outbound = obj
        .get("route")
        .and_then(|r| r.get("final"))
        .and_then(|v| v.as_str())
        .unwrap_or(TAG_MANUAL);
    form.default_outbound = match final_outbound {
        t if t == TAG_AUTO => "auto".to_string(),
        t if t == TAG_MANUAL => "manual".to_string(),
        other => {
            return Err(format!(
                "模板默认出站「{other}」无法用表单表达，请使用 JSON 模式编辑"
            ))
        }
    };

    let outbounds = obj
        .get("outbounds")
        .and_then(|v| v.as_array())
        .ok_or("模板缺少 outbounds")?;
    let outbound_tags: HashSet<&str> = outbounds
        .iter()
        .filter_map(|o| o.get("tag").and_then(|v| v.as_str()))
        .collect();

    // 测速地址
    form.urltest_url = outbounds
        .iter()
        .find(|o| o.get("tag").and_then(|v| v.as_str()) == Some(TAG_AUTO))
        .and_then(|o| o.get("url"))
        .and_then(|v| v.as_str())
        .map(str::to_string);

    // 业务分流组
    form.app_groups = APP_GROUP_TAGS
        .iter()
        .filter(|t| outbound_tags.contains(**t))
        .map(|t| t.to_string())
        .collect();

    // DNS 服务器
    let dns_servers = obj
        .get("dns")
        .and_then(|d| d.get("servers"))
        .and_then(|v| v.as_array());
    let server_addr = |tag: &str| -> Option<String> {
        dns_servers?
            .iter()
            .find(|s| s.get("tag").and_then(|v| v.as_str()) == Some(tag))
            .and_then(reconstruct_dns_address)
    };
    form.dns_proxy = server_addr(DNS_PROXY);
    form.dns_cn = server_addr(DNS_CN);
    form.dns_resolver = server_addr(DNS_RESOLVER);
    let has_dns_server = |tag: &str| -> bool {
        dns_servers
            .map(|s| {
                s.iter()
                    .any(|x| x.get("tag").and_then(|v| v.as_str()) == Some(tag))
            })
            .unwrap_or(false)
    };
    form.fake_dns_enabled = has_dns_server(DNS_FAKEIP);
    form.dns_use_mdns = has_dns_server(DNS_MDNS);

    // 路由规则
    let empty_rules: Vec<Value> = Vec::new();
    let route_rules = obj
        .get("route")
        .and_then(|r| r.get("rules"))
        .and_then(|v| v.as_array())
        .unwrap_or(&empty_rules);

    form.block_ads = route_rules.iter().any(|r| {
        r.get("rule_set").and_then(|v| v.as_str()) == Some(RS_GEOSITE_ADS)
            && r.get("action").and_then(|v| v.as_str()) == Some("reject")
    });
    form.dns_hijack = route_rules.iter().any(|r| {
        r.get("protocol").and_then(|v| v.as_str()) == Some("dns")
            && r.get("action").and_then(|v| v.as_str()) == Some("hijack-dns")
    });

    let mut custom_rules = Vec::new();
    for rule in route_rules {
        if !is_standard_route_rule(rule) {
            custom_rules
                .push(custom_rule_from_route_rule(rule).map_err(|e| format!("{e}；rule={rule}"))?);
        }
    }
    form.custom_rules = custom_rules;

    Ok(form)
}

/// 把骨架里的 DNS 服务器对象重建为应用设置语义的地址字符串（与 build_dns_server_config 互逆）。
/// 无法识别的类型返回 None（表单留空 = 跟随应用设置）。
fn reconstruct_dns_address(server: &Value) -> Option<String> {
    let server_type = server.get("type").and_then(|v| v.as_str())?;
    let host = server.get("server").and_then(|v| v.as_str())?;
    let port = server.get("server_port").and_then(|v| v.as_u64());
    let path = server.get("path").and_then(|v| v.as_str()).unwrap_or("");

    let with_port = |default: u64| -> String {
        match port {
            Some(p) if p != default => format!(":{p}"),
            _ => String::new(),
        }
    };

    match server_type {
        "local" => Some("local".to_string()),
        "dhcp" => Some(format!("dhcp://{host}")),
        "https" | "h3" => Some(format!("{server_type}://{host}{}{path}", with_port(443))),
        "quic" => Some(format!("quic://{host}{}", with_port(443))),
        "tls" => Some(format!("tls://{host}{}", with_port(853))),
        "tcp" => Some(format!("tcp://{host}{}", with_port(53))),
        "udp" => Some(match port {
            Some(p) => format!("{host}:{p}"),
            None => host.to_string(),
        }),
        _ => None,
    }
}

/// 判断是否为骨架内置规则（非用户自定义）。
fn is_standard_route_rule(rule: &Value) -> bool {
    // 动作类内置规则
    if matches!(
        rule.get("action").and_then(|v| v.as_str()),
        Some("sniff") | Some("resolve")
    ) {
        return true;
    }
    if rule.get("protocol").is_some() {
        return true; // hijack-dns
    }
    if rule.get("clash_mode").is_some() || rule.get("server").is_some() {
        return true;
    }

    // 标准 rule_set（字符串或数组形式）
    if let Some(rs) = rule.get("rule_set") {
        if let Some(tag) = rs.as_str() {
            if STANDARD_RULE_SETS.contains(&tag) {
                return true;
            }
            return false; // 未知 rule_set 交由 custom_rule_from_route_rule 报错
        }
        if let Some(list) = rs.as_array() {
            // [geosite-cn, geoip-cn] 合并直连段
            return list
                .iter()
                .filter_map(|v| v.as_str())
                .all(|tag| STANDARD_RULE_SETS.contains(&tag));
        }
    }

    // 私网直连段 / fakeip 回收段
    if let Some(cidrs) = rule.get("ip_cidr").and_then(|v| v.as_array()) {
        let cidr_strs: Vec<&str> = cidrs.iter().filter_map(|v| v.as_str()).collect();
        if cidr_strs.contains(&PRIVATE_IP_CIDRS[0]) || cidr_strs.contains(&"198.18.0.0/15") {
            return true;
        }
    }

    false
}

/// 把"表单可表达"的自定义路由规则还原成 CustomRule。
fn custom_rule_from_route_rule(rule: &Value) -> Result<CustomRule, String> {
    const MATCH_FIELDS: [(&str, CustomRuleMatchType); 4] = [
        ("domain", CustomRuleMatchType::Domain),
        ("domain_suffix", CustomRuleMatchType::DomainSuffix),
        ("domain_keyword", CustomRuleMatchType::DomainKeyword),
        ("ip_cidr", CustomRuleMatchType::IpCidr),
    ];

    let matched: Vec<(&str, &Value)> = MATCH_FIELDS
        .iter()
        .filter_map(|(field, _)| rule.get(*field).map(|v| (*field, v)))
        .collect();
    if matched.len() != 1 {
        return Err(format!(
            "规则匹配字段数量为 {}（需恰好 1 个），无法用表单表达，请使用 JSON 模式编辑",
            matched.len()
        ));
    }
    let (field, values) = matched[0];
    let match_type = MATCH_FIELDS
        .iter()
        .find(|(f, _)| *f == field)
        .map(|(_, kind)| kind.clone())
        .expect("field 来自 MATCH_FIELDS，必然存在");

    let payload = values
        .as_array()
        .map(|arr| {
            arr.iter()
                .filter_map(|v| v.as_str())
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_default();
    if payload.is_empty() {
        return Err("路由规则匹配内容为空，无法用表单表达".to_string());
    }

    let action = if rule.get("action").and_then(|v| v.as_str()) == Some("reject") {
        CustomRuleAction::Block
    } else {
        match rule.get("outbound").and_then(|v| v.as_str()) {
            Some(t) if t == TAG_DIRECT => CustomRuleAction::Direct,
            Some(t) if t == TAG_AUTO || t == TAG_MANUAL => CustomRuleAction::Proxy,
            Some(other) => {
                return Err(format!(
                    "规则出站「{other}」无法用表单表达，请使用 JSON 模式编辑"
                ))
            }
            None => return Err("路由规则缺少出站/动作，无法用表单表达".to_string()),
        }
    };

    let now = chrono::Utc::now();
    Ok(CustomRule {
        id: uuid::Uuid::new_v4().to_string(),
        enabled: true,
        match_type,
        payload,
        action,
        outbound: None,
        note: None,
        created_at: now,
        updated_at: now,
    })
}

#[cfg(test)]
#[path = "template_form.tests.rs"]
mod tests;
