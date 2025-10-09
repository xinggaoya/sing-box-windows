use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub auto_start_kernel: bool,
    pub prefer_ipv6: bool,
    pub proxy_port: u16,
    pub api_port: u16,
    pub proxy_mode: String,
    pub tray_instance_id: Option<String>,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            auto_start_kernel: false,
            prefer_ipv6: false,
            proxy_port: 12080,
            api_port: 12081,
            proxy_mode: "manual".to_string(),
            tray_instance_id: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub is_dark: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self { is_dark: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub auto_check: bool,
    pub last_check: i64,
    pub last_version: Option<String>,
    pub skip_version: Option<String>,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            auto_check: true,
            last_check: 0,
            last_version: None,
            skip_version: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Subscription {
    pub name: String,
    pub url: String,
    pub is_loading: bool,
    pub last_update: Option<u64>,
    pub is_manual: bool,
    pub manual_content: Option<String>,
    pub use_original_config: bool,
}