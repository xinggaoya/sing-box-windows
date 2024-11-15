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

// 运行内核
#[tauri::command]
pub fn start_kernel() {
    // 命令执行 不堵塞
    let child = std::process::Command::new("./sing-box/sing-box")
        .arg("run")
        .arg("-D")
        .arg("./sing-box")
        .creation_flags(0x08000000)
        .spawn()
        .expect("Failed to start child process");

    // 记录pid到文件
    let pid = child.id();
    let pid_file = "./sing-box/pid.txt";
    let file = File::create(pid_file);
    match file {
        Ok(mut file) => match file.write_all(pid.to_string().as_bytes()) {
            Ok(_) => {}
            Err(e) => {
                println!("Error writing to file: {}", e);
            }
        },
        Err(e) => {
            println!("Error creating file: {}", e);
        }
    }
}

// 停止
#[tauri::command]
pub fn stop_kernel() {
    if let Err(e) = stop_kernel_impl() {
        println!("Error: {}", e);
    }
}

fn stop_kernel_impl() -> Result<(), Box<dyn Error>> {
    let pid_file = "./sing-box/pid.txt";
    let mut file = File::open(pid_file)?;

    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    let pid: u32 = buffer.trim().parse()?;
    kill_process(pid)?;

    info!("进程已杀死");
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

    if !result.status.success() {
        return Err(format!("Error killing process: {:?}", result.stderr).into());
    }
    Ok(())
}


// 下载订阅
#[tauri::command]
pub async fn download_subscription(url: String) {
    if let Err(e) = download_and_process_subscription(url).await {
        println!("Error: {:?}", e);
    }
    set_system_proxy()
}

async fn download_and_process_subscription(url: String) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    let user_agent = reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36");
    headers.insert(reqwest::header::USER_AGENT, user_agent);

    let response = client.get(url).headers(headers).send().await?;
    let text = response.text().await?;

    let path = Path::new("./sing-box/config.json");
    let mut file = File::create(path)?;
    file.write_all(text.as_bytes())?;

    let mut json_util = ConfigUtil::new("./sing-box/config.json")?;
    let target_keys = vec!["experimental"];
    let config = Config {
        clash_api: ClashApiConfig {
            external_controller: "127.0.0.1:9090".to_string(),
            external_ui: "metacubexd".to_string(),
            external_ui_download_url: "https://github.com/MetaCubeX/metacubexd/archive/refs/heads/gh-pages.zip".to_string(),
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
    let dist_dir = "./sing-box";
    let client = reqwest::Client::new();
    // 设置请求头
    let mut headers = reqwest::header::HeaderMap::new();
    let user_agent = reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36");
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
            let path = Path::new(dist_dir).join(&asset.name);
            download_file(asset.browser_download_url, path.to_str().unwrap())
                .await
                .map_err(|e| format!("Failed to download file: {}", e))?;
            // 解压文件
            unzip_file(&format!("./sing-box/{}", asset.name), "./sing-box").await?;
        }
    }
    Ok(())
}

// 修改代理模式为系统代理
#[tauri::command]
pub fn set_system_proxy() {
    let json_util = ConfigUtil::new("./sing-box/config.json");

    match json_util {
        Ok(mut json_util) => {
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
            json_util.modify_property(&target_keys, serde_json::to_value(new_structs).unwrap());
            json_util.save().unwrap()
        }
        Err(e) => {
            error!("修改配置文件失败: {}", e)
        }
    }
    info!("代理模式已修改")
}
// 修改TUN 模式为代理模式
#[tauri::command]
pub fn set_tun_proxy() {
    if let Err(e) = set_tun_proxy_impl() {
        error!("设置TUN代理失败: {}", e);
    }
}

fn set_tun_proxy_impl() -> Result<(), Box<dyn Error>> {
    let mut json_util = ConfigUtil::new("./sing-box/config.json")?;

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
