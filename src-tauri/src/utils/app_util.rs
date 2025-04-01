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
        let template_path = manifest_dir.join("src/config/template.json");
        tracing::info!("开发环境使用模板路径: {:?}", template_path);
        
        if !template_path.exists() {
            tracing::error!("模板文件不存在: {:?}", template_path);
        }
        
        template_path
    }
    
    #[cfg(not(debug_assertions))]
    {
        use std::path::Path;
        
        // 首先尝试在可执行文件所在目录的resources文件夹中查找
        let exe_path = match std::env::current_exe() {
            Ok(path) => path,
            Err(e) => {
                tracing::error!("无法获取当前可执行文件路径: {}", e);
                return PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/config/template.json");
            }
        };
        
        let app_dir = exe_path.parent().unwrap_or_else(|| Path::new(".")).to_path_buf();
        tracing::info!("应用目录: {:?}", app_dir);
            
        // 尝试多个可能的模板位置
        let possible_paths = vec![
            app_dir.join("resources/src/config/template.json"),
            app_dir.join("resources/config/template.json"),
            app_dir.join("resources/template.json"),
            app_dir.join("template.json"),
            // 回退到CARGO_MANIFEST_DIR作为最后的选择
            PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("src/config/template.json")
        ];
        
        for path in &possible_paths {
            tracing::info!("尝试模板路径: {:?}, 存在: {}", path, path.exists());
            if path.exists() {
                return path.clone();
            }
        }
        
        // 如果找不到任何模板，创建一个基本模板并返回
        let fallback_path = app_dir.join("template.json");
        tracing::error!("未找到模板文件，创建基本模板: {:?}", fallback_path);
        
        // 尝试创建目录
        if let Some(parent) = fallback_path.parent() {
            if !parent.exists() {
                if let Err(e) = std::fs::create_dir_all(parent) {
                    tracing::error!("创建目录失败: {}", e);
                }
            }
        }
        
        // 写入基本模板内容
        let basic_template = r#"{
  "log": {
    "level": "info",
    "timestamp": true
  },
  "dns": {
    "servers": [
      {
        "tag": "dns-remote",
        "address": "https://1.1.1.1/dns-query"
      },
      {
        "tag": "dns-local",
        "address": "https://223.5.5.5/dns-query"
      }
    ],
    "rules": []
  },
  "inbounds": [
    {
      "type": "mixed",
      "tag": "mixed-in",
      "listen": "127.0.0.1",
      "listen_port": 12080
    }
  ],
  "outbounds": [
    {
      "type": "selector",
      "tag": "手动切换",
      "outbounds": [
        "自动选择"
      ]
    },
    {
      "type": "urltest",
      "tag": "自动选择",
      "outbounds": [],
      "url": "https://www.gstatic.com/generate_204",
      "interval": "10m",
      "tolerance": 100
    },
    {
      "type": "direct",
      "tag": "direct"
    },
    {
      "type": "block",
      "tag": "block"
    },
    {
      "type": "dns",
      "tag": "dns-out"
    }
  ],
  "route": {
    "rules": [],
    "final": "手动切换"
  },
  "experimental": {
    "clash_api": {
      "external_controller": "127.0.0.1:12081",
      "external_ui": "metacubexd",
      "external_ui_download_url": "",
      "external_ui_download_detour": "手动切换",
      "default_mode": "rule"
    },
    "cache_file": {
      "enabled": true
    }
  }
}"#;
        
        if let Err(e) = std::fs::write(&fallback_path, basic_template) {
            tracing::error!("写入基本模板失败: {}", e);
            // 如果写入失败，返回CARGO_MANIFEST_DIR路径
            return possible_paths.last().unwrap().clone();
        }
        
        fallback_path
    }
}
