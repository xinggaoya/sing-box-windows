use crate::entity::config_model::{CacheFileConfig, ClashApiConfig, Config};
use crate::app::constants::{paths, messages};
use crate::utils::config_util::ConfigUtil;
use crate::utils::app_util::get_work_dir;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use tracing::info;
use base64;

// 下载订阅
#[tauri::command]
pub async fn download_subscription(url: String) -> Result<(), String> {
    download_and_process_subscription(url)
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_SUBSCRIPTION_FAILED, e))?;
    let _ = crate::app::proxy_service::set_system_proxy();
    Ok(())
}

// 手动添加订阅内容
#[tauri::command]
pub async fn add_manual_subscription(content: String) -> Result<(), String> {
    process_subscription_content(content)
        .map_err(|e| format!("{}: {}", messages::ERR_PROCESS_SUBSCRIPTION_FAILED, e))?;
    let _ = crate::app::proxy_service::set_system_proxy();
    Ok(())
}

// 获取当前配置文件内容
#[tauri::command]
pub fn get_current_config() -> Result<String, String> {
    let config_path = paths::get_config_path();
    
    // 检查文件是否存在
    if !config_path.exists() {
        return Err(messages::ERR_CONFIG_READ_FAILED.to_string());
    }
    
    // 读取文件内容
    match std::fs::read_to_string(config_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e)),
    }
}

async fn download_and_process_subscription(url: String) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    let user_agent = reqwest::header::HeaderValue::from_static("sing-box-windows/1.0 (sing-box; compatible; Windows NT 10.0)");
    headers.insert(reqwest::header::USER_AGENT, user_agent);

    let response = client.get(url).headers(headers).send().await?;
    let response_text = response.text().await?;

    // 检查内容是否为base64编码，并在需要时进行解码
    let text = if is_base64_encoded(&response_text) {
        info!("检测到base64编码内容，正在解码...");
        let decoded = match base64::decode(&response_text.trim()) {
            Ok(data) => data,
            Err(_) => {
                // 如果标准解码失败，尝试URL安全的base64变体
                base64::decode_config(&response_text.trim(), base64::URL_SAFE)
                    .map_err(|e| format!("Base64解码失败: {}", e))?
            }
        };
        
        // 尝试将解码后的内容解析为有效的UTF-8字符串
        match String::from_utf8(decoded.clone()) {
            Ok(s) => {
                // 检查解码后的内容是否是有效的JSON或配置格式
                if s.trim_start().starts_with('{') || s.contains("proxies:") {
                    s // 返回解码后的文本
                } else {
                    // 解码后的内容不像是有效的配置，可能是误判，使用原始文本
                    info!("解码后的内容不是有效的配置格式，使用原始内容");
                    response_text
                }
            },
            Err(_) => {
                // 如果不是有效的UTF-8，返回原始文本
                info!("解码后的内容不是有效的UTF-8，使用原始内容");
                response_text
            }
        }
    } else {
        response_text
    };

    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let mut file = File::create(path.to_str().unwrap())?;
    file.write_all(text.as_bytes())?;

    let mut json_util = ConfigUtil::new(path.to_str().unwrap())?;
    let target_keys = vec!["experimental"];
    let config = Config {
        clash_api: ClashApiConfig {
            external_controller: "127.0.0.1:12081".to_string(),
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

// 处理订阅内容（无论是从URL获取还是手动添加）
fn process_subscription_content(content: String) -> Result<(), Box<dyn Error>> {
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let mut file = File::create(path.to_str().unwrap())?;
    file.write_all(content.as_bytes())?;

    let mut json_util = ConfigUtil::new(path.to_str().unwrap())?;
    let target_keys = vec!["experimental"];
    let config = Config {
        clash_api: ClashApiConfig {
            external_controller: "127.0.0.1:12081".to_string(),
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

// 改进base64检测逻辑
fn is_base64_encoded(text: &str) -> bool {
    // 先进行基本字符集检查
    let is_valid_charset = text.trim().chars().all(|c| 
        c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' || 
        c == '-' || c == '_' // 支持URL安全变体
    );
    
    if !is_valid_charset {
        return false;
    }
    
    let trimmed = text.trim();
    
    // 检查长度（标准base64长度应为4的倍数，可能有结尾的'='填充）
    if trimmed.len() % 4 != 0 && !trimmed.ends_with('=') {
        return false;
    }
    
    // 避免误判普通文本
    if trimmed.len() < 8 || trimmed.contains(" ") {
        return false;
    }
    
    // 尝试解码看是否成功（更准确但性能较低的方法）
    let standard_decode_ok = base64::decode(trimmed).is_ok();
    let url_safe_decode_ok = base64::decode_config(trimmed, base64::URL_SAFE).is_ok();
    
    // 如果能成功解码，再检查解码后内容是否合理（避免误判）
    if standard_decode_ok || url_safe_decode_ok {
        // 检查是否为常见订阅格式特征
        if trimmed.starts_with("ey") || trimmed.starts_with("dm") {
            return true; // 常见的JSON或YAML base64编码的开头特征
        }
    }
    
    false
} 