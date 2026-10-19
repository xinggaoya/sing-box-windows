//! 配置意图（ConfigIntent）：`AppConfig` → 配置语义 的**唯一映射产物**。
//!
//! ## 为什么需要它
//!
//! 此前配置语义散落在两处：
//!
//! - `config_generator.rs::generate_base_config` —— 从零生成一份配置
//! - `settings_patch.rs::apply_app_settings_to_config` —— 在已有配置上按 tag 打补丁
//!
//! 两者都从 `AppConfig` 读同一批字段、都调用同一批 `normalize_*` 归一化函数
//! （`normalize_default_outbound` / `dns_strategy` / `dns_bootstrap_strategy` /
//! `normalize_fakeip_range` / `normalize_dns_mode` / `normalize_fake_dns_filter_mode` …）。
//! 结果是**加一个设置项要改两处**，漏改任何一处都不会报错——
//! 只会让「生成的配置」与「补丁后的配置」悄悄分叉，用户拿到一份与设置项不匹配的配置。
//!
//! `ConfigIntent` 把「AppConfig 翻译成配置语义」收敛到一个**纯函数**
//! [`ConfigIntent::from_app_config`]：两条路径都只消费 intent，语义只有一份。
//!
//! ## 使用约定
//!
//! - intent 只承载**配置语义**，不携带 IO、不读取全局状态。
//! - 需要读取"已有配置"才能决定的语义（如 TUN 路由排除网段的兜底），
//!   在 intent 里保留 `Option`（用户显式设置），由使用方按场景回退。

use super::common::{
    dns_bootstrap_strategy, dns_strategy, node_domain_resolver_strategy,
    normalize_default_outbound, normalize_dns_mode, normalize_download_detour,
    normalize_fake_dns_filter_mode, FAKE_DNS_FILTER_GLOBAL_NON_CN,
};
use crate::app::storage::state_model::AppConfig;

/// 假 DNS（FakeIP）语义
///
/// ⚠️ 地址段与过滤模式**始终**按用户配置归一化，与是否启用无关：
/// 关闭假 DNS 时，补丁路径仍需要用这些值去清理上一份配置里遗留的 fakeip 规则。
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FakeDnsIntent {
    /// 是否启用假 DNS
    pub enabled: bool,
    /// IPv4 地址段（已归一化，非空）
    pub ipv4_range: String,
    /// IPv6 地址段（已归一化，非空）
    pub ipv6_range: String,
    /// 过滤模式：`proxy_only` / `global_non_cn`
    pub filter_mode: &'static str,
}

impl FakeDnsIntent {
    /// 是否为"非国内流量全部走 fakeip"模式
    pub fn is_global_non_cn(&self) -> bool {
        self.filter_mode == FAKE_DNS_FILTER_GLOBAL_NON_CN
    }
}

/// TUN 入站语义
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TunIntent {
    pub enabled: bool,
    pub ipv4: String,
    pub ipv6: String,
    pub enable_ipv6: bool,
    pub mtu: u16,
    pub auto_route: bool,
    pub strict_route: bool,
    /// `system` / `gvisor` / `mixed`
    pub stack: String,
    /// 1.14 `dns_mode`：`hijack` / `native` / `disabled`（已归一化）
    pub dns_mode: &'static str,
    /// 用户显式设置的路由排除网段；`None` 表示沿用配置里已有的值或平台默认值
    pub route_exclude_address: Option<Vec<String>>,
}

impl TunIntent {
    /// TUN 地址列表（IPv6 仅在开启时加入）
    pub fn addresses(&self) -> Vec<String> {
        let mut addresses = vec![self.ipv4.clone()];
        if self.enable_ipv6 {
            addresses.push(self.ipv6.clone());
        }
        addresses
    }
}

/// Tailscale endpoint（1.14）语义
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TailscaleIntent {
    pub run_ssh_server: bool,
    /// Taildrop 收件箱目录；空串表示不写入该字段
    pub taildrop_directory: String,
}

/// 配置语义全集
#[derive(Debug, Clone, PartialEq)]
pub struct ConfigIntent {
    // ---- 入口 ----
    pub proxy_port: u16,
    /// 代理入站监听地址（局域网访问开启时为 0.0.0.0）
    pub proxy_listen: &'static str,
    pub system_proxy_enabled: bool,
    pub tun: TunIntent,

    // ---- 1.14 gRPC API 服务 ----
    pub api_port: u16,
    pub web_dashboard: bool,

    // ---- 出站 ----
    /// 非国内流量的默认出站（`manual` / `auto`）
    pub default_outbound: &'static str,
    /// 规则集下载出站
    pub download_detour: &'static str,
    /// urltest 组的测速地址
    pub urltest_url: String,
    /// 是否启用业务分流组（Telegram / YouTube / Netflix / OpenAI / Google）
    pub enable_app_groups: bool,

    // ---- DNS ----
    pub dns_proxy: String,
    pub dns_cn: String,
    pub dns_resolver: String,
    /// 出口侧域名解析策略
    pub dns_strategy: &'static str,
    /// DNS 服务器自身域名的 bootstrap 解析策略
    pub dns_bootstrap_strategy: &'static str,
    /// 节点服务器域名的解析策略
    pub node_resolver_strategy: &'static str,
    pub dns_hijack: bool,
    pub dns_optimistic: bool,
    pub dns_timeout: String,
    pub dns_use_mdns: bool,
    /// 假 DNS 语义（地址段与过滤模式始终存在，`enabled` 决定是否真正启用）
    pub fake_dns: FakeDnsIntent,

    // ---- 路由 ----
    /// 是否拦截广告（基于 geosite-category-ads-all）
    pub block_ads: bool,

    // ---- Tailscale ----
    /// `None` 表示未启用 Tailscale endpoint
    pub tailscale: Option<TailscaleIntent>,
}

impl ConfigIntent {
    /// `AppConfig` → 配置语义 的唯一映射点。
    ///
    /// 纯函数：不读全局状态、不做 IO。所有归一化（默认值、枚举白名单、地址段裁剪）
    /// 都在这里完成，调用方拿到的 intent 一定是"可直接写进配置"的形式。
    pub fn from_app_config(app_config: &AppConfig) -> Self {
        let fake_dns = FakeDnsIntent {
            enabled: app_config.singbox_fake_dns_enabled,
            ipv4_range: normalize_fakeip_range(
                &app_config.singbox_fake_dns_ipv4_range,
                "198.18.0.0/15",
            ),
            ipv6_range: normalize_fakeip_range(
                &app_config.singbox_fake_dns_ipv6_range,
                "fc00::/18",
            ),
            filter_mode: normalize_fake_dns_filter_mode(app_config),
        };

        let tailscale = if app_config.enable_tailscale_endpoint {
            Some(TailscaleIntent {
                run_ssh_server: app_config.tailscale_run_ssh_server,
                taildrop_directory: app_config.tailscale_taildrop_directory.clone(),
            })
        } else {
            None
        };

        Self {
            proxy_port: app_config.proxy_port,
            proxy_listen: if app_config.allow_lan_access {
                "0.0.0.0"
            } else {
                "127.0.0.1"
            },
            system_proxy_enabled: app_config.system_proxy_enabled,
            tun: TunIntent {
                enabled: app_config.tun_enabled,
                ipv4: app_config.tun_ipv4.clone(),
                ipv6: app_config.tun_ipv6.clone(),
                enable_ipv6: app_config.tun_enable_ipv6,
                mtu: app_config.tun_mtu,
                auto_route: app_config.tun_auto_route,
                strict_route: app_config.tun_strict_route,
                stack: app_config.tun_stack.clone(),
                dns_mode: normalize_dns_mode(&app_config.tun_dns_mode),
                route_exclude_address: app_config.tun_route_exclude_address.clone(),
            },

            api_port: app_config.api_port,
            web_dashboard: app_config.enable_web_dashboard,

            default_outbound: normalize_default_outbound(app_config),
            download_detour: normalize_download_detour(app_config),
            urltest_url: app_config.singbox_urltest_url.clone(),
            enable_app_groups: app_config.singbox_enable_app_groups,

            dns_proxy: app_config.singbox_dns_proxy.clone(),
            dns_cn: app_config.singbox_dns_cn.clone(),
            dns_resolver: app_config.singbox_dns_resolver.clone(),
            dns_strategy: dns_strategy(app_config),
            dns_bootstrap_strategy: dns_bootstrap_strategy(app_config),
            node_resolver_strategy: node_domain_resolver_strategy(app_config),
            dns_hijack: app_config.singbox_dns_hijack,
            dns_optimistic: app_config.singbox_dns_optimistic_cache,
            dns_timeout: app_config.singbox_dns_timeout.clone(),
            dns_use_mdns: app_config.singbox_dns_use_mdns,
            fake_dns,

            block_ads: app_config.singbox_block_ads,

            tailscale,
        }
    }

    /// 假 DNS 是否启用
    pub fn fake_dns_enabled(&self) -> bool {
        self.fake_dns.enabled
    }

    /// 假 DNS 的过滤模式
    pub fn fake_dns_filter_mode(&self) -> &'static str {
        self.fake_dns.filter_mode
    }
}

/// 地址段归一化：空串回退到默认值
fn normalize_fakeip_range(raw: &str, fallback: &str) -> String {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        fallback.to_string()
    } else {
        trimmed.to_string()
    }
}
