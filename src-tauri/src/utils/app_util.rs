use std::path::PathBuf;
use tracing::error;
use crate::app::constants::messages;
use tauri::{AppHandle, Manager};
use tauri::path::BaseDirectory;

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

/// 获取模板文件路径
/// 使用Tauri的资源API直接获取模板文件
pub fn get_template_path(app_handle: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 直接从Tauri资源目录获取模板文件
    let template_path = app_handle.path().resolve("src/config/template.json", BaseDirectory::Resource)?;
    tracing::info!("使用模板路径: {:?}", template_path);
    
    if !template_path.exists() {
        let err_msg = format!("找不到模板文件: {:?}", template_path);
        tracing::error!("{}", err_msg);
        return Err(err_msg.into());
    }
    
    Ok(template_path)
}
