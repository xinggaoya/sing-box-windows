//! 配置补丁路径：把 `AppConfig` 的设置同步到**已有**配置上。
//!
//! ## 两条路径，一条语义
//!
//! 配置有两个入口：
//!
//! - **生成路径**（`config_generator::generate_base_config`）：从零拼一份配置
//! - **补丁路径**（本模块）：在已有配置上按 tag 打补丁
//!
//! 两者读同一批设置、产出同一份配置语义。语义由 [`ConfigIntent`] 唯一承载，
//! 渲染细节由 [`config_render`] 共享——本模块只负责"**在已有结构上改哪些字段**"。
//!
//! ## 归属（Ownership）
//!
//! 补丁的前提是"这份配置是本程序的"。此前靠结构猜测判断，
//! 猜错就会把用户导入的原始配置改坏（入口被重建、路由终点被改写）。
//! 现在改为显式的 [`Ownership`]：
//!
//! - 归属为本程序 → 按设置完整同步
//! - 归属为外来配置 → 调用方退化为 [`apply_port_settings_only`]，只改端口与监听地址

use super::common::{
    build_dns_server_config, ensure_kernel_log_output, RS_GEOSITE_ADS,
    RS_GEOSITE_GEOLOCATION_NOT_CN, RS_GEOSITE_GOOGLE, RS_GEOSITE_NETFLIX, RS_GEOSITE_OPENAI,
    RS_GEOSITE_TELEGRAM, RS_GEOSITE_YOUTUBE, TAG_AUTO, TAG_DIRECT, TAG_GOOGLE, TAG_NETFLIX,
    TAG_OPENAI, TAG_TELEGRAM, TAG_YOUTUBE,
};
use super::config_generator::remote_rule_set_sources;
use super::config_intent::ConfigIntent;
use super::config_render::{
    build_http_clients, build_inbounds, ensure_sniff_route_rule,
    extract_existing_tun_route_exclude_address, extract_fake_dns_server_range_pair,
    sync_fake_dns_route_rules, sync_fake_dns_rules, sync_fake_dns_server, HTTP_CLIENT_DEFAULT_TAG,
    HTTP_CLIENT_DIRECT_DOWNLOAD_TAG,
};
use super::ownership::Ownership;
use crate::app::storage::state_model::AppConfig;
use serde_json::{json, Map, Value};

/// 将应用设置（端口 / System Proxy / TUN / IPv6 偏好等）同步到 sing-box 配置。
///
/// 兼容入口：内部先把 `AppConfig` 翻译成 [`ConfigIntent`]（配置语义的唯一来源），
/// 再按配置内容推断归属后落盘。
///
/// 需要精确控制归属时（例如归属已落库、或明确知道这份配置是外来配置），
/// 请直接调用 [`apply_intent_to_config`]。
pub fn apply_app_settings_to_config(config: &mut Value, app_config: &AppConfig) {
    let intent = ConfigIntent::from_app_config(app_config);
    let ownership = Ownership::inferred(config);
    apply_intent_to_config(config, &intent, &ownership);
}

/// 按配置意图 + 归属同步到已有配置。
///
/// - `ownership.managed == false`（外来配置）：**不动**入口 / DNS / 路由 / 出站，
///   只做内核版本兼容清理与 gRPC API 服务兜底。
/// - `ownership.managed == true`：按设置完整同步。
pub fn apply_intent_to_config(config: &mut Value, intent: &ConfigIntent, ownership: &Ownership) {
    if let Some(config_obj) = config.as_object_mut() {
        ensure_kernel_log_output(config_obj);

        // 入口 / DNS / 路由 / 出站的重建只对本程序拥有的配置做，
        // 否则会把用户导入的原始配置改坏。
        if ownership.managed {
            apply_inbounds_settings(config_obj, intent);
            apply_owned_profile_settings(config_obj, intent, ownership);
        }

        // 1.14 替代 `experimental.clash_api`：完全清理历史残留，
        // 实际 gRPC API 由 `services: [api]` 提供。
        // 外来配置同样要清理——1.14 内核已不接受该字段，留着会直接启动失败。
        if let Some(experimental) = config_obj
            .get_mut("experimental")
            .and_then(|v| v.as_object_mut())
        {
            experimental.remove("clash_api");
        }

        // 兜底补齐 1.14 gRPC api 服务：api 服务此前只在配置生成时注入,升级前的
        // 旧 config.json（订阅刷新失败/离线时不会被再生成）被删掉 clash_api 后
        // 就没有任何 API 通道,内核能启动但状态/代理组/模式切换全部不可用。
        // 外来配置也必须补——否则 UI 完全控制不了内核。
        ensure_api_service(config_obj, intent);

        // cache_file：1.14 替代 `store_rdrc` → `store_dns`（1.16 移除 store_rdrc）。
        if ownership.managed {
            let experimental = config_obj
                .entry("experimental".to_string())
                .or_insert(json!({}));
            if let Some(exp_obj) = experimental.as_object_mut() {
                let cache_file = exp_obj.entry("cache_file".to_string()).or_insert(json!({}));
                if let Some(cache_obj) = cache_file.as_object_mut() {
                    cache_obj.insert("enabled".to_string(), json!(true));
                    // 兼容老字段（用户可能从 1.13 升级上来）→ 1.14 同时接受，1.16 后忽略
                    cache_obj.remove("store_rdrc");
                    if intent.fake_dns_enabled() {
                        cache_obj.insert("store_dns".to_string(), json!(true));
                    } else {
                        cache_obj.remove("store_dns");
                    }
                }
            }
        }

        // `dns.strategy` 已在 sing-box 1.12 废弃；迁移到 domain_resolver 对象策略。
        // 1.14 移除 `independent_cache`（强制按 transport 隔离缓存）。
        // 这两项是内核版本兼容清理，对外来配置同样执行——留着会直接启动失败。
        if let Some(dns_obj) = config_obj.get_mut("dns").and_then(|v| v.as_object_mut()) {
            dns_obj.remove("strategy");
            dns_obj.remove("independent_cache");

            if ownership.managed {
                if intent.fake_dns_enabled() {
                    dns_obj.insert("reverse_mapping".to_string(), json!(true));
                } else {
                    dns_obj.remove("reverse_mapping");
                }
                // 1.14 新增：乐观 DNS 缓存
                if intent.dns_optimistic {
                    dns_obj.insert("optimistic".to_string(), json!(true));
                } else {
                    dns_obj.remove("optimistic");
                }
                // 1.14 新增：per-server DNS 超时
                if !intent.dns_timeout.is_empty() {
                    dns_obj.insert("timeout".to_string(), json!(intent.dns_timeout.clone()));
                } else {
                    dns_obj.remove("timeout");
                }
            }
        }
    }
}

/// 最小侵入补丁：只同步端口与监听地址。
///
/// 用于"使用订阅原始配置"的场景——用户导入的配置结构由机场决定，
/// 本程序只该改自己负责的入口参数，不应动其余部分。
pub fn apply_port_settings_only(config: &mut Value, app_config: &AppConfig) {
    let intent = ConfigIntent::from_app_config(app_config);
    apply_port_settings_only_with_intent(config, &intent);
}

/// 最小侵入补丁（intent 版本）
pub fn apply_port_settings_only_with_intent(config: &mut Value, intent: &ConfigIntent) {
    if let Some(config_obj) = config.as_object_mut() {
        // 1.14 替代 clash_api：清空 external_controller 残留（端口同步在 services: [api] 处理）
        if let Some(experimental) = config_obj
            .get_mut("experimental")
            .and_then(|v| v.as_object_mut())
        {
            if let Some(clash_api) = experimental
                .get_mut("clash_api")
                .and_then(|v| v.as_object_mut())
            {
                if clash_api.contains_key("external_controller") {
                    clash_api.insert(
                        "external_controller".to_string(),
                        json!(format!("127.0.0.1:{}", intent.api_port)),
                    );
                }
            }
        }

        if let Some(inbounds) = config_obj
            .get_mut("inbounds")
            .and_then(|v| v.as_array_mut())
        {
            for inbound in inbounds.iter_mut() {
                if let Some(inbound_obj) = inbound.as_object_mut() {
                    let is_mixed = matches!(
                        inbound_obj.get("type").and_then(|v| v.as_str()),
                        Some("mixed")
                    );
                    let is_mixed_in = matches!(
                        inbound_obj.get("tag").and_then(|v| v.as_str()),
                        Some("mixed-in")
                    );
                    if is_mixed || is_mixed_in {
                        if inbound_obj.contains_key("listen_port") {
                            inbound_obj.insert("listen_port".to_string(), json!(intent.proxy_port));
                        }
                        inbound_obj.insert("listen".to_string(), json!(intent.proxy_listen));
                    }
                }
            }
        }
    }
}

/// 同步"本程序生成的配置"里的高级选项。
///
/// 每一项都按归属过滤：只改本程序认领的 tag，用户自己加的同名结构不受影响。
fn apply_owned_profile_settings(
    config_obj: &mut Map<String, Value>,
    intent: &ConfigIntent,
    ownership: &Ownership,
) {
    let default_outbound = intent.default_outbound;
    let mut fake_dns_route_cleanup_pairs = vec![
        (
            intent.fake_dns.ipv4_range.clone(),
            intent.fake_dns.ipv6_range.clone(),
        ),
        ("198.18.0.0/15".to_string(), "fc00::/18".to_string()),
    ];

    // 1) 更新 urltest 的 URL
    if ownership.owns_group(TAG_AUTO) {
        if let Some(outbounds) = config_obj
            .get_mut("outbounds")
            .and_then(|v| v.as_array_mut())
        {
            for outbound in outbounds.iter_mut() {
                if outbound.get("tag").and_then(|t| t.as_str()) == Some(TAG_AUTO) {
                    if let Some(obj) = outbound.as_object_mut() {
                        // 强制启用切换时中断旧连接，避免长时间后台运行连接数膨胀
                        obj.insert("interrupt_exist_connections".to_string(), json!(true));
                        // 缩短空闲回收时间，防止长尾连接占满列表
                        obj.insert("idle_timeout".to_string(), json!("10m"));
                        obj.insert("url".to_string(), json!(intent.urltest_url));
                    }
                }
            }
        }
    }

    // 2) 更新 DNS servers（按 tag 定位）
    if let Some(dns_obj) = config_obj.get_mut("dns").and_then(|v| v.as_object_mut()) {
        if let Some(servers) = dns_obj.get_mut("servers").and_then(|v| v.as_array_mut()) {
            for server in servers.iter() {
                if let Some(range_pair) = extract_fake_dns_server_range_pair(server) {
                    fake_dns_route_cleanup_pairs.push(range_pair);
                }
            }

            for server in servers.iter_mut() {
                let tag = server
                    .get("tag")
                    .and_then(|t| t.as_str())
                    .unwrap_or("")
                    .to_string();
                if let Some(obj) = server.as_object_mut() {
                    match tag {
                        t if t == super::common::DNS_PROXY && ownership.owns_dns_server(&t) => {
                            if let Ok(cfg) = build_dns_server_config(
                                super::common::DNS_PROXY,
                                &intent.dns_proxy,
                                Some(intent.dns_bootstrap_strategy),
                                Some(default_outbound),
                                Some(super::common::DNS_RESOLVER),
                            ) {
                                if let Ok(new_server) = serde_json::to_value(cfg) {
                                    if let Some(new_obj) = new_server.as_object() {
                                        *obj = new_obj.clone();
                                    }
                                }
                            }
                        }
                        t if t == super::common::DNS_CN && ownership.owns_dns_server(&t) => {
                            if let Ok(cfg) = build_dns_server_config(
                                super::common::DNS_CN,
                                &intent.dns_cn,
                                Some(intent.dns_bootstrap_strategy),
                                Some(TAG_DIRECT),
                                Some(super::common::DNS_RESOLVER),
                            ) {
                                if let Ok(new_server) = serde_json::to_value(cfg) {
                                    if let Some(new_obj) = new_server.as_object() {
                                        *obj = new_obj.clone();
                                    }
                                }
                            }
                        }
                        t if t == super::common::DNS_RESOLVER && ownership.owns_dns_server(&t) => {
                            if let Ok(cfg) = build_dns_server_config(
                                super::common::DNS_RESOLVER,
                                &intent.dns_resolver,
                                Some(intent.dns_bootstrap_strategy),
                                Some(TAG_DIRECT),
                                None,
                            ) {
                                if let Ok(new_server) = serde_json::to_value(cfg) {
                                    if let Some(new_obj) = new_server.as_object() {
                                        *obj = new_obj.clone();
                                    }
                                }
                            }
                        }
                        _ => {}
                    }
                }
            }

            sync_fake_dns_server(servers, intent);
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

            if intent.block_ads {
                if ads_rule_index.is_none() {
                    // 尽量插入在前面：优先拦截广告域名的解析
                    rules.insert(0, json!({ "rule_set": RS_GEOSITE_ADS, "action": "reject" }));
                } else if let Some(i) = ads_rule_index {
                    if let Some(obj) = rules.get_mut(i).and_then(|v| v.as_object_mut()) {
                        obj.insert("action".to_string(), json!("reject"));
                        obj.remove("server");
                    }
                }
            } else if let Some(i) = ads_rule_index {
                rules.remove(i);
            }

            sync_fake_dns_rules(rules, intent);
        }
    }

    // 4) route：同步 final / hijack-dns / ads reject / rule_set download_detour
    if let Some(route_obj) = config_obj.get_mut("route").and_then(|v| v.as_object_mut()) {
        route_obj.insert("final".to_string(), json!(default_outbound));
        route_obj.insert(
            "default_domain_resolver".to_string(),
            json!({
                "server": super::common::DNS_RESOLVER,
                "strategy": intent.dns_strategy
            }),
        );
        // 兼容旧配置残留字段，避免触发 1.14 前置报错。
        route_obj.remove("default_domain_strategy");

        if let Some(rule_sets) = route_obj.get_mut("rule_set").and_then(|v| v.as_array_mut()) {
            // 1.14 替代 `download_detour`：移除所有 remote rule-set 的 download_detour，
            // 统一通过顶层 `route.default_http_client` + `http_clients` 表达。
            // 1.16 后 `download_detour` 字段将被内核直接拒绝。
            for rs in rule_sets.iter_mut() {
                if let Some(obj) = rs.as_object_mut() {
                    if obj.get("type").and_then(|v| v.as_str()) == Some("remote") {
                        obj.remove("download_detour");
                    }
                }
            }

            // 按开关移除不再需要的规则集，避免后台持续下载无用文件
            if !intent.block_ads {
                rule_sets
                    .retain(|rs| rs.get("tag").and_then(|v| v.as_str()) != Some(RS_GEOSITE_ADS));
            }
            if !intent.enable_app_groups {
                rule_sets.retain(|rs| {
                    let tag = rs.get("tag").and_then(|v| v.as_str()).unwrap_or("");
                    !matches!(
                        tag,
                        RS_GEOSITE_TELEGRAM
                            | RS_GEOSITE_YOUTUBE
                            | RS_GEOSITE_NETFLIX
                            | RS_GEOSITE_OPENAI
                            | RS_GEOSITE_GOOGLE
                    )
                });
            }

            // 同步规则集上的 `http_client` 引用（与顶层 `http_clients` 保持一致）
            sync_rule_set_http_client_refs(rule_sets, intent);
        }

        // 1.14 新增：默认 HTTP 客户端（替代 download_detour）—— 必须在 rule_set 借用结束后
        if !route_obj.contains_key("default_http_client") {
            route_obj.insert(
                "default_http_client".to_string(),
                json!(HTTP_CLIENT_DEFAULT_TAG),
            );
        }

        if let Some(rules) = route_obj.get_mut("rules").and_then(|v| v.as_array_mut()) {
            let sniff_index = ensure_sniff_route_rule(rules);

            // 让规则里的 global/非CN 默认走用户选择的出站（manual/auto）
            for rule in rules.iter_mut() {
                if let Some(obj) = rule.as_object_mut() {
                    if obj.get("clash_mode").and_then(|v| v.as_str()) == Some("global") {
                        obj.insert("outbound".to_string(), json!(default_outbound));
                    }
                    if obj.get("rule_set").and_then(|v| v.as_str())
                        == Some(RS_GEOSITE_GEOLOCATION_NOT_CN)
                    {
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
            if intent.dns_hijack {
                if hijack_index.is_none() {
                    // 放在 sniff 后面通常更合适
                    let insert_index = (sniff_index + 1).min(rules.len());
                    rules.insert(
                        insert_index,
                        json!({ "protocol": "dns", "action": "hijack-dns" }),
                    );
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
            if intent.block_ads {
                if ads_index.is_none() {
                    rules.push(json!({ "rule_set": RS_GEOSITE_ADS, "action": "reject" }));
                }
            } else if let Some(i) = ads_index {
                rules.remove(i);
            }

            // 业务分流组：关闭后移除相关规则，避免"空组/无意义分流"
            if !intent.enable_app_groups {
                rules.retain(|rule| {
                    let rs = rule.get("rule_set").and_then(|v| v.as_str()).unwrap_or("");
                    !matches!(
                        rs,
                        RS_GEOSITE_TELEGRAM
                            | RS_GEOSITE_YOUTUBE
                            | RS_GEOSITE_NETFLIX
                            | RS_GEOSITE_OPENAI
                            | RS_GEOSITE_GOOGLE
                    )
                });
            }

            sync_fake_dns_route_rules(rules, intent, &fake_dns_route_cleanup_pairs);
        }
    }

    // 5) outbounds：按开关移除业务分流组（如果存在）
    if let Some(outbounds) = config_obj
        .get_mut("outbounds")
        .and_then(|v| v.as_array_mut())
    {
        if !intent.enable_app_groups {
            outbounds.retain(|ob| {
                let tag = ob.get("tag").and_then(|v| v.as_str()).unwrap_or("");
                !matches!(
                    tag,
                    TAG_TELEGRAM | TAG_YOUTUBE | TAG_NETFLIX | TAG_OPENAI | TAG_GOOGLE
                )
            });
        }
    }

    // 6) 顶层 `http_clients`：与生成路径共用同一份渲染逻辑（必须在 route 借用结束后）
    sync_http_clients(config_obj, intent, ownership);
}

/// 同步顶层 `http_clients`（sing-box 1.14 替代 `rule_set.download_detour` 的原生写法）。
///
/// 只动本程序创建的两个客户端（`default` / `direct-download`），用户自己加的其它
/// `http_client` 原样保留；外来配置（`ownership.managed == false`）一个字节都不动。
///
/// ## ⚠️ 直连客户端不能写 `detour`
///
/// sing-box 1.14+ 的 `common/dialer/detour.go` 会对"detour 指向空 direct 出站"
/// 直接判定为无意义，并让内核**启动失败**：
///
/// ```text
/// start service: initialize rule-set[0]: initial rule-set: geosite-cn:
///   Get "https://.../geosite-cn.srs": detour to an empty direct outbound makes no sense
/// ```
///
/// 不写 `detour` 时 `http_client` 走系统 dialer，语义同样是直连——所以直连必须表达成
/// "没有 `detour` 字段"。存量配置里写的是 `detour: "direct"`，本函数负责纠正回来。
fn sync_http_clients(
    config_obj: &mut Map<String, Value>,
    intent: &ConfigIntent,
    ownership: &Ownership,
) {
    if !ownership.managed {
        // 外来配置：用户自己的 http_client 绝不改写
        return;
    }

    let desired = build_http_clients(intent);
    // 用**自有类型**的 tag 列表（而不是借用 `desired`），后面 `desired` 会被 move 掉。
    let desired_tags: Vec<String> = desired.iter().map(|client| client.tag.clone()).collect();

    let clients = config_obj
        .entry("http_clients".to_string())
        .or_insert_with(|| json!([]));
    if !clients.is_array() {
        *clients = json!([]);
    }
    let Some(arr) = clients.as_array_mut() else {
        return;
    };

    // 先清掉本程序的历史客户端（直连偏好下不再需要 `direct-download`），
    // 避免留下指向不存在客户端的悬空配置。
    // ⚠️ 只清 tag 为 `default` / `direct-download` 的——这两个是本程序创建的；
    // 用户自己加的其它 http_client 原样保留。
    arr.retain(|client| {
        let tag = client.get("tag").and_then(|v| v.as_str()).unwrap_or("");
        let owned = tag == HTTP_CLIENT_DEFAULT_TAG || tag == HTTP_CLIENT_DIRECT_DOWNLOAD_TAG;
        !owned || desired_tags.iter().any(|wanted| wanted == tag)
    });

    // 再按当前偏好写入本程序的客户端（已存在则原地替换，保持顺序不变）
    for client in desired {
        let tag = client.tag.clone();
        let value = serde_json::to_value(&client).expect("http client 序列化失败");
        match arr
            .iter_mut()
            .find(|item| item.get("tag").and_then(|v| v.as_str()) == Some(tag.as_str()))
        {
            Some(slot) => *slot = value,
            None => arr.push(value),
        }
    }

    // `route.default_http_client` 也不能指向已被删除的客户端：
    // 内核对不存在的 http_client 会直接启动失败。
    // 典型触发：配置被手工改过，或经其它工具往返处理过。
    if let Some(route) = config_obj.get_mut("route").and_then(|v| v.as_object_mut()) {
        let current = route
            .get("default_http_client")
            .and_then(|v| v.as_str())
            .unwrap_or("");
        let points_at_removed_client = (current == HTTP_CLIENT_DEFAULT_TAG
            || current == HTTP_CLIENT_DIRECT_DOWNLOAD_TAG)
            && !desired_tags.iter().any(|wanted| wanted == current);
        if points_at_removed_client {
            route.insert(
                "default_http_client".to_string(),
                json!(HTTP_CLIENT_DEFAULT_TAG),
            );
        }
    }
}

/// 同步远程 rule-set 上的 `http_client` 引用。
///
/// - 偏好为直连：本程序不再创建 `direct-download` 客户端，残留引用必须清掉
///   （引用不存在的 client 会让内核启动失败）。
/// - 偏好为走手动选择出站：本机专用规则集（geosite-private）固定走直连客户端。
fn sync_rule_set_http_client_refs(rule_sets: &mut [Value], intent: &ConfigIntent) {
    let direct_tags: Vec<&str> = remote_rule_set_sources(intent)
        .iter()
        .filter(|source| source.direct_detour)
        .map(|source| source.tag)
        .collect();

    for rs in rule_sets.iter_mut() {
        let Some(obj) = rs.as_object_mut() else {
            continue;
        };
        if obj.get("type").and_then(|v| v.as_str()) != Some("remote") {
            continue;
        }

        let tag = obj.get("tag").and_then(|v| v.as_str()).unwrap_or("");
        if intent.download_via_manual() {
            if direct_tags.contains(&tag) {
                obj.insert(
                    "http_client".to_string(),
                    json!(HTTP_CLIENT_DIRECT_DOWNLOAD_TAG),
                );
            }
        } else if obj.get("http_client").and_then(|v| v.as_str())
            == Some(HTTP_CLIENT_DIRECT_DOWNLOAD_TAG)
        {
            obj.remove("http_client");
        }
    }
}

fn apply_inbounds_settings(config_obj: &mut Map<String, Value>, intent: &ConfigIntent) {
    // TUN 路由排除网段：用户显式设置优先，否则沿用配置里已有的值，最后回退平台默认。
    let existing = extract_existing_tun_route_exclude_address(config_obj);
    let inbounds = build_inbounds(intent, existing);
    config_obj.insert("inbounds".to_string(), json!(inbounds));
}

/// 确保 `services` 数组存在且包含 1.14 gRPC api 服务（形状与 config_generator 注入的
/// 保持一致），已有 api 服务时同步端口 / dashboard 开关。
fn ensure_api_service(config_obj: &mut serde_json::Map<String, Value>, intent: &ConfigIntent) {
    let mut api_service = json!({
        "type": "api",
        "listen": "127.0.0.1",
        "listen_port": intent.api_port,
        "dashboard": { "enabled": intent.web_dashboard }
    });
    if intent.web_dashboard {
        // 启用 dashboard 时同时提供 secret（用 listen_port 派生一个本地 secret）
        // 防止 dashboard 在公网意外暴露时无鉴权
        api_service["secret"] = json!(format!("{:x}", intent.api_port));
    }

    let services = config_obj
        .entry("services".to_string())
        .or_insert(json!([]));
    let Some(arr) = services.as_array_mut() else {
        return;
    };
    if let Some(existing) = arr
        .iter_mut()
        .find(|s| s.get("type").and_then(|t| t.as_str()) == Some("api"))
    {
        if let (Some(existing_obj), Some(new_obj)) =
            (existing.as_object_mut(), api_service.as_object())
        {
            for (key, value) in new_obj {
                existing_obj.insert(key.clone(), value.clone());
            }
        }
    } else {
        arr.push(api_service);
    }
}

#[cfg(test)]
#[path = "settings_patch.tests.rs"]
mod tests;
