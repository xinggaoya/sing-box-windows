use crate::app::constants::{config, messages, network_config, paths};
use crate::entity::config_model;
use crate::utils::app_util::get_work_dir;
use crate::utils::config_util::ConfigUtil;
use crate::utils::http_client;
use serde_json::{json, Value};
use std::error::Error;
use std::path::Path;
use std::time::Duration;
use tauri::{Emitter, Runtime};
use tracing::{error, info, warn};

// 修改代理模式为系统代理
#[tauri::command]
pub fn set_system_proxy(port: u16) -> Result<(), String> {
    let config_path = paths::get_config_path();
    let config_path_str = config_path.to_str().ok_or("配置文件路径包含无效字符")?;

    let json_util = ConfigUtil::new(config_path_str)
        .map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))?;

    let mut json_util = json_util;
    let target_keys = vec!["inbounds"];
    let new_structs = vec![config_model::Inbound {
        r#type: config::DEFAULT_INBOUND_TYPE.to_string(),
        tag: config::DEFAULT_INBOUND_TAG.to_string(),
        listen: Some(network_config::DEFAULT_LISTEN_ADDRESS.to_string()),
        listen_port: Some(port),
        address: None,
        auto_route: None,
        strict_route: None,
        stack: None,
        sniff: None,
        set_system_proxy: Some(true),
    }];

    json_util.update_key(
        target_keys.clone(),
        serde_json::to_value(new_structs).map_err(|e| format!("序列化配置失败: {}", e))?,
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
pub fn set_manual_proxy(port: u16) -> Result<(), String> {
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
        listen_port: Some(port),
        address: None,
        auto_route: None,
        strict_route: None,
        stack: None,
        sniff: None,
        set_system_proxy: Some(false),
    }];

    json_util.update_key(
        target_keys.clone(),
        serde_json::to_value(new_structs).map_err(|e| format!("序列化配置失败: {}", e))?,
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
pub fn set_tun_proxy(port: u16) -> Result<(), String> {
    set_tun_proxy_impl(port).map_err(|e| format!("设置TUN代理失败: {}", e))
}

fn set_tun_proxy_impl(port: u16) -> Result<(), Box<dyn Error>> {
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let path_str = path.to_str().ok_or("配置文件路径包含无效字符")?;
    let mut json_util = ConfigUtil::new(path_str)?;

    let target_keys = vec!["inbounds"]; // 修改为你的属性路径
    let new_structs = vec![
        config_model::Inbound {
            r#type: "mixed".to_string(),
            tag: "mixed-in".to_string(),
            listen: Some(network_config::DEFAULT_CLASH_API_ADDRESS.to_string()),
            listen_port: Some(port),
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
