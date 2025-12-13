use crate::utils::proxy_util::DEFAULT_BYPASS_LIST;
use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/AppConfig.ts")]
pub struct AppConfig {
    pub auto_start_kernel: bool,
    pub auto_start_app: bool,
    pub prefer_ipv6: bool,
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
    pub active_config_path: Option<String>,
    pub installed_kernel_version: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_start_kernel: true,
            auto_start_app: false,
            prefer_ipv6: false,
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
            active_config_path: None,
            installed_kernel_version: None,
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
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_check: true,
            last_check: 0,
            last_version: None,
            skip_version: None,
            accept_prerelease: false,
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
}
