use crate::app::storage::state_model::AppConfig;

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
pub const DNS_PROXY: &str = "dns_proxy";
pub const DNS_CN: &str = "dns_cn";
pub const DNS_RESOLVER: &str = "dns_resolver";
pub const DNS_BLOCK: &str = "dns_block";

// Rule-set tags (官方 SagerNet 规则集)
pub const RS_GEOSITE_CN: &str = "geosite-cn";
pub const RS_GEOSITE_GEOLOCATION_NOT_CN: &str = "geosite-geolocation-!cn";
pub const RS_GEOSITE_PRIVATE: &str = "geosite-private";
pub const RS_GEOSITE_ADS: &str = "geosite-category-ads-all";
pub const RS_GEOSITE_TELEGRAM: &str = "geosite-telegram";
pub const RS_GEOSITE_YOUTUBE: &str = "geosite-youtube";
pub const RS_GEOSITE_NETFLIX: &str = "geosite-netflix";
pub const RS_GEOSITE_OPENAI: &str = "geosite-openai";
pub const RS_GEOIP_CN: &str = "geoip-cn";
pub const RS_GEOIP_PRIVATE: &str = "geoip-private";

pub fn normalize_default_outbound(app_config: &AppConfig) -> &'static str {
    match app_config.singbox_default_proxy_outbound.as_str() {
        "auto" => TAG_AUTO,
        _ => TAG_MANUAL,
    }
}

pub fn normalize_download_detour(app_config: &AppConfig) -> &'static str {
    match app_config.singbox_download_detour.as_str() {
        "direct" => TAG_DIRECT,
        // 订阅规则集/Clash UI 下载默认走“手动切换”，便于用户用可用节点下载（适配国内网络）
        _ => TAG_MANUAL,
    }
}

pub fn dns_strategy(app_config: &AppConfig) -> &'static str {
    if app_config.prefer_ipv6 {
        "prefer_ipv6"
    } else {
        "ipv4_only"
    }
}

pub fn node_domain_resolver_strategy(app_config: &AppConfig) -> &'static str {
    if app_config.prefer_ipv6 {
        "prefer_ipv6"
    } else {
        // 节点域名解析默认走 IPv4，能显著降低“有 AAAA 但本机 IPv6 不可用”导致的连接失败。
        "ipv4_only"
    }
}

