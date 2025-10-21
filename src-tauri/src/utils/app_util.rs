use crate::app::constants::messages;
use std::path::PathBuf;
use tauri::path::BaseDirectory;
use tauri::{AppHandle, Manager};
use tracing::error;

// 获取工作目录（同步版本）
pub fn get_work_dir_sync() -> String {
    let cache_dir = if cfg!(target_os = "windows") {
        // Windows: %LOCALAPPDATA%\sing-box-windows
        std::env::var("LOCALAPPDATA")
            .map(|p| PathBuf::from(p).join("sing-box-windows"))
            .unwrap_or_else(|_| PathBuf::from(r"C:\ProgramData\sing-box-windows"))
    } else if cfg!(target_os = "linux") {
        // Linux: ~/.local/share/sing-box-windows
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("sing-box-windows")
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

// 获取工作目录
pub async fn get_work_dir() -> String {
    let cache_dir = if cfg!(target_os = "windows") {
        // Windows: %LOCALAPPDATA%\sing-box-windows
        std::env::var("LOCALAPPDATA")
            .map(|p| PathBuf::from(p).join("sing-box-windows"))
            .unwrap_or_else(|_| PathBuf::from(r"C:\ProgramData\sing-box-windows"))
    } else if cfg!(target_os = "linux") {
        // Linux: ~/.local/share/sing-box-windows
        dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("sing-box-windows")
    } else {
        // 其他系统使用默认缓存目录
        dirs::cache_dir()
            .unwrap_or_else(|| PathBuf::from("/tmp"))
            .join("sing-box-windows")
    };

    // 确保目录存在
    if let Err(e) = tokio::fs::create_dir_all(&cache_dir).await {
        error!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e);
    }

    cache_dir.to_str().unwrap_or(".").to_string()
}

/// 获取模板文件路径
/// 使用Tauri的资源API直接获取模板文件
pub fn get_template_path(app_handle: &AppHandle) -> Result<PathBuf, Box<dyn std::error::Error>> {
    // 直接从Tauri资源目录获取模板文件
    let template_path = app_handle
        .path()
        .resolve("src/config/template.json", BaseDirectory::Resource)?;
    tracing::info!("使用模板路径: {:?}", template_path);

    if !template_path.exists() {
        let err_msg = format!("找不到模板文件: {:?}", template_path);
        error!("{}", err_msg);
        return Err(err_msg.into());
    }

    Ok(template_path)
}

/// 获取服务路径
pub fn get_service_path() -> PathBuf {
    // 获取可执行程序路径
    let exe_path = std::env::current_exe().expect("无法获取可执行程序路径");
    let work_dir = exe_path
        .parent()
        .expect("无法获取可执行程序父目录")
        .to_str()
        .expect("无法将父目录路径转换为字符串");
    PathBuf::from(&work_dir)
        .join("src")
        .join("config")
        .join("sing-box-service.exe")
}
