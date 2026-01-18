pub mod helpers;
mod mode;
mod parser;
pub mod auto_update;

use crate::app::constants::{messages, paths};
use crate::app::core::kernel_auto_manage::auto_manage_with_saved_config;
use crate::app::core::proxy_service::apply_proxy_runtime_state;
use crate::app::singbox::config_generator;
use crate::app::singbox::settings_patch::apply_port_settings_only;
use crate::app::storage::enhanced_storage_service::{
    db_get_app_config, db_get_subscriptions, db_save_app_config, db_save_subscriptions,
};
use crate::app::storage::state_model::AppConfig;
use crate::utils::http_client;
use base64::{Engine as _, engine::general_purpose};
use helpers::{
    backup_existing_config, resolve_target_config_path, runtime_state_from_config,
};
use parser::extract_nodes_from_subscription;
use reqwest::header::HeaderMap;
use serde::Serialize;
use serde_json::Value;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use tauri::{AppHandle, Manager};
use tracing::{error, info, warn};

/// 尝试把订阅内容当作 Base64 解码成 UTF-8 文本。
///
/// 说明：不少机场会把 Clash YAML / URI 列表做一次 Base64 封装，且可能包含换行。
fn try_decode_base64_to_text(raw: &str) -> Option<String> {
    let mut s: String = raw.split_whitespace().collect();
    if s.is_empty() {
        return None;
    }

    // 补齐 padding，兼容省略 '=' 的情况
    let rem = s.len() % 4;
    if rem != 0 {
        s.push_str(&"=".repeat(4 - rem));
    }

    let bytes = general_purpose::STANDARD
        .decode(&s)
        .or_else(|_| general_purpose::URL_SAFE.decode(&s))
        .ok()?;
    String::from_utf8(bytes).ok()
}

#[derive(Debug, Clone, Serialize)]
pub struct SubscriptionPersistResult {
    pub config_path: String,
    pub subscription_upload: Option<u64>,
    pub subscription_download: Option<u64>,
    pub subscription_total: Option<u64>,
    pub subscription_expire: Option<u64>,
}

#[derive(Debug, Clone)]
struct SubscriptionUserInfo {
    upload: Option<u64>,
    download: Option<u64>,
    total: Option<u64>,
    expire: Option<u64>,
}

fn parse_subscription_userinfo(raw: &str) -> Option<SubscriptionUserInfo> {
    let mut info = SubscriptionUserInfo {
        upload: None,
        download: None,
        total: None,
        expire: None,
    };

    let mut has_value = false;
    for segment in raw.split(';') {
        let segment = segment.trim();
        if segment.is_empty() {
            continue;
        }
        let (key, value) = match segment.split_once('=') {
            Some(pair) => pair,
            None => continue,
        };

        let value = value.trim().parse::<u64>().ok();
        match key.trim().to_ascii_lowercase().as_str() {
            "upload" => {
                info.upload = value;
                has_value = true;
            }
            "download" => {
                info.download = value;
                has_value = true;
            }
            "total" => {
                info.total = value;
                has_value = true;
            }
            "expire" => {
                info.expire = value;
                has_value = true;
            }
            _ => {}
        }
    }

    if has_value { Some(info) } else { None }
}

fn extract_subscription_userinfo(headers: &HeaderMap) -> Option<SubscriptionUserInfo> {
    let header = headers
        .get("subscription-userinfo")
        .or_else(|| headers.get("Subscription-Userinfo"))?;
    let raw = header.to_str().ok()?;
    parse_subscription_userinfo(raw)
}

async fn fetch_subscription_content(
    url: &str,
) -> Result<(String, Option<SubscriptionUserInfo>), Box<dyn Error>> {
    let response = http_client::get_client().get(url).send().await?;
    response.error_for_status_ref()?;
    let headers = response.headers().clone();
    let body = response.text().await?;
    let userinfo = extract_subscription_userinfo(&headers);
    Ok((body, userinfo))
}

async fn update_subscription_userinfo(
    app_handle: &AppHandle,
    target_path: &Path,
    url: &str,
    userinfo: Option<SubscriptionUserInfo>,
) -> Result<(), String> {
    let mut subscriptions = db_get_subscriptions(app_handle.clone())
        .await
        .map_err(|e| format!("读取订阅配置失败: {}", e))?;

    let trimmed_url = url.trim();
    let target_path = target_path.to_string_lossy();
    let now_ms = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_err(|e| format!("获取时间失败: {}", e))?
        .as_millis() as u64;

    let mut updated = false;
    for sub in subscriptions.iter_mut() {
        let path_match = sub
            .config_path
            .as_deref()
            .map(|path| path == target_path.as_ref())
            .unwrap_or(false);
        let url_match = !trimmed_url.is_empty() && sub.url.trim() == trimmed_url;

        if path_match || url_match {
            sub.last_update = Some(now_ms);
            if let Some(info) = &userinfo {
                sub.subscription_upload = info.upload;
                sub.subscription_download = info.download;
                sub.subscription_total = info.total;
                sub.subscription_expire = info.expire;
            } else {
                sub.subscription_upload = None;
                sub.subscription_download = None;
                sub.subscription_total = None;
                sub.subscription_expire = None;
            }
            updated = true;
        }
    }

    if updated {
        db_save_subscriptions(subscriptions, app_handle.clone())
            .await
            .map_err(|e| format!("保存订阅配置失败: {}", e))?;
    }

    Ok(())
}

#[tauri::command]
#[allow(clippy::too_many_arguments)] // Tauri 接口需与前端参数保持一致
pub async fn download_subscription(
    url: String,
    use_original_config: bool,
    file_name: Option<String>,
    config_path: Option<String>,
    apply_runtime: Option<bool>,
    window: tauri::Window,
    proxy_port: Option<u16>,
    api_port: Option<u16>,
) -> Result<SubscriptionPersistResult, String> {
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
    let trimmed_url = url.trim();
    let userinfo = download_and_process_subscription(
        trimmed_url,
        use_original_config,
        app_handle,
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
        if let Err(e) = apply_proxy_runtime_state(app_handle, &runtime_state).await {
            warn!("应用代理配置失败: {}", e);
        }
        auto_manage_with_saved_config(app_handle, true, "subscription-download").await;
    }

    if let Err(e) = update_subscription_userinfo(
        &app_handle,
        &target_path,
        trimmed_url,
        userinfo.clone(),
    )
    .await
    {
        warn!("同步订阅信息失败: {}", e);
    }

    Ok(SubscriptionPersistResult {
        config_path: target_path.to_string_lossy().to_string(),
        subscription_upload: userinfo.as_ref().and_then(|info| info.upload),
        subscription_download: userinfo.as_ref().and_then(|info| info.download),
        subscription_total: userinfo.as_ref().and_then(|info| info.total),
        subscription_expire: userinfo.as_ref().and_then(|info| info.expire),
    })
}

#[tauri::command]
#[allow(clippy::too_many_arguments)] // Tauri 接口需与前端参数保持一致
pub async fn add_manual_subscription(
    content: String,
    use_original_config: bool,
    file_name: Option<String>,
    config_path: Option<String>,
    apply_runtime: Option<bool>,
    window: tauri::Window,
    proxy_port: Option<u16>,
    api_port: Option<u16>,
) -> Result<SubscriptionPersistResult, String> {
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
        app_handle,
        &app_config,
        &target_path,
    )
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
        if let Err(e) = apply_proxy_runtime_state(app_handle, &runtime_state).await {
            warn!("应用代理配置失败: {}", e);
        }
        auto_manage_with_saved_config(app_handle, true, "subscription-manual").await;
    }

    Ok(SubscriptionPersistResult {
        config_path: target_path.to_string_lossy().to_string(),
        subscription_upload: None,
        subscription_download: None,
        subscription_total: None,
        subscription_expire: None,
    })
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
    url: &str,
    use_original_config: bool,
    _app_handle: &AppHandle,
    app_config: &AppConfig,
    target_path: &Path,
) -> Result<Option<SubscriptionUserInfo>, Box<dyn Error>> {
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

    let (response_text, userinfo) = fetch_subscription_content(url)
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_SUBSCRIPTION_FAILED, e))?;

    info!("订阅下载成功，内容长度: {} 字节", response_text.len());

    if use_original_config {
        info!("使用原始订阅内容，仅修改必要的端口和地址");
        process_original_config(&response_text, app_config, target_path)?;
        return Ok(userinfo);
    }

    let mut extracted_nodes = extract_nodes_from_subscription(&response_text)?;
    info!("从原始内容提取到 {} 个节点", extracted_nodes.len());

    if extracted_nodes.is_empty() {
        info!("未从原始内容提取到节点，尝试base64解码...");

        if let Some(decoded_text) = try_decode_base64_to_text(&response_text) {
            info!("base64 解码成功，重新从解码内容提取节点...");
            extracted_nodes = extract_nodes_from_subscription(&decoded_text)?;
            info!("从 base64 解码内容提取到 {} 个节点", extracted_nodes.len());
        }
    }

    if extracted_nodes.is_empty() {
        info!("标准解码方法均未提取到节点，尝试移除前缀后再解码...");

        let stripped_text = response_text
            .trim()
            .replace("vmess://", "")
            .replace("ss://", "")
            .replace("trojan://", "")
            .replace("vless://", "")
            .replace("hysteria2://", "");

        if let Ok(decoded) = general_purpose::STANDARD.decode(&stripped_text) {
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
        return Err(
            "无法从订阅内容提取节点信息（支持 sing-box JSON / Clash YAML / URI 列表，且可 base64 封装），请检查订阅链接或内容格式"
                .into(),
        );
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
    Ok(userinfo)
}

fn process_subscription_content(
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
        if let Some(decoded_text) = try_decode_base64_to_text(&content) {
            info!("手动内容 base64 解码成功，重新提取节点");
            extracted_nodes = extract_nodes_from_subscription(&decoded_text)?;
            info!("从解码内容提取到 {} 个节点", extracted_nodes.len());
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
    info!("处理原始订阅配置，仅调整端口");

    let mut config: Value = serde_json::from_str(content)?;
    apply_port_settings_only(&mut config, app_config);

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

#[cfg(test)]
mod tests {
    use super::{try_decode_base64_to_text, extract_nodes_from_subscription};
    use base64::{engine::general_purpose, Engine as _};

    #[test]
    fn base64_uri_list_should_extract_nodes_after_decode() {
        let uri_list = "trojan://password@example.com:443#test\nvless://uuid@example.com:443?security=tls&sni=example.com#vless\n";
        let b64 = general_purpose::STANDARD.encode(uri_list.as_bytes());

        let decoded = try_decode_base64_to_text(&b64).expect("decode should work");
        let nodes = extract_nodes_from_subscription(&decoded).expect("extract should work");
        assert_eq!(nodes.len(), 2);
    }
}
