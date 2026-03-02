use serde::{Deserialize, Serialize};

pub const TRAY_ICON_ID: &str = "main_tray";

pub mod menu_ids {
    pub const SHOW_WINDOW: &str = "tray_show_window";
    pub const KERNEL_SUBMENU: &str = "tray_kernel_submenu";
    pub const KERNEL_STATUS: &str = "tray_kernel_status";
    pub const KERNEL_RESTART: &str = "tray_kernel_restart";
    pub const PROXY_SUBMENU: &str = "tray_proxy_submenu";
    pub const PROXY_CURRENT: &str = "tray_proxy_current";
    pub const PROXY_SYSTEM: &str = "tray_proxy_system";
    pub const PROXY_TUN: &str = "tray_proxy_tun";
    pub const PROXY_MANUAL: &str = "tray_proxy_manual";
    pub const QUIT: &str = "tray_quit";
}

pub mod events {
    pub const ACTION_SHOW_WINDOW: &str = "tray-action-show-window";
    pub const ACTION_HIDE_WINDOW: &str = "tray-action-hide-window";
    pub const ACTION_NAVIGATE_LAST_ROUTE: &str = "tray-action-navigate-last-route";
    pub const ACTION_EXIT_REQUESTED: &str = "tray-action-exit-requested";
    pub const ACTION_RESTART_KERNEL: &str = "tray-action-restart-kernel";
    pub const ACTION_SWITCH_PROXY_MODE: &str = "tray-action-switch-proxy-mode";
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum TrayProxyMode {
    System,
    Tun,
    #[default]
    Manual,
}

impl TrayProxyMode {
    pub fn from_raw(value: &str) -> Self {
        match value {
            "system" => Self::System,
            "tun" => Self::Tun,
            _ => Self::Manual,
        }
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            Self::System => "system",
            Self::Tun => "tun",
            Self::Manual => "manual",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(default, rename_all = "camelCase")]
pub struct TrayRuntimeStateInput {
    pub kernel_running: bool,
    pub proxy_mode: String,
    pub active_subscription_name: Option<String>,
    pub locale: String,
    pub window_visible: bool,
}

impl Default for TrayRuntimeStateInput {
    fn default() -> Self {
        Self {
            kernel_running: false,
            proxy_mode: "manual".to_string(),
            active_subscription_name: None,
            locale: "en-US".to_string(),
            window_visible: true,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TrayNavigatePayload {
    pub path: String,
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct TraySwitchProxyModePayload {
    pub mode: String,
}
