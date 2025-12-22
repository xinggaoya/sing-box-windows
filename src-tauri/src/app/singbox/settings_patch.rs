use crate::app::core::tun_profile::TUN_ROUTE_EXCLUDES;
use crate::app::storage::state_model::AppConfig;
use super::common::{
    dns_strategy, normalize_default_outbound, normalize_download_detour, DNS_BLOCK, DNS_CN, DNS_PROXY,
    DNS_RESOLVER, RS_GEOSITE_ADS, RS_GEOSITE_GEOLOCATION_NOT_CN, RS_GEOSITE_NETFLIX,
    RS_GEOSITE_OPENAI, RS_GEOSITE_TELEGRAM, RS_GEOSITE_YOUTUBE, TAG_AUTO,
    TAG_NETFLIX, TAG_OPENAI, TAG_TELEGRAM, TAG_YOUTUBE,
};
use serde_json::{json, Map, Value};

/// 将应用设置（端口 / System Proxy / TUN / IPv6 偏好等）同步到 sing-box 配置。
///
/// 说明：
/// - 这是“设置页面操作会影响配置”的核心入口之一。
/// - 该函数会覆盖/重建 `inbounds`，确保 mixed/tun 与端口设置始终与 AppConfig 一致。
pub fn apply_app_settings_to_config(config: &mut Value, app_config: &AppConfig) {
    if let Some(config_obj) = config.as_object_mut() {
        apply_inbounds_settings(config_obj, app_config);

        // 针对“本程序生成的订阅配置”，尝试同步高级选项。
        // 采用“按 tag 定位并局部更新”的方式：如果用户导入的是原始订阅配置（结构不同），则不会强行改动。
        apply_profile_settings_if_present(config_obj, app_config);

        // clash_api 主要用于前端 UI 通过 Clash API 读取代理组/切换节点。
        let experimental = config_obj
            .entry("experimental".to_string())
            .or_insert(json!({}));
        if let Some(exp_obj) = experimental.as_object_mut() {
            let clash_api = exp_obj.entry("clash_api".to_string()).or_insert(json!({}));
            if let Some(clash_api_obj) = clash_api.as_object_mut() {
                clash_api_obj.insert(
                    "external_controller".to_string(),
                    json!(format!("127.0.0.1:{}", app_config.api_port)),
                );
                // 允许用户指定 UI/规则集下载走哪个出站（国内网络通常需要走代理）
                clash_api_obj.insert(
                    "external_ui_download_detour".to_string(),
                    json!(normalize_download_detour(app_config)),
                );
            }
        }

        // dns.strategy 用于切换“仅 IPv4 / IPv6 优先”等行为。
        let dns = config_obj.entry("dns".to_string()).or_insert(json!({}));
        if let Some(dns_obj) = dns.as_object_mut() {
            dns_obj.insert("strategy".to_string(), json!(dns_strategy(app_config)));
        }
    }
}

pub fn apply_port_settings_only(config: &mut Value, app_config: &AppConfig) {
    if let Some(config_obj) = config.as_object_mut() {
        if let Some(experimental) = config_obj.get_mut("experimental").and_then(|v| v.as_object_mut())
        {
            if let Some(clash_api) = experimental.get_mut("clash_api").and_then(|v| v.as_object_mut())
            {
                if clash_api.contains_key("external_controller") {
                    clash_api.insert(
                        "external_controller".to_string(),
                        json!(format!("127.0.0.1:{}", app_config.api_port)),
                    );
                }
            }
        }

        if let Some(inbounds) = config_obj.get_mut("inbounds").and_then(|v| v.as_array_mut()) {
            for inbound in inbounds.iter_mut() {
                if let Some(inbound_obj) = inbound.as_object_mut() {
                    let is_mixed =
                        matches!(inbound_obj.get("type").and_then(|v| v.as_str()), Some("mixed"));
                    let is_mixed_in = matches!(
                        inbound_obj.get("tag").and_then(|v| v.as_str()),
                        Some("mixed-in")
                    );
                    if (is_mixed || is_mixed_in) && inbound_obj.contains_key("listen_port") {
                        inbound_obj
                            .insert("listen_port".to_string(), json!(app_config.proxy_port));
                    }
                }
            }
        }
    }
}

fn apply_profile_settings_if_present(config_obj: &mut Map<String, Value>, app_config: &AppConfig) {
    let default_outbound = normalize_default_outbound(app_config);
    let download_detour = normalize_download_detour(app_config);

    // 1) 更新 urltest 的 URL
    if let Some(outbounds) = config_obj.get_mut("outbounds").and_then(|v| v.as_array_mut()) {
        for outbound in outbounds.iter_mut() {
            if outbound.get("tag").and_then(|t| t.as_str()) == Some(TAG_AUTO) {
                if let Some(obj) = outbound.as_object_mut() {
                    // 强制启用切换时中断旧连接，避免长时间后台运行连接数膨胀
                    obj.insert("interrupt_exist_connections".to_string(), json!(true));
                    // 缩短空闲回收时间，防止长尾连接占满列表
                    obj.insert("idle_timeout".to_string(), json!("10m"));
                    obj.insert("url".to_string(), json!(app_config.singbox_urltest_url));
                }
            }
        }
    }

    // 2) 更新 DNS servers（按 tag 定位）
    if let Some(dns_obj) = config_obj.get_mut("dns").and_then(|v| v.as_object_mut()) {
        if let Some(servers) = dns_obj.get_mut("servers").and_then(|v| v.as_array_mut()) {
            for server in servers.iter_mut() {
                let tag = server
                    .get("tag")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                if let Some(obj) = server.as_object_mut() {
                    match tag {
                        t if t == DNS_PROXY => {
                            obj.insert("address".to_string(), json!(app_config.singbox_dns_proxy));
                            // DNS_PROXY 默认走代理（防污染/更接近 Clash 的体验）。
                            // 循环依赖问题由 config_generator 在“注入节点”时给节点补 `domain_resolver=dns_resolver` 来解决：
                            // - 节点域名用 dns_resolver（直连）解析
                            // - DNS_PROXY 的 DoH/DoH3 请求可以走代理出站
                            obj.insert("detour".to_string(), json!(default_outbound));
                        }
                        t if t == DNS_CN => {
                            obj.insert("address".to_string(), json!(app_config.singbox_dns_cn));
                        }
                        t if t == DNS_RESOLVER => {
                            obj.insert(
                                "address".to_string(),
                                json!(app_config.singbox_dns_resolver),
                            );
                        }
                        _ => {}
                    }
                }
            }
        }

        // 3) 广告拦截：同步 dns.rules（如果存在/可定位）
        if let Some(rules) = dns_obj.get_mut("rules").and_then(|v| v.as_array_mut()) {
            let mut ads_rule_index: Option<usize> = None;
            for (idx, rule) in rules.iter().enumerate() {
                if rule.get("rule_set").and_then(|v| v.as_str()) == Some(RS_GEOSITE_ADS) {
                    ads_rule_index = Some(idx);
                    break;
                }
            }

            if app_config.singbox_block_ads {
                if ads_rule_index.is_none() {
                    // 尽量插入在前面：优先拦截广告域名的解析
                    rules.insert(0, json!({ "rule_set": RS_GEOSITE_ADS, "server": DNS_BLOCK }));
                } else if let Some(i) = ads_rule_index {
                    if let Some(obj) = rules.get_mut(i).and_then(|v| v.as_object_mut()) {
                        obj.insert("server".to_string(), json!(DNS_BLOCK));
                    }
                }
            } else if let Some(i) = ads_rule_index {
                rules.remove(i);
            }
        }
    }

    // 4) route：同步 final / hijack-dns / ads reject / rule_set download_detour
    if let Some(route_obj) = config_obj.get_mut("route").and_then(|v| v.as_object_mut()) {
        route_obj.insert("final".to_string(), json!(default_outbound));
        route_obj.insert("default_domain_resolver".to_string(), json!(DNS_RESOLVER));

        if let Some(rule_sets) = route_obj.get_mut("rule_set").and_then(|v| v.as_array_mut()) {
            // 仅对 remote 规则集更新 download_detour，避免影响本地文件规则集
            for rs in rule_sets.iter_mut() {
                if let Some(obj) = rs.as_object_mut() {
                    if obj.get("type").and_then(|v| v.as_str()) == Some("remote") {
                        obj.insert("download_detour".to_string(), json!(download_detour));
                    }
                }
            }

            // 按开关移除不再需要的规则集，避免后台持续下载无用文件
            if !app_config.singbox_block_ads {
                rule_sets.retain(|rs| {
                    rs.get("tag").and_then(|v| v.as_str()) != Some(RS_GEOSITE_ADS)
                });
            }
            if !app_config.singbox_enable_app_groups {
                rule_sets.retain(|rs| {
                    let tag = rs.get("tag").and_then(|v| v.as_str()).unwrap_or("");
                    !matches!(
                        tag,
                        RS_GEOSITE_TELEGRAM | RS_GEOSITE_YOUTUBE | RS_GEOSITE_NETFLIX | RS_GEOSITE_OPENAI
                    )
                });
            }
        }

        if let Some(rules) = route_obj.get_mut("rules").and_then(|v| v.as_array_mut()) {
            // 让规则里的 global/非CN 默认走用户选择的出站（manual/auto）
            for rule in rules.iter_mut() {
                if let Some(obj) = rule.as_object_mut() {
                    if obj.get("clash_mode").and_then(|v| v.as_str()) == Some("global") {
                        obj.insert("outbound".to_string(), json!(default_outbound));
                    }
                    if obj.get("rule_set").and_then(|v| v.as_str()) == Some(RS_GEOSITE_GEOLOCATION_NOT_CN) {
                        obj.insert("outbound".to_string(), json!(default_outbound));
                    }
                }
            }

            // hijack-dns
            let mut hijack_index: Option<usize> = None;
            for (idx, rule) in rules.iter().enumerate() {
                if rule.get("protocol").and_then(|v| v.as_str()) == Some("dns")
                    && rule.get("action").and_then(|v| v.as_str()) == Some("hijack-dns")
                {
                    hijack_index = Some(idx);
                    break;
                }
            }
            if app_config.singbox_dns_hijack {
                if hijack_index.is_none() {
                    // 放在 sniff 后面通常更合适
                    rules.insert(1, json!({ "protocol": "dns", "action": "hijack-dns" }));
                }
            } else if let Some(i) = hijack_index {
                rules.remove(i);
            }

            // 广告拦截 route.rules（按 rule_set + action 定位）
            let mut ads_index: Option<usize> = None;
            for (idx, rule) in rules.iter().enumerate() {
                if rule.get("rule_set").and_then(|v| v.as_str()) == Some(RS_GEOSITE_ADS)
                    && rule.get("action").and_then(|v| v.as_str()).is_some()
                {
                    ads_index = Some(idx);
                    break;
                }
            }
            if app_config.singbox_block_ads {
                if ads_index.is_none() {
                    rules.push(json!({ "rule_set": RS_GEOSITE_ADS, "action": "reject" }));
                }
            } else if let Some(i) = ads_index {
                rules.remove(i);
            }

            // 业务分流组：关闭后移除相关规则，避免“空组/无意义分流”
            if !app_config.singbox_enable_app_groups {
                rules.retain(|rule| {
                    let rs = rule.get("rule_set").and_then(|v| v.as_str()).unwrap_or("");
                    !matches!(
                        rs,
                        RS_GEOSITE_TELEGRAM | RS_GEOSITE_YOUTUBE | RS_GEOSITE_NETFLIX | RS_GEOSITE_OPENAI
                    )
                });
            }
        }
    }

    // 5) outbounds：按开关移除业务分流组（如果存在）
    if let Some(outbounds) = config_obj.get_mut("outbounds").and_then(|v| v.as_array_mut()) {
        if !app_config.singbox_enable_app_groups {
            outbounds.retain(|ob| {
                let tag = ob.get("tag").and_then(|v| v.as_str()).unwrap_or("");
                !matches!(tag, TAG_TELEGRAM | TAG_YOUTUBE | TAG_NETFLIX | TAG_OPENAI)
            });
        }
    }
}

fn apply_inbounds_settings(config_obj: &mut Map<String, Value>, app_config: &AppConfig) {
    let mut tun_addresses = vec![app_config.tun_ipv4.clone()];
    if app_config.tun_enable_ipv6 {
        tun_addresses.push(app_config.tun_ipv6.clone());
    }

    let mut inbounds = Vec::new();

    // mixed 是桌面端最通用的入口（HTTP + SOCKS），便于系统代理/浏览器直接使用。
    inbounds.push(json!({
        "type": "mixed",
        "tag": "mixed-in",
        "listen": "127.0.0.1",
        "listen_port": app_config.proxy_port,
        "sniff": true,
        "set_system_proxy": app_config.system_proxy_enabled
    }));

    // TUN 模式依赖 sing-box 配置里显式存在 tun inbound，所以这里根据设置开关动态添加/移除。
    if app_config.tun_enabled {
        inbounds.push(json!({
            "type": "tun",
            "tag": "tun-in",
            "address": tun_addresses,
            "auto_route": app_config.tun_auto_route,
            "strict_route": app_config.tun_strict_route,
            "stack": app_config.tun_stack,
            "mtu": app_config.tun_mtu,
            "sniff": true,
            "sniff_override_destination": true,
            "route_exclude_address": TUN_ROUTE_EXCLUDES
        }));
    }

    config_obj.insert("inbounds".to_string(), json!(inbounds));
}
