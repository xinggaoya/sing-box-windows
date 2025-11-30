use crate::entity::config_model;
use serde::{Deserialize, Serialize};

/// 默认的 TUN IPv4 地址段
pub const DEFAULT_TUN_IPV4: &str = "172.19.0.1/30";
/// 默认的 TUN IPv6 地址段
pub const DEFAULT_TUN_IPV6: &str = "fdfe:dcba:9876::1/126";
/// 默认排除的本地网段
pub const TUN_ROUTE_EXCLUDES: [&str; 6] = [
    "127.0.0.1/8",
    "10.0.0.0/8",
    "172.16.0.0/12",
    "192.168.0.0/16",
    "::1/128",
    "fc00::/7",
];

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
    pub interface_name: Option<String>,
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
            interface_name: None,
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
}

impl TunProfile {
    pub fn from_options(options: &TunProxyOptions) -> Self {
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
                sniff: Some(true),
                sniff_override_destination: Some(true),
                mtu: None,
                route_address: None,
                route_exclude_address: None,
                set_system_proxy: None,
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
                sniff: Some(true),
                sniff_override_destination: Some(true),
                mtu: Some(self.mtu),
                route_address: None,
                route_exclude_address: Some(
                    TUN_ROUTE_EXCLUDES
                        .iter()
                        .map(|s| s.to_string())
                        .collect::<Vec<_>>(),
                ),
                set_system_proxy: None,
            },
        ]
    }
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
