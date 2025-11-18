use crate::app::constants::{config, messages, network_config, paths};
use crate::app::core::tun_profile::{TunProfile, TunProxyOptions};
use crate::app::system::config_service;
use crate::entity::config_model;
use crate::utils::app_util::get_work_dir_sync;
use crate::utils::config_util::ConfigUtil;
use crate::utils::http_client;
use crate::utils::proxy_util::{disable_system_proxy, enable_system_proxy, DEFAULT_BYPASS_LIST};
use serde_json::{json, Value};
use std::error::Error;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tauri::{Emitter, Runtime};
use tracing::{error, info, warn};

// 修改代理模式为系统代理
#[tauri::command]
pub fn set_system_proxy(port: u16, system_proxy_bypass: Option<String>) -> Result<(), String> {
    config_service::ensure_singbox_config().map_err(|e| format!("准备配置失败: {}", e))?;
    let config_path = paths::get_config_path();
    let config_path_str = config_path.to_str().ok_or("配置文件路径包含无效字符")?;

    let json_util = ConfigUtil::new(config_path_str)
        .map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))?;

    let mut json_util = json_util;
    let target_keys = vec!["inbounds"];
    let new_structs = vec![config_model::Inbound {
        r#type: config::DEFAULT_INBOUND_TYPE.to_string(),
        tag: config::DEFAULT_INBOUND_TAG.to_string(),
        listen: Some(network_config::DEFAULT_CLASH_API_ADDRESS.to_string()),
        interface_name: None,
        listen_port: Some(port),
        address: None,
        auto_route: None,
        strict_route: None,
        stack: None,
        sniff: Some(true),
        sniff_override_destination: Some(true),
        mtu: None,
        route_address: None,
        route_exclude_address: None,
        set_system_proxy: Some(true),
    }];

    json_util.update_key(
        target_keys.clone(),
        serde_json::to_value(new_structs).map_err(|e| format!("序列化配置失败: {}", e))?,
    );
    json_util
        .save_to_file()
        .map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))?;

    let bypass_value = system_proxy_bypass.filter(|b| !b.trim().is_empty());
    enable_system_proxy(
        network_config::DEFAULT_CLASH_API_ADDRESS,
        port,
        bypass_value.as_deref(),
    )
    .map_err(|e| format!("设置系统代理失败: {}", e))?;

    info!(
        "系统代理模式已启用，端口 {}，绕过列表: {}",
        port,
        bypass_value.unwrap_or_else(|| DEFAULT_BYPASS_LIST.to_string())
    );
    Ok(())
}

// 设置手动代理模式（不自动设置系统代理）
#[tauri::command]
pub fn set_manual_proxy(port: u16) -> Result<(), String> {
    config_service::ensure_singbox_config().map_err(|e| format!("准备配置失败: {}", e))?;
    let config_path = paths::get_config_path();
    let config_path_str = config_path.to_str().ok_or("配置文件路径包含无效字符")?;

    let json_util = ConfigUtil::new(config_path_str)
        .map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))?;

    let mut json_util = json_util;
    let target_keys = vec!["inbounds"];
    let new_structs = vec![config_model::Inbound {
        r#type: config::DEFAULT_INBOUND_TYPE.to_string(),
        tag: config::DEFAULT_INBOUND_TAG.to_string(),
        listen: Some(network_config::DEFAULT_CLASH_API_ADDRESS.to_string()),
        interface_name: None,
        listen_port: Some(port),
        address: None,
        auto_route: None,
        strict_route: None,
        stack: None,
        sniff: Some(true),
        sniff_override_destination: Some(true),
        mtu: None,
        route_address: None,
        route_exclude_address: None,
        set_system_proxy: Some(false),
    }];

    json_util.update_key(
        target_keys.clone(),
        serde_json::to_value(new_structs).map_err(|e| format!("序列化配置失败: {}", e))?,
    );
    json_util
        .save_to_file()
        .map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))?;

    if let Err(err) = disable_system_proxy() {
        warn!("关闭系统代理失败: {}", err);
    }

    info!("手动代理模式已启用，需要手动设置系统代理");
    Ok(())
}

// 修改TUN 模式为代理模式
#[tauri::command]
pub fn set_tun_proxy(port: u16, tun_options: Option<TunProxyOptions>) -> Result<(), String> {
    set_tun_proxy_impl(port, tun_options.unwrap_or_default())
        .map_err(|e| format!("设置TUN代理失败: {}", e))
}

fn set_tun_proxy_impl(port: u16, options: TunProxyOptions) -> Result<(), Box<dyn Error>> {
    config_service::ensure_singbox_config().map_err(|e| format!("准备配置失败: {}", e))?;
    let work_dir = get_work_dir_sync();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let path_str = path.to_str().ok_or("配置文件路径包含无效字符")?;
    let mut json_util = ConfigUtil::new(path_str)?;
    let profile = TunProfile::from_options(&options);

    let mut inbounds = profile.to_inbounds(port);
    if let Some(mixed) = inbounds.get_mut(0) {
        mixed.listen = Some(network_config::DEFAULT_CLASH_API_ADDRESS.to_string());
    }

    json_util.modify_property(
        &["inbounds"],
        serde_json::to_value(inbounds).map_err(|e| format!("序列化配置失败: {}", e))?,
    );
    json_util
        .save()
        .map_err(|e| format!("保存配置文件失败: {}", e))?;

    if let Err(err) = disable_system_proxy() {
        warn!("关闭系统代理失败: {}", err);
    }

    info!("TUN代理模式已设置");
    Ok(())
}

pub fn update_dns_strategy(prefer_ipv6: bool) -> Result<(), String> {
    let config_path = paths::get_config_path();
    let content =
        fs::read_to_string(&config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;
    let mut config: Value =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    let strategy_value = if prefer_ipv6 {
        "prefer_ipv6"
    } else {
        "ipv4_only"
    };

    let dns_object = config
        .as_object_mut()
        .and_then(|obj| obj.get_mut("dns"))
        .and_then(|dns| dns.as_object_mut())
        .ok_or_else(|| "配置文件缺少DNS配置".to_string())?;

    dns_object.insert(
        "strategy".to_string(),
        Value::String(strategy_value.to_string()),
    );

    let serialized =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_path, serialized).map_err(|e| format!("保存配置文件失败: {}", e))?;

    Ok(())
}

// 切换 IPV6版本模式
#[tauri::command]
pub fn toggle_ip_version(prefer_ipv6: bool) -> Result<(), String> {
    info!(
        "开始切换IP版本模式: {}",
        if prefer_ipv6 { "IPv6优先" } else { "仅IPv4" }
    );

    update_dns_strategy(prefer_ipv6)?;

    info!(
        "IP版本模式已成功切换为: {}",
        if prefer_ipv6 { "IPv6优先" } else { "仅IPv4" }
    );

    Ok(())
}

// 获取API令牌
#[tauri::command]
pub fn get_api_token() -> String {
    // 目前返回空字符串
    "".to_string()
}

// 获取代理节点列表
#[tauri::command]
pub async fn get_proxies(port: u16) -> Result<Value, String> {
    let url = format!("http://127.0.0.1:{}/proxies", port);

    match http_client::get_json::<Value>(&url).await {
        Ok(data) => Ok(data),
        Err(e) => {
            error!("获取代理列表失败: {}", e);
            Err(format!("获取代理列表失败: {}", e))
        }
    }
}

// 切换代理节点
#[tauri::command]
pub async fn change_proxy(group: String, proxy: String, port: u16) -> Result<(), String> {
    let url = format!("http://127.0.0.1:{}/proxies/{}", port, group);
    let data = json!({
        "name": proxy
    });

    let client = http_client::get_client();
    match client
        .put(&url)
        .json(&data)
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                info!("代理节点已切换: {} -> {}", group, proxy);
                Ok(())
            } else {
                let error_msg = format!("切换代理节点失败，HTTP状态码: {}", response.status());
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
        Err(e) => {
            let error_msg = format!("切换代理节点请求失败: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// 测试代理组延迟
#[tauri::command]
pub async fn test_group_delay<R: Runtime>(
    window: tauri::Window<R>,
    group: String,
    server: Option<String>,
    port: u16,
) -> Result<(), String> {
    let test_url = server.unwrap_or_else(|| "http://cp.cloudflare.com".to_string());
    let url = format!(
        "http://127.0.0.1:{}/group/{}/delay?timeout=5000&url={}",
        port, group, test_url
    );

    let client = http_client::get_client();
    match client
        .get(&url) // 改为GET方法
        .timeout(Duration::from_secs(10))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Value>().await {
                    Ok(result) => {
                        // 发送结果到前端
                        if let Err(e) = window.emit("proxy-group-delay-result", &result) {
                            warn!("发送延迟测试结果失败: {}", e);
                        }
                        info!("代理组 {} 延迟测试完成", group);
                        Ok(())
                    }
                    Err(e) => {
                        let error_msg = format!("解析延迟测试结果失败: {}", e);
                        error!("{}", error_msg);
                        Err(error_msg)
                    }
                }
            } else {
                let error_msg = format!("延迟测试失败，HTTP状态码: {}", response.status());
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
        Err(e) => {
            let error_msg = format!("延迟测试请求失败: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

// 获取版本信息
#[tauri::command]
pub async fn get_version_info(port: u16) -> Result<Value, String> {
    let url = format!("http://127.0.0.1:{}/version", port);

    match http_client::get_json::<Value>(&url).await {
        Ok(data) => Ok(data),
        Err(e) => {
            error!("获取版本信息失败: {}", e);
            Err(format!("获取版本信息失败: {}", e))
        }
    }
}

// 获取规则信息
#[tauri::command]
pub async fn get_rules(port: u16) -> Result<Value, String> {
    let url = format!("http://127.0.0.1:{}/rules", port);

    match http_client::get_json::<Value>(&url).await {
        Ok(data) => Ok(data),
        Err(e) => {
            error!("获取规则信息失败: {}", e);
            Err(format!("获取规则信息失败: {}", e))
        }
    }
}

// 测试单个节点延迟
#[tauri::command]
pub async fn test_node_delay<R: Runtime>(
    window: tauri::Window<R>,
    proxy: String,
    server: Option<String>,
    port: u16,
) -> Result<(), String> {
    let test_url = server.unwrap_or_else(|| "http://cp.cloudflare.com".to_string());
    let url = format!(
        "http://127.0.0.1:{}/proxies/{}/delay?timeout=5000&url={}",
        port, proxy, test_url
    );

    let client = http_client::get_client();
    match client
        .get(&url) // 改为GET方法
        .timeout(Duration::from_secs(8))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                match response.json::<Value>().await {
                    Ok(data) => {
                        let delay = data.get("delay").and_then(|d| d.as_u64()).unwrap_or(0);

                        let result = json!({
                            "proxy": proxy,
                            "delay": delay
                        });

                        // 发送结果到前端
                        if let Err(e) = window.emit("proxy-delay-result", &result) {
                            warn!("发送延迟测试结果失败: {}", e);
                        }

                        info!("节点 {} 延迟测试完成: {}ms", proxy, delay);
                        Ok(())
                    }
                    Err(e) => {
                        let error_msg = format!("解析延迟测试结果失败: {}", e);
                        error!("{}", error_msg);
                        Err(error_msg)
                    }
                }
            } else {
                let error_msg = format!("节点延迟测试失败，HTTP状态码: {}", response.status());
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
        Err(e) => {
            let error_msg = format!("节点延迟测试请求失败: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}
