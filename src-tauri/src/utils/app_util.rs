use std::path::PathBuf;
use tracing::error;
use crate::app::constants::messages;

// 获取工作目录
pub fn get_work_dir() -> String {
    let cache_dir = if cfg!(target_os = "windows") {
        // Windows: %LOCALAPPDATA%\sing-box-windows
        std::env::var("LOCALAPPDATA")
            .map(|p| PathBuf::from(p).join("sing-box-windows"))
            .unwrap_or_else(|_| PathBuf::from(r"C:\ProgramData\sing-box-windows"))
    } else {
        // 其他系统使用默认缓存目录
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("sing-box-windows")
    };

    // 确保目录存在
    if let Err(e) = std::fs::create_dir_all(&cache_dir) {
        error!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e);
    }

    cache_dir.to_str().unwrap_or(".").to_string()
}
