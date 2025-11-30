use crate::app::constants::paths;
use serde_json::json;
use serde_json::Value;
use std::error::Error;
use std::fs;
use std::path::Path;
use tracing::{error, info, warn};

const DEFAULT_SINGBOX_CONFIG: &str = include_str!("../../config/template.json");

fn backup_corrupted_config(path: &Path) {
    if !path.exists() {
        return;
    }

    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_else(|_| std::time::Duration::from_secs(0))
        .as_secs();
    let backup_path = path.with_extension(format!("bak-{}", timestamp));

    if let Err(e) = fs::rename(path, &backup_path) {
        warn!("备份损坏的配置失败: {}", e);
    } else {
        info!("已备份损坏的配置到 {:?}", backup_path);
    }
}

fn restore_default_config() -> Result<(), String> {
    let config_path = paths::get_config_path();
    if let Some(parent) = config_path.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
    }
    fs::write(&config_path, DEFAULT_SINGBOX_CONFIG)
        .map_err(|e| format!("写入默认配置失败: {}", e))?;
    info!("已恢复默认 sing-box 配置");
    Ok(())
}

pub fn ensure_singbox_config() -> Result<(), String> {
    let config_path = paths::get_config_path();

    if !config_path.exists() {
        info!("sing-box 配置文件不存在，使用默认模板恢复");
        return restore_default_config();
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => {
            if serde_json::from_str::<Value>(&content).is_ok() {
                Ok(())
            } else {
                warn!("检测到 sing-box 配置损坏，正在恢复默认模板");
                backup_corrupted_config(&config_path);
                restore_default_config()
            }
        }
        Err(e) => {
            warn!("读取 sing-box 配置失败: {}，尝试恢复默认模板", e);
            backup_corrupted_config(&config_path);
            restore_default_config()
        }
    }
}

// 更新sing-box配置文件中的端口设置
fn update_singbox_config_ports(proxy_port: u16, api_port: u16) -> Result<(), Box<dyn Error>> {
    let config_path = paths::get_config_path();

    // 检查配置文件是否存在
    if !config_path.exists() {
        info!("sing-box配置文件不存在，跳过更新");
        return Ok(());
    }

    info!(
        "正在更新sing-box配置文件中的端口设置: 代理端口={}, API端口={}",
        proxy_port, api_port
    );

    // 读取现有的配置文件
    let config_content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            error!("读取sing-box配置失败: {}", e);
            return Err(Box::new(e));
        }
    };

    // 解析为JSON
    let mut config: Value = match serde_json::from_str(&config_content) {
        Ok(json) => json,
        Err(e) => {
            error!("解析sing-box配置JSON失败: {}", e);
            return Err(Box::new(e));
        }
    };

    // 修改API端口和代理端口
    if let Some(config_obj) = config.as_object_mut() {
        // 修改experimental.clash_api配置（如果存在）
        if let Some(experimental) = config_obj.get_mut("experimental") {
            if let Some(exp_obj) = experimental.as_object_mut() {
                // 添加或修改clash_api配置
                let clash_api = exp_obj.entry("clash_api").or_insert(json!({}));

                if let Some(clash_api_obj) = clash_api.as_object_mut() {
                    // 设置external_controller为本地端口
                    clash_api_obj.insert(
                        "external_controller".to_string(),
                        json!(format!("127.0.0.1:{}", api_port)),
                    );
                }
            }
        } else {
            // 如果不存在experimental字段，添加它
            config_obj.insert(
                "experimental".to_string(),
                json!({
                    "clash_api": {
                        "external_controller": format!("127.0.0.1:{}", api_port),
                        "external_ui": "metacubexd",
                        "default_mode": "rule"
                    }
                }),
            );
        }

        // 修改入站端口（如果有inbounds）
        if let Some(inbounds) = config_obj.get_mut("inbounds") {
            if let Some(inbounds_array) = inbounds.as_array_mut() {
                for inbound in inbounds_array {
                    if let Some(inbound_obj) = inbound.as_object_mut() {
                        if inbound_obj.get("tag").and_then(|t| t.as_str()) == Some("mixed-in") {
                            inbound_obj.insert("listen_port".to_string(), json!(proxy_port));
                        }
                    }
                }
            }
        }
    }

    // 保存修改后的配置
    match fs::write(&config_path, serde_json::to_string_pretty(&config)?) {
        Ok(_) => {
            info!("已更新sing-box配置文件中的端口设置");
            Ok(())
        }
        Err(e) => {
            error!("保存sing-box配置失败: {}", e);
            Err(Box::new(e))
        }
    }
}

// 更新sing-box配置文件中的端口设置（供外部调用）
#[tauri::command]
pub fn update_singbox_ports(proxy_port: u16, api_port: u16) -> Result<bool, String> {
    // 验证端口范围
    if proxy_port < 1024 || api_port < 1024 {
        return Err("端口号必须在1024-65535之间".to_string());
    }

    // 验证端口不冲突
    if proxy_port == api_port {
        return Err("代理端口和API端口不能相同".to_string());
    }

    // 更新sing-box配置文件中的端口设置
    match update_singbox_config_ports(proxy_port, api_port) {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("更新sing-box配置端口失败: {}", e)),
    }
}
