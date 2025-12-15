pub mod helpers;
mod mode;
mod parser;
pub mod auto_update;

use crate::app::constants::{messages, paths};
use crate::app::core::kernel_auto_manage::auto_manage_with_saved_config;
use crate::app::core::proxy_service::apply_proxy_runtime_state;
use crate::app::singbox::config_generator;
use crate::app::singbox::settings_patch::apply_app_settings_to_config;
use crate::app::storage::enhanced_storage_service::{db_get_app_config, db_save_app_config};
use crate::app::storage::state_model::AppConfig;
use crate::utils::http_client;
use base64;
use helpers::{
    backup_existing_config, resolve_target_config_path, runtime_state_from_config,
};
use parser::{extract_nodes_from_subscription, parse_to_value};
use serde_json::{json, Value};
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tracing::{error, info, warn};

#[tauri::command]
pub async fn download_subscription(
    url: String,
    use_original_config: bool,
    file_name: Option<String>,
    config_path: Option<String>,
    apply_runtime: Option<bool>,
    window: tauri::Window,
    proxy_port: Option<u16>,
    api_port: Option<u16>,
) -> Result<String, String> {
    let app_handle = window.app_handle();
    let apply_runtime = apply_runtime.unwrap_or(true);

    let mut app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("读取设置失败: {}", e))?;

    if let Some(port) = proxy_port {
        app_config.proxy_port = port;
    }
    if let Some(port) = api_port {
        app_config.api_port = port;
    }

    let target_path = resolve_target_config_path(file_name, config_path)?;
    download_and_process_subscription(
        url,
        use_original_config,
        &app_handle,
        &app_config,
        &target_path,
    )
    .await
    .map_err(|e| format!("{}: {}", messages::ERR_SUBSCRIPTION_FAILED, e))?;

    if apply_runtime {
        if let Err(e) = set_active_config_path(
            app_handle.clone(),
            Some(target_path.to_string_lossy().to_string()),
        )
        .await
        {
            warn!("写入激活配置指针失败: {}", e);
        }

        let runtime_state = runtime_state_from_config(&app_config);
        if let Err(e) = apply_proxy_runtime_state(&app_handle, &runtime_state).await {
            warn!("应用代理配置失败: {}", e);
        }
        auto_manage_with_saved_config(&app_handle, true, "subscription-download").await;
    }

    Ok(target_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn add_manual_subscription(
    content: String,
    use_original_config: bool,
    file_name: Option<String>,
    config_path: Option<String>,
    apply_runtime: Option<bool>,
    window: tauri::Window,
    proxy_port: Option<u16>,
    api_port: Option<u16>,
) -> Result<String, String> {
    let app_handle = window.app_handle();
    let apply_runtime = apply_runtime.unwrap_or(true);

    let mut app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("读取设置失败: {}", e))?;

    if let Some(port) = proxy_port {
        app_config.proxy_port = port;
    }
    if let Some(port) = api_port {
        app_config.api_port = port;
    }

    let target_path = resolve_target_config_path(file_name, config_path)?;

    process_subscription_content(
        content,
        use_original_config,
        &app_handle,
        &app_config,
        &target_path,
    )
    .await
    .map_err(|e| format!("{}: {}", messages::ERR_PROCESS_SUBSCRIPTION_FAILED, e))?;

    if apply_runtime {
        if let Err(e) = set_active_config_path(
            app_handle.clone(),
            Some(target_path.to_string_lossy().to_string()),
        )
        .await
        {
            warn!("写入激活配置指针失败: {}", e);
        }

        let runtime_state = runtime_state_from_config(&app_config);
        if let Err(e) = apply_proxy_runtime_state(&app_handle, &runtime_state).await {
            warn!("应用代理配置失败: {}", e);
        }
        auto_manage_with_saved_config(&app_handle, true, "subscription-manual").await;
    }

    Ok(target_path.to_string_lossy().to_string())
}

#[tauri::command]
pub async fn get_current_config(app_handle: AppHandle) -> Result<String, String> {
    let app_config = db_get_app_config(app_handle)
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;

    let config_path = if let Some(path_str) = app_config.active_config_path {
        std::path::PathBuf::from(path_str)
    } else {
        paths::get_config_dir().join("config.json")
    };

    if !config_path.exists() {
        return Err(messages::ERR_CONFIG_READ_FAILED.to_string());
    }

    std::fs::read_to_string(config_path)
        .map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))
}

#[tauri::command]
pub async fn set_active_config_path(
    app_handle: AppHandle,
    config_path: Option<String>,
) -> Result<(), String> {
    let mut app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;

    // 记录 active_config_path 的变更，便于排查“过一段时间配置指针被改写”的问题
    let previous = app_config.active_config_path.clone();
    app_config.active_config_path = config_path;
    info!(
        "设置 active_config_path: {:?} -> {:?}",
        previous, app_config.active_config_path
    );

    db_save_app_config(app_config, app_handle)
        .await
        .map_err(|e| format!("保存配置路径失败: {}", e))
}

#[tauri::command]
pub fn delete_subscription_config(config_path: String) -> Result<(), String> {
    let path = PathBuf::from(&config_path);

    if path.exists() {
        std::fs::remove_file(&path).map_err(|e| format!("删除配置文件失败: {}", e))?;
    }

    let backup = path.with_extension("bak");
    if backup.exists() {
        let _ = std::fs::remove_file(&backup);
    }

    Ok(())
}

#[tauri::command]
pub fn rollback_subscription_config(config_path: String) -> Result<String, String> {
    let path = PathBuf::from(&config_path);
    let backup = path.with_extension("bak");

    if !backup.exists() {
        return Err("未找到可用于回滚的备份文件".to_string());
    }

    std::fs::copy(&backup, &path).map_err(|e| format!("回滚配置失败: {}", e))?;

    Ok(config_path)
}

#[tauri::command]
pub async fn toggle_proxy_mode(app_handle: AppHandle, mode: String) -> Result<String, String> {
    mode::toggle_proxy_mode_impl(app_handle, mode).await
}

#[tauri::command]
pub fn get_current_proxy_mode() -> Result<String, String> {
    mode::get_current_proxy_mode_impl()
}

async fn download_and_process_subscription(
    url: String,
    use_original_config: bool,
    _app_handle: &AppHandle,
    app_config: &AppConfig,
    target_path: &Path,
) -> Result<(), Box<dyn Error>> {
    let work_dir = crate::utils::app_util::get_work_dir_sync();
    let sing_box_dir = Path::new(&work_dir).join("sing-box");

    if !sing_box_dir.exists() {
        info!("正在创建Sing-Box目录: {:?}", sing_box_dir);
        if let Err(e) = std::fs::create_dir_all(&sing_box_dir) {
            let err_msg = format!("创建Sing-Box目录失败: {}", e);
            error!("{}", err_msg);
            return Err(err_msg.into());
        }
    }

    info!("开始下载订阅: {}", url);

    let response_text = http_client::get_text(url.trim())
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_SUBSCRIPTION_FAILED, e))?;

    info!("订阅下载成功，内容长度: {} 字节", response_text.len());

    if use_original_config {
        info!("使用原始订阅内容，仅修改必要的端口和地址");
        process_original_config(&response_text, app_config, target_path)?;
        return Ok(());
    }

    let mut extracted_nodes = extract_nodes_from_subscription(&response_text)?;
    if extracted_nodes.is_empty() {
        if let Ok(provider_nodes) = try_load_proxy_providers(&response_text).await {
            if !provider_nodes.is_empty() {
                info!("从 proxy-providers 提取到 {} 个节点", provider_nodes.len());
                extracted_nodes = provider_nodes;
            }
        }
    }
    info!("从原始内容提取到 {} 个节点", extracted_nodes.len());

    if extracted_nodes.is_empty() {
        info!("未从原始内容提取到节点，尝试base64解码...");

        if let Ok(decoded) = base64::decode(&response_text.trim()) {
            if let Ok(decoded_text) = String::from_utf8(decoded.clone()) {
                info!("base64标准解码成功，重新从解码内容提取节点...");
                extracted_nodes = extract_nodes_from_subscription(&decoded_text)?;
                if extracted_nodes.is_empty() {
                    if let Ok(provider_nodes) = try_load_proxy_providers(&decoded_text).await {
                        if !provider_nodes.is_empty() {
                            info!("从 proxy-providers 提取到 {} 个节点", provider_nodes.len());
                            extracted_nodes = provider_nodes;
                        }
                    }
                }
                info!(
                    "从标准base64解码内容提取到 {} 个节点",
                    extracted_nodes.len()
                );
            }
        } else if let Ok(decoded) = base64::decode_config(&response_text.trim(), base64::URL_SAFE) {
            if let Ok(decoded_text) = String::from_utf8(decoded.clone()) {
                info!("URL安全base64解码成功，重新从解码内容提取节点...");
                extracted_nodes = extract_nodes_from_subscription(&decoded_text)?;
                if extracted_nodes.is_empty() {
                    if let Ok(provider_nodes) = try_load_proxy_providers(&decoded_text).await {
                        if !provider_nodes.is_empty() {
                            info!("从 proxy-providers 提取到 {} 个节点", provider_nodes.len());
                            extracted_nodes = provider_nodes;
                        }
                    }
                }
                info!(
                    "从URL安全base64解码内容提取到 {} 个节点",
                    extracted_nodes.len()
                );
            }
        }
    }

    if extracted_nodes.is_empty() {
        info!("标准解码方法均未提取到节点，尝试移除前缀后再解码...");

        let stripped_text = response_text
            .trim()
            .replace("vmess://", "")
            .replace("ss://", "")
            .replace("trojan://", "")
            .replace("vless://", "");

        if let Ok(decoded) = base64::decode(&stripped_text) {
            if let Ok(decoded_text) = String::from_utf8(decoded) {
                extracted_nodes = extract_nodes_from_subscription(&decoded_text)?;
                info!(
                    "从移除前缀后解码内容提取到 {} 个节点",
                    extracted_nodes.len()
                );
            }
        }
    }

    if extracted_nodes.is_empty() {
        error!("无法从订阅内容提取节点信息，已尝试所有解码方式");
        return Err("无法从订阅内容提取节点信息，请检查订阅链接或内容格式".into());
    }

    info!(
        "成功提取到 {} 个节点，准备应用到配置",
        extracted_nodes.len()
    );

    let dir = target_path
        .parent()
        .map(|p| p.to_path_buf())
        .unwrap_or_else(|| PathBuf::from(&work_dir).join("sing-box"));
    if let Err(e) = std::fs::create_dir_all(&dir) {
        error!("{}: {}", messages::ERR_CREATE_DIR_FAILED, e);
    }

    // 不再读取/替换模板文件：直接根据 AppConfig 生成一份通用配置骨架，然后注入订阅节点。
    let config = config_generator::generate_config_with_nodes(app_config, &extracted_nodes)
        .map_err(|e| format!("生成配置失败: {}", e))?;

    info!("正在保存配置到: {:?}", target_path);

    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            info!("创建配置目录: {:?}", parent);
            if let Err(e) = std::fs::create_dir_all(parent) {
                let err_msg = format!("创建配置目录失败: {}", e);
                error!("{}", err_msg);
                return Err(err_msg.into());
            }
        }
    }

    let _backup = backup_existing_config(target_path);

    let config_str = serde_json::to_string_pretty(&config)?;
    let mut file = File::create(target_path)?;
    file.write_all(config_str.as_bytes())?;

    info!("配置已成功保存到: {:?}", target_path);
    info!("订阅已更新并应用到模板，配置已保存");
    Ok(())
}

async fn process_subscription_content(
    content: String,
    use_original_config: bool,
    _app_handle: &AppHandle,
    app_config: &AppConfig,
    target_path: &Path,
) -> Result<(), Box<dyn Error>> {
    if use_original_config {
        info!("使用原始配置内容，仅调整端口和地址");
        process_original_config(&content, app_config, target_path)?;
        return Ok(());
    }

    let mut extracted_nodes = extract_nodes_from_subscription(&content)?;
    info!("从手动内容提取到 {} 个节点", extracted_nodes.len());

    if extracted_nodes.is_empty() {
        if let Ok(provider_nodes) = try_load_proxy_providers(&content).await {
            if !provider_nodes.is_empty() {
                info!("从 proxy-providers 提取到 {} 个节点", provider_nodes.len());
                extracted_nodes = provider_nodes;
            }
        }
    }

    if extracted_nodes.is_empty() {
        if let Ok(decoded) = base64::decode(&content.trim()) {
            if let Ok(decoded_text) = String::from_utf8(decoded.clone()) {
                info!("手动内容 base64 解码成功，重新提取节点");
                extracted_nodes = extract_nodes_from_subscription(&decoded_text)?;
                if extracted_nodes.is_empty() {
                    if let Ok(provider_nodes) = try_load_proxy_providers(&decoded_text).await {
                        if !provider_nodes.is_empty() {
                            info!("从 proxy-providers 提取到 {} 个节点", provider_nodes.len());
                            extracted_nodes = provider_nodes;
                        }
                    }
                }
                info!("从解码内容提取到 {} 个节点", extracted_nodes.len());
            }
        }
    }

    if extracted_nodes.is_empty() {
        return Err("无法从配置内容提取节点，请检查格式".into());
    }

    // 手动输入的订阅内容（URI/节点列表等）同样走“生成骨架 + 注入节点”的路径。
    let config = config_generator::generate_config_with_nodes(app_config, &extracted_nodes)
        .map_err(|e| format!("生成配置失败: {}", e))?;

    info!("正在保存手动配置到: {:?}", target_path);

    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }

    let _backup = backup_existing_config(target_path);

    let config_str = serde_json::to_string_pretty(&config)?;
    let mut file = File::create(target_path)?;
    file.write_all(config_str.as_bytes())?;

    info!("手动配置已保存");
    Ok(())
}

fn process_original_config(
    content: &str,
    app_config: &AppConfig,
    target_path: &Path,
) -> Result<(), Box<dyn Error>> {
    info!("处理原始订阅配置，调整端口和地址");

    let mut config: Value = serde_json::from_str(content)?;
    apply_app_settings_to_config(&mut config, app_config);

    if let Some(config_obj) = config.as_object_mut() {
        if let Some(experimental) = config_obj.get_mut("experimental") {
            if let Some(exp_obj) = experimental.as_object_mut() {
                let clash_api = exp_obj.entry("clash_api".to_string()).or_insert(json!({}));

                if let Some(clash_api_obj) = clash_api.as_object_mut() {
                    clash_api_obj.insert(
                        "external_controller".to_string(),
                        json!(format!("127.0.0.1:{}", app_config.api_port)),
                    );

                    clash_api_obj.insert("external_ui".to_string(), json!("metacubexd"));

                    if !clash_api_obj.contains_key("default_mode") {
                        clash_api_obj.insert("default_mode".to_string(), json!("rule"));
                    }
                }
            }
        } else {
            config_obj.insert(
                "experimental".to_string(),
                json!({
                    "clash_api": {
                        "external_controller": format!("127.0.0.1:{}", app_config.api_port),
                        "external_ui": "metacubexd",
                        "default_mode": "rule"
                    }
                }),
            );
        }

        if let Some(inbounds) = config_obj.get_mut("inbounds") {
            if let Some(inbounds_array) = inbounds.as_array_mut() {
                for inbound in inbounds_array {
                    if let Some(inbound_obj) = inbound.as_object_mut() {
                        if inbound_obj.get("tag").and_then(|t| t.as_str()) == Some("mixed-in") {
                            inbound_obj
                                .insert("listen_port".to_string(), json!(app_config.proxy_port));
                        }
                    }
                }
            }
        }
    }

    info!("正在保存配置到: {:?}", target_path);

    if let Some(parent) = target_path.parent() {
        if !parent.exists() {
            info!("创建配置目录: {:?}", parent);
            if let Err(e) = std::fs::create_dir_all(parent) {
                let err_msg = format!("创建配置目录失败: {}", e);
                error!("{}", err_msg);
                return Err(err_msg.into());
            }
        }
    }

    let config_str = serde_json::to_string_pretty(&config)?;

    let _backup = backup_existing_config(target_path);

    let mut file = File::create(target_path)?;
    file.write_all(config_str.as_bytes())?;

    info!("原始订阅配置（修改端口后）已成功保存");
    Ok(())
}

/// 尝试解析并拉取 proxy-providers 中的节点列表（Clash/Mihomo 配置常见写法）。
async fn try_load_proxy_providers(raw: &str) -> Result<Vec<Value>, String> {
    let (content_json, _) = match parse_to_value(raw) {
        Ok(v) => v,
        Err(_) => return Ok(Vec::new()),
    };

    let providers = match content_json
        .get("proxy-providers")
        .or_else(|| content_json.get("proxy_providers"))
        .and_then(|v| v.as_object())
    {
        Some(p) => p,
        None => return Ok(Vec::new()),
    };

    let mut collected = Vec::new();
    for (name, provider) in providers {
        let url = match provider.get("url").and_then(|v| v.as_str()) {
            Some(u) if !u.is_empty() => u.trim(),
            _ => continue,
        };

        info!("拉取 proxy-provider: {} => {}", name, url);
        match http_client::get_text(url).await {
            Ok(body) => match extract_nodes_from_subscription(&body) {
                Ok(mut nodes) => {
                    if !nodes.is_empty() {
                        info!("proxy-provider {} 获取到 {} 个节点", name, nodes.len());
                        collected.append(&mut nodes);
                    }
                }
                Err(e) => warn!("解析 proxy-provider {} 失败: {}", name, e),
            },
            Err(e) => warn!("下载 proxy-provider {} 失败: {}", name, e),
        }
    }

    Ok(collected)
}
