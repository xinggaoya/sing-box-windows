use serde::{Deserialize, Serialize};

/// 应用状态数据结构
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppState {
    pub app_config: AppConfig,
    pub theme_config: ThemeConfig,
    pub locale_config: LocaleConfig,
    pub window_config: WindowConfig,
    pub update_config: UpdateConfig,
    pub subscriptions: Vec<Subscription>,
    pub kernel_info: KernelInfo,
}

/// 应用配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub auto_start_kernel: bool,
    pub prefer_ipv6: bool,
    pub proxy_port: u16,
    pub api_port: u16,
    pub proxy_mode: ProxyMode,
    pub tray_instance_id: Option<String>,
}

/// 代理模式
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProxyMode {
    System,
    Tun,
    Manual,
}

impl Default for ProxyMode {
    fn default() -> Self {
        ProxyMode::System
    }
}

/// 主题配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThemeConfig {
    pub is_dark: bool,
}

impl Default for ThemeConfig {
    fn default() -> Self {
        Self { is_dark: true }
    }
}

/// 语言配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocaleConfig {
    pub locale: Locale,
}

/// 支持的语言
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Locale {
    Auto,
    ZhCN,
    EnUS,
    RuRU,
    JaJP,
}

impl Default for Locale {
    fn default() -> Self {
        Locale::Auto
    }
}

/// 窗口配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WindowConfig {
    pub is_visible: bool,
    pub is_fullscreen: bool,
    pub is_maximized: bool,
    pub last_visible_path: String,
}

impl Default for WindowConfig {
    fn default() -> Self {
        Self {
            is_visible: true,
            is_fullscreen: false,
            is_maximized: false,
            last_visible_path: "/home".to_string(),
        }
    }
}

/// 更新配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateConfig {
    pub app_version: String,
    pub auto_check_update: bool,
    pub skip_version: Option<String>,
    pub accept_prerelease: bool,
}

impl Default for UpdateConfig {
    fn default() -> Self {
        Self {
            app_version: "0.0.0".to_string(),
            auto_check_update: true,
            skip_version: None,
            accept_prerelease: false,
        }
    }
}

/// 订阅信息
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

/// 内核信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelInfo {
    pub version: Option<VersionInfo>,
    pub new_version: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionInfo {
    pub version: String,
    pub meta: bool,
    pub premium: bool,
    pub environment: Option<String>,
    pub tags: Option<Vec<String>>,
    pub revision: Option<String>,
    pub cgo: Option<String>,
}

/// 数据库操作的错误类型
#[derive(Debug, thiserror::Error)]
pub enum StorageError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
    
    #[error("Database error: {0}")]
    Database(String),
}

pub type StorageResult<T> = Result<T, StorageError>;