use std::fmt;
use thiserror::Error;

#[derive(Debug, Error, Clone)]
pub enum ProcessError {
    #[error("进程已在运行中")]
    AlreadyRunning,

    #[error("进程未运行")]
    NotRunning,

    #[error("启动失败: {0}")]
    StartFailed(String),

    #[error("停止失败: {0}")]
    StopFailed(String),

    #[error("进程状态检查失败: {0}")]
    StatusCheckFailed(String),

    #[error("配置错误: {0}")]
    ConfigError(String),

    #[error("系统错误: {0}")]
    SystemError(String),

    #[error("权限错误: {0}")]
    PermissionError(String),

    #[error("网络错误: {0}")]
    NetworkError(String),

    #[error("未知错误: {0}")]
    Unknown(String),
}

#[derive(Debug, Clone)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed(String),
}

impl fmt::Display for ProcessStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ProcessStatus::Starting => write!(f, "正在启动"),
            ProcessStatus::Running => write!(f, "运行中"),
            ProcessStatus::Stopping => write!(f, "正在停止"),
            ProcessStatus::Stopped => write!(f, "已停止"),
            ProcessStatus::Failed(err) => write!(f, "失败: {}", err),
        }
    }
}

pub type Result<T> = std::result::Result<T, ProcessError>;