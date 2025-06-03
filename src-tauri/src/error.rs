use serde::{Deserialize, Serialize};
use std::fmt;

/// 应用自定义错误类型
#[derive(Debug, Serialize, Deserialize)]
pub enum AppError {
    /// 内核相关错误
    Kernel(KernelError),
    /// 网络相关错误
    Network(NetworkError),
    /// 配置相关错误
    Config(ConfigError),
    /// 系统相关错误
    System(SystemError),
    /// 通用错误
    General(String),
}

/// 内核错误类型
#[derive(Debug, Serialize, Deserialize)]
pub enum KernelError {
    /// 启动失败
    StartFailed(String),
    /// 停止失败
    StopFailed(String),
    /// 版本检查失败
    VersionCheckFailed(String),
    /// 内核未找到
    NotFound,
    /// API服务不可用
    ApiUnavailable,
}

/// 网络错误类型
#[derive(Debug, Serialize, Deserialize)]
pub enum NetworkError {
    /// HTTP请求失败
    HttpRequestFailed(String),
    /// 下载失败
    DownloadFailed(String),
    /// 连接超时
    Timeout,
    /// 订阅处理失败
    SubscriptionFailed(String),
}

/// 配置错误类型
#[derive(Debug, Serialize, Deserialize)]
pub enum ConfigError {
    /// 配置文件不存在
    FileNotFound(String),
    /// 配置解析失败
    ParseFailed(String),
    /// 配置验证失败
    ValidationFailed(String),
    /// 配置保存失败
    SaveFailed(String),
}

/// 系统错误类型
#[derive(Debug, Serialize, Deserialize)]
pub enum SystemError {
    /// 权限不足
    PermissionDenied,
    /// 进程操作失败
    ProcessFailed(String),
    /// 注册表操作失败
    RegistryFailed(String),
    /// 文件系统错误
    FileSystemError(String),
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AppError::Kernel(e) => write!(f, "内核错误: {}", e),
            AppError::Network(e) => write!(f, "网络错误: {}", e),
            AppError::Config(e) => write!(f, "配置错误: {}", e),
            AppError::System(e) => write!(f, "系统错误: {}", e),
            AppError::General(msg) => write!(f, "通用错误: {}", msg),
        }
    }
}

impl fmt::Display for KernelError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            KernelError::StartFailed(msg) => write!(f, "内核启动失败: {}", msg),
            KernelError::StopFailed(msg) => write!(f, "内核停止失败: {}", msg),
            KernelError::VersionCheckFailed(msg) => write!(f, "版本检查失败: {}", msg),
            KernelError::NotFound => write!(f, "内核文件未找到"),
            KernelError::ApiUnavailable => write!(f, "内核API服务不可用"),
        }
    }
}

impl fmt::Display for NetworkError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NetworkError::HttpRequestFailed(msg) => write!(f, "HTTP请求失败: {}", msg),
            NetworkError::DownloadFailed(msg) => write!(f, "下载失败: {}", msg),
            NetworkError::Timeout => write!(f, "连接超时"),
            NetworkError::SubscriptionFailed(msg) => write!(f, "订阅处理失败: {}", msg),
        }
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ConfigError::FileNotFound(path) => write!(f, "配置文件未找到: {}", path),
            ConfigError::ParseFailed(msg) => write!(f, "配置解析失败: {}", msg),
            ConfigError::ValidationFailed(msg) => write!(f, "配置验证失败: {}", msg),
            ConfigError::SaveFailed(msg) => write!(f, "配置保存失败: {}", msg),
        }
    }
}

impl fmt::Display for SystemError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SystemError::PermissionDenied => write!(f, "权限不足"),
            SystemError::ProcessFailed(msg) => write!(f, "进程操作失败: {}", msg),
            SystemError::RegistryFailed(msg) => write!(f, "注册表操作失败: {}", msg),
            SystemError::FileSystemError(msg) => write!(f, "文件系统错误: {}", msg),
        }
    }
}

impl std::error::Error for AppError {}
impl std::error::Error for KernelError {}
impl std::error::Error for NetworkError {}
impl std::error::Error for ConfigError {}
impl std::error::Error for SystemError {}

// 从各种标准错误类型转换
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::System(SystemError::FileSystemError(error.to_string()))
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        AppError::Network(NetworkError::HttpRequestFailed(error.to_string()))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::Config(ConfigError::ParseFailed(error.to_string()))
    }
}

// 便捷的结果类型别名
pub type AppResult<T> = Result<T, AppError>;

// 便捷的错误创建函数
impl AppError {
    pub fn kernel_start_failed(msg: impl Into<String>) -> Self {
        AppError::Kernel(KernelError::StartFailed(msg.into()))
    }

    pub fn network_timeout() -> Self {
        AppError::Network(NetworkError::Timeout)
    }

    pub fn config_not_found(path: impl Into<String>) -> Self {
        AppError::Config(ConfigError::FileNotFound(path.into()))
    }

    pub fn permission_denied() -> Self {
        AppError::System(SystemError::PermissionDenied)
    }

    pub fn general(msg: impl Into<String>) -> Self {
        AppError::General(msg.into())
    }
}

// Tauri命令错误转换
impl From<AppError> for String {
    fn from(error: AppError) -> Self {
        error.to_string()
    }
} 