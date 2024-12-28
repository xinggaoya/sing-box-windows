use std::error::Error;
use crate::entity::{config_model, github_model};
use crate::utils::config_util::ConfigUtil;
use crate::utils::file_util::{download_file, unzip_file};
use log::{error, info};
use std::fs::File;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::path::Path;
use winreg::enums::{HKEY_CURRENT_USER, KEY_SET_VALUE};
use winreg::RegKey;
use crate::entity::config_model::{CacheFileConfig, ClashApiConfig, Config};
use crate::utils::app_util::get_work_dir;
use serde_json::json;

// 以管理员权限重启
#[tauri::command]
pub fn restart_as_admin() -> Result<(), String> {
    let current_exe = std::env::current_exe().map_err(|e| format!("获取当前程序路径失败: {}", e))?;
    
    let result = std::process::Command::new("powershell")
        .arg("Start-Process")
        .arg(current_exe.to_str().unwrap())
        .arg("-Verb")
        .arg("RunAs")
        .creation_flags(0x08000000)
        .spawn();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("重启失败: {}", e))
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
        Err(_) => false
    }
}

// 运行内核
#[tauri::command]
pub fn start_kernel() -> Result<(), String> {
    let word_dir = get_work_dir();
    let kernel_path = Path::new(&word_dir).join("sing-box/sing-box");
    let kernel_word_dir = Path::new(&word_dir).join("sing-box");
    
    let res = std::process::Command::new(kernel_path.to_str().unwrap())
        .arg("run")
        .arg("-D")
        .arg(kernel_word_dir.to_str().unwrap())
        .creation_flags(0x08000000)
        .spawn();

    let child = match res {
        Ok(child) => child,
        Err(e) => {
            error!("Error starting kernel: {}", e);
            return Err(format!("启动内核失败: {}", e));
        }
    };

    // 记录pid到文件
    let pid = child.id();
    let pid_path = Path::new(&word_dir).join("sing-box/pid.txt");
    let file = File::create(pid_path.to_str().unwrap());
    match file {
        Ok(mut file) => match file.write_all(pid.to_string().as_bytes()) {
            Ok(_) => Ok(()),
            Err(e) => Err(format!("写入PID文件失败: {}", e))
        },
        Err(e) => Err(format!("创建PID文件失败: {}", e))
    }
}

// 停止
#[tauri::command]
pub fn stop_kernel() -> Result<(), String> {
    stop_kernel_impl().map_err(|e| format!("停止内核失败: {}", e))
}

fn stop_kernel_impl() -> Result<(), Box<dyn Error>> {
    let word_dir = get_work_dir();
    let pid_file = Path::new(&word_dir).join("sing-box/pid.txt");
    
    // 如果pid文件不存在，说明进程已经不在运行，直接返回成功
    if !pid_file.exists() {
        info!("PID文件不存在，内核可能已经停止");
        disable_proxy()?;
        return Ok(());
    }

    // 尝试读取pid文件
    let mut file = match File::open(&pid_file) {
        Ok(file) => file,
        Err(e) => {
            info!("无法打开PID文件: {}, 内核可能已经停止", e);
            disable_proxy()?;
            return Ok(());
        }
    };

    let mut buffer = String::new();
    if let Err(e) = file.read_to_string(&mut buffer) {
        info!("无法读取PID文件内容: {}, 内核可能已经停止", e);
        disable_proxy()?;
        return Ok(());
    }

    let pid: u32 = match buffer.trim().parse() {
        Ok(pid) => pid,
        Err(e) => {
            info!("PID���式无效: {}, 内核可能已经停止", e);
            disable_proxy()?;
            return Ok(());
        }
    };

    // 尝试结束进程
    if let Err(e) = kill_process(pid) {
        info!("结束进程失败: {}, 内核可能已经停止", e);
    }

    info!("进程已停止,进程id: {}", pid);
    disable_proxy()?;
    Ok(())
}

fn kill_process(pid: u32) -> Result<(), Box<dyn Error>> {
    let result = std::process::Command::new("taskkill")
        .arg("/F")
        .arg("/PID")
        .arg(pid.to_string())
        .creation_flags(0x08000000)
        .output()?;

    // 如果进程不存在，taskkill 会返回错误，但这种情况我们也认为是成功的
    if !result.status.success() {
        let stderr = String::from_utf8_lossy(&result.stderr);
        // 如果错误信息表明进程不存在，我们认为这是成功的情况
        if stderr.contains("进程未运行") || stderr.contains("not found") || stderr.contains("不存在") {
            info!("进程 {} 已经不存在", pid);
            return Ok(());
        }
        return Err(format!("Error killing process: {}", stderr).into());
    }
    Ok(())
}


// 下载订阅
#[tauri::command]
pub async fn download_subscription(url: String) -> Result<(), String> {
    download_and_process_subscription(url).await
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
        cache_file: CacheFileConfig {
            enabled: true,
        },
    };
    json_util.modify_property(&target_keys, serde_json::to_value(config)?);
    json_util.save()?;

    info!("订阅已更新");
    Ok(())
}


// 下载内核
#[tauri::command]
pub async fn download_latest_kernel() -> Result<(), String> {
    let url = "https://api.github.com/repos/SagerNet/sing-box/releases/latest";
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/");
    let client = reqwest::Client::new();
    // 设置请求头
    let mut headers = reqwest::header::HeaderMap::new();
    let user_agent = reqwest::header::HeaderValue
    ::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36\
     (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36");
    headers.insert(reqwest::header::USER_AGENT, user_agent);

    let response = client
        .get(url)
        .headers(headers)
        .send()
        .await
        .map_err(|e| format!("Error: {:?}", e.status()))?;

    let text = response
        .text()
        .await
        .map_err(|e| format!("Failed to get response text: {}", e))?;
    // json 转实体
    let json: github_model::Release =
        serde_json::from_str(&text).map_err(|e| format!("Error parsing JSON: {}", e))?;

    // 获取当前系统平台
    let platform = std::env::consts::OS;
    // 获取系统 比如amd64
    let mut arch = std::env::consts::ARCH;
    if arch == "x86_64" {
        arch = "amd64";
    }

    let str = format!("{}-{}.zip", platform, arch);
    for asset in json.assets {
        if asset.name.contains(&str) {
            info!("Asset: {}", asset.name);
            // 下载文件
            let path = Path::new(&path.to_str().unwrap()).join(&asset.name);
            // 加速下载
            let speed_url = format!("https://ghgo.xyz/{}", asset.browser_download_url);
            download_file(speed_url, path.to_str().unwrap())
                .await
                .map_err(|e| format!("Failed to download file: {}", e))?;
            // 解压文件
            let word_dir = get_work_dir();
            let out_path = Path::new(&word_dir).join("sing-box");
            let to_path = out_path.join(asset.name);
            unzip_file(&to_path.to_str().unwrap(), out_path.to_str().unwrap()).await?;

            info!("内核已下载,路径: {}", path.to_str().unwrap())
        }
    }
    Ok(())
}

// 修改代理模式为系统代理
#[tauri::command]
pub fn set_system_proxy() -> Result<(), String> {
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let json_util = ConfigUtil::new(path.to_str().unwrap())
        .map_err(|e| format!("读取配置文件失败: {}", e))?;

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
    
    json_util.modify_property(&target_keys, serde_json::to_value(new_structs)
        .map_err(|e| format!("序列化配置失败: {}", e))?);
    json_util.save()
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
    
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("请求内存数据失败: {}", e))?;
        
    let data = response.json::<serde_json::Value>()
        .await
        .map_err(|e| format!("解析内存数据失败: {}", e))?;
        
    Ok(data)
}

// 获取流量数据
#[tauri::command]
pub async fn get_traffic_data() -> Result<serde_json::Value, String> {
    let client = reqwest::Client::new();
    let url = "http://127.0.0.1:9090/traffic";
    
    let response = client.get(url)
        .send()
        .await
        .map_err(|e| format!("请求流量数据失败: {}", e))?;
        
    let mut data = response.json::<serde_json::Value>()
        .await
        .map_err(|e| format!("解析流量数据失败: {}", e))?;
        
    // 确保数据中包含总流量
    if let Some(obj) = data.as_object_mut() {
        // 如果没有总流量字段，添加它们
        if !obj.contains_key("upTotal") {
            if let Some(up) = obj.get("up").and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()) {
                obj.insert("upTotal".to_string(), json!(up));
            }
        }
        if !obj.contains_key("downTotal") {
            if let Some(down) = obj.get("down").and_then(|v| v.as_str()).and_then(|s| s.parse::<u64>().ok()) {
                obj.insert("downTotal".to_string(), json!(down));
            }
        }
    }
        
    Ok(data)
}
