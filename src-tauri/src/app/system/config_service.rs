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

use crate::app::storage::enhanced_storage_service::{
    db_get_app_config, db_save_app_config, db_save_app_config_internal,
};
use tauri::AppHandle;

async fn restore_default_config(app_handle: &AppHandle) -> Result<(), String> {
    // 获取配置目录
    let config_dir = paths::get_config_dir();
    if !config_dir.exists() {
        fs::create_dir_all(&config_dir).map_err(|e| format!("创建配置目录失败: {}", e))?;
    }

    // 默认配置路径
    let default_config_path = config_dir.join("config.json");

    // 写入默认配置
    fs::write(&default_config_path, DEFAULT_SINGBOX_CONFIG)
        .map_err(|e| format!("写入默认配置失败: {}", e))?;

    // 更新数据库中的激活配置路径
    let mut app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;

    app_config.active_config_path = Some(default_config_path.to_string_lossy().to_string());

    db_save_app_config_internal(app_config, app_handle.clone())
        .await
        .map_err(|e| format!("保存应用配置失败: {}", e))?;

    info!("已恢复默认 sing-box 配置");
    Ok(())
}

pub async fn ensure_singbox_config(app_handle: &AppHandle) -> Result<(), String> {
    // 从数据库获取配置路径
    let app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;

    let config_path = if let Some(path_str) = app_config.active_config_path {
        std::path::PathBuf::from(path_str)
    } else {
        paths::get_config_dir().join("config.json")
    };

    if !config_path.exists() {
        info!("sing-box 配置文件不存在，使用默认模板恢复");
        return restore_default_config(app_handle).await;
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => {
            if serde_json::from_str::<Value>(&content).is_ok() {
                Ok(())
            } else {
                warn!("检测到 sing-box 配置损坏，正在恢复默认模板");
                backup_corrupted_config(&config_path);
                restore_default_config(app_handle).await
            }
        }
        Err(e) => {
            warn!("读取 sing-box 配置失败: {}，尝试恢复默认模板", e);
            backup_corrupted_config(&config_path);
            restore_default_config(app_handle).await
        }
    }
}

// 更新sing-box配置文件中的端口设置
// 更新sing-box配置文件中的端口设置
async fn update_singbox_config_ports(
    app_handle: &AppHandle,
    proxy_port: u16,
    api_port: u16,
) -> Result<(), Box<dyn Error>> {
    // 从数据库获取配置路径
    let app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;

    let config_path = if let Some(path_str) = app_config.active_config_path {
        std::path::PathBuf::from(path_str)
    } else {
        paths::get_config_dir().join("config.json")
    };

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
// 更新sing-box配置文件中的端口设置（供外部调用）
#[tauri::command]
pub async fn update_singbox_ports(
    app_handle: AppHandle,
    proxy_port: u16,
    api_port: u16,
) -> Result<bool, String> {
    // 验证端口范围
    if proxy_port < 1024 || api_port < 1024 {
        return Err("端口号必须在1024-65535之间".to_string());
    }

    // 验证端口不冲突
    if proxy_port == api_port {
        return Err("代理端口和API端口不能相同".to_string());
    }

    // 更新sing-box配置文件中的端口设置
    match update_singbox_config_ports(&app_handle, proxy_port, api_port).await {
        Ok(_) => Ok(true),
        Err(e) => Err(format!("更新sing-box配置端口失败: {}", e)),
    }
}
