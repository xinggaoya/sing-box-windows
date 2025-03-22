use crate::entity::config_model;
use crate::utils::app_util::get_work_dir;
use crate::utils::config_util::ConfigUtil;
use std::error::Error;
use std::path::Path;
use tracing::info;
use crate::app::constants::{paths, network, config as config_constants, messages};
use serde_json::{json, Value};
use reqwest::Client;
use tauri::{Runtime, Emitter};

// 修改代理模式为系统代理
#[tauri::command]
pub fn set_system_proxy() -> Result<(), String> {
    let config_path = paths::get_config_path();
    let json_util =
        ConfigUtil::new(config_path.to_str().unwrap()).map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))?;

    let mut json_util = json_util;
    let target_keys = vec!["inbounds"];
    let new_structs = vec![config_model::Inbound {
        r#type: config_constants::DEFAULT_INBOUND_TYPE.to_string(),
        tag: config_constants::DEFAULT_INBOUND_TAG.to_string(),
        listen: Some(network::DEFAULT_LISTEN_ADDRESS.to_string()),
        listen_port: Some(network::DEFAULT_PROXY_PORT),
        address: None,
        auto_route: None,
        strict_route: None,
        stack: None,
        sniff: None,
        set_system_proxy: Some(true),
    }];

    json_util.update_key(target_keys.clone(), serde_json::to_value(new_structs).unwrap());
    match json_util.save_to_file() {
        Ok(_) => {
            info!("{}", messages::INFO_PROXY_MODE_ENABLED);
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

    // 直接替换字符串
    let modified_content = if prefer_ipv6 {
        content.replace("\"ipv4_only\"", "\"prefer_ipv6\"")
    } else {
        content.replace("\"prefer_ipv6\"", "\"ipv4_only\"")
    };

    // 验证修改后的内容是否是有效的 JSON
    serde_json::from_str::<serde_json::Value>(&modified_content)
        .map_err(|e| format!("修改后的配置不是有效的 JSON: {}", e))?;

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
    network::DEFAULT_API_TOKEN.to_string()
}

/// 获取代理列表
#[tauri::command]
pub async fn get_proxies() -> Result<Value, String> {
    let token = get_api_token();
    let url = format!("http://{}:{}/proxies?token={}", 
        network::DEFAULT_CLASH_API_ADDRESS, 
        network::DEFAULT_CLASH_API_PORT,
        token);
    
    // 创建禁用代理的HTTP客户端
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    
    // 发送请求并获取响应
    let response = client.get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("获取代理列表失败: {}", e))?;
    
    // 解析响应为JSON
    let json = response.json::<Value>()
        .await
        .map_err(|e| format!("解析代理列表失败: {}", e))?;
    
    Ok(json)
}

/// 切换代理
#[tauri::command]
pub async fn change_proxy(group: String, proxy: String) -> Result<(), String> {
    let token = get_api_token();
    let url = format!("http://{}:{}/proxies/{}?token={}", 
        network::DEFAULT_CLASH_API_ADDRESS, 
        network::DEFAULT_CLASH_API_PORT, 
        group, 
        token);
    
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
    let response = client.put(&url)
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

/// 测试节点延迟
#[tauri::command]
pub async fn test_node_delay(name: String) -> Result<u64, String> {
    // 使用默认测试URL
    let test_url = "https://www.gstatic.com/generate_204";
    let token = get_api_token();
    let url = format!("http://{}:{}/proxies/{}/delay?url={}&timeout=5000&token={}", 
        network::DEFAULT_CLASH_API_ADDRESS, 
        network::DEFAULT_CLASH_API_PORT, 
        name, 
        urlencoding::encode(test_url),
        token);
    
    // 创建禁用代理的HTTP客户端
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    
    // 发送请求并获取响应
    let response = client.get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("测试节点延迟失败: {}", e))?;
    
    // 解析响应为JSON
    let json = response.json::<Value>()
        .await
        .map_err(|e| format!("解析测试结果失败: {}", e))?;
    
    // 获取延迟值
    let delay = json["delay"]
        .as_u64()
        .unwrap_or(0);
    
    Ok(delay)
}

/// 批量测试节点延迟
#[tauri::command]
pub async fn batch_test_nodes<R: Runtime>(
    window: tauri::Window<R>,
    nodes: Vec<String>, 
    server: Option<String>
) -> Result<(), String> {
    // 使用默认测试URL或指定的URL
    let test_url = server.unwrap_or_else(|| "https://www.gstatic.com/generate_204".to_string());
    let token = get_api_token();
    
    // 创建禁用代理的HTTP客户端
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    
    // 遍历节点列表进行测试
    for (index, name) in nodes.iter().enumerate() {
        // 构建请求URL
        let url = format!("http://{}:{}/proxies/{}/delay?url={}&timeout=5000&token={}", 
            network::DEFAULT_CLASH_API_ADDRESS, 
            network::DEFAULT_CLASH_API_PORT,
            name,
            urlencoding::encode(&test_url),
            token);
        
        // 发送进度事件
        let _ = window.emit("test-nodes-progress", json!({
            "current": index + 1,
            "total": nodes.len(),
            "node": name,
            "status": "testing"
        }));
        
        // 发送请求并获取结果
        match client.get(&url)
            .header("Accept", "application/json")
            .header("Content-Type", "application/json")
            .send().await {
            Ok(response) => {
                match response.json::<Value>().await {
                    Ok(data) => {
                        // 发送测试结果事件
                        let _ = window.emit("test-node-result", json!({
                            "name": name,
                            "delay": data["delay"],
                            "success": true
                        }));
                    },
                    Err(e) => {
                        // 发送测试失败事件
                        let _ = window.emit("test-node-result", json!({
                            "name": name,
                            "delay": 0,
                            "success": false,
                            "error": format!("解析结果失败: {}", e)
                        }));
                    }
                }
            },
            Err(e) => {
                // 发送测试失败事件
                let _ = window.emit("test-node-result", json!({
                    "name": name,
                    "delay": 0,
                    "success": false,
                    "error": format!("请求失败: {}", e)
                }));
            }
        }
        
        // 短暂延迟以避免过快发送请求
        tokio::time::sleep(tokio::time::Duration::from_millis(200)).await;
    }
    
    // 发送测试完成事件
    let _ = window.emit("test-nodes-complete", json!({
        "total": nodes.len()
    }));
    
    Ok(())
}

/// 获取内核版本信息
#[tauri::command]
pub async fn get_version_info() -> Result<Value, String> {
    let token = get_api_token();
    let url = format!("http://{}:{}/version?token={}", 
        network::DEFAULT_CLASH_API_ADDRESS, 
        network::DEFAULT_CLASH_API_PORT,
        token);
    
    // 创建禁用代理的HTTP客户端
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
    
    // 发送请求并获取响应
    let response = client.get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("获取版本信息失败: {}", e))?;
    
    // 解析响应为JSON
    let json = response.json::<Value>()
        .await
        .map_err(|e| format!("解析版本信息失败: {}", e))?;
    
    Ok(json)
}

/// 获取规则列表
#[tauri::command]
pub async fn get_rules() -> Result<Value, String> {
    let token = get_api_token();
    let url = format!("http://{}:{}/rules?token={}", 
        network::DEFAULT_CLASH_API_ADDRESS, 
        network::DEFAULT_CLASH_API_PORT,
        token);
    
    // 创建禁用代理的HTTP客户端
    let client = Client::builder()
        .no_proxy()
        .build()
        .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;

    info!("获取规则列表{}", url);
    // 发送请求并获取响应
    let response = client.get(&url)
        .header("Accept", "application/json")
        .header("Content-Type", "application/json")
        .send()
        .await
        .map_err(|e| format!("获取规则列表失败: {}", e))?;
    
    // 解析响应为JSON
    let json = response.json::<Value>()
        .await
        .map_err(|e| format!("解析规则列表失败: {}", e))?;
    
    Ok(json)
} 