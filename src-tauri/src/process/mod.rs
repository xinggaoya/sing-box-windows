use std::fmt;

pub mod manager;

// 进程错误类型
#[derive(Debug, Clone)]
pub enum ProcessError {
    AlreadyRunning,
    NotRunning,
    StartFailed(String),
    StopFailed(String),
    StatusCheckFailed(String),
    ConfigError(String),
    SystemError(String),
    PermissionError(String),
    NetworkError(String),
    Unknown(String),
    Other(String),
}

impl From<std::io::Error> for ProcessError {
    fn from(err: std::io::Error) -> Self {
        ProcessError::SystemError(err.to_string())
    }
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessError::AlreadyRunning => write!(f, "进程已在运行"),
            ProcessError::NotRunning => write!(f, "进程未运行"),
            ProcessError::StartFailed(msg) => write!(f, "启动失败: {}", msg),
            ProcessError::StopFailed(msg) => write!(f, "停止失败: {}", msg),
            ProcessError::StatusCheckFailed(msg) => write!(f, "进程状态检查失败: {}", msg),
            ProcessError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
            ProcessError::SystemError(msg) => write!(f, "系统错误: {}", msg),
            ProcessError::PermissionError(msg) => write!(f, "权限错误: {}", msg),
            ProcessError::NetworkError(msg) => write!(f, "网络错误: {}", msg),
            ProcessError::Unknown(msg) => write!(f, "未知错误: {}", msg),
            ProcessError::Other(msg) => write!(f, "其他错误: {}", msg),
        }
    }
}

impl std::error::Error for ProcessError {}

pub type Result<T> = std::result::Result<T, ProcessError>;
