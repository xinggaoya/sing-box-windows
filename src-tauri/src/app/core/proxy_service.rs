use crate::app::constants::{config, messages, network_config, paths};
use crate::entity::config_model;
use crate::utils::app_util::get_work_dir;
use crate::utils::config_util::ConfigUtil;
use reqwest::Client;
use serde_json::{json, Value};
use std::error::Error;
use std::path::Path;
use tauri::{Emitter, Runtime};
use tracing::info;

// 修改代理模式为系统代理
#[tauri::command]
pub fn set_system_proxy() -> Result<(), String> {
    let config_path = paths::get_config_path();
    let json_util = ConfigUtil::new(config_path.to_str().unwrap())
        .map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))?;

    let mut json_util = json_util;
    let target_keys = vec!["inbounds"];
    let new_structs = vec![config_model::Inbound {
        r#type: config::DEFAULT_INBOUND_TYPE.to_string(),
        tag: config::DEFAULT_INBOUND_TAG.to_string(),
        listen: Some(network_config::DEFAULT_LISTEN_ADDRESS.to_string()),
        listen_port: Some(network_config::DEFAULT_PROXY_PORT),
        address: None,
        auto_route: None,
        strict_route: None,
        stack: None,
        sniff: None,
        set_system_proxy: Some(true),
    }];

    json_util.update_key(
        target_keys.clone(),
        serde_json::to_value(new_structs).unwrap(),
    );
    match json_util.save_to_file() {
        Ok(_) => {
            info!("{}", messages::INFO_PROXY_MODE_ENABLED);
            Ok(())
        }
        Err(e) => Err(format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e)),
    }
}

// 设置手动代理模式（不自动设置系统代理）
#[tauri::command]
pub fn set_manual_proxy() -> Result<(), String> {
    let config_path = paths::get_config_path();
    let json_util = ConfigUtil::new(config_path.to_str().unwrap())
        .map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))?;

    let mut json_util = json_util;
    let target_keys = vec!["inbounds"];
    let new_structs = vec![config_model::Inbound {
        r#type: config::DEFAULT_INBOUND_TYPE.to_string(),
        tag: config::DEFAULT_INBOUND_TAG.to_string(),
        listen: Some(network_config::DEFAULT_LISTEN_ADDRESS.to_string()),
        listen_port: Some(network_config::DEFAULT_PROXY_PORT),
        address: None,
        auto_route: None,
        strict_route: None,
        stack: None,
        sniff: None,
        set_system_proxy: Some(false),
    }];

    json_util.update_key(
        target_keys.clone(),
        serde_json::to_value(new_structs).unwrap(),
    );
    match json_util.save_to_file() {
        Ok(_) => {
            info!("手动代理模式已启用，需要手动设置系统代理");
            Ok(())
        }
        Err(e) => Err(format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e)),
    }
}

// 修改TUN 模式为代理模式
#[tauri::command]
pub fn set_tun_proxy() -> Result<(), String> {
    set_tun_proxy_impl().map_err(|e| format!("设置TUN代理失败: {}", e))
}

fn set_tun_proxy_impl() -> Result<(), Box<dyn Error>> {
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let mut json_util = ConfigUtil::new(path.to_str().unwrap())?;

    let target_keys = vec!["inbounds"]; // 修改为你的属性路径
    let new_structs = vec![
        config_model::Inbound {
            r#type: "mixed".to_string(),
            tag: "mixed-in".to_string(),
            listen: Some("0.0.0.0".to_string()),
            listen_port: Some(2080),
            address: None,
            auto_route: None,
            strict_route: None,
            stack: None,
            sniff: None,
            set_system_proxy: None,
        },
        config_model::Inbound {
            r#type: "tun".to_string(),
            tag: "tun-in".to_string(),
            listen: None,
            listen_port: None,
            address: Some(vec![
                "172.18.0.1/30".to_string(),
                "fdfe:dcba:9876::1/126".to_string(),
            ]),
            auto_route: Some(true),
            strict_route: Some(true),
            stack: Some("mixed".to_string()),
            sniff: None,
            set_system_proxy: None,
        },
    ];

    json_util.modify_property(
        &target_keys,
        serde_json::to_value(new_structs).map_err(|e| format!("序列化配置失败: {}", e))?,
    );
    json_util
        .save()
        .map_err(|e| format!("保存配置文件失败: {}", e))?;

    info!("TUN代理模式已设置");
    Ok(())
}

// 切换 IPV6版本模式
#[tauri::command]
pub fn toggle_ip_version(prefer_ipv6: bool) -> Result<(), String> {
    info!(
        "开始切换IP版本模式: {}",
        if prefer_ipv6 { "IPv6优先" } else { "仅IPv4" }
    );

    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");

    // 读取文件内容
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 先尝试解析为JSON以验证是否有效
    let json_config: Value =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 检查DNS配置结构是否存在
    if !json_config
        .get("dns")
        .and_then(|dns| dns.get("servers"))
        .is_some()
    {
        return Err("配置文件缺少DNS服务器配置".to_string());
    }

    // 对服务器配置中的每个策略进行修改
    let mut modified_content = content;

    // 识别现有策略和目标策略
    let current_strategy = if prefer_ipv6 {
        "ipv4_only"
    } else {
        "prefer_ipv6"
    };
    let target_strategy = if prefer_ipv6 {
        "prefer_ipv6"
    } else {
        "ipv4_only"
    };

    // 替换策略字符串，但确保是在正确的上下文中
    // 通过查找 "strategy": "current_strategy" 模式来定位
    let pattern = format!("\"strategy\": \"{}\"", current_strategy);
    let replacement = format!("\"strategy\": \"{}\"", target_strategy);

    // 执行替换
    modified_content = modified_content.replace(&pattern, &replacement);

    // 验证修改后的内容仍然是有效的JSON
    let _: Value = serde_json::from_str(&modified_content)
        .map_err(|e| format!("修改后的配置不是有效的JSON: {}", e))?;

    // 保存修改后的内容
    std::fs::write(&path, modified_content).map_err(|e| format!("保存配置文件失败: {}", e))?;

    info!(
        "IP版本模式已成功切换为: {}",
        if prefer_ipv6 { "IPv6优先" } else { "仅IPv4" }
    );

    Ok(())
}

// 获取API Token
#[tauri::command]
pub fn get_api_token() -> String {
    network_config::DEFAULT_API_TOKEN.to_string()
}

/// 获取代理列表
#[tauri::command]
pub async fn get_proxies() -> Result<Value, String> {
    let token = get_api_token();
    let url = format!(
        "http://{}:{}/proxies?token={}",
        network_config::DEFAULT_CLASH_API_ADDRESS,
        network_config::DEFAULT_CLASH_API_PORT,
        token
    );

    // 创建禁用代理的HTTP客户端
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    // 发送请求并获取响应
    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("获取代理列表失败: {}", e))?;

    // 解析响应为JSON
    let json = response
        .json::<Value>()
        .await
        .map_err(|e| format!("解析代理列表失败: {}", e))?;

    Ok(json)
}

/// 切换代理
#[tauri::command]
pub async fn change_proxy(group: String, proxy: String) -> Result<(), String> {
    let token = get_api_token();
    let url = format!(
        "http://{}:{}/proxies/{}?token={}",
        network_config::DEFAULT_CLASH_API_ADDRESS,
        network_config::DEFAULT_CLASH_API_PORT,
        group,
        token
    );

    // 创建禁用代理的HTTP客户端
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    // 请求体
    let payload = json!({
        "name": proxy
    });

    // 发送请求并获取响应
    let response = client
        .put(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .json(&payload)
        .send()
        .await
        .map_err(|e| format!("切换代理失败: {}", e))?;

    // 检查响应状态
    if !response.status().is_success() {
        return Err(format!("服务器返回错误: {}", response.status()));
    }

    Ok(())
}

/// 测试节点组延迟
#[tauri::command]
pub async fn test_group_delay<R: Runtime>(
    window: tauri::Window<R>,
    group: String,
    server: Option<String>,
) -> Result<(), String> {
    // 使用默认测试URL或指定的URL
    let test_url = server.unwrap_or_else(|| "https://www.gstatic.com/generate_204".to_string());
    let token = get_api_token();

    // 构建请求URL
    let url = format!(
        "http://{}:{}/group/{}/delay?url={}&timeout=2000&token={}",
        network_config::DEFAULT_CLASH_API_ADDRESS,
        network_config::DEFAULT_CLASH_API_PORT,
        urlencoding::encode(&group),
        urlencoding::encode(&test_url),
        token
    );

    // 创建禁用代理的HTTP客户端
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    // 发送请求并获取结果
    match client
        .get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<Value>().await {
                Ok(data) => {
                    // 发送测试结果事件
                    let _ = window.emit(
                        "test-group-result",
                        json!({
                            "group": group,
                            "results": data,
                            "success": true
                        }),
                    );
                }
                Err(e) => {
                    // 发送测试失败事件
                    let _ = window.emit(
                        "test-group-result",
                        json!({
                            "group": group,
                            "success": false,
                            "error": format!("解析结果失败: {}", e)
                        }),
                    );
                }
            }
        }
        Err(e) => {
            // 发送测试失败事件
            let _ = window.emit(
                "test-group-result",
                json!({
                    "group": group,
                    "success": false,
                    "error": format!("请求失败: {}", e)
                }),
            );
        }
    }

    Ok(())
}

/// 获取内核版本信息
#[tauri::command]
pub async fn get_version_info() -> Result<Value, String> {
    let token = get_api_token();
    let url = format!(
        "http://{}:{}/version?token={}",
        network_config::DEFAULT_CLASH_API_ADDRESS,
        network_config::DEFAULT_CLASH_API_PORT,
        token
    );

    // 创建禁用代理的HTTP客户端
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    // 发送请求并获取响应
    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("获取版本信息失败: {}", e))?;

    // 解析响应为JSON
    let json = response
        .json::<Value>()
        .await
        .map_err(|e| format!("解析版本信息失败: {}", e))?;

    Ok(json)
}

/// 获取规则列表
#[tauri::command]
pub async fn get_rules() -> Result<Value, String> {
    let token = get_api_token();
    let url = format!(
        "http://{}:{}/rules?token={}",
        network_config::DEFAULT_CLASH_API_ADDRESS,
        network_config::DEFAULT_CLASH_API_PORT,
        token
    );

    // 创建禁用代理的HTTP客户端
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    info!("获取规则列表{}", url);
    // 发送请求并获取响应
    let response = client
        .get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("获取规则列表失败: {}", e))?;

    // 解析响应为JSON
    let json = response
        .json::<Value>()
        .await
        .map_err(|e| format!("解析规则列表失败: {}", e))?;

    Ok(json)
}

/// 测试单个节点延迟
#[tauri::command]
pub async fn test_node_delay<R: Runtime>(
    window: tauri::Window<R>,
    proxy: String,
    server: Option<String>,
) -> Result<(), String> {
    // 使用默认测试URL或指定的URL
    let test_url = server.unwrap_or_else(|| "https://www.gstatic.com/generate_204".to_string());
    let token = get_api_token();

    // 构建请求URL
    let url = format!(
        "http://{}:{}/proxies/{}/delay?url={}&timeout=5000&token={}",
        network_config::DEFAULT_CLASH_API_ADDRESS,
        network_config::DEFAULT_CLASH_API_PORT,
        urlencoding::encode(&proxy),
        urlencoding::encode(&test_url),
        token
    );

    // 创建禁用代理的HTTP客户端
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    // 发送请求并获取结果
    match client
        .get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
    {
        Ok(response) => {
            match response.json::<Value>().await {
                Ok(data) => {
                    let delay = data.get("delay").and_then(|d| d.as_u64()).unwrap_or(0);

                    // 发送测试结果事件
                    let _ = window.emit(
                        "test-node-result",
                        json!({
                            "proxy": proxy,
                            "delay": delay,
                            "success": true
                        }),
                    );
                }
                Err(e) => {
                    // 发送测试失败事件
                    let _ = window.emit(
                        "test-node-result",
                        json!({
                            "proxy": proxy,
                            "success": false,
                            "error": format!("解析结果失败: {}", e)
                        }),
                    );
                }
            }
        }
        Err(e) => {
            // 发送测试失败事件
            let _ = window.emit(
                "test-node-result",
                json!({
                    "proxy": proxy,
                    "success": false,
                    "error": format!("请求失败: {}", e)
                }),
            );
        }
    }

    Ok(())
}
