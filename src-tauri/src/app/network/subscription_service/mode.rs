use crate::app::constants::paths;
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use serde_json::json;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tauri::AppHandle;
use tracing::{error, info};

pub async fn toggle_proxy_mode_impl(app_handle: AppHandle, mode: String) -> Result<String, String> {
    if !["global", "rule"].contains(&mode.as_str()) {
        return Err(format!("无效的代理模式: {}", mode));
    }

    info!("正在切换代理模式为: {}", mode);

    let app_config = db_get_app_config(app_handle)
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;

    let path = if let Some(path_str) = app_config.active_config_path {
        std::path::PathBuf::from(path_str)
    } else {
        paths::get_config_dir().join("config.json")
    };

    if !path.exists() {
        return Err("配置文件不存在，请先添加订阅".to_string());
    }

    match modify_default_mode(&path, mode.clone(), None) {
        Ok(_) => {
            info!("代理模式已切换为: {}", mode);
            Ok(format!("代理模式已切换为: {}", mode))
        }
        Err(e) => {
            error!("切换代理模式失败: {}", e);
            Err(format!("切换代理模式失败: {}", e))
        }
    }
}

pub fn get_current_proxy_mode_impl() -> Result<String, String> {
    info!("正在获取当前代理模式");

    let path = paths::get_config_dir().join("config.json");

    if !path.exists() {
        return Ok("rule".to_string());
    }

    match read_proxy_mode_from_config(&path) {
        Ok(mode) => {
            info!("当前代理模式为: {}", mode);
            Ok(mode)
        }
        Err(e) => {
            error!("获取代理模式失败: {}", e);
            Ok("rule".to_string())
        }
    }
}

pub fn modify_default_mode(
    config_path: &Path,
    mode: String,
    api_port: Option<u16>,
) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(config_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mut config: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(config_obj) = config.as_object_mut() {
        if let Some(experimental) = config_obj.get_mut("experimental") {
            if let Some(clash_api) = experimental.get_mut("clash_api") {
                if let Some(clash_api_obj) = clash_api.as_object_mut() {
                    clash_api_obj.insert("default_mode".to_string(), json!(mode));

                    if let Some(port) = api_port {
                        clash_api_obj.insert(
                            "external_controller".to_string(),
                            json!(format!("127.0.0.1:{}", port)),
                        );
                    }

                    clash_api_obj.insert("external_ui".to_string(), json!("metacubexd"));
                } else {
                    return Err("clash_api 不是对象".into());
                }
            } else {
                let mut clash_api = serde_json::Map::new();
                clash_api.insert("default_mode".to_string(), json!(mode));
                clash_api.insert("external_ui".to_string(), json!("metacubexd"));

                if let Some(port) = api_port {
                    clash_api.insert(
                        "external_controller".to_string(),
                        json!(format!("127.0.0.1:{}", port)),
                    );
                }

                if let Some(exp_obj) = experimental.as_object_mut() {
                    exp_obj.insert("clash_api".to_string(), json!(clash_api));
                } else {
                    return Err("experimental 不是对象".into());
                }
            }
        } else {
            let mut experimental = serde_json::Map::new();

            let mut clash_api = serde_json::Map::new();
            clash_api.insert("default_mode".to_string(), json!(mode));
            clash_api.insert("external_ui".to_string(), json!("metacubexd"));

            if let Some(port) = api_port {
                clash_api.insert(
                    "external_controller".to_string(),
                    json!(format!("127.0.0.1:{}", port)),
                );
            }

            experimental.insert("clash_api".to_string(), json!(clash_api));

            config_obj.insert("experimental".to_string(), json!(experimental));
        }

        if let Some(experimental) = config_obj.get_mut("experimental") {
            if let Some(experimental_obj) = experimental.as_object_mut() {
                experimental_obj.insert(
                    "cache_file".to_string(),
                    json!({
                        "enabled": true
                    }),
                );
            }
        }

        let updated_content = serde_json::to_string_pretty(&config)?;
        let mut file = File::create(config_path)?;
        file.write_all(updated_content.as_bytes())?;

        info!("已成功更新代理模式为: {}", mode);
    } else {
        return Err("配置文件格式错误：根对象不是JSON对象".into());
    }

    Ok(())
}

fn read_proxy_mode_from_config(config_path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(config_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let json: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(experimental) = json.get("experimental") {
        if let Some(clash_api) = experimental.get("clash_api") {
            if let Some(default_mode) = clash_api.get("default_mode") {
                if let Some(mode) = default_mode.as_str() {
                    return Ok(mode.to_string());
                }
            }
        }
    }

    Ok("rule".to_string())
}
