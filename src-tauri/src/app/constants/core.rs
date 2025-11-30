//! 核心服务相关常量
//!
//! 包含进程管理、文件路径等核心功能相关的常量

/// 进程相关常量
pub mod process {
    /// Windows 创建进程时隐藏控制台窗口的标志
    #[cfg(target_os = "windows")]
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
    use std::path::{Path, PathBuf};
    use std::fs;

    /// 获取 sing-box 配置目录
    pub fn get_config_dir() -> PathBuf {
        let work_dir = get_work_dir_sync();
        PathBuf::from(&work_dir).join("sing-box")
    }

    /// 获取当前激活配置指针文件路径
    pub fn get_active_config_indicator() -> PathBuf {
        get_config_dir().join("active_config_path")
    }

    /// 获取 Sing-Box 可执行文件路径
    pub fn get_kernel_path() -> PathBuf {
        let exe_name = if cfg!(target_os = "windows") {
            "sing-box.exe"
        } else {
            "sing-box"
        };
        get_config_dir().join(exe_name)
    }

    /// 获取 Sing-Box 工作目录
    pub fn get_kernel_work_dir() -> PathBuf {
        get_config_dir()
    }

    /// 获取配置文件路径
    pub fn get_config_path() -> PathBuf {
        let config_dir = get_config_dir();
        let indicator = get_active_config_indicator();

        if let Ok(raw) = fs::read_to_string(&indicator) {
            let trimmed = raw.trim();
            if !trimmed.is_empty() {
                let candidate = PathBuf::from(trimmed);
                if candidate.exists() {
                    return candidate;
                }
            }
        }

        config_dir.join("config.json")
    }

    /// 写入激活配置文件路径指针；None 表示清除指向，回退到默认 config.json
    pub fn set_active_config_path(path: Option<&Path>) -> std::io::Result<()> {
        let indicator = get_active_config_indicator();
        if let Some(parent) = indicator.parent() {
            fs::create_dir_all(parent)?;
        }

        match path {
            Some(p) => fs::write(indicator, p.to_string_lossy().to_string()),
            None => {
                if indicator.exists() {
                    fs::remove_file(indicator)?;
                }
                Ok(())
            }
        }
    }
}

/// 配置常量
pub mod config {
    /// 默认的 Inbound 标签
    pub const DEFAULT_INBOUND_TAG: &str = "mixed-in";

    /// 默认的 Inbound 类型
    pub const DEFAULT_INBOUND_TYPE: &str = "mixed";
}
