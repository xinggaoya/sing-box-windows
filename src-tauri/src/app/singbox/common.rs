use super::config_schema::DnsServerConfig;
use crate::app::storage::state_model::AppConfig;
use serde_json::json;
use std::net::IpAddr;
use url::Url;

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
pub const TAG_GOOGLE: &str = "Google";

// DNS server tags
pub const DNS_PROXY: &str = "dns_proxy";
pub const DNS_CN: &str = "dns_cn";
pub const DNS_RESOLVER: &str = "dns_resolver";
pub const DNS_FAKEIP: &str = "dns_fakeip";

pub const FAKE_DNS_FILTER_PROXY_ONLY: &str = "proxy_only";
pub const FAKE_DNS_FILTER_GLOBAL_NON_CN: &str = "global_non_cn";

// Rule-set tags (官方 SagerNet 规则集)
pub const RS_GEOSITE_CN: &str = "geosite-cn";
pub const RS_GEOSITE_GEOLOCATION_NOT_CN: &str = "geosite-geolocation-!cn";
pub const RS_GEOSITE_PRIVATE: &str = "geosite-private";
pub const RS_GEOSITE_ADS: &str = "geosite-category-ads-all";
pub const RS_GEOSITE_TELEGRAM: &str = "geosite-telegram";
pub const RS_GEOSITE_YOUTUBE: &str = "geosite-youtube";
pub const RS_GEOSITE_NETFLIX: &str = "geosite-netflix";
pub const RS_GEOSITE_OPENAI: &str = "geosite-openai";
pub const RS_GEOSITE_GOOGLE: &str = "geosite-google";
pub const RS_GEOIP_CN: &str = "geoip-cn";
pub const RS_GEOIP_PRIVATE: &str = "geoip-private";
pub const PRIVATE_IP_CIDRS: &[&str] = &[
    "10.0.0.0/8",
    "100.64.0.0/10",
    "127.0.0.0/8",
    "169.254.0.0/16",
    "172.16.0.0/12",
    "192.168.0.0/16",
    "::1/128",
    "fc00::/7",
    "fe80::/10",
];

pub fn normalize_default_outbound(app_config: &AppConfig) -> &'static str {
    match app_config.singbox_default_proxy_outbound.as_str() {
        "auto" => TAG_AUTO,
        _ => TAG_MANUAL,
    }
}

pub fn normalize_download_detour(app_config: &AppConfig) -> &'static str {
    match app_config.singbox_download_detour.as_str() {
        "manual" => TAG_MANUAL,
        // 默认值调整为直连：gh-proxy 已经加速，避免多余的代理链路
        _ => TAG_DIRECT,
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

pub fn normalize_fake_dns_filter_mode(app_config: &AppConfig) -> &'static str {
    match app_config.singbox_fake_dns_filter_mode.as_str() {
        FAKE_DNS_FILTER_GLOBAL_NON_CN => FAKE_DNS_FILTER_GLOBAL_NON_CN,
        _ => FAKE_DNS_FILTER_PROXY_ONLY,
    }
}

fn default_dns_server_port(server_type: &str) -> u16 {
    match server_type {
        "https" | "h3" | "tls" | "quic" => 443,
        _ => 53,
    }
}

fn is_domain(value: &str) -> bool {
    value.parse::<IpAddr>().is_err()
}

fn parse_legacy_host_port(input: &str) -> Result<(String, Option<u16>), String> {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return Err("DNS 服务器地址为空".to_string());
    }

    if trimmed.starts_with('[') {
        // 支持 [IPv6]:port 形式
        if let Some(end) = trimmed.find(']') {
            let host = &trimmed[1..end];
            if host.is_empty() {
                return Err("DNS IPv6 地址为空".to_string());
            }
            let port = if end + 1 < trimmed.len() {
                let suffix = &trimmed[end + 1..];
                if let Some(port_str) = suffix.strip_prefix(':') {
                    Some(
                        port_str
                            .parse::<u16>()
                            .map_err(|_| format!("无效的 DNS 端口: {}", port_str))?,
                    )
                } else {
                    return Err(format!("无效的 DNS 地址格式: {}", trimmed));
                }
            } else {
                None
            };
            return Ok((host.to_string(), port));
        }
    }

    // 裸 IPv6（不带端口）直接作为 host 使用
    if trimmed.matches(':').count() > 1 && trimmed.parse::<IpAddr>().is_ok() {
        return Ok((trimmed.to_string(), None));
    }

    if let Some((host, port)) = trimmed.rsplit_once(':') {
        if !host.is_empty() && port.chars().all(|c| c.is_ascii_digit()) {
            return Ok((
                host.to_string(),
                Some(
                    port.parse::<u16>()
                        .map_err(|_| format!("无效的 DNS 端口: {}", port))?,
                ),
            ));
        }
    }

    Ok((trimmed.to_string(), None))
}

/// 将旧 `address` 语义转换为 sing-box 1.12+ 的 DNS 服务器新格式。
///
/// 说明：
/// - 当 `address` 是域名且 `resolver_tag` 非空时，会自动写入 `domain_resolver`（对象格式）。
/// - 仅转换本项目会用到的常见类型：udp/tcp/tls/https/h3/quic/local/dhcp。
pub(crate) fn build_dns_server_config(
    tag: &str,
    address: &str,
    strategy: Option<&str>,
    detour: Option<&str>,
    resolver_tag: Option<&str>,
) -> Result<DnsServerConfig, String> {
    let raw = address.trim();
    if raw.is_empty() {
        return Err(format!("DNS 服务器地址为空: tag={}", tag));
    }

    // 新版 sing-box 中，DNS server 默认就是 direct dial。
    // 显式设置 detour=direct 会触发 "detour to an empty direct outbound makes no sense"。
    let normalized_detour = detour.and_then(|d| {
        let trimmed = d.trim();
        if trimmed.is_empty() || trimmed.eq_ignore_ascii_case(TAG_DIRECT) {
            None
        } else {
            Some(trimmed.to_string())
        }
    });

    if raw.eq_ignore_ascii_case("local") {
        return Ok(DnsServerConfig {
            tag: tag.to_string(),
            server_type: Some("local".to_string()),
            server: None,
            server_port: None,
            path: None,
            interface: None,
            inet4_range: None,
            inet6_range: None,
            domain_resolver: None,
            detour: None,
        });
    }

    let mut server_type = "udp".to_string();
    let mut server: Option<String> = None;
    let mut server_port: Option<u16> = None;
    let mut path: Option<String> = None;
    let mut interface: Option<String> = None;

    if raw.contains("://") {
        if raw.starts_with("dhcp://") {
            server_type = "dhcp".to_string();
            let value = raw.trim_start_matches("dhcp://").trim();
            if !value.is_empty() && !value.eq_ignore_ascii_case("auto") {
                interface = Some(value.to_string());
            }
        } else {
            let url = Url::parse(raw).map_err(|e| format!("无效的 DNS 地址: {} ({})", raw, e))?;
            server_type = match url.scheme() {
                "https" => "https".to_string(),
                "h3" => "h3".to_string(),
                "quic" => "quic".to_string(),
                "tls" => "tls".to_string(),
                "tcp" => "tcp".to_string(),
                "udp" => "udp".to_string(),
                unknown => {
                    return Err(format!(
                        "不支持的 DNS 协议: {} (tag={}, address={})",
                        unknown, tag, raw
                    ))
                }
            };

            let host = url
                .host_str()
                .ok_or_else(|| format!("DNS 地址缺少主机: {}", raw))?;
            server = Some(host.to_string());
            server_port = Some(url.port().unwrap_or(default_dns_server_port(&server_type)));

            if matches!(server_type.as_str(), "https" | "h3") {
                let mut p = url.path().to_string();
                if p.is_empty() || p == "/" {
                    p = "/dns-query".to_string();
                }
                path = Some(p);
            }
        }
    } else {
        let (host, port) = parse_legacy_host_port(raw)?;
        server = Some(host);
        server_port = Some(port.unwrap_or(53));
    }

    let mut domain_resolver = None;
    if let (Some(host), Some(resolver)) = (server.as_deref(), resolver_tag) {
        if !resolver.is_empty() && is_domain(host) && tag != resolver {
            domain_resolver = Some(match strategy {
                Some(s) if !s.trim().is_empty() => json!({
                    "server": resolver,
                    "strategy": s
                }),
                _ => json!({
                    "server": resolver
                }),
            });
        }
    }

    Ok(DnsServerConfig {
        tag: tag.to_string(),
        server_type: Some(server_type),
        server,
        server_port,
        path,
        interface,
        inet4_range: None,
        inet6_range: None,
        domain_resolver,
        detour: normalized_detour,
    })
}

#[cfg(test)]
mod tests {
    use super::build_dns_server_config;

    #[test]
    fn should_convert_https_legacy_address_to_new_dns_server() {
        let server = build_dns_server_config(
            "dns_proxy",
            "https://1.1.1.1/dns-query",
            Some("ipv4_only"),
            Some("manual"),
            Some("dns_resolver"),
        )
        .expect("https 地址应可转换");

        assert_eq!(server.server_type.as_deref(), Some("https"));
        assert_eq!(server.server.as_deref(), Some("1.1.1.1"));
        assert_eq!(server.server_port, Some(443));
        assert_eq!(server.path.as_deref(), Some("/dns-query"));
    }

    #[test]
    fn should_set_domain_resolver_for_domain_server() {
        let server = build_dns_server_config(
            "dns_proxy",
            "https://dns.google/dns-query",
            Some("prefer_ipv4"),
            Some("manual"),
            Some("dns_resolver"),
        )
        .expect("域名 DNS 地址应可转换");

        assert_eq!(
            server
                .domain_resolver
                .as_ref()
                .and_then(|v| v.get("server"))
                .and_then(|v| v.as_str()),
            Some("dns_resolver")
        );
        assert_eq!(
            server
                .domain_resolver
                .as_ref()
                .and_then(|v| v.get("strategy"))
                .and_then(|v| v.as_str()),
            Some("prefer_ipv4")
        );
    }

    #[test]
    fn should_omit_direct_detour_for_dns_server() {
        let server = build_dns_server_config(
            "dns_resolver",
            "223.5.5.5",
            Some("prefer_ipv4"),
            Some("direct"),
            None,
        )
        .expect("dns_resolver 构建应成功");

        assert!(server.detour.is_none());
    }
}
