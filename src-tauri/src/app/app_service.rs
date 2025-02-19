use crate::entity::config_model::{CacheFileConfig, ClashApiConfig, Config};
use crate::entity::config_model;
use crate::process::manager::ProcessManager;
use crate::utils::app_util::get_work_dir;
use crate::utils::config_util::ConfigUtil;
use crate::utils::file_util::{download_file, unzip_file};
use log::{error, info};
use serde_json::json;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::sync::Arc;
use tauri::Emitter;

// 全局进程管理器
lazy_static::lazy_static! {
    pub(crate) static ref PROCESS_MANAGER: Arc<ProcessManager> = Arc::new(ProcessManager::new());
}

// 获取内存使用情况
#[tauri::command]
pub async fn get_memory_usage() -> Result<String, String> {
    let output = std::process::Command::new("wmic")
        .args(["process", "where", "name='sing-box.exe'", "get", "WorkingSetSize"])
        .creation_flags(0x08000000)
        .output()
        .map_err(|e| e.to_string())?;

    let output_str = String::from_utf8_lossy(&output.stdout);
    let lines: Vec<&str> = output_str.lines().collect();
    if lines.len() < 2 {
        return Ok("0".to_string());
    }

    let memory = lines[1].trim();
    if memory.is_empty() {
        Ok("0".to_string())
    } else {
        Ok((memory.parse::<u64>().unwrap_or(0) / 1024 / 1024).to_string())
    }
}

// 获取流量数据
#[tauri::command]
pub async fn get_traffic_data() -> Result<String, String> {
    // 这里实现获取流量数据的逻辑
    // 由于需要实际的网络接口数据，这里返回一个示例数据
    Ok(json!({
        "upload": 0,
        "download": 0
    }).to_string())
}

// 以管理员权限重启
#[tauri::command]
pub fn restart_as_admin() -> Result<(), String> {
    let current_exe =
        std::env::current_exe().map_err(|e| format!("获取当前程序路径失败: {}", e))?;

    let result = std::process::Command::new("powershell")
        .arg("Start-Process")
        .arg(current_exe.to_str().unwrap())
        .arg("-Verb")
        .arg("RunAs")
        .creation_flags(0x08000000)
        .spawn();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("重启失败: {}", e)),
    }
}

// 检查是否有管理员权限
#[tauri::command]
pub fn check_admin() -> bool {
    let result = std::process::Command::new("net")
        .arg("session")
        .creation_flags(0x08000000)
        .output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

// 运行内核
#[tauri::command]
pub async fn start_kernel() -> Result<(), String> {
    PROCESS_MANAGER.start().await.map_err(|e| e.to_string())
}

// 停止内核
#[tauri::command]
pub async fn stop_kernel() -> Result<(), String> {
    PROCESS_MANAGER.stop().await.map_err(|e| e.to_string())
}

// 获取进程状态
#[tauri::command]
pub async fn get_process_status() -> Result<String, String> {
    let status = PROCESS_MANAGER.get_status().await;
    serde_json::to_string(&status).map_err(|e| e.to_string())
}

// 下载订阅
#[tauri::command]
pub async fn download_subscription(url: String) -> Result<(), String> {
    download_and_process_subscription(url)
        .await
        .map_err(|e| format!("下载订阅失败: {}", e))?;
        let _ = set_system_proxy();
    Ok(())
}

async fn download_and_process_subscription(url: String) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    let user_agent = reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36");
    headers.insert(reqwest::header::USER_AGENT, user_agent);

    let response = client.get(url).headers(headers).send().await?;
    let text = response.text().await?;

    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let mut file = File::create(path.to_str().unwrap())?;
    file.write_all(text.as_bytes())?;

    let mut json_util = ConfigUtil::new(path.to_str().unwrap())?;
    let target_keys = vec!["experimental"];
    let config = Config {
        clash_api: ClashApiConfig {
            external_controller: "127.0.0.1:9090".to_string(),
            external_ui: "metacubexd".to_string(),
            external_ui_download_url: "".to_string(),
            external_ui_download_detour: "手动切换".to_string(),
            default_mode: "rule".to_string(),
        },
        cache_file: CacheFileConfig { enabled: true },
    };
    json_util.modify_property(&target_keys, serde_json::to_value(config)?);
    json_util.save()?;

    info!("订阅已更新");
    Ok(())
}

// 下载内核
#[tauri::command]
pub async fn download_latest_kernel(window: tauri::Window) -> Result<(), String> {
    let url = "https://github.com/SagerNet/sing-box/releases/latest/download";
    let work_dir = get_work_dir();
    info!("当前工作目录: {}", work_dir);

    let path = Path::new(&work_dir).join("sing-box/");
    info!("目标下载目录: {}", path.display());

    // 如果目录已存在，先检查是否为有效目录
    if path.exists() {
        if !path.is_dir() {
            error!("sing-box 路径存在但不是目录");
            return Err("sing-box 路径存在但不是目录".to_string());
        }
    }

    // 确保目录存在
    if let Err(e) = std::fs::create_dir_all(&path) {
        error!("创建目录失败: {}", e);
        return Err(format!("创建目录失败: {}", e));
    }
    info!("已确保下载目录存在");

    info!("正在准备下载最新版本...");
    // 发送进度事件
    let _ = window.emit(
        "download-progress",
        json!({
            "status": "checking",
            "progress": 0,
            "message": "正在准备下载最新版本..."
        }),
    );

    // 获取当前系统平台
    let platform = std::env::consts::OS;
    // 获取系统架构
    let mut arch = std::env::consts::ARCH;
    if arch == "x86_64" {
        arch = "amd64";
    }

    let target_file = format!("{}-{}", platform, arch);
    info!("目标文件名: {}", target_file);

    let download_path = Path::new(&path).join(format!("sing-box-{}.zip", target_file));
    info!("目标下载路径: {}", download_path.display());

    // 发送进度事件
    let _ = window.emit(
        "download-progress",
        json!({
            "status": "downloading",
            "progress": 20,
            "message": format!("开始下载文件: sing-box-{}.zip", target_file)
        }),
    );

    // 尝试多个下载源
    let download_urls = vec![
        format!("{}/sing-box-{}.zip", url, target_file),
    ];

    let mut success = false;
    let mut last_error = String::new();

    for url in &download_urls {
        info!("尝试从 {} 下载", url);
        let window_clone = window.clone();
        match download_file(
            url.clone(),
            download_path.to_str().unwrap(),
            move |progress| {
                let real_progress = 20 + (progress as f64 * 0.6) as u32; // 20-80%的进度用于下载
                let _ = window_clone.emit(
                    "download-progress",
                    json!({
                        "status": "downloading",
                        "progress": real_progress,
                        "message": format!("正在下载: {}%", progress)
                    }),
                );
            },
        )
        .await
        {
            Ok(_) => {
                success = true;
                info!("下载成功");
                break;
            }
            Err(e) => {
                error!("从 {} 下载失败: {}", url, e);
                last_error = e;
                continue;
            }
        }
    }

    // 所有下载源都失败时才返回错误
    if !success {
        error!("所有下载源均失败");
        return Err("所有下载源均失败，请检查网络连接或稍后重试".to_string());
    }

    // 解压文件
    info!("开始解压文件...");
    // 发送进度事件
    let _ = window.emit(
        "download-progress",
        json!({
            "status": "extracting",
            "progress": 80,
            "message": "正在解压文件..."
        }),
    );

    let out_path = Path::new(&work_dir).join("sing-box");
    match unzip_file(download_path.to_str().unwrap(), out_path.to_str().unwrap()).await {
        Ok(_) => {
            info!("内核已下载并解压到: {}", out_path.display());
            // 发送完成事件
            let _ = window.emit(
                "download-progress",
                json!({
                    "status": "completed",
                    "progress": 100,
                    "message": "下载完成！"
                }),
            );
        }
        Err(e) => {
            error!("解压文件失败: {}", e);
            return Err(format!("解压文件失败: {}", e));
        }
    }

    Ok(())
}

// 修改代理模式为系统代理
#[tauri::command]
pub fn set_system_proxy() -> Result<(), String> {
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let json_util =
        ConfigUtil::new(path.to_str().unwrap()).map_err(|e| format!("读取配置文件失败: {}", e))?;

    let mut json_util = json_util;
    let target_keys = vec!["inbounds"];
    let new_structs = vec![config_model::Inbound {
        r#type: "mixed".to_string(),
        tag: "mixed-in".to_string(),
        listen: Some("0.0.0.0".to_string()),
        listen_port: Some(2080),
        address: None,
        auto_route: None,
        strict_route: None,
        stack: None,
        sniff: None,
        set_system_proxy: Some(true),
    }];

    json_util.modify_property(
        &target_keys,
        serde_json::to_value(new_structs).map_err(|e| format!("序列化配置失败: {}", e))?,
    );
    json_util
        .save()
        .map_err(|e| format!("保存配置文件失败: {}", e))?;

    info!("代理模式已修改");
    Ok(())
}

// 修改TUN 模式为代理模式
#[tauri::command]
pub fn set_tun_proxy() -> Result<(), String> {
    set_tun_proxy_impl().map_err(|e| format!("设置TUN代理失败: {}", e))
}

fn set_tun_proxy_impl() -> Result<(), Box<dyn Error>> {

    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let mut json_util = ConfigUtil::new(path.to_str().unwrap())?;

    let target_keys = vec!["inbounds"]; // 修改为你的属性路径
    let new_structs = vec![
        config_model::Inbound {
            r#type: "mixed".to_string(),
            tag: "mixed-in".to_string(),
            listen: Some("0.0.0.0".to_string()),
            listen_port: Some(2080),
            address: None,
            auto_route: None,
            strict_route: None,
            stack: None,
            sniff: None,
            set_system_proxy: None,
        },
        config_model::Inbound {
            r#type: "tun".to_string(),
            tag: "tun-in".to_string(),
            listen: None,
            listen_port: None,
            address: Some(vec![
                "172.18.0.1/30".to_string(),
               "fdfe:dcba:9876::1/126".to_string()
           ]),
            auto_route: Some(true),
            strict_route: Some(true),
            stack: Some("mixed".to_string()),
            sniff: None,
            set_system_proxy: None,
        }
    ];

    json_util.modify_property(
        &target_keys,
        serde_json::to_value(new_structs).map_err(|e| format!("序列化配置失败: {}", e))?,
    );
    json_util
        .save()
        .map_err(|e| format!("保存配置文件失败: {}", e))?;

    info!("TUN代理模式已设置");
    Ok(())
}
