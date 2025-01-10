use serde::{Deserialize, Serialize};
use std::fmt;

pub mod manager;

// 进程状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed(String),
}

// 进程错误类型
#[derive(Debug, Clone)]
pub enum ProcessError {
    AlreadyRunning,
    NotRunning,
    StartFailed(String),
    StopFailed(String),
    ConfigError(String),
    IoError(String),
}

impl From<std::io::Error> for ProcessError {
    fn from(err: std::io::Error) -> Self {
        ProcessError::IoError(err.to_string())
    }
}

impl fmt::Display for ProcessError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            ProcessError::AlreadyRunning => write!(f, "进程已在运行"),
            ProcessError::NotRunning => write!(f, "进程未运行"),
            ProcessError::StartFailed(msg) => write!(f, "启动失败: {}", msg),
            ProcessError::StopFailed(msg) => write!(f, "停止失败: {}", msg),
            ProcessError::ConfigError(msg) => write!(f, "配置错误: {}", msg),
            ProcessError::IoError(e) => write!(f, "IO错误: {}", e),
        }
    }
}

impl std::error::Error for ProcessError {}

// 进程信息结构体
#[derive(Debug, Clone, Serialize)]
pub struct ProcessInfo {
    pub pid: Option<u32>,
    pub status: ProcessStatus,
    pub last_error: Option<String>,
}

pub type Result<T> = std::result::Result<T, ProcessError>;

// 进程配置结构体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProcessConfig {
    pub graceful_timeout: u64,      // 优雅关闭超时时间(秒)
    pub health_check_interval: u64, // 健康检查间隔(秒)
    pub max_restart_attempts: u32,  // 最大重启尝试次数
    pub restart_delay: u64,         // 重启延迟时间(秒)
}

impl Default for ProcessConfig {
    fn default() -> Self {
        Self {
            graceful_timeout: 5,
            health_check_interval: 30,
            max_restart_attempts: 3,
            restart_delay: 1,
        }
    }
}
