//! 配置归属（Ownership）：记录"这份配置里哪些 tag 是本程序创建的"。
//!
//! ## 为什么需要它
//!
//! 补丁路径此前用**结构猜测**判断"这份配置是不是本程序的"：
//! `settings_patch::apply_profile_settings_if_present` 看到 `dns.servers` 里有 `dns_cn`
//! 就当成自己的服务器去重建、看到 `outbounds` 里有 `auto` 就去改它的测速地址、
//! 看到 `route.rules` 里有一条 `rule_set` 规则就去覆盖 `route.final`……
//!
//! 猜错的代价很大：用户导入的机场原始配置一旦被"接管"，
//! 入口会被整个重建、路由终点会被改写、`http_clients` 会被塞进一个不存在的引用——
//! 配置直接失效，而这种失败**不会报错**，只表现为"能连但走错线路"。
//!
//! ## 归属怎么来
//!
//! - **生成时**：本程序生成的配置，把它创建的 tag 清单写进 `generic_config`
//!   （key = `managed_config_tags:<配置文件路径>`，与 `custom_rules` 同一套 KV 通道）。
//! - **补丁时**：先查落库的归属；查不到（例如老库、或用户手改过配置）就按配置内容**推断**。
//! - **两者都判定为"不是本程序的配置"** → 退化为 `PortsOnly`（只改端口与监听地址），
//!   而不是继续猜结构——猜错会改坏用户导入的原始配置。
//!
//! ## 判定标准
//!
//! `looks_managed` 只认本程序生成配置时留下的**标记**（`mixed-in` 入口、
//! `dns_proxy`/`dns_cn`/`dns_resolver` 等本程序自定义的 DNS server tag）。
//! 刻意不拿"自动选择/手动切换"这类分组名当标记——用户自己的配置里也可能出现同名分组。
//!
//! ⚠️ 这些标记**不是本程序独占的**：`mixed-in` 是 sing-box 官方示例里的常见 tag，
//! 沿用它的机场配置会被判定为已接管。因此归属判定只用于**决定能不能改写**，
//! 不用于判断"配置是谁写的"；真正的兜底是"判定为外来配置就只同步端口"。
//! 另见 `enhanced_storage_service::resolve_ownership` 的落库条件。

use super::common::{
    DNS_CN, DNS_FAKEIP, DNS_MDNS, DNS_PROXY, DNS_RESOLVER, RS_GEOIP_CN, RS_GEOSITE_ADS,
    RS_GEOSITE_CN, RS_GEOSITE_GEOLOCATION_NOT_CN, RS_GEOSITE_GOOGLE, RS_GEOSITE_NETFLIX,
    RS_GEOSITE_OPENAI, RS_GEOSITE_PRIVATE, RS_GEOSITE_TELEGRAM, RS_GEOSITE_YOUTUBE, TAG_AUTO,
    TAG_GOOGLE, TAG_MANUAL, TAG_NETFLIX, TAG_OPENAI, TAG_TELEGRAM, TAG_YOUTUBE,
};
use serde::{Deserialize, Serialize};
use serde_json::Value;

/// `generic_config` 中存放"本程序管理的 tag 清单"所用的 key 前缀。
/// 按配置文件路径分桶：一份订阅配置一份归属，切换订阅互不污染。
pub const STORAGE_KEY_PREFIX: &str = "managed_config_tags";

/// 本程序创建的入口（inbound）tag
pub const OWNED_INBOUND_TAGS: [&str; 2] = ["mixed-in", "tun-in"];

/// 本程序创建的 DNS server tag
pub const OWNED_DNS_SERVER_TAGS: [&str; 5] =
    [DNS_PROXY, DNS_CN, DNS_RESOLVER, DNS_FAKEIP, DNS_MDNS];

/// 本程序创建的出站分组 tag
pub const OWNED_GROUP_TAGS: [&str; 7] = [
    TAG_AUTO,
    TAG_MANUAL,
    TAG_TELEGRAM,
    TAG_YOUTUBE,
    TAG_NETFLIX,
    TAG_OPENAI,
    TAG_GOOGLE,
];

/// 本程序创建的远程 rule-set tag
pub const OWNED_RULE_SET_TAGS: [&str; 10] = [
    RS_GEOSITE_ADS,
    RS_GEOSITE_CN,
    RS_GEOSITE_GEOLOCATION_NOT_CN,
    RS_GEOSITE_TELEGRAM,
    RS_GEOSITE_YOUTUBE,
    RS_GEOSITE_NETFLIX,
    RS_GEOSITE_OPENAI,
    RS_GEOSITE_GOOGLE,
    RS_GEOSITE_PRIVATE,
    RS_GEOIP_CN,
];

/// 某个配置文件对应的归属存储 key
pub fn storage_key(config_path: &str) -> String {
    format!("{STORAGE_KEY_PREFIX}:{config_path}")
}

/// 本程序负责维护的 tag 集合。
#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq, Eq)]
pub struct Ownership {
    /// 本程序是否拥有这份配置。
    /// `false` = 外来配置（用户导入的机场原始配置等），只允许最小侵入补丁。
    pub managed: bool,
    /// 本程序创建的入口 tag
    pub inbounds: Vec<String>,
    /// 本程序创建的 DNS server tag
    pub dns_servers: Vec<String>,
    /// 本程序创建的出站分组 tag
    pub groups: Vec<String>,
    /// 本程序创建的远程 rule-set tag
    pub rule_sets: Vec<String>,
}

impl Ownership {
    /// 空归属：这份配置不属于本程序
    pub fn empty() -> Self {
        Self::default()
    }

    /// 从配置内容**推断**归属。
    ///
    /// 判定为"本程序的配置"时，只认领配置里**实际存在**的本程序 tag；
    /// 判定为"外来配置"时返回空归属（调用方据此退化为最小侵入补丁）。
    pub fn inferred(config: &Value) -> Self {
        if !looks_managed(config) {
            return Self::empty();
        }
        Self::claimed(config)
    }

    /// 认领配置里实际存在的、本程序认识的 tag。
    ///
    /// 用于生成路径：配置是我们自己写的，归属必然是本程序的。
    pub fn claimed(config: &Value) -> Self {
        let inbound_tags = tags_of(config, |config| config.get("inbounds"));
        let dns_server_tags = tags_of(config, |config| {
            config.get("dns").and_then(|dns| dns.get("servers"))
        });
        let outbound_tags = tags_of(config, |config| config.get("outbounds"));
        let rule_set_tags = tags_of(config, |config| {
            config.get("route").and_then(|route| route.get("rule_set"))
        });

        Self {
            managed: true,
            inbounds: intersect(&inbound_tags, &OWNED_INBOUND_TAGS),
            dns_servers: intersect(&dns_server_tags, &OWNED_DNS_SERVER_TAGS),
            groups: intersect(&outbound_tags, &OWNED_GROUP_TAGS),
            rule_sets: intersect(&rule_set_tags, &OWNED_RULE_SET_TAGS),
        }
    }

    /// 是否拥有入口 tag
    pub fn owns_inbound(&self, tag: &str) -> bool {
        self.inbounds.iter().any(|owned| owned == tag)
    }

    /// 是否拥有 DNS server tag
    pub fn owns_dns_server(&self, tag: &str) -> bool {
        self.dns_servers.iter().any(|owned| owned == tag)
    }

    /// 是否拥有出站分组 tag
    pub fn owns_group(&self, tag: &str) -> bool {
        self.groups.iter().any(|owned| owned == tag)
    }

    /// 是否拥有远程 rule-set tag
    pub fn owns_rule_set(&self, tag: &str) -> bool {
        self.rule_sets.iter().any(|owned| owned == tag)
    }

    /// 是否**一个 tag 都没认领**。
    ///
    /// 出现在两种情况：配置是"空壳"（既没有本程序的入口，也没有本程序的 DNS server），
    /// 或配置读取失败被当成空壳。这种归属记录**没有信息量**，落库后反而会
    /// 让后续补丁拿一份空清单去过滤（DNS 地址、测速 URL 等设置会静默不生效）。
    pub fn claims_nothing(&self) -> bool {
        self.inbounds.is_empty()
            && self.dns_servers.is_empty()
            && self.groups.is_empty()
            && self.rule_sets.is_empty()
    }
}

/// 这份配置看起来是不是本程序生成的。
///
/// 只认本程序的**专属标记**：
/// - 入口 `mixed-in`（本程序创建）
/// - 本程序自定义的 DNS server tag（`dns_proxy` / `dns_cn` / `dns_resolver` / `dns_fakeip` / `dns_mdns`）
///
/// 另外，"空壳配置"（没有任何入口 / DNS server / 出站 / rule-set）也视为本程序的配置——
/// 那是本程序刚生成、还没来得及注入节点的中间态，交给补丁路径补齐是安全的。
pub fn looks_managed(config: &Value) -> bool {
    let inbound_tags = tags_of(config, |config| config.get("inbounds"));
    let dns_server_tags = tags_of(config, |config| {
        config.get("dns").and_then(|dns| dns.get("servers"))
    });
    let outbound_tags = tags_of(config, |config| config.get("outbounds"));
    let rule_set_tags = tags_of(config, |config| {
        config.get("route").and_then(|route| route.get("rule_set"))
    });

    if inbound_tags.iter().any(|tag| tag == "mixed-in") {
        return true;
    }
    if OWNED_DNS_SERVER_TAGS
        .iter()
        .any(|tag| dns_server_tags.iter().any(|owned| owned == tag))
    {
        return true;
    }

    // 空壳配置：没有可管理的结构，交给补丁路径补齐是安全的
    inbound_tags.is_empty()
        && dns_server_tags.is_empty()
        && outbound_tags.is_empty()
        && rule_set_tags.is_empty()
}

/// 收集某个数组里所有对象的 `tag` 字段
fn tags_of(config: &Value, pick: impl Fn(&Value) -> Option<&Value>) -> Vec<String> {
    pick(config)
        .and_then(|value| value.as_array())
        .map(|items| {
            items
                .iter()
                .filter_map(|item| item.get("tag").and_then(|tag| tag.as_str()))
                .map(|tag| tag.to_string())
                .collect()
        })
        .unwrap_or_default()
}

/// 取两组 tag 的交集（保持 `known` 的顺序）
fn intersect(present: &[String], known: &[&str]) -> Vec<String> {
    known
        .iter()
        .filter(|tag| present.iter().any(|owned| owned == *tag))
        .map(|tag| tag.to_string())
        .collect()
}

#[cfg(test)]
#[path = "ownership.tests.rs"]
mod tests;
