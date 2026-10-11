use crate::app::singbox::common::{normalize_dns_mode, PRIVATE_IP_CIDRS};
use crate::entity::config_model;
use serde::{Deserialize, Serialize};
use std::net::IpAddr;

/// 默认的 TUN IPv4 地址段
pub const DEFAULT_TUN_IPV4: &str = "172.19.0.1/30";
/// 默认的 TUN IPv6 地址段
pub const DEFAULT_TUN_IPV6: &str = "fdfe:dcba:9876::1/126";
/// 默认排除的 TUN 路由网段。
///
/// 与 `PRIVATE_IP_CIDRS` 共享同一份 canonical 列表，避免 TUN 默认值和直连私网规则漂移。
pub fn default_tun_route_exclude_addresses() -> Vec<String> {
    PRIVATE_IP_CIDRS
        .iter()
        .map(|cidr| cidr.to_string())
        .collect()
}

pub fn normalize_tun_route_exclude_address(
    route_exclude_address: Option<Vec<String>>,
) -> Result<Option<Vec<String>>, String> {
    let Some(route_exclude_address) = route_exclude_address else {
        return Ok(None);
    };

    let mut normalized = Vec::with_capacity(route_exclude_address.len());
    for cidr in route_exclude_address {
        let trimmed = cidr.trim();
        if trimmed.is_empty() {
            continue;
        }

        validate_cidr(trimmed)?;
        let trimmed = trimmed.to_string();
        if !normalized.contains(&trimmed) {
            normalized.push(trimmed);
        }
    }

    if normalized.is_empty() {
        Ok(None)
    } else {
        Ok(Some(normalized))
    }
}

pub fn normalize_persisted_tun_route_exclude_address(
    route_exclude_address: Option<Vec<String>>,
) -> Option<Vec<String>> {
    match normalize_tun_route_exclude_address(route_exclude_address) {
        Ok(normalized) => normalized,
        Err(error) => {
            tracing::warn!(
                "检测到无效的已持久化 tun_route_exclude_address，已回退为 None: {}",
                error
            );
            None
        }
    }
}

#[derive(Debug, Clone, Deserialize, Serialize)]
#[serde(default)]
pub struct TunProxyOptions {
    pub ipv4_address: String,
    pub ipv6_address: String,
    pub mtu: u16,
    pub auto_route: bool,
    pub strict_route: bool,
    pub stack: String,
    pub enable_ipv6: bool,
    pub route_exclude_address: Option<Vec<String>>,
    pub interface_name: Option<String>,
    /// 1.14 新增：对系统 DNS 的接管方式（hijack / native / disabled）
    pub dns_mode: String,
}

impl Default for TunProxyOptions {
    fn default() -> Self {
        Self {
            ipv4_address: DEFAULT_TUN_IPV4.to_string(),
            ipv6_address: DEFAULT_TUN_IPV6.to_string(),
            mtu: 1500,
            auto_route: true,
            strict_route: true,
            stack: "mixed".to_string(),
            enable_ipv6: true,
            route_exclude_address: None,
            interface_name: None,
            dns_mode: "hijack".to_string(),
        }
    }
}

/// 归一化后的 TUN 运行时配置，便于在多个模块之间复用
#[derive(Debug, Clone)]
pub struct TunProfile {
    pub ipv4_address: String,
    pub ipv6_address: Option<String>,
    pub auto_route: bool,
    pub strict_route: bool,
    pub stack: String,
    pub mtu: u16,
    pub interface_name: String,
    pub route_exclude_address: Vec<String>,
    pub dns_mode: String,
}

impl TunProfile {
    pub fn from_options(
        options: &TunProxyOptions,
        route_exclude_address_override: Option<&[String]>,
    ) -> Self {
        let interface_name = options
            .interface_name
            .clone()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or_else(default_interface_name);

        let ipv6_address = if options.enable_ipv6 && !options.ipv6_address.trim().is_empty() {
            Some(options.ipv6_address.clone())
        } else {
            None
        };

        Self {
            ipv4_address: if options.ipv4_address.trim().is_empty() {
                DEFAULT_TUN_IPV4.to_string()
            } else {
                options.ipv4_address.clone()
            },
            ipv6_address,
            auto_route: options.auto_route,
            strict_route: options.strict_route,
            stack: normalize_stack(&options.stack),
            mtu: options.mtu,
            interface_name,
            route_exclude_address: route_exclude_address_override
                .map(|cidrs| cidrs.to_vec())
                .or_else(|| options.route_exclude_address.clone())
                .unwrap_or_else(default_tun_route_exclude_addresses),
            dns_mode: normalize_dns_mode(&options.dns_mode).to_string(),
        }
    }

    pub fn address_list(&self) -> Vec<String> {
        let mut addresses = vec![self.ipv4_address.clone()];
        if let Some(v6) = &self.ipv6_address {
            addresses.push(v6.clone());
        }
        addresses
    }

    pub fn to_inbounds(&self, port: u16) -> Vec<config_model::Inbound> {
        vec![
            config_model::Inbound {
                r#type: "mixed".to_string(),
                tag: "mixed-in".to_string(),
                listen: Some("127.0.0.1".to_string()),
                interface_name: None,
                listen_port: Some(port),
                address: None,
                auto_route: None,
                strict_route: None,
                stack: None,
                mtu: None,
                route_address: None,
                route_exclude_address: None,
                set_system_proxy: None,
                dns_mode: None,
            },
            config_model::Inbound {
                r#type: "tun".to_string(),
                tag: "tun-in".to_string(),
                listen: None,
                interface_name: Some(self.interface_name.clone()),
                listen_port: None,
                address: Some(self.address_list()),
                auto_route: Some(self.auto_route),
                strict_route: Some(self.strict_route),
                stack: Some(self.stack.clone()),
                mtu: Some(self.mtu),
                route_address: None,
                route_exclude_address: Some(self.route_exclude_address.clone()),
                set_system_proxy: None,
                dns_mode: Some(self.dns_mode.clone()),
            },
        ]
    }
}

fn validate_cidr(value: &str) -> Result<(), String> {
    let (address, prefix) = value
        .split_once('/')
        .ok_or_else(|| format!("无效的 CIDR: {}", value))?;
    let ip = address
        .parse::<IpAddr>()
        .map_err(|_| format!("无效的 CIDR: {}", value))?;
    let prefix = prefix
        .parse::<u8>()
        .map_err(|_| format!("无效的 CIDR: {}", value))?;

    let max_prefix = match ip {
        IpAddr::V4(_) => 32,
        IpAddr::V6(_) => 128,
    };

    if prefix > max_prefix {
        return Err(format!("无效的 CIDR: {}", value));
    }

    Ok(())
}

fn normalize_stack(stack: &str) -> String {
    match stack {
        "system" | "gvisor" | "mixed" => stack.to_string(),
        _ => "mixed".to_string(),
    }
}

fn default_interface_name() -> String {
    #[cfg(target_os = "macos")]
    {
        use std::time::{SystemTime, UNIX_EPOCH};
        if let Ok(duration) = SystemTime::now().duration_since(UNIX_EPOCH) {
            let idx = (duration.subsec_millis() % 90) + 5;
            return format!("utun{}", idx);
        }
        "utun5".to_string()
    }
    #[cfg(not(target_os = "macos"))]
    {
        "singbox_tun".to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::{
        default_tun_route_exclude_addresses, normalize_persisted_tun_route_exclude_address,
        normalize_tun_route_exclude_address, TunProfile, TunProxyOptions,
    };

    #[test]
    fn normalize_tun_route_exclude_address_should_trim_and_drop_blank_entries() {
        let normalized = normalize_tun_route_exclude_address(Some(vec![
            " 10.0.0.0/8 ".to_string(),
            "".to_string(),
            "   ".to_string(),
        ]))
        .expect("route exclude addresses should normalize");

        assert_eq!(normalized, Some(vec!["10.0.0.0/8".to_string()]));
    }

    #[test]
    fn normalize_tun_route_exclude_address_should_collapse_empty_input_to_none() {
        let normalized =
            normalize_tun_route_exclude_address(Some(vec!["".to_string(), "   ".to_string()]))
                .expect("empty route exclude addresses should normalize");

        assert_eq!(normalized, None);
    }

    #[test]
    fn normalize_tun_route_exclude_address_should_reject_invalid_cidrs() {
        let error = normalize_tun_route_exclude_address(Some(vec!["not-a-cidr".to_string()]))
            .expect_err("invalid CIDR should be rejected");

        assert!(
            error.contains("not-a-cidr"),
            "error should mention invalid cidr, got: {}",
            error
        );
    }

    #[test]
    fn normalize_tun_route_exclude_address_should_deduplicate_while_preserving_order() {
        let normalized = normalize_tun_route_exclude_address(Some(vec![
            "10.0.0.0/8".to_string(),
            "10.0.0.0/8".to_string(),
            "192.168.0.0/16".to_string(),
        ]))
        .expect("duplicate route exclude addresses should normalize");

        assert_eq!(
            normalized,
            Some(vec!["10.0.0.0/8".to_string(), "192.168.0.0/16".to_string()])
        );
    }

    #[test]
    fn normalize_persisted_tun_route_exclude_address_should_drop_invalid_values() {
        assert_eq!(
            normalize_persisted_tun_route_exclude_address(Some(vec!["bad".to_string()])),
            None
        );
    }

    #[test]
    fn tun_profile_should_use_explicit_route_exclude_address_override() {
        let options = TunProxyOptions::default();
        let override_values = vec!["203.0.113.0/24".to_string()];
        let profile = TunProfile::from_options(&options, Some(&override_values));

        assert_eq!(profile.route_exclude_address, override_values);
    }

    #[test]
    fn tun_profile_should_use_options_route_exclude_address_when_override_absent() {
        let options = TunProxyOptions {
            route_exclude_address: Some(vec!["198.51.100.0/24".to_string()]),
            ..TunProxyOptions::default()
        };
        let profile = TunProfile::from_options(&options, None);

        assert_eq!(
            profile.route_exclude_address,
            vec!["198.51.100.0/24".to_string()]
        );
    }

    #[test]
    fn tun_profile_should_fallback_to_canonical_default_route_excludes() {
        let options = TunProxyOptions::default();
        let profile = TunProfile::from_options(&options, None);

        assert_eq!(
            profile.route_exclude_address,
            default_tun_route_exclude_addresses()
        );
    }
}
