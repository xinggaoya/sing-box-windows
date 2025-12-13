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
    db_get_app_config, db_save_app_config_internal,
};
use tauri::AppHandle;

fn write_default_config(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
    }

    fs::write(path, DEFAULT_SINGBOX_CONFIG).map_err(|e| format!("写入默认配置失败: {}", e))?;
    Ok(())
}

/// 尝试从同路径的 `.bak` 备份恢复配置（订阅配置写入时会维护该备份）。
/// 返回 `Ok(true)` 表示已成功恢复并可继续使用该配置文件。
fn try_restore_from_bak(path: &Path) -> Result<bool, String> {
    let backup = path.with_extension("bak");
    if !backup.exists() {
        return Ok(false);
    }

    let content =
        fs::read_to_string(&backup).map_err(|e| format!("读取备份配置失败: {}", e))?;
    if serde_json::from_str::<Value>(&content).is_err() {
        warn!("发现备份配置也不是有效 JSON，跳过恢复: {:?}", backup);
        return Ok(false);
    }

    fs::copy(&backup, path).map_err(|e| format!("从备份恢复配置失败: {}", e))?;
    info!("已从备份恢复配置: {:?} -> {:?}", backup, path);
    Ok(true)
}

async fn persist_active_config_path_if_missing(
    app_handle: &AppHandle,
    path: &Path,
) -> Result<(), String> {
    let mut app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;

    if app_config.active_config_path.is_none() {
        app_config.active_config_path = Some(path.to_string_lossy().to_string());
        db_save_app_config_internal(app_config, app_handle.clone())
            .await
            .map_err(|e| format!("保存应用配置失败: {}", e))?;
    }

    Ok(())
}

pub async fn ensure_singbox_config(app_handle: &AppHandle) -> Result<(), String> {
    // 从数据库获取配置路径
    let app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;

    let active_config_path = app_config.active_config_path.clone();
    let config_path = active_config_path
        .as_deref()
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| paths::get_config_dir().join("config.json"));

    if !config_path.exists() {
        info!("sing-box 配置文件不存在，尝试从备份恢复: {:?}", config_path);
        if try_restore_from_bak(&config_path)? {
            return Ok(());
        }

        // 关键修复：
        // 以前这里会直接把 active_config_path 重置为默认 `config.json`，导致“订阅选择”被悄悄改变，
        // 从而出现“内核运行配置与前端选中订阅不一致”的现象。
        // 现在优先在原路径恢复/写入默认模板，不主动切换 active_config_path（除非原本就没有设置）。
        info!("未找到可用备份，写入默认模板到: {:?}", config_path);
        write_default_config(&config_path)?;
        persist_active_config_path_if_missing(app_handle, &config_path).await?;
        info!("已恢复 sing-box 配置（保持 active_config_path 不变）");
        return Ok(());
    }

    match fs::read_to_string(&config_path) {
        Ok(content) => {
            if serde_json::from_str::<Value>(&content).is_ok() {
                Ok(())
            } else {
                warn!("检测到 sing-box 配置损坏，正在恢复默认模板");
                backup_corrupted_config(&config_path);

                // 优先从 `.bak` 恢复（订阅配置文件通常会有可回滚的备份）
                if try_restore_from_bak(&config_path)? {
                    return Ok(());
                }

                warn!("未找到可用备份，写入默认模板到: {:?}", config_path);
                write_default_config(&config_path)?;
                persist_active_config_path_if_missing(app_handle, &config_path).await?;
                info!("已恢复 sing-box 配置（保持 active_config_path 不变）");
                Ok(())
            }
        }
        Err(e) => {
            warn!("读取 sing-box 配置失败: {}，尝试恢复默认模板", e);
            backup_corrupted_config(&config_path);
            if try_restore_from_bak(&config_path)? {
                return Ok(());
            }

            warn!("未找到可用备份，写入默认模板到: {:?}", config_path);
            write_default_config(&config_path)?;
            persist_active_config_path_if_missing(app_handle, &config_path).await?;
            info!("已恢复 sing-box 配置（保持 active_config_path 不变）");
            Ok(())
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
