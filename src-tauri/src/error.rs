use serde::{Deserialize, Serialize};
use std::fmt;

/// 应用错误类型
#[derive(Debug, Serialize, Deserialize)]
pub struct AppError {
    pub code: ErrorCode,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub details: Option<String>,
}

/// 错误代码枚举
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ErrorCode {
    // 内核相关错误
    KernelNotFound,
    KernelStartFailed,
    KernelStopFailed,
    KernelAlreadyRunning,
    KernelNotRunning,

    // 配置相关错误
    ConfigNotFound,
    ConfigInvalid,
    ConfigParseFailed,

    // 网络相关错误
    NetworkRequestFailed,
    NetworkTimeout,
    WebSocketConnectionFailed,

    // 权限相关错误
    PermissionDenied,
    AdminRequired,

    // 系统相关错误
    SystemError,
    FileOperationFailed,
    ProcessOperationFailed,

    // 其他错误
    Unknown,
}

impl AppError {
    /// 创建新的应用错误
    pub fn new(code: ErrorCode, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            details: None,
        }
    }

    /// 添加错误详情
    pub fn with_details(mut self, details: impl Into<String>) -> Self {
        self.details = Some(details.into());
        self
    }
}

impl fmt::Display for AppError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{:?}] {}", self.code, self.message)?;
        if let Some(ref details) = self.details {
            write!(f, " - {}", details)?;
        }
        Ok(())
    }
}

impl std::error::Error for AppError {}

/// 从标准错误类型转换
impl From<std::io::Error> for AppError {
    fn from(error: std::io::Error) -> Self {
        AppError::new(ErrorCode::SystemError, format!("IO错误: {}", error))
    }
}

impl From<serde_json::Error> for AppError {
    fn from(error: serde_json::Error) -> Self {
        AppError::new(
            ErrorCode::ConfigParseFailed,
            format!("JSON解析错误: {}", error),
        )
    }
}

impl From<reqwest::Error> for AppError {
    fn from(error: reqwest::Error) -> Self {
        AppError::new(
            ErrorCode::NetworkRequestFailed,
            format!("网络请求失败: {}", error),
        )
    }
}

/// 为Tauri命令提供的Result类型别名
pub type AppResult<T> = Result<T, AppError>;

/// 快捷创建错误的宏
#[macro_export]
macro_rules! app_error {
    ($code:expr, $msg:expr) => {
        $crate::error::AppError::new($code, $msg)
    };
    ($code:expr, $msg:expr, $details:expr) => {
        $crate::error::AppError::new($code, $msg).with_details($details)
    };
}
