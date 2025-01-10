use crate::entity::config_model::{CacheFileConfig, ClashApiConfig, Config};
use crate::entity::{config_model, github_model};
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
use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
use winreg::RegKey;

// 全局进程管理器
lazy_static::lazy_static! {
    pub(crate) static ref PROCESS_MANAGER: Arc<ProcessManager> = Arc::new(ProcessManager::new());
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
    set_system_proxy();
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
    let url = "https://api.github.com/repos/SagerNet/sing-box/releases/latest";
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

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(30))
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    // 设置请求头
    let mut headers = reqwest::header::HeaderMap::new();
    let user_agent = reqwest::header::HeaderValue::from_static(
        "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36"
    );
    headers.insert(reqwest::header::USER_AGENT, user_agent);

    info!("正在获取最新版本信息...");
    // 发送进度事件
    let _ = window.emit(
        "download-progress",
        json!({
            "status": "checking",
            "progress": 0,
            "message": "正在获取最新版本信息..."
        }),
    );

    let response = client
        .get(url)
        .headers(headers.clone())
        .send()
        .await
        .map_err(|e| format!("请求GitHub API失败: {}", e))?;

    if !response.status().is_success() {
        error!("GitHub API请求失败: {}", response.status());
        return Err(format!("GitHub API请求失败: {}", response.status()));
    }

    let text = response
        .text()
        .await
        .map_err(|e| format!("读取响应内容失败: {}", e))?;

    info!("正在解析版本信息...");
    let json: github_model::Release =
        serde_json::from_str(&text).map_err(|e| format!("解析JSON失败: {}", e))?;

    info!("最新版本: {}", json.tag_name);
    // 发送进度事件
    let _ = window.emit(
        "download-progress",
        json!({
            "status": "found",
            "progress": 10,
            "message": format!("找到最新版本: {}", json.tag_name)
        }),
    );

    // 获取当前系统平台
    let platform = std::env::consts::OS;
    // 获取系统架构
    let mut arch = std::env::consts::ARCH;
    if arch == "x86_64" {
        arch = "amd64";
    }

    let target_file = format!("{}-{}.zip", platform, arch);
    info!("目标文件名: {}", target_file);

    let mut found = false;
    let mut download_attempted = false;

    for asset in json.assets {
        if asset.name.contains(&target_file) {
            info!("找到匹配的文件: {}", asset.name);
            found = true;
            download_attempted = true;

            let download_path = Path::new(&path).join(&asset.name);

            // 发送进度事件
            let _ = window.emit(
                "download-progress",
                json!({
                    "status": "downloading",
                    "progress": 20,
                    "message": format!("开始下载文件: {}", asset.name)
                }),
            );

            // 尝试多个下载源
            let download_urls = vec![
                format!("https://ghgo.xyz/{}", asset.browser_download_url),
                format!("https://gh.api.99988866.xyz/{}", asset.browser_download_url),
                asset.browser_download_url.clone(),
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

            if !success {
                return Err(format!("所有下载源均失败，最后一次错误: {}", last_error));
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
                    break;
                }
                Err(e) => {
                    error!("解压文件失败: {}", e);
                    return Err(format!("解压文件失败: {}", e));
                }
            }
        }
    }

    if !found {
        error!("未找到适配当前系统的版本: {}-{}", platform, arch);
        return Err(format!("未找到适配当前系统的版本: {}-{}", platform, arch));
    }

    if !download_attempted {
        error!("未尝试下载任何文件");
        return Err("未尝试下载任何文件".to_string());
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
        inet4_address: None,
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
            inet4_address: None,
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
            inet4_address: Some("172.19.0.1/30".to_string()),
            auto_route: Some(true),
            strict_route: Some(true),
            stack: Some("mixed".to_string()),
            sniff: Some(true),
            set_system_proxy: None,
        },
    ];

    json_util.modify_property(&target_keys, serde_json::to_value(new_structs).unwrap());
    json_util.save()?;
    Ok(())
}

pub fn set_proxy_mode(mode: String) -> Result<(), String> {
    if let Err(e) = set_system_proxy() {
        return Err(format!("设置系统代理失败: {}", e));
    }
    Ok(())
}

#[allow(dead_code)]
fn disable_proxy() -> Result<(), Box<dyn Error>> {
    // 打开注册表键
    let hkcu = RegKey::predef(HKEY_CURRENT_USER);
    let internet_settings = hkcu.open_subkey_with_flags(
        "Software\\Microsoft\\Windows\\CurrentVersion\\Internet Settings",
        KEY_SET_VALUE,
    )?;

    // 关闭代理
    internet_settings.set_value("ProxyEnable", &0u32)?;

    // 清空代理服务器地址
    internet_settings.set_value("ProxyServer", &"")?;

    Ok(())
}

// 获取内存使用情况
#[tauri::command]
pub async fn get_memory_usage() -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:9090/memory";

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求内存数据失败: {}", e))?;

    let data = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("解析内存数据失败: {}", e))?;

    Ok(data)
}

// 获取流量数据
#[tauri::command]
pub async fn get_traffic_data() -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:9090/traffic";

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| format!("请求流量数据失败: {}", e))?;

    let mut data = response
        .json::<serde_json::Value>()
        .await
        .map_err(|e| format!("解析流量数据失败: {}", e))?;

    // 确保数据中包含总流量
    if let Some(obj) = data.as_object_mut() {
        // 如果没有总流量字段，添加它们
        if !obj.contains_key("upTotal") {
            if let Some(up) = obj
                .get("up")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
            {
                obj.insert("upTotal".to_string(), json!(up));
            }
        }
        if !obj.contains_key("downTotal") {
            if let Some(down) = obj
                .get("down")
                .and_then(|v| v.as_str())
                .and_then(|s| s.parse::<u64>().ok())
            {
                obj.insert("downTotal".to_string(), json!(down));
            }
        }
    }

    Ok(data)
}
