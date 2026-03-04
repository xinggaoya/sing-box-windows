use super::common::{
    build_dns_server_config, dns_strategy, node_domain_resolver_strategy,
    normalize_default_outbound, normalize_download_detour, normalize_fake_dns_filter_mode, DNS_CN,
    DNS_FAKEIP, DNS_PROXY, DNS_RESOLVER, FAKE_DNS_FILTER_GLOBAL_NON_CN, PRIVATE_IP_CIDRS,
    RS_GEOIP_CN, RS_GEOSITE_ADS, RS_GEOSITE_CN, RS_GEOSITE_GEOLOCATION_NOT_CN, RS_GEOSITE_GOOGLE,
    RS_GEOSITE_NETFLIX, RS_GEOSITE_OPENAI, RS_GEOSITE_PRIVATE, RS_GEOSITE_TELEGRAM,
    RS_GEOSITE_YOUTUBE,
};
use super::config_schema::{
    CacheFileConfig, ClashApiConfig, DnsConfig, DnsServerConfig, ExperimentalConfig, LogConfig,
    RemoteRuleSetConfig, RouteConfig, SingBoxConfig,
};
use crate::app::singbox::settings_patch::apply_app_settings_to_config;
use crate::app::storage::state_model::AppConfig;
use serde_json::{json, Value};
// 兼容旧引用：这些 tag 之前是 `config_generator` 的 `pub const`，保留同名导出以降低未来重构的破坏性。
pub use super::common::{
    TAG_AUTO, TAG_BLOCK, TAG_DIRECT, TAG_GOOGLE, TAG_MANUAL, TAG_NETFLIX, TAG_OPENAI, TAG_TELEGRAM,
    TAG_YOUTUBE,
};

/// 生成一份“通用且更适合国内环境”的 sing-box 配置骨架（不依赖模板文件）。
///
/// 目标：
/// - 默认规则：国内域名/IP 直连，其他走代理（可“绕过国内域名”）。
/// - DNS：国内用国内 DNS，非国内用 DoH（尽量避免污染）。
/// - 兼容：保留 Clash API（前端节点选择/延迟测试依赖）。
pub fn generate_base_config(app_config: &AppConfig) -> Value {
    let dns_strategy = dns_strategy(app_config);

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
            // 手动切换分组只暴露“自动选择 + 订阅节点”，不暴露 direct，避免 UI 优选/自动选择误选到直连。
            "outbounds": [TAG_AUTO]
        }),
    ];

    // 应用分流组：默认开启（对大多数用户比较实用），可在设置页关闭。
    if app_config.singbox_enable_app_groups {
        outbounds.extend([
            json!({
                "type": "selector",
                "tag": TAG_TELEGRAM,
                "outbounds": [TAG_MANUAL, TAG_AUTO]
            }),
            json!({
                "type": "selector",
                "tag": TAG_YOUTUBE,
                "outbounds": [TAG_MANUAL, TAG_AUTO]
            }),
            json!({
                "type": "selector",
                "tag": TAG_NETFLIX,
                "outbounds": [TAG_MANUAL, TAG_AUTO]
            }),
            json!({
                "type": "selector",
                "tag": TAG_OPENAI,
                "outbounds": [TAG_MANUAL, TAG_AUTO]
            }),
            json!({
                "type": "selector",
                "tag": TAG_GOOGLE,
                "outbounds": [TAG_MANUAL, TAG_AUTO]
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
        dns_rules.insert(2, json!({ "rule_set": RS_GEOSITE_ADS, "action": "reject" }));
    }

    apply_fake_dns_rules(&mut dns_rules, app_config);

    let mut rule_sets: Vec<Value> = Vec::new();
    if app_config.singbox_block_ads {
        rule_sets.push(remote_rule_set_value(
            RS_GEOSITE_ADS,
            "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-category-ads-all.srs",
            download_detour,
            "1d",
        ));
    }

    rule_sets.extend([
        remote_rule_set_value(
            RS_GEOSITE_CN,
            "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-cn.srs",
            download_detour,
            "1d",
        ),
        remote_rule_set_value(
            RS_GEOSITE_GEOLOCATION_NOT_CN,
            "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-geolocation-!cn.srs",
            download_detour,
            "1d",
        ),
    ]);

    if app_config.singbox_enable_app_groups {
        rule_sets.extend([
            remote_rule_set_value(
                RS_GEOSITE_TELEGRAM,
                "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-telegram.srs",
                download_detour,
                "7d",
            ),
            remote_rule_set_value(
                RS_GEOSITE_YOUTUBE,
                "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-youtube.srs",
                download_detour,
                "7d",
            ),
            remote_rule_set_value(
                RS_GEOSITE_NETFLIX,
                "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-netflix.srs",
                download_detour,
                "7d",
            ),
            remote_rule_set_value(
                RS_GEOSITE_OPENAI,
                "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-openai.srs",
                download_detour,
                "7d",
            ),
            remote_rule_set_value(
                RS_GEOSITE_GOOGLE,
                "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-google.srs",
                download_detour,
                "7d",
            ),
        ]);
    }

    rule_sets.extend([
        remote_rule_set_value(
            RS_GEOSITE_PRIVATE,
            "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geosite/rule-set/geosite-private.srs",
            TAG_DIRECT,
            "7d",
        ),
        remote_rule_set_value(
            RS_GEOIP_CN,
            "https://gh-proxy.com/https://raw.githubusercontent.com/SagerNet/sing-geoip/rule-set/geoip-cn.srs",
            download_detour,
            "1d",
        ),
    ]);

    let mut route_rules: Vec<Value> = vec![json!({ "action": "sniff" })];

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
            // Google 服务分流（包含 Gemini、DeepMind、搜索、云服务等所有 Google 相关域名）
            json!({ "rule_set": RS_GEOSITE_GOOGLE, "outbound": TAG_GOOGLE }),
        ]);
    }

    // 直接内置私网段，避免依赖不存在的 geoip-private 规则集导致启动 404 退出
    route_rules.extend([
        json!({ "rule_set": RS_GEOSITE_PRIVATE, "outbound": TAG_DIRECT }),
        json!({ "ip_cidr": PRIVATE_IP_CIDRS, "outbound": TAG_DIRECT }),
        json!({ "rule_set": [RS_GEOSITE_CN, RS_GEOIP_CN], "outbound": TAG_DIRECT }),
        json!({ "rule_set": RS_GEOSITE_GEOLOCATION_NOT_CN, "outbound": default_outbound }),
    ]);

    if app_config.singbox_fake_dns_enabled {
        // fakeip 生成的地址段需要显式直连，确保连接能够回到内核并完成域名逆向映射。
        route_rules.push(json!({
            "ip_cidr": [
                normalize_fakeip_range(&app_config.singbox_fake_dns_ipv4_range, "198.18.0.0/15"),
                normalize_fakeip_range(&app_config.singbox_fake_dns_ipv6_range, "fc00::/18")
            ],
            "outbound": TAG_DIRECT
        }));
    }

    // 注意：这里的 outbounds 只是骨架，订阅节点注入后会补齐 TAG_AUTO/TAG_MANUAL
    // 以及各业务分流组的候选列表。
    //
    // 这里用结构体序列化生成 JSON，减少“字符串 key + json! 拼装”的维护成本：
    // - 字段改名/移动时更容易被编译器发现
    // - 减少复制粘贴造成的漏字段/错字段
    let base = SingBoxConfig {
        log: LogConfig {
            disabled: false,
            level: "info".to_string(),
            timestamp: true,
        },
        experimental: ExperimentalConfig {
            cache_file: CacheFileConfig {
                enabled: true,
                // Fake DNS 依赖反向域名映射缓存，开启持久化可降低切换网络后的映射丢失。
                store_rdrc: app_config.singbox_fake_dns_enabled.then_some(true),
            },
            clash_api: ClashApiConfig {
                external_controller: format!("127.0.0.1:{}", app_config.api_port),
                external_ui: "metacubexd".to_string(),
                // 让 sing-box 自动下载 UI（国内网络可能被墙，下载走代理可提高成功率）
                external_ui_download_url:
                    "https://github.com/MetaCubeX/metacubexd/archive/refs/heads/gh-pages.zip"
                        .to_string(),
                external_ui_download_detour: download_detour.to_string(),
                default_mode: "rule".to_string(),
            },
        },
        dns: DnsConfig {
            servers: build_dns_servers(app_config, dns_strategy, default_outbound),
            rules: dns_rules,
            independent_cache: true,
            reverse_mapping: app_config.singbox_fake_dns_enabled.then_some(true),
            final_server: DNS_PROXY.to_string(),
        },
        inbounds: Vec::new(),
        outbounds,
        route: RouteConfig {
            rule_set: rule_sets,
            rules: route_rules,
            final_outbound: default_outbound.to_string(),
            auto_detect_interface: true,
            default_domain_resolver: Some(json!({
                "server": DNS_RESOLVER,
                "strategy": dns_strategy
            })),
        },
    };

    let mut config = serde_json::to_value(base).expect("SingBoxConfig 序列化失败");

    // 统一由 settings_patch 负责把端口/TUN/IPv6 偏好写入配置，确保行为一致。
    apply_app_settings_to_config(&mut config, app_config);
    config
}

fn build_dns_servers(
    app_config: &AppConfig,
    dns_strategy: &str,
    default_outbound: &str,
) -> Vec<DnsServerConfig> {
    let mut servers = vec![
        build_dns_server_with_fallback(
            DNS_PROXY,
            &app_config.singbox_dns_proxy,
            Some(dns_strategy),
            Some(default_outbound),
            Some(DNS_RESOLVER),
            "https://1.1.1.1/dns-query",
        ),
        build_dns_server_with_fallback(
            DNS_CN,
            &app_config.singbox_dns_cn,
            Some(dns_strategy),
            Some(TAG_DIRECT),
            Some(DNS_RESOLVER),
            "h3://dns.alidns.com/dns-query",
        ),
        build_dns_server_with_fallback(
            DNS_RESOLVER,
            &app_config.singbox_dns_resolver,
            Some(dns_strategy),
            Some(TAG_DIRECT),
            None,
            "114.114.114.114",
        ),
    ];

    if app_config.singbox_fake_dns_enabled {
        servers.push(build_fakeip_dns_server(app_config));
    }

    servers
}

fn build_dns_server_with_fallback(
    tag: &str,
    raw_address: &str,
    strategy: Option<&str>,
    detour: Option<&str>,
    resolver_tag: Option<&str>,
    fallback_address: &str,
) -> DnsServerConfig {
    build_dns_server_config(tag, raw_address, strategy, detour, resolver_tag).unwrap_or_else(|_| {
        build_dns_server_config(tag, fallback_address, strategy, detour, resolver_tag)
            .expect("内置 DNS fallback 地址必须可解析")
    })
}

fn build_fakeip_dns_server(app_config: &AppConfig) -> DnsServerConfig {
    DnsServerConfig {
        tag: DNS_FAKEIP.to_string(),
        server_type: Some("fakeip".to_string()),
        server: None,
        server_port: None,
        path: None,
        interface: None,
        inet4_range: Some(normalize_fakeip_range(
            &app_config.singbox_fake_dns_ipv4_range,
            "198.18.0.0/15",
        )),
        inet6_range: Some(normalize_fakeip_range(
            &app_config.singbox_fake_dns_ipv6_range,
            "fc00::/18",
        )),
        domain_resolver: None,
        detour: None,
    }
}

fn normalize_fakeip_range(raw: &str, fallback: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}

fn apply_fake_dns_rules(dns_rules: &mut Vec<Value>, app_config: &AppConfig) {
    // 先清理所有历史 fakeip 规则，再按当前开关重建，避免重复叠加。
    dns_rules.retain(|rule| rule.get("server").and_then(|v| v.as_str()) != Some(DNS_FAKEIP));

    if !app_config.singbox_fake_dns_enabled {
        return;
    }

    match normalize_fake_dns_filter_mode(app_config) {
        FAKE_DNS_FILTER_GLOBAL_NON_CN => {
            // 全局非 CN：保留前面的 CN 规则，其他 A/AAAA 查询统一落到 fakeip。
            dns_rules.push(json!({
                "query_type": ["A", "AAAA"],
                "server": DNS_FAKEIP
            }));
        }
        _ => {
            // 仅代理流量：非 CN 域名走 fakeip，国内域名仍按原逻辑直连解析。
            let rule = json!({
                "query_type": ["A", "AAAA"],
                "rule_set": RS_GEOSITE_GEOLOCATION_NOT_CN,
                "server": DNS_FAKEIP
            });

            let insert_idx = dns_rules
                .iter()
                .position(|item| {
                    item.get("rule_set").and_then(|v| v.as_str())
                        == Some(RS_GEOSITE_GEOLOCATION_NOT_CN)
                })
                .unwrap_or(dns_rules.len());
            dns_rules.insert(insert_idx, rule);
        }
    }
}

fn remote_rule_set_value(
    tag: &str,
    url: &str,
    download_detour: &str,
    update_interval: &str,
) -> Value {
    let rs = RemoteRuleSetConfig {
        tag: tag.to_string(),
        kind: "remote".to_string(),
        format: "binary".to_string(),
        url: url.to_string(),
        download_detour: download_detour.to_string(),
        update_interval: update_interval.to_string(),
    };
    serde_json::to_value(rs).expect("RemoteRuleSetConfig 序列化失败")
}

/// 基于骨架配置注入节点，并更新“自动选择/手动切换”等组的候选列表。
pub fn generate_config_with_nodes(
    app_config: &AppConfig,
    nodes: &[Value],
) -> Result<Value, String> {
    let mut config = generate_base_config(app_config);
    inject_nodes(&mut config, app_config, nodes)?;
    Ok(config)
}

pub fn inject_nodes(
    config: &mut Value,
    app_config: &AppConfig,
    nodes: &[Value],
) -> Result<(), String> {
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

    let resolver_strategy = node_domain_resolver_strategy(app_config);

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
        if node_obj
            .get("type")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .trim()
            .is_empty()
        {
            return Err(format!("节点缺少 type: tag={}, index={}", raw_tag, idx));
        }

        // 若 tag 冲突，则自动改名，避免覆盖内置分组/出站。
        let mut tag = raw_tag.clone();
        if existing_tags.contains(&tag) {
            // 先用 index 尝试一次，避免同名节点时生成可读且相对稳定的 tag。
            let candidate = format!("节点-{}-{}", raw_tag, idx);
            if !existing_tags.contains(&candidate) {
                tag = candidate;
            } else {
                // 极端情况下仍可能冲突（例如订阅自带同名 + 已存在相同格式 tag），这里兜底确保唯一性。
                let mut counter = 1usize;
                loop {
                    let candidate = format!("节点-{}-{}", raw_tag, counter);
                    if !existing_tags.contains(&candidate) {
                        tag = candidate;
                        break;
                    }
                    counter = counter.saturating_add(1);
                }
            }
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
    // 2) 更新 TAG_MANUAL(selector) 包含自动选择 + 每个节点（不包含 direct，避免 UI 误选直连）。
    // 3) 业务分流组补齐节点列表，避免只剩“自动/手动”无法直选节点。
    ensure_urltest_and_selector(outbounds, &group_node_tags)?;
    ensure_app_group_selectors(outbounds, &group_node_tags)?;

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

fn ensure_urltest_and_selector(
    outbounds: &mut Vec<Value>,
    node_tags: &[String],
) -> Result<(), String> {
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

    // 手动切换候选列表：自动选择 + 每个节点
    let mut manual_list = Vec::<Value>::with_capacity(1 + node_tags.len());
    manual_list.push(Value::String(TAG_AUTO.to_string()));
    for tag in node_tags {
        manual_list.push(Value::String(tag.clone()));
    }
    {
        let manual = outbounds
            .get_mut(manual_idx)
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| format!("outbound(tag={}) 不是对象", TAG_MANUAL))?;
        manual.insert("outbounds".to_string(), Value::Array(manual_list));
    }

    Ok(())
}

fn ensure_app_group_selectors(
    outbounds: &mut Vec<Value>,
    node_tags: &[String],
) -> Result<(), String> {
    let group_tags = [
        TAG_TELEGRAM,
        TAG_YOUTUBE,
        TAG_NETFLIX,
        TAG_OPENAI,
        TAG_GOOGLE,
    ];

    for group_tag in group_tags {
        let Some(idx) = outbounds
            .iter()
            .position(|o| o.get("tag").and_then(|t| t.as_str()) == Some(group_tag))
        else {
            continue;
        };

        let mut group_list = Vec::<Value>::with_capacity(2 + node_tags.len());
        group_list.push(Value::String(TAG_MANUAL.to_string()));
        group_list.push(Value::String(TAG_AUTO.to_string()));
        for tag in node_tags {
            group_list.push(Value::String(tag.clone()));
        }

        let group = outbounds
            .get_mut(idx)
            .and_then(|v| v.as_object_mut())
            .ok_or_else(|| format!("outbound(tag={}) 不是对象", group_tag))?;
        group.insert("outbounds".to_string(), Value::Array(group_list));
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::storage::state_model::AppConfig;

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
    fn ads_dns_rule_should_use_reject_action() {
        let mut app_config = AppConfig::default();
        app_config.singbox_block_ads = true;

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
        let mut app_config = AppConfig::default();
        app_config.singbox_fake_dns_enabled = true;

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
        let mut app_config = AppConfig::default();
        app_config.singbox_fake_dns_enabled = true;
        app_config.singbox_fake_dns_filter_mode = "global_non_cn".to_string();

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
}
