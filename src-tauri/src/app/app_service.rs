use std::fs::File;
use std::io::{Read, Write};
use std::os::windows::process::CommandExt;
use std::path::Path;
use log::{error, info};
use crate::entity::{config_model, github_model};
use crate::utils::config_util::ConfigUtil;
use crate::utils::file_util::{download_file, unzip_file};

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
        Ok(mut file) => {
            match file.write_all(pid.to_string().as_bytes()) {
                Ok(_) => {}
                Err(e) => {
                    println!("Error writing to file: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error creating file: {}", e);
        }
    }
}

// 停止
#[tauri::command]
pub fn stop_kernel() {
    // 读取pid
    let pid_file = "./sing-box/pid.txt";
    let file = File::open(pid_file);
    match file {
        Ok(mut file) => {
            let mut buffer = String::new();
            match file.read_to_string(&mut buffer) {
                Ok(_) => {
                    let pid = buffer.trim().parse::<u32>().unwrap();
                    // 杀死进程
                    let result = std::process::Command::new("taskkill")
                        .arg("/F")
                        .arg("/PID")
                        .arg(pid.to_string())
                        .creation_flags(0x08000000)
                        .output();
                    match result {
                        Ok(_) => {
                            info!("进程已杀死")
                        }
                        Err(e) => {
                            println!("Error killing process: {}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("Error reading file: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error opening file: {}", e);
        }
    }
}

// 下载订阅
#[tauri::command]
pub async fn download_subscription(url: String) {
    // 请求订阅
    let client = reqwest::Client::new();
    // 设置请求头
    let mut headers = reqwest::header::HeaderMap::new();
    let user_agent = reqwest::header::HeaderValue::from_static("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36");
    headers.insert(reqwest::header::USER_AGENT, user_agent);
    let response = client.get(url).headers(headers).send().await;
    match response {
        Ok(response) => {
            let text = response.text().await.unwrap();
            // 写入文件
            let path = Path::new("./sing-box/config.json");
            let mut file = File::create(path).expect("Failed to create file");
            match file.write(text.as_bytes()) {
                Ok(_) => {
                    info!("订阅已更新")
                }
                Err(e) => {
                    println!("Error writing to file: {}", e);
                }
            }
        }
        Err(e) => {
            println!("Error: {:?}", e.status());
        }
    }
    set_system_proxy()
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

    let response = client.get(url).headers(headers).send().await.map_err(|e| format!("Error: {:?}", e.status()))?;

    let text = response.text().await.map_err(|e| format!("Failed to get response text: {}", e))?;
    // json 转实体
    let json: github_model::Release = serde_json::from_str(&text).map_err(|e| format!("Error parsing JSON: {}", e))?;

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
                .await.map_err(|e| format!("Failed to download file: {}", e))?;
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
    let json_util = ConfigUtil::new("./sing-box/config.json");
    match json_util {
        Ok(mut json_util) => {
            let target_keys = vec!["inbounds"]; // 修改为你的属性路径
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
                set_system_proxy: None,
            }, config_model::Inbound {
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
            }
            ];

            json_util.modify_property(&target_keys, serde_json::to_value(new_structs).unwrap());
            json_util.save().unwrap()
        }
        Err(e) => {
            error!("修改配置文件失败: {}", e)
        }
    }
}