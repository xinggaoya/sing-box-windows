use crate::app::constants::paths;
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use serde_json::json;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tauri::AppHandle;
use tracing::{error, info, warn};

/// Read the Clash API port from a config file's experimental.clash_api.external_controller
fn read_api_port_from_config(config_path: &Path) -> Option<u16> {
    let mut file = File::open(config_path).ok()?;
    let mut content = String::new();
    file.read_to_string(&mut content).ok()?;
    let json: serde_json::Value = serde_json::from_str(&content).ok()?;
    let controller = json
        .get("experimental")?
        .get("clash_api")?
        .get("external_controller")?
        .as_str()?;
    // Parse "127.0.0.1:12081" -> 12081
    controller.rsplit(':').next()?.parse::<u16>().ok()
}

/// Patch the running Clash API mode via HTTP PATCH /configs
fn patch_clash_api_mode(api_port: u16, mode: &str) -> Result<(), Box<dyn Error>> {
    let url = format!("http://127.0.0.1:{}/configs", api_port);
    let body = format!(r#"{{"mode":"{}"}}"#, mode);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .no_proxy()
        .build()?;

    let resp = client
        .patch(&url)
        .header("Content-Type", "application/json")
        .body(body)
        .send()?;

    if resp.status().is_success() {
        info!("Clash API mode patched to: {}", mode);
        Ok(())
    } else {
        Err(format!("Clash API returned status {}", resp.status()).into())
    }
}

/// Query the running Clash API mode via HTTP GET /configs
fn query_clash_api_mode(api_port: u16) -> Result<String, Box<dyn Error>> {
    let url = format!("http://127.0.0.1:{}/configs", api_port);

    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(3))
        .no_proxy()
        .build()?;

    let resp = client.get(&url).send()?;
    let json: serde_json::Value = resp.json()?;
    if let Some(mode) = json.get("mode").and_then(|m| m.as_str()) {
        Ok(mode.to_string())
    } else {
        Err("mode field not found in Clash API response".into())
    }
}

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

    // 1. Update the config file (for persistence across restarts)
    match modify_default_mode(&path, mode.clone(), None) {
        Ok(_) => {
            info!("配置文件代理模式已更新为: {}", mode);
        }
        Err(e) => {
            error!("更新配置文件代理模式失败: {}", e);
            return Err(format!("切换代理模式失败: {}", e));
        }
    }

    // 2. Also update config.json if it's different from active config
    let config_json_path = paths::get_config_dir().join("config.json");
    if config_json_path.exists() && config_json_path != path {
        if let Err(e) = modify_default_mode(&config_json_path, mode.clone(), None) {
            warn!("更新 config.json 代理模式失败 (非致命): {}", e);
        }
    }

    // 3. Patch the running Clash API to switch mode immediately
    let api_port = read_api_port_from_config(&path)
        .or_else(|| read_api_port_from_config(&config_json_path))
        .unwrap_or(app_config.api_port as u16);

    match patch_clash_api_mode(api_port, &mode) {
        Ok(_) => {
            info!("Clash API 代理模式已实时切换为: {}", mode);
        }
        Err(e) => {
            warn!("Clash API 实时切换失败 (配置文件已更新，重启后生效): {}", e);
        }
    }

    Ok(format!("代理模式已切换为: {}", mode))
}

pub fn get_current_proxy_mode_impl() -> Result<String, String> {
    info!("正在获取当前代理模式");

    // 1. Try to get the live mode from Clash API first (most accurate)
    let config_path = paths::get_config_dir().join("config.json");
    if let Some(api_port) = read_api_port_from_config(&config_path) {
        match query_clash_api_mode(api_port) {
            Ok(mode) => {
                info!("从 Clash API 获取当前代理模式: {}", mode);
                return Ok(mode);
            }
            Err(e) => {
                warn!("从 Clash API 获取模式失败，回退到配置文件: {}", e);
            }
        }
    }

    // 2. Fall back to reading from config file
    if !config_path.exists() {
        return Ok("rule".to_string());
    }

    match read_proxy_mode_from_config(&config_path) {
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
