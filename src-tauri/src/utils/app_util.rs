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

/// 获取模板文件路径
pub fn get_template_path() -> PathBuf {
    // 获取应用资源目录，Tauri 2.0使用不同的API
    #[cfg(debug_assertions)]
    {
        // 开发环境下直接使用项目目录中的模板
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        manifest_dir.join("src/config/template.json")
    }
    
    #[cfg(not(debug_assertions))]
    {
        use std::path::Path;
        
        // 生产环境尝试获取打包后的资源目录
        // Tauri 2.0中资源在应用目录下的resources文件夹
        let app_dir = std::env::current_exe()
            .unwrap_or_else(|_| PathBuf::from("."))
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
            
        let resource_path = app_dir.join("resources/src/config/template.json");
        
        if resource_path.exists() {
            resource_path
        } else {
            // 如果找不到资源，使用环境变量中的路径作为后备
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/config/template.json")
        }
    }
}
