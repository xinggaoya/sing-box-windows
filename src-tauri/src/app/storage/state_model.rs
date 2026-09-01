use crate::utils::proxy_util::DEFAULT_BYPASS_LIST;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/AppConfig.ts")]
pub struct AppConfig {
    pub auto_start_kernel: bool,
    pub auto_start_app: bool,
    pub auto_hide_to_tray_on_autostart: bool,
    pub tray_close_behavior: String,
    pub prefer_ipv6: bool,
    pub allow_lan_access: bool,
    pub proxy_port: u16,
    pub api_port: u16,
    pub proxy_mode: String,
    pub system_proxy_enabled: bool,
    pub tun_enabled: bool,
    pub tray_instance_id: Option<String>,
    pub system_proxy_bypass: String,
    pub tun_auto_route: bool,
    pub tun_strict_route: bool,
    pub tun_mtu: u16,
    pub tun_ipv4: String,
    pub tun_ipv6: String,
    pub tun_stack: String,
    pub tun_enable_ipv6: bool,
    pub tun_route_exclude_address: Option<Vec<String>>,
    pub active_config_path: Option<String>,
    pub installed_kernel_version: Option<String>,

    // --- sing-box 配置生成（订阅模板）高级选项 ---
    // 说明：这些字段仅影响“本程序生成的订阅配置”，不会强行覆盖用户导入的原始订阅配置结构。
    pub singbox_dns_proxy: String,
    pub singbox_dns_cn: String,
    pub singbox_dns_resolver: String,
    pub singbox_urltest_url: String,
    /// 非国内流量默认走的出站：manual/auto
    pub singbox_default_proxy_outbound: String,
    /// 是否启用广告拦截（基于 geosite-category-ads-all）
    pub singbox_block_ads: bool,
    /// 规则集/Clash UI 下载走的出站：manual/direct
    pub singbox_download_detour: String,
    /// 是否启用 DNS 劫持（hijack-dns）
    pub singbox_dns_hijack: bool,
    /// 是否启用 Fake DNS（fakeip）
    pub singbox_fake_dns_enabled: bool,
    /// Fake DNS IPv4 地址池（CIDR）
    pub singbox_fake_dns_ipv4_range: String,
    /// Fake DNS IPv6 地址池（CIDR）
    pub singbox_fake_dns_ipv6_range: String,
    /// Fake DNS 生效范围：proxy_only/global_non_cn
    pub singbox_fake_dns_filter_mode: String,
    /// 是否启用 Telegram/YouTube/Netflix/OpenAI 分流组
    pub singbox_enable_app_groups: bool,
    /// 是否启用 TUN 连通性自愈
    pub tun_self_heal_enabled: bool,
    /// TUN 自愈冷却时间（秒）
    pub tun_self_heal_cooldown_secs: u16,

    // === sing-box 1.14 升级新增字段 ===

    /// 1.14 内核下载/升级通道：stable / oldstable / beta / testing
    /// 默认 stable；oldstable 适合老配置兼容性回退
    pub kernel_update_track: String,
    /// 1.14 DNS 乐观缓存（optimistic）：重复查询命中过期缓存立即返回 + 后台刷新
    pub singbox_dns_optimistic_cache: bool,
    /// 1.14 per-server / per-query DNS 超时（如 "5s"）；空字符串表示用内核默认
    pub singbox_dns_timeout: String,
    /// 1.14 mDNS server（*.local / link-local）；默认开
    pub singbox_dns_use_mdns: bool,
    /// 1.14 启用 `tls.spoof`（SNI 诱骗，抗 SNI 过滤）
    /// **仅 Windows x64/x86 + Admin**，ARM64 / Linux / macOS 需自动隐藏
    pub singbox_enable_tls_spoof: bool,
    /// 1.14 TUN `dns_mode`：hijack（默认）/ tun / off
    /// hijack 会改平台 DNS，1.14 起为默认行为
    pub tun_dns_mode: String,
    /// 1.14 TUN `include_mac_address`：仅代理指定网卡的流量
    pub tun_include_macs: Vec<String>,
    /// 1.14 TUN `exclude_mac_address`：排除指定网卡
    pub tun_exclude_macs: Vec<String>,
    /// 1.14 Hysteria2 `disable_chrome_parrot`：服务器用 Ed25519 证书时必须开
    pub hysteria2_disable_chrome_parrot: bool,
    /// 1.14 Hysteria2 obfs 类型：salamander（默认）/ gecko
    pub hysteria2_obfs_type: String,
    /// 1.14 Clash Mode 快捷切换持久化：rule / global / direct
    pub clash_mode: String,
    /// 1.14 启用 `type: api` 的 `dashboard.enabled`（暴露 sing-box-dashboard 到 listen_port）
    /// 默认 false（继续用本项目 Vue UI）
    pub enable_web_dashboard: bool,
    /// 1.14 启用 Tailscale endpoint（实验性，需要 Tailscale auth key）
    pub enable_tailscale_endpoint: bool,
    /// Tailscale 节点 SSH server（tailnet:22）
    pub tailscale_run_ssh_server: bool,
    /// Tailscale Taildrop 收件箱目录（默认 "Taildrop"）
    pub tailscale_taildrop_directory: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_start_kernel: true,
            auto_start_app: false,
            auto_hide_to_tray_on_autostart: true,
            tray_close_behavior: "hide".to_string(),
            prefer_ipv6: false,
            allow_lan_access: false,
            proxy_port: 12080,
            api_port: 12081,
            proxy_mode: "manual".to_string(),
            system_proxy_enabled: false,
            tun_enabled: false,
            tray_instance_id: None,
            system_proxy_bypass: DEFAULT_BYPASS_LIST.to_string(),
            tun_auto_route: true,
            tun_strict_route: true,
            tun_mtu: 1500,
            tun_ipv4: "172.19.0.1/30".to_string(),
            tun_ipv6: "fdfe:dcba:9876::1/126".to_string(),
            tun_stack: "mixed".to_string(),
            // 新安装默认关闭：避免首次安装即启用 IPv6 TUN 造成意外行为
            tun_enable_ipv6: false,
            tun_route_exclude_address: None,
            active_config_path: None,
            installed_kernel_version: None,

            // sing-box 配置生成高级选项默认值：
            // - 适配国内网络：国内域名直连，其他走代理；DNS 使用 CN/Proxy 双路分流
            singbox_dns_proxy: "https://1.1.1.1/dns-query".to_string(),
            singbox_dns_cn: "h3://dns.alidns.com/dns-query".to_string(),
            singbox_dns_resolver: "114.114.114.114".to_string(),
            singbox_urltest_url: "http://cp.cloudflare.com/generate_204".to_string(),
            singbox_default_proxy_outbound: "manual".to_string(),
            singbox_block_ads: true,
            // gh-proxy 已经做加速，默认走直连下载，避免额外经过代理
            singbox_download_detour: "direct".to_string(),
            singbox_dns_hijack: true,
            singbox_fake_dns_enabled: false,
            singbox_fake_dns_ipv4_range: "198.18.0.0/15".to_string(),
            singbox_fake_dns_ipv6_range: "fc00::/18".to_string(),
            singbox_fake_dns_filter_mode: "proxy_only".to_string(),
            singbox_enable_app_groups: true,
            tun_self_heal_enabled: true,
            tun_self_heal_cooldown_secs: 90,

            // sing-box 1.14 升级默认值
            kernel_update_track: "stable".to_string(),
            singbox_dns_optimistic_cache: true,
            singbox_dns_timeout: "5s".to_string(),
            singbox_dns_use_mdns: true,
            // TLS spoof 涉及抓包权限，默认关；UI 在 Windows x64/x86 + Admin 上才显示
            singbox_enable_tls_spoof: false,
            tun_dns_mode: "hijack".to_string(),
            tun_include_macs: Vec::new(),
            tun_exclude_macs: Vec::new(),
            hysteria2_disable_chrome_parrot: false,
            hysteria2_obfs_type: "salamander".to_string(),
            clash_mode: "rule".to_string(),
            enable_web_dashboard: false,
            enable_tailscale_endpoint: false,
            tailscale_run_ssh_server: false,
            tailscale_taildrop_directory: "Taildrop".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default)]
pub struct StartupPreferences {
    pub auto_start_app: bool,
    pub auto_hide_to_tray_on_autostart: bool,
    pub tray_close_behavior: String,
}

impl Default for StartupPreferences {
    fn default() -> Self {
        Self {
            auto_start_app: false,
            auto_hide_to_tray_on_autostart: true,
            tray_close_behavior: "hide".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/ThemeConfig.ts")]
pub struct ThemeConfig {
    pub is_dark: bool,
    pub mode: String,
    pub accent_color: String,
    pub compact_mode: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self {
            is_dark: true,
            mode: "system".to_string(),
            accent_color: "#6366f1".to_string(),
            compact_mode: false,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/LocaleConfig.ts")]
pub struct LocaleConfig {
    pub locale: String,
}

impl Default for LocaleConfig {
    fn default() -> Self {
        Self {
            locale: "zh-CN".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/WindowConfig.ts")]
pub struct WindowConfig {
    pub is_maximized: bool,
    pub width: u32,
    pub height: u32,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            is_maximized: false,
            width: 1000,
            height: 700,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/UpdateConfig.ts")]
pub struct UpdateConfig {
    pub auto_check: bool,
    pub last_check: i64,
    pub last_version: Option<String>,
    pub skip_version: Option<String>,
    pub accept_prerelease: bool,
    pub update_channel: Option<String>,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_check: true,
            last_check: 0,
            last_version: None,
            skip_version: None,
            accept_prerelease: false,
            update_channel: Some("stable".to_string()),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/Subscription.ts")]
pub struct Subscription {
    pub name: String,
    pub url: String,
    pub is_loading: bool,
    pub last_update: Option<u64>,
    pub is_manual: bool,
    pub manual_content: Option<String>,
    pub use_original_config: bool,
    pub config_path: Option<String>,
    pub backup_path: Option<String>,
    pub auto_update_interval_minutes: Option<u64>,
    pub subscription_upload: Option<u64>,
    pub subscription_download: Option<u64>,
    pub subscription_total: Option<u64>,
    pub subscription_expire: Option<u64>,
    // 自动更新健康状态（用于失败退避与可观测性）
    pub auto_update_fail_count: Option<u32>,
    pub last_auto_update_attempt: Option<u64>,
    pub last_auto_update_error: Option<String>,
    pub last_auto_update_error_type: Option<String>,
    pub last_auto_update_backoff_until: Option<u64>,
}
