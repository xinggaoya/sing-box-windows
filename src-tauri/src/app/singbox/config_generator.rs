use crate::app::singbox::settings_patch::apply_app_settings_to_config;
use crate::app::storage::state_model::AppConfig;
use serde_json::{json, Value};

// 代理组/出站标签（这些标签会暴露在 Clash API 里，尽量保持稳定，避免前端/用户习惯被破坏）。
pub const TAG_AUTO: &str = "自动选择";
pub const TAG_MANUAL: &str = "手动切换";
pub const TAG_DIRECT: &str = "direct";
pub const TAG_BLOCK: &str = "block";

// 业务分流组（可选，但对大多数用户比较实用）
pub const TAG_TELEGRAM: &str = "Telegram";
pub const TAG_YOUTUBE: &str = "YouTube";
pub const TAG_NETFLIX: &str = "Netflix";
pub const TAG_OPENAI: &str = "OpenAI";

// DNS server tags
const DNS_PROXY: &str = "dns_proxy";
const DNS_CN: &str = "dns_cn";
const DNS_RESOLVER: &str = "dns_resolver";
const DNS_BLOCK: &str = "dns_block";

// Rule-set tags (官方 SagerNet 规则集)
const RS_GEOSITE_CN: &str = "geosite-cn";
const RS_GEOSITE_GEOLOCATION_NOT_CN: &str = "geosite-geolocation-!cn";
const RS_GEOSITE_PRIVATE: &str = "geosite-private";
const RS_GEOSITE_ADS: &str = "geosite-category-ads-all";
const RS_GEOSITE_TELEGRAM: &str = "geosite-telegram";
const RS_GEOSITE_YOUTUBE: &str = "geosite-youtube";
const RS_GEOSITE_NETFLIX: &str = "geosite-netflix";
const RS_GEOSITE_OPENAI: &str = "geosite-openai";
const RS_GEOIP_CN: &str = "geoip-cn";
const RS_GEOIP_PRIVATE: &str = "geoip-private";

fn normalize_default_outbound(app_config: &AppConfig) -> &'static str {
    match app_config.singbox_default_proxy_outbound.as_str() {
        "auto" => TAG_AUTO,
        _ => TAG_MANUAL,
    }
}

fn normalize_download_detour(app_config: &AppConfig) -> &'static str {
    match app_config.singbox_download_detour.as_str() {
        "direct" => TAG_DIRECT,
        // 订阅规则集/Clash UI 下载默认走“手动切换”，便于用户用可用节点下载（适配国内网络）
        _ => TAG_MANUAL,
    }
}

/// 生成一份“通用且更适合国内环境”的 sing-box 配置骨架（不依赖模板文件）。
///
/// 目标：
/// - 默认规则：国内域名/IP 直连，其他走代理（可“绕过国内域名”）。
/// - DNS：国内用国内 DNS，非国内用 DoH（尽量避免污染）。
/// - 兼容：保留 Clash API（前端节点选择/延迟测试依赖）。
pub fn generate_base_config(app_config: &AppConfig) -> Value {
    let dns_strategy = if app_config.prefer_ipv6 {
        "prefer_ipv6"
    } else {
        "ipv4_only"
    };

    let default_outbound = normalize_default_outbound(app_config);
    let download_detour = normalize_download_detour(app_config);

    let mut outbounds: Vec<Value> = vec![
        json!({
            "type": "urltest",
            "tag": TAG_AUTO,
            "outbounds": [TAG_DIRECT],
            "url": app_config.singbox_urltest_url,
            // 保障切换节点时主动中断旧连接，避免连接数长期堆积
            "interrupt_exist_connections": true,
            // 缩短空闲回收时间，配合上面的中断行为防止连接滞留
            "idle_timeout": "10m",
            "interval": "3m",
            "tolerance": 50
        }),
        json!({
            "type": "selector",
            "tag": TAG_MANUAL,
            "outbounds": [TAG_AUTO, TAG_DIRECT]
        }),
    ];

    // 应用分流组：默认开启（对大多数用户比较实用），可在设置页关闭。
    if app_config.singbox_enable_app_groups {
        outbounds.extend([
            json!({
                "type": "selector",
                "tag": TAG_TELEGRAM,
                "outbounds": [TAG_MANUAL, TAG_AUTO, TAG_DIRECT]
            }),
            json!({
                "type": "selector",
                "tag": TAG_YOUTUBE,
                "outbounds": [TAG_MANUAL, TAG_AUTO, TAG_DIRECT]
            }),
            json!({
                "type": "selector",
                "tag": TAG_NETFLIX,
                "outbounds": [TAG_MANUAL, TAG_AUTO, TAG_DIRECT]
            }),
            json!({
                "type": "selector",
                "tag": TAG_OPENAI,
                "outbounds": [TAG_MANUAL, TAG_AUTO, TAG_DIRECT]
            }),
        ]);
    }

    outbounds.extend([
        json!({ "type": "direct", "tag": TAG_DIRECT }),
        json!({ "type": "block", "tag": TAG_BLOCK }),
    ]);

    let mut dns_rules: Vec<Value> = vec![
        json!({ "clash_mode": "direct", "server": DNS_CN }),
        json!({ "clash_mode": "global", "server": DNS_PROXY }),
        json!({ "rule_set": [RS_GEOSITE_CN, RS_GEOIP_CN], "server": DNS_CN }),
        json!({ "rule_set": RS_GEOSITE_GEOLOCATION_NOT_CN, "server": DNS_PROXY }),
    ];

    if app_config.singbox_block_ads {
        dns_rules.insert(2, json!({ "rule_set": RS_GEOSITE_ADS, "server": DNS_BLOCK }));
    }

    let mut rule_sets: Vec<Value> = Vec::new();
    if app_config.singbox_block_ads {
        rule_sets.push(json!({
            "tag": RS_GEOSITE_ADS,
            "type": "remote",
            "format": "binary",
            "url": "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-category-ads-all.srs",
            "download_detour": download_detour,
            "update_interval": "1d"
        }));
    }

    rule_sets.extend([
        json!({
            "tag": RS_GEOSITE_CN,
            "type": "remote",
            "format": "binary",
            "url": "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-cn.srs",
            "download_detour": download_detour,
            "update_interval": "1d"
        }),
        json!({
            "tag": RS_GEOSITE_GEOLOCATION_NOT_CN,
            "type": "remote",
            "format": "binary",
            "url": "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-geolocation-!cn.srs",
            "download_detour": download_detour,
            "update_interval": "1d"
        }),
    ]);

    if app_config.singbox_enable_app_groups {
        rule_sets.extend([
            json!({
                "tag": RS_GEOSITE_TELEGRAM,
                "type": "remote",
                "format": "binary",
                "url": "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-telegram.srs",
                "download_detour": download_detour,
                "update_interval": "7d"
            }),
            json!({
                "tag": RS_GEOSITE_YOUTUBE,
                "type": "remote",
                "format": "binary",
                "url": "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-youtube.srs",
                "download_detour": download_detour,
                "update_interval": "7d"
            }),
            json!({
                "tag": RS_GEOSITE_NETFLIX,
                "type": "remote",
                "format": "binary",
                "url": "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-netflix.srs",
                "download_detour": download_detour,
                "update_interval": "7d"
            }),
            json!({
                "tag": RS_GEOSITE_OPENAI,
                "type": "remote",
                "format": "binary",
                "url": "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-openai.srs",
                "download_detour": download_detour,
                "update_interval": "7d"
            }),
        ]);
    }

    rule_sets.extend([
        json!({
            "tag": RS_GEOSITE_PRIVATE,
            "type": "remote",
            "format": "binary",
            "url": "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-private.srs",
            "download_detour": TAG_DIRECT,
            "update_interval": "7d"
        }),
        json!({
            "tag": RS_GEOIP_PRIVATE,
            "type": "remote",
            "format": "binary",
            "url": "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set/geoip-private.srs",
            "download_detour": TAG_DIRECT,
            "update_interval": "7d"
        }),
        json!({
            "tag": RS_GEOIP_CN,
            "type": "remote",
            "format": "binary",
            "url": "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set/geoip-cn.srs",
            "download_detour": download_detour,
            "update_interval": "1d"
        }),
    ]);

    let mut route_rules: Vec<Value> = vec![
        json!({ "action": "sniff" }),
    ];

    if app_config.singbox_dns_hijack {
        route_rules.push(json!({ "protocol": "dns", "action": "hijack-dns" }));
    }

    route_rules.extend([
        json!({ "clash_mode": "global", "outbound": default_outbound }),
        json!({ "clash_mode": "direct", "outbound": TAG_DIRECT }),
    ]);

    if app_config.singbox_block_ads {
        route_rules.push(json!({ "rule_set": RS_GEOSITE_ADS, "action": "reject" }));
    }

    if app_config.singbox_enable_app_groups {
        route_rules.extend([
            json!({ "rule_set": RS_GEOSITE_TELEGRAM, "outbound": TAG_TELEGRAM }),
            json!({ "rule_set": RS_GEOSITE_YOUTUBE, "outbound": TAG_YOUTUBE }),
            json!({ "rule_set": RS_GEOSITE_NETFLIX, "outbound": TAG_NETFLIX }),
            json!({ "rule_set": RS_GEOSITE_OPENAI, "outbound": TAG_OPENAI }),
        ]);
    }

    route_rules.extend([
        json!({ "rule_set": [RS_GEOSITE_PRIVATE, RS_GEOIP_PRIVATE], "outbound": TAG_DIRECT }),
        json!({ "rule_set": [RS_GEOSITE_CN, RS_GEOIP_CN], "outbound": TAG_DIRECT }),
        json!({ "rule_set": RS_GEOSITE_GEOLOCATION_NOT_CN, "outbound": default_outbound }),
    ]);

    // 注意：这里的 outbounds 只是骨架，订阅节点注入后会补齐 TAG_AUTO/TAG_MANUAL 的候选列表。
    let mut config = json!({
        "log": {
            "disabled": false,
            "level": "info",
            "timestamp": true
        },
        "experimental": {
            "cache_file": { "enabled": true },
            "clash_api": {
                "external_controller": format!("127.0.0.1:{}", app_config.api_port),
                "external_ui": "metacubexd",
                // 让 sing-box 自动下载 UI（国内网络可能被墙，下载走代理可提高成功率）
                "external_ui_download_url": "https://github.com/MetaCubeX/metacubexd/archive/refs/heads/gh-pages.zip",
                "external_ui_download_detour": download_detour,
                "default_mode": "rule"
            }
        },
        "dns": {
            "servers": [
                {
                    "tag": DNS_PROXY,
                    "address": app_config.singbox_dns_proxy,
                    "address_resolver": DNS_RESOLVER,
                    "strategy": dns_strategy,
                    // DNS_PROXY 默认走代理（更符合“防污染/可解析被墙域名”的预期），
                    // 但为了避免形成“DNS 走代理 -> 代理节点域名又需要 DNS”的循环依赖，
                    // 我们会在注入节点时为每个节点设置 `domain_resolver=dns_resolver`（直连解析节点域名）。
                    "detour": default_outbound
                },
                {
                    "tag": DNS_CN,
                    "address": app_config.singbox_dns_cn,
                    "address_resolver": DNS_RESOLVER,
                    "strategy": dns_strategy,
                    "detour": TAG_DIRECT
                },
                {
                    "tag": DNS_RESOLVER,
                    "address": app_config.singbox_dns_resolver,
                    "strategy": dns_strategy,
                    "detour": TAG_DIRECT
                },
                {
                    "tag": DNS_BLOCK,
                    "address": "rcode://success"
                }
            ],
            "rules": dns_rules,
            "independent_cache": true,
            "final": DNS_PROXY
        },
        "inbounds": [],
        "outbounds": outbounds,
        "route": {
            "rule_set": rule_sets,
            "rules": route_rules,
            "final": default_outbound,
            "auto_detect_interface": true
        }
    });

    // 统一由 settings_patch 负责把端口/TUN/IPv6 偏好写入配置，确保行为一致。
    apply_app_settings_to_config(&mut config, app_config);
    config
}

/// 基于骨架配置注入节点，并更新“自动选择/手动切换”等组的候选列表。
pub fn generate_config_with_nodes(app_config: &AppConfig, nodes: &[Value]) -> Result<Value, String> {
    let mut config = generate_base_config(app_config);
    inject_nodes(&mut config, app_config, nodes)?;
    Ok(config)
}

pub fn inject_nodes(config: &mut Value, app_config: &AppConfig, nodes: &[Value]) -> Result<(), String> {
    let outbounds = ensure_outbounds_array(config)?;

    // 预先收集已有 tag，避免节点 tag 与内置出站/分组冲突。
    let mut existing_tags = std::collections::HashSet::<String>::new();
    for ob in outbounds.iter() {
        if let Some(tag) = ob.get("tag").and_then(|t| t.as_str()) {
            existing_tags.insert(tag.to_string());
        }
    }

    let mut normalized_nodes = Vec::<Value>::with_capacity(nodes.len());
    // 用于注入到“自动选择/手动切换”等分组的节点列表。
    // 注意：订阅里可能会夹带“提示节点/占位节点”（如 server=0.0.0.0），放进 urltest 会导致启动时默认选中无效节点，表现为全部无法联网。
    let mut group_node_tags = Vec::<String>::with_capacity(nodes.len());

    let resolver_strategy = if app_config.prefer_ipv6 {
        "prefer_ipv6"
    } else {
        // 节点域名解析默认走 IPv4，能显著降低“有 AAAA 但本机 IPv6 不可用”导致的连接失败。
        "ipv4_only"
    };

    for (idx, node) in nodes.iter().cloned().enumerate() {
        let mut node_obj = node
            .as_object()
            .cloned()
            .ok_or_else(|| format!("节点不是对象: index={}", idx))?;

        let raw_tag = node_obj
            .get("tag")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .to_string();
        if raw_tag.is_empty() {
            return Err(format!("节点缺少 tag: index={}", idx));
        }

        // 若 tag 冲突，则自动改名，避免覆盖内置分组/出站。
        let mut tag = raw_tag.clone();
        if existing_tags.contains(&tag) {
            tag = format!("节点-{}-{}", raw_tag, idx);
        }
        existing_tags.insert(tag.clone());
        node_obj.insert("tag".to_string(), Value::String(tag.clone()));

        // 为“节点 server 是域名”的出站补上 domain_resolver，避免出现 DNS 循环依赖：
        // - DNS_PROXY 的 DoH/DoH3 可以走代理出站（防污染/可解析被墙域名）
        // - 代理节点本身的域名用 dns_resolver（直连）解析
        // 这样即便 DNS_PROXY 需要走代理，也不会反过来依赖 DNS_PROXY 来解析节点域名。
        if let Some(server) = node_obj.get("server").and_then(|v| v.as_str()) {
            let server = server.trim();
            if !server.is_empty()
                && server != "0.0.0.0"
                && server.parse::<std::net::IpAddr>().is_err()
                && !node_obj.contains_key("domain_resolver")
            {
                node_obj.insert(
                    "domain_resolver".to_string(),
                    json!({
                        "server": DNS_RESOLVER,
                        "strategy": resolver_strategy
                    }),
                );
            }
        }

        // 只把“看起来可用”的节点加入分组候选，避免 urltest 初始选择到无效节点（如 server=0.0.0.0）。
        if should_include_node_in_groups(&node_obj) {
            group_node_tags.push(tag.clone());
        }
        normalized_nodes.push(Value::Object(node_obj));
    }

    // 1) 更新 TAG_AUTO(urltest) 只包含节点（避免把 direct 当作最快导致全直连）。
    // 2) 更新 TAG_MANUAL(selector) 包含自动选择 + 每个节点 + direct（便于用户手动回退直连）。
    ensure_urltest_and_selector(outbounds, &group_node_tags)?;

    // 追加节点出站
    for node in normalized_nodes {
        outbounds.push(node);
    }

    Ok(())
}

fn should_include_node_in_groups(node_obj: &serde_json::Map<String, Value>) -> bool {
    // 订阅里经常会夹带提示节点：server=0.0.0.0 或空字符串。
    // 这些节点在 Clash 内核里通常不会被默认选中，但放进 sing-box 的 urltest 初始候选会导致“启动即断网”。
    let server = node_obj
        .get("server")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .trim();

    if server.is_empty() {
        return false;
    }
    // 明确屏蔽不可路由地址
    if server == "0.0.0.0" {
        return false;
    }

    true
}

fn ensure_outbounds_array(config: &mut Value) -> Result<&mut Vec<Value>, String> {
    let root = config
        .as_object_mut()
        .ok_or_else(|| "配置根不是 JSON 对象".to_string())?;
    if !root.contains_key("outbounds") {
        root.insert("outbounds".to_string(), json!([]));
    }
    root.get_mut("outbounds")
        .and_then(|v| v.as_array_mut())
        .ok_or_else(|| "outbounds 不是数组".to_string())
}

fn ensure_urltest_and_selector(outbounds: &mut Vec<Value>, node_tags: &[String]) -> Result<(), String> {
    let auto_idx = ensure_outbound_index(outbounds, TAG_AUTO, || {
        json!({
            "type": "urltest",
            "tag": TAG_AUTO,
            "outbounds": [],
            "interrupt_exist_connections": true,
            "idle_timeout": "10m",
            "url": "http://cp.cloudflare.com/generate_204",
            "interval": "3m",
            "tolerance": 50
        })
    })?;

    let manual_idx = ensure_outbound_index(outbounds, TAG_MANUAL, || {
        json!({
            "type": "selector",
            "tag": TAG_MANUAL,
            "outbounds": []
        })
    })?;

    // 自动选择候选列表
    let auto_list = if node_tags.is_empty() {
        vec![Value::String(TAG_DIRECT.to_string())]
    } else {
        node_tags.iter().cloned().map(Value::String).collect()
    };
    {
        let auto = outbounds
            .get_mut(auto_idx)
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| format!("outbound(tag={}) 不是对象", TAG_AUTO))?;
        auto.insert("outbounds".to_string(), Value::Array(auto_list));
    }

    // 手动切换候选列表：自动选择 + 每个节点 + direct
    let mut manual_list = Vec::<Value>::with_capacity(2 + node_tags.len());
    manual_list.push(Value::String(TAG_AUTO.to_string()));
    for tag in node_tags {
        manual_list.push(Value::String(tag.clone()));
    }
    manual_list.push(Value::String(TAG_DIRECT.to_string()));
    {
        let manual = outbounds
            .get_mut(manual_idx)
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| format!("outbound(tag={}) 不是对象", TAG_MANUAL))?;
        manual.insert("outbounds".to_string(), Value::Array(manual_list));
    }

    Ok(())
}

fn ensure_outbound_index<F>(
    outbounds: &mut Vec<Value>,
    tag: &str,
    create: F,
) -> Result<usize, String>
where
    F: FnOnce() -> Value,
{
    if let Some((idx, _)) = outbounds
        .iter()
        .enumerate()
        .find(|(_, o)| o.get("tag").and_then(|t| t.as_str()) == Some(tag))
    {
        return Ok(idx);
    }

    outbounds.push(create());
    Ok(outbounds.len().saturating_sub(1))
}
