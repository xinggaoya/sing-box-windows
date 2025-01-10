pub mod manager;
pub mod process;

// 进程状态枚举
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ProcessStatus {
    Starting,
    Running,
    Stopping,
    Stopped,
    Failed(String),
}

// ... rest of the code ...
