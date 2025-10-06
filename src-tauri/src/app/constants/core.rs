//! 核心服务相关常量
//!
//! 包含进程管理、文件路径等核心功能相关的常量

/// 进程相关常量
pub mod process {
    /// Windows 创建进程时隐藏控制台窗口的标志
    pub const CREATE_NO_WINDOW: u32 = 0x08000000;

    /// 进程超时和延迟常量（秒）
    pub const GRACEFUL_TIMEOUT: u64 = 5;
    pub const HEALTH_CHECK_INTERVAL: u64 = 30;
    pub const MAX_RESTART_ATTEMPTS: u32 = 3;
    pub const RESTART_DELAY: u64 = 1;
}

/// 文件路径常量
pub mod paths {
    use crate::utils::app_util::get_work_dir_sync;
    use std::path::PathBuf;

    /// 获取 Sing-Box 可执行文件路径
    pub fn get_kernel_path() -> PathBuf {
        let work_dir = get_work_dir_sync();
        PathBuf::from(&work_dir)
            .join("sing-box")
            .join("sing-box.exe")
    }

    /// 获取 Sing-Box 工作目录
    pub fn get_kernel_work_dir() -> PathBuf {
        let work_dir = get_work_dir_sync();
        PathBuf::from(&work_dir).join("sing-box")
    }

    /// 获取配置文件路径
    pub fn get_config_path() -> PathBuf {
        let work_dir = get_work_dir_sync();
        PathBuf::from(&work_dir)
            .join("sing-box")
            .join("config.json")
    }
}

/// 配置常量
pub mod config {
    /// 默认的 Inbound 标签
    pub const DEFAULT_INBOUND_TAG: &str = "mixed-in";

    /// 默认的 Inbound 类型
    pub const DEFAULT_INBOUND_TYPE: &str = "mixed";
}
