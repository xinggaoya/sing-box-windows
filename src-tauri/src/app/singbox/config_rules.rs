//! 路由 / DNS 规则的**共享语义层**：生成路径与补丁路径产出同一份规则。
//!
//! ## 为什么单独成模块
//!
//! 规则（尤其是 `route.rules` 的顺序与动作类型）是"配置语义"里最容易被两条路径
//! 写歪的部分：生成路径从零拼一份，补丁路径在已有配置上按特征定位后改写。
//! 只要两边对"一条规则长什么样、放在哪个位置"的判断有一丁点不同，
//! 用户拿到的运行时配置就会和设置项对不上——而且**不会报错**，只会表现为
//! "某些网站分流不对"这类极难定位的问题。
//!
//! 所以这里把规则的**定义**（[`build_route_rules`] / [`build_dns_rules`]）与
//! **对齐**（[`sync_route_rules`] / [`sync_dns_rules`]）收敛到一处，
//! 两条路径都只调用这两个函数。
//!
//! ## 关键语义：sing-box 1.11+ 的 `resolve` 动作
//!
//! sing-box 1.11 起，路由规则里的 **IP 匹配**（`ip_cidr` / `geoip-*` 规则集）
//! 只在目标**已经是 IP** 时才会命中；目标仍是域名时**不会隐式解析**。
//! 要让"按 IP 判定国内"的 `geoip-cn` 对域名流量生效，必须显式插入
//! `{"action": "resolve"}`，把域名先解析成 IP，后续 IP 规则才能匹配。
//!
//! 这正是"国内网站误走代理"的根因：
//!
//! - `geosite-cn` 是**域名清单**，永远不完整（实测 `rum-openway.guance.com`、
//!   `filecdn.minimax.chat`、`ts2.tc.mm.bing.net` 等国内站点的域名都不在其中）；
//! - `geoip-cn` 是**IP 规则集**，没有 `resolve` 动作时对域名流量**完全不生效**；
//! - 于是这些国内站点既不匹配域名规则、也匹配不到 IP 规则，
//!   一路落到 `final` → **走代理**。
//!
//! `resolve` 是**非终结**动作：解析之后继续匹配后续规则，
//! 域名规则仍然按域名匹配（实测 `bilibili.com` / `www.youtube.com` 均正常）。

use super::common::{
    DNS_CN, DNS_MDNS, DNS_PROXY, PRIVATE_IP_CIDRS, RS_GEOIP_CN, RS_GEOSITE_ADS, RS_GEOSITE_CN,
    RS_GEOSITE_GEOLOCATION_NOT_CN, RS_GEOSITE_GOOGLE, RS_GEOSITE_NETFLIX, RS_GEOSITE_OPENAI,
    RS_GEOSITE_PRIVATE, RS_GEOSITE_TELEGRAM, RS_GEOSITE_YOUTUBE, TAG_DIRECT, TAG_GOOGLE,
    TAG_MANUAL, TAG_NETFLIX, TAG_OPENAI, TAG_TELEGRAM, TAG_YOUTUBE,
};
use super::config_intent::ConfigIntent;
use serde_json::{json, Value};

/// mDNS 覆盖的域名后缀。
///
/// 与 sing-box 1.14 `dns/transport/mdns` 的 `mdnsLocalZones` 一一对应；
/// 不用 `geosite-private`（那里面混着 `*.lan` 等非 mDNS 名称，发去多播组不会有应答）。
const MDNS_DOMAIN_SUFFIXES: &[&str] = &[
    "local",
    "254.169.in-addr.arpa",
    "8.e.f.ip6.arpa",
    "9.e.f.ip6.arpa",
    "a.e.f.ip6.arpa",
    "b.e.f.ip6.arpa",
];

/// 构造一条"路由到指定出站"的路由规则。
///
/// sing-box 1.11+ 的官方形态是 `action: "route"` + `outbound`；
/// 裸写 `outbound` 是 1.11 之前的旧写法（`route` 动作是默认动作，行为等价，
/// 但显式写出来更利于跨版本阅读与工具解析）。
fn route_to(outbound: &str, extra: Value) -> Value {
    let mut rule = extra;
    if let Some(obj) = rule.as_object_mut() {
        obj.insert("action".to_string(), json!("route"));
        obj.insert("outbound".to_string(), json!(outbound));
    }
    rule
}

/// 构造一条"把解析交给指定 DNS 服务器"的 DNS 规则。
///
/// sing-box 1.11+ 的官方形态是 `action: "route"` + `server`。
fn dns_to(server: &str, extra: Value) -> Value {
    let mut rule = extra;
    if let Some(obj) = rule.as_object_mut() {
        obj.insert("action".to_string(), json!("route"));
        obj.insert("server".to_string(), json!(server));
    }
    rule
}

/// 构造"把域名目标解析成 IP"的非终结动作规则。
///
/// - `strategy` 与全局 `dns_strategy` 保持一致（IPv6 偏好开启时用 `prefer_ipv6`）。
/// - 不指定 `server`：解析走**正常的 DNS 分流**（`dns.rules`），
///   国内域名仍然由国内解析器解析、境外域名由境外解析器解析。
pub fn build_resolve_rule(intent: &ConfigIntent) -> Value {
    json!({
        "action": "resolve",
        "strategy": intent.dns_strategy
    })
}

/// 生成本程序托管的**完整路由规则清单**（生成路径）。
///
/// 顺序即优先级（sing-box 自上而下匹配，第一条命中即生效）：
///
/// 1. `sniff` —— 从 TLS SNI / HTTP Host 里还原域名（TUN 模式下这是拿到域名的唯一途径）
/// 2. `hijack-dns` —— 接管系统 DNS 查询
/// 3. `clash_mode` 覆盖 —— 全局 / 直连模式优先于一切分流规则
/// 4. 广告拦截
/// 5. 业务分流组（Telegram / YouTube / Netflix / OpenAI / Google）
/// 6. 私网直连（`geosite-private` + 内置私网网段）
/// 7. **国内域名直连**（`geosite-cn`，域名规则，无需解析即可定去向）
/// 8. **境外域名交代理**（`geosite-geolocation-!cn`，域名直送代理，由远端解析）
/// 9. **`resolve`** —— 把剩下的域名解析成 IP
/// 10. **国内 IP 直连**（`geoip-cn`，依赖第 9 步的解析结果）
///
/// 第 7/8 步刻意放在 `resolve` **之前**：命中的域名立刻定去向，
/// 既省掉一次 DNS 解析，也让境外域名保持"域名"形态交给代理（由远端解析，
/// 避免本地 DNS 被污染，也避免"代理 → 本地 DNS → 代理"的链路依赖）。
pub fn build_route_rules(intent: &ConfigIntent) -> Vec<Value> {
    let default_outbound = intent.default_outbound;
    let mut rules: Vec<Value> = vec![json!({ "action": "sniff" })];

    if intent.dns_hijack {
        rules.push(json!({ "protocol": "dns", "action": "hijack-dns" }));
    }

    rules.push(route_to(
        default_outbound,
        json!({ "clash_mode": "global" }),
    ));
    rules.push(route_to(TAG_DIRECT, json!({ "clash_mode": "direct" })));

    if intent.block_ads {
        rules.push(json!({ "rule_set": RS_GEOSITE_ADS, "action": "reject" }));
    }

    if intent.enable_app_groups {
        rules.push(route_to(
            TAG_TELEGRAM,
            json!({ "rule_set": RS_GEOSITE_TELEGRAM }),
        ));
        rules.push(route_to(
            TAG_YOUTUBE,
            json!({ "rule_set": RS_GEOSITE_YOUTUBE }),
        ));
        rules.push(route_to(
            TAG_NETFLIX,
            json!({ "rule_set": RS_GEOSITE_NETFLIX }),
        ));
        rules.push(route_to(
            TAG_OPENAI,
            json!({ "rule_set": RS_GEOSITE_OPENAI }),
        ));
        rules.push(route_to(
            TAG_GOOGLE,
            json!({ "rule_set": RS_GEOSITE_GOOGLE }),
        ));
    }

    rules.push(route_to(
        TAG_DIRECT,
        json!({ "rule_set": RS_GEOSITE_PRIVATE }),
    ));
    rules.push(route_to(TAG_DIRECT, json!({ "ip_cidr": PRIVATE_IP_CIDRS })));

    rules.push(route_to(TAG_DIRECT, json!({ "rule_set": RS_GEOSITE_CN })));
    rules.push(route_to(
        default_outbound,
        json!({ "rule_set": RS_GEOSITE_GEOLOCATION_NOT_CN }),
    ));

    rules.push(build_resolve_rule(intent));

    rules.push(route_to(TAG_DIRECT, json!({ "rule_set": RS_GEOIP_CN })));

    rules
}

/// 生成本程序托管的**完整 DNS 规则清单**（生成路径）。
///
/// 关键取舍：
///
/// - **`dns.rules` 里不再出现 `geoip-cn`**。
///   IP 规则集在 DNS 阶段无法匹配"尚未解析的域名"（先有鸡还是先有蛋），
///   属于 sing-box 1.14 起废弃的 "Legacy Address Filter Fields"
///   （1.16 将被移除，内核会打印弃用告警）。IP 判定统一交给 `route.rules`。
/// - **兜底 DNS 走国内解析器**（见 [`ConfigIntent::dns_final`]）：
///   未命中任何规则的域名由国内解析器解析，**不依赖代理可用性**，
///   代理挂掉时国内站点照常解析、照常直连。
/// - 只有明确判定为境外的域名（`geosite-geolocation-!cn`）才走境外解析器，
///   且该解析器经代理出站，天然免疫污染。
pub fn build_dns_rules(intent: &ConfigIntent) -> Vec<Value> {
    let mut rules: Vec<Value> = Vec::new();

    // mDNS 规则必须排在所有其他 DNS 规则之前：*.local / link-local 反查只应答于
    // 本地多播，落到 CN/代理解析都会失败。
    if intent.dns_use_mdns {
        rules.push(dns_to(
            DNS_MDNS,
            json!({ "domain_suffix": MDNS_DOMAIN_SUFFIXES }),
        ));
    }

    // 模式覆盖优先于分流规则
    rules.push(dns_to(DNS_CN, json!({ "clash_mode": "direct" })));
    rules.push(dns_to(DNS_PROXY, json!({ "clash_mode": "global" })));

    if intent.block_ads {
        rules.push(json!({ "rule_set": RS_GEOSITE_ADS, "action": "reject" }));
    }

    rules.push(dns_to(DNS_CN, json!({ "rule_set": RS_GEOSITE_CN })));
    rules.push(dns_to(
        DNS_PROXY,
        json!({ "rule_set": RS_GEOSITE_GEOLOCATION_NOT_CN }),
    ));

    rules
}

/// 规则里"托管特征"的分类结果。
///
/// 补丁路径据此识别"哪些规则是本程序生成的"，从而只重写这些规则、
/// 保留用户自己加的规则（自定义规则、外来配置里的额外分流）。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ManagedRule {
    /// `action: resolve`
    Resolve,
    /// `geosite-cn` 域名规则
    GeoSiteCn,
    /// `geoip-cn` IP 规则
    GeoIpCn,
    /// `geosite-geolocation-!cn` 规则
    GeoLocationNotCn,
}

/// 判断一条路由规则是否属于本程序托管的"国内/境外分流"规则。
fn classify_managed_rule(rule: &Value) -> Option<ManagedRule> {
    if rule.get("action").and_then(|v| v.as_str()) == Some("resolve") {
        return Some(ManagedRule::Resolve);
    }

    let rule_sets = collect_rule_set_tags(rule);
    if rule_sets.is_empty() {
        return None;
    }

    let has_cn = rule_sets.iter().any(|tag| tag == RS_GEOSITE_CN);
    let has_geoip_cn = rule_sets.iter().any(|tag| tag == RS_GEOIP_CN);
    let has_not_cn = rule_sets
        .iter()
        .any(|tag| tag == RS_GEOSITE_GEOLOCATION_NOT_CN);

    if has_not_cn {
        return Some(ManagedRule::GeoLocationNotCn);
    }
    if has_cn && has_geoip_cn {
        // 旧版把两个规则集合在一条规则里；拆成两条（域名一条、IP 一条）
        return Some(ManagedRule::GeoSiteCn);
    }
    if has_cn {
        return Some(ManagedRule::GeoSiteCn);
    }
    if has_geoip_cn {
        return Some(ManagedRule::GeoIpCn);
    }
    None
}

/// 收集一条规则里引用到的 rule-set tag（兼容 `rule_set` 是字符串或数组两种写法）。
fn collect_rule_set_tags(rule: &Value) -> Vec<String> {
    match rule.get("rule_set") {
        Some(Value::String(tag)) => vec![tag.clone()],
        Some(Value::Array(items)) => items
            .iter()
            .filter_map(|item| item.as_str().map(|s| s.to_string()))
            .collect(),
        _ => Vec::new(),
    }
}

/// 本程序托管的 rule-set tag（用于把旧写法归一到 `action: "route"` + `outbound`）。
const MANAGED_RULE_SET_TAGS: &[&str] = &[
    RS_GEOSITE_TELEGRAM,
    RS_GEOSITE_YOUTUBE,
    RS_GEOSITE_NETFLIX,
    RS_GEOSITE_OPENAI,
    RS_GEOSITE_GOOGLE,
    RS_GEOSITE_PRIVATE,
];

/// 把一条"本程序托管"的路由规则归一到 1.11+ 官方写法。
///
/// sing-box 1.11+ 的路由规则用 `action: "route"` + `outbound` 表达"路由到某个出站"；
/// 裸写 `outbound` 是 1.11 之前的旧写法（`route` 是默认动作，行为等价，
/// 但旧写法会在未来版本被移除）。只改写**本程序认领**的规则，
/// 用户自己写的规则原样保留。
fn normalize_managed_route_rule(rule: &mut Value) {
    let rule_sets = collect_rule_set_tags(rule);
    let is_managed = rule_sets
        .iter()
        .any(|tag| MANAGED_RULE_SET_TAGS.contains(&tag.as_str()))
        // clash_mode 覆盖规则也是本程序生成的
        || rule.get("clash_mode").and_then(|v| v.as_str()).is_some();
    if !is_managed {
        return;
    }
    let Some(obj) = rule.as_object_mut() else {
        return;
    };
    if obj.contains_key("outbound") && !obj.contains_key("action") {
        obj.insert("action".to_string(), json!("route"));
    }
}

/// 把已有 `route.rules` 对齐到本程序托管的形态（补丁路径）。
///
/// 做法：识别出托管规则 → 移除 → 用当前语义重建 → 插回原来的位置。
/// 用户自定义规则（无法识别的）原样保留。
pub fn sync_route_rules(rules: &mut Vec<Value>, intent: &ConfigIntent) {
    let default_outbound = intent.default_outbound;

    // 记录托管规则所在的位置（用于插回），以及非托管规则是否已经"在前面"
    let original_len = rules.len();
    let mut managed_positions: Vec<usize> = Vec::new();
    let mut kept: Vec<Value> = Vec::with_capacity(original_len);
    for (idx, rule) in rules.drain(..).enumerate() {
        match classify_managed_rule(&rule) {
            Some(_) => managed_positions.push(idx),
            None => kept.push(rule),
        }
    }

    // 重建托管规则
    let mut rebuilt: Vec<Value> = Vec::new();
    rebuilt.push(route_to(TAG_DIRECT, json!({ "rule_set": RS_GEOSITE_CN })));
    rebuilt.push(route_to(
        default_outbound,
        json!({ "rule_set": RS_GEOSITE_GEOLOCATION_NOT_CN }),
    ));
    rebuilt.push(build_resolve_rule(intent));
    rebuilt.push(route_to(TAG_DIRECT, json!({ "rule_set": RS_GEOIP_CN })));

    // 插回位置：托管规则原本的第一处；没有托管规则时追加到末尾
    // （注意：不能用 `rules.len()`——上面已经 drain 过，此时为 0）
    let insert_at = managed_positions
        .first()
        .copied()
        .unwrap_or(original_len)
        .min(kept.len());

    let mut merged: Vec<Value> = Vec::with_capacity(kept.len() + rebuilt.len());
    merged.extend_from_slice(&kept[..insert_at]);
    merged.extend(rebuilt);
    merged.extend_from_slice(&kept[insert_at..]);

    // 归一化其余托管规则的写法（业务分流组 / 私网直连）
    for rule in merged.iter_mut() {
        normalize_managed_route_rule(rule);
    }

    *rules = merged;
}

/// 把已有 `dns.rules` 对齐到本程序托管的形态（补丁路径）。
///
/// 主要修正三件事：
///
/// 1. 旧版的 `[geosite-cn, geoip-cn]` 合并规则拆成 `geosite-cn` 一条
///    （`geoip-cn` 属于 1.14 起废弃的 Legacy Address Filter Fields，
///    且在 DNS 阶段本就无法匹配未解析的域名）；
/// 2. 统一成 `action: "route"` + `server` 的 1.11+ 官方形态；
/// 3. 广告拦截规则与模式覆盖规则的相对顺序对齐生成路径。
pub fn sync_dns_rules(rules: &mut Vec<Value>, intent: &ConfigIntent) {
    let mut rebuilt: Vec<Value> = Vec::new();

    // mDNS 规则：保留用户已有的写法（域名后缀集合可能被用户改过）
    if let Some(existing) = rules.iter().find(|rule| {
        rule.get("server").and_then(|v| v.as_str()) == Some(DNS_MDNS)
            || rule.get("action").and_then(|v| v.as_str()) == Some("route")
                && rule.get("server").and_then(|v| v.as_str()) == Some(DNS_MDNS)
    }) {
        rebuilt.push(existing.clone());
    } else if intent.dns_use_mdns {
        rebuilt.push(dns_to(
            DNS_MDNS,
            json!({ "domain_suffix": MDNS_DOMAIN_SUFFIXES }),
        ));
    }

    // 模式覆盖
    rebuilt.push(dns_to(DNS_CN, json!({ "clash_mode": "direct" })));
    rebuilt.push(dns_to(DNS_PROXY, json!({ "clash_mode": "global" })));

    // 广告拦截
    if intent.block_ads {
        rebuilt.push(json!({ "rule_set": RS_GEOSITE_ADS, "action": "reject" }));
    }

    // 国内域名 → 国内解析器；境外域名 → 境外解析器
    rebuilt.push(dns_to(DNS_CN, json!({ "rule_set": RS_GEOSITE_CN })));
    rebuilt.push(dns_to(
        DNS_PROXY,
        json!({ "rule_set": RS_GEOSITE_GEOLOCATION_NOT_CN }),
    ));

    // 保留无法识别的用户规则（追加在托管规则之后）
    for rule in rules.iter() {
        let server = rule.get("server").and_then(|v| v.as_str());
        let action = rule.get("action").and_then(|v| v.as_str());
        let rule_sets = collect_rule_set_tags(rule);
        let is_managed = matches!(server, Some(DNS_CN) | Some(DNS_PROXY) | Some(DNS_MDNS))
            || (action == Some("reject") && rule_sets.iter().any(|tag| tag == RS_GEOSITE_ADS))
            || rule_sets
                .iter()
                .any(|tag| tag == RS_GEOSITE_CN || tag == RS_GEOIP_CN);
        if !is_managed {
            rebuilt.push(rule.clone());
        }
    }

    *rules = rebuilt;
}

/// `dns.final`（未命中任何 DNS 规则时的兜底解析器）对应的 **server tag**。
///
/// - `dns_cn`：国内解析器直连兜底（默认）。**不依赖代理可用性**，
///   代理挂掉时国内站点照常解析；未收录的境外域名也能拿到真实 IP。
/// - `dns_proxy`：境外解析器（经代理）。适合“境外站点必须走境外 DNS”的场景，
///   但代理不可用时全部解析都会失败。
pub fn dns_final_server(intent: &ConfigIntent) -> &'static str {
    match intent.dns_final {
        super::common::DNS_FINAL_STRATEGY_PROXY => DNS_PROXY,
        _ => DNS_CN,
    }
}

/// 供补丁路径判断"手动切换"分组是否已包含直连兜底成员。
pub fn manual_group_has_direct(outbounds: &[Value]) -> bool {
    outbounds.iter().any(|ob| {
        ob.get("tag").and_then(|v| v.as_str()) == Some(TAG_MANUAL)
            && ob
                .get("outbounds")
                .and_then(|v| v.as_array())
                .map(|list| list.iter().any(|item| item.as_str() == Some(TAG_DIRECT)))
                .unwrap_or(false)
    })
}

#[cfg(test)]
#[path = "config_rules.tests.rs"]
mod tests;
