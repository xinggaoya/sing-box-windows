use crate::entity::config_model::{CacheFileConfig, ClashApiConfig, Config};
use crate::app::constants::{paths, messages, network};
use crate::utils::config_util::ConfigUtil;
use crate::utils::app_util::{get_work_dir, get_template_path};
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::Path;
use tracing::{info, error};
use base64;
use serde_json::{json, Value};

// 下载订阅
#[tauri::command]
pub async fn download_subscription(url: String) -> Result<(), String> {
    download_and_process_subscription(url)
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_SUBSCRIPTION_FAILED, e))?;
    let _ = crate::app::proxy_service::set_system_proxy();
    Ok(())
}

// 手动添加订阅内容
#[tauri::command]
pub async fn add_manual_subscription(content: String) -> Result<(), String> {
    process_subscription_content(content)
        .map_err(|e| format!("{}: {}", messages::ERR_PROCESS_SUBSCRIPTION_FAILED, e))?;
    let _ = crate::app::proxy_service::set_system_proxy();
    Ok(())
}

// 获取当前配置文件内容
#[tauri::command]
pub fn get_current_config() -> Result<String, String> {
    let config_path = paths::get_config_path();
    
    // 检查文件是否存在
    if !config_path.exists() {
        return Err(messages::ERR_CONFIG_READ_FAILED.to_string());
    }
    
    // 读取文件内容
    match std::fs::read_to_string(config_path) {
        Ok(content) => Ok(content),
        Err(e) => Err(format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e)),
    }
}

// 切换代理模式（global、rule或tun）
#[tauri::command]
pub fn toggle_proxy_mode(mode: String) -> Result<String, String> {
    // 验证模式参数
    if !["global", "rule", "tun"].contains(&mode.as_str()) {
        return Err(format!("无效的代理模式: {}", mode));
    }
    
    info!("正在切换代理模式为: {}", mode);
    
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    
    // 检查文件是否存在
    if !path.exists() {
        return Err("配置文件不存在，请先添加订阅".to_string());
    }
    
    // 修改配置文件
    match modify_default_mode(&path, mode.clone()) {
        Ok(_) => {
            info!("代理模式已切换为: {}", mode);
            Ok(format!("代理模式已切换为: {}", mode))
        },
        Err(e) => {
            error!("切换代理模式失败: {}", e);
            Err(format!("切换代理模式失败: {}", e))
        }
    }
}

// 修改配置文件中的default_mode
fn modify_default_mode(config_path: &Path, mode: String) -> Result<(), Box<dyn Error>> {
    // 读取现有配置
    let mut json_util = ConfigUtil::new(config_path.to_str().unwrap())?;
    
    // 我们不使用get_value方法，因为它不存在
    // 而是直接创建新的配置并修改
    let target_keys = vec!["experimental"];
    
    // 创建新的配置，设置mode
    let config = Config {
        clash_api: ClashApiConfig {
            external_controller: format!("127.0.0.1:{}", network::DEFAULT_CLASH_API_PORT),
            external_ui: "metacubexd".to_string(),
            external_ui_download_url: "".to_string(),
            external_ui_download_detour: "手动切换".to_string(),
            default_mode: mode, // 设置为传入的模式
        },
        cache_file: CacheFileConfig { enabled: true },
    };
    
    // 更新配置
    json_util.modify_property(&target_keys, serde_json::to_value(config)?);
    json_util.save()?;
    
    Ok(())
}

async fn download_and_process_subscription(url: String) -> Result<(), Box<dyn Error>> {
    let client = reqwest::Client::new();
    let mut headers = reqwest::header::HeaderMap::new();
    let user_agent = reqwest::header::HeaderValue::from_static("sing-box-windows/1.0 (sing-box; compatible; Windows NT 10.0)");
    headers.insert(reqwest::header::USER_AGENT, user_agent);

    let response = client.get(url).headers(headers).send().await?;
    let response_text = response.text().await?;

    // 检查内容是否为base64编码，并在需要时进行解码
    let text = if is_base64_encoded(&response_text) {
        info!("检测到base64编码内容，正在解码...");
        let decoded = match base64::decode(&response_text.trim()) {
            Ok(data) => data,
            Err(_) => {
                // 如果标准解码失败，尝试URL安全的base64变体
                base64::decode_config(&response_text.trim(), base64::URL_SAFE)
                    .map_err(|e| format!("Base64解码失败: {}", e))?
            }
        };
        
        // 尝试将解码后的内容解析为有效的UTF-8字符串
        match String::from_utf8(decoded.clone()) {
            Ok(s) => {
                // 检查解码后的内容是否是有效的JSON或配置格式
                if s.trim_start().starts_with('{') || s.contains("proxies:") {
                    s // 返回解码后的文本
                } else {
                    // 解码后的内容不像是有效的配置，可能是误判，使用原始文本
                    info!("解码后的内容不是有效的配置格式，使用原始内容");
                    response_text
                }
            },
            Err(_) => {
                // 如果不是有效的UTF-8，返回原始文本
                info!("解码后的内容不是有效的UTF-8，使用原始内容");
                response_text
            }
        }
    } else {
        response_text
    };

    // 使用模板和提取的节点信息创建新的配置
    let work_dir = get_work_dir();
    let config_path = Path::new(&work_dir).join("sing-box/config.json");
    
    // 提取订阅内容中的节点信息
    let extracted_nodes = extract_nodes_from_subscription(&text)?;
    
    // 读取模板文件
    let template_path = get_template_path();
    let mut template_file = File::open(&template_path)?;
    let mut template_content = String::new();
    template_file.read_to_string(&mut template_content)?;
    
    // 将模板内容解析为JSON对象
    let mut config: Value = serde_json::from_str(&template_content)?;
    
    // 将提取的节点添加到模板配置中
    if let Some(config_obj) = config.as_object_mut() {
        if let Some(outbounds) = config_obj.get_mut("outbounds") {
            if let Some(outbounds_array) = outbounds.as_array_mut() {
                // 找到"自动选择"出站
                if let Some(auto_select) = outbounds_array.iter_mut().find(|o| {
                    o.get("tag").and_then(|t| t.as_str()) == Some("自动选择")
                }) {
                    // 更新自动选择的outbounds列表
                    if let Some(outbound_tags) = auto_select.get_mut("outbounds") {
                        // 设置所有节点的标签列表
                        let node_tags: Vec<Value> = extracted_nodes.iter()
                            .map(|node| json!(node.get("tag").unwrap().as_str().unwrap()))
                            .collect();
                        *outbound_tags = json!(node_tags);
                    }
                }
                
                // 找到"手动切换"出站
                if let Some(proxy_select) = outbounds_array.iter_mut().find(|o| {
                    o.get("tag").and_then(|t| t.as_str()) == Some("手动切换")
                }) {
                    // 更新手动切换的outbounds列表
                    if let Some(outbound_tags) = proxy_select.get_mut("outbounds") {
                        let mut tags = vec![json!("自动选择")];
                        // 添加所有节点标签
                        for node in &extracted_nodes {
                            tags.push(json!(node.get("tag").unwrap().as_str().unwrap()));
                        }
                        *outbound_tags = json!(tags);
                    }
                }
                
                // 更新其他选择器的outbounds列表
                update_selector_outbounds(outbounds_array, &extracted_nodes);
                
                // 将节点添加到outbounds数组末尾
                for node in extracted_nodes {
                    outbounds_array.push(node);
                }
            }
        }
    }
    
    // 保存配置到文件
    let mut config_file = File::create(config_path)?;
    let config_str = serde_json::to_string_pretty(&config)?;
    config_file.write_all(config_str.as_bytes())?;

    info!("订阅已更新并应用到模板");
    Ok(())
}

// 从订阅内容中提取节点信息
fn extract_nodes_from_subscription(content: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    // 解析内容为JSON（如果是JSON格式）
    let content_json: Result<Value, _> = serde_json::from_str(content);
    
    let mut nodes = Vec::new();
    
    match content_json {
        Ok(json) => {
            // 如果是JSON格式，尝试从中提取outbounds或proxies
            if let Some(outbounds) = json.get("outbounds").and_then(|o| o.as_array()) {
                // 从sing-box格式的配置中提取节点
                for outbound in outbounds {
                    let outbound_type = outbound.get("type").and_then(|t| t.as_str());
                    match outbound_type {
                        Some("vless") | Some("vmess") | Some("trojan") | Some("shadowsocks") | 
                        Some("shadowsocksr") | Some("socks") | Some("http") => {
                            nodes.push(outbound.clone());
                        },
                        _ => {} // 忽略其他类型的出站
                    }
                }
            } else if let Some(proxies) = json.get("proxies").and_then(|p| p.as_array()) {
                // 从Clash格式的配置中提取节点并转换为sing-box格式
                for proxy in proxies {
                    if let Some(converted_node) = convert_clash_node_to_singbox(proxy) {
                        nodes.push(converted_node);
                    }
                }
            }
        },
        Err(_) => {
            // 尝试解析为Clash YAML格式（简化处理，实际中可能需要更复杂的YAML解析）
            if content.contains("proxies:") {
                info!("检测到可能的Clash YAML格式，需要更多处理...");
                // 这里应该添加YAML格式解析逻辑，简化实现
                // 实际中需要使用yaml解析库提取节点并转换
            }
        }
    }
    
    info!("从订阅中提取了 {} 个节点", nodes.len());
    Ok(nodes)
}

// 将Clash格式的节点转换为sing-box格式
fn convert_clash_node_to_singbox(clash_node: &Value) -> Option<Value> {
    // 获取节点类型
    let node_type = clash_node.get("type").and_then(|t| t.as_str())?;
    let name = clash_node.get("name").and_then(|n| n.as_str())?;
    let server = clash_node.get("server").and_then(|s| s.as_str())?;
    let port = clash_node.get("port").and_then(|p| p.as_u64())?;
    
    // 根据不同类型转换节点
    match node_type {
        "vmess" => {
            let uuid = clash_node.get("uuid").and_then(|u| u.as_str())?;
            let mut node = json!({
                "tag": name,
                "type": "vmess",
                "server": server,
                "server_port": port,
                "uuid": uuid,
                "security": clash_node.get("cipher").and_then(|c| c.as_str()).unwrap_or("auto"),
                "alter_id": clash_node.get("alterId").and_then(|a| a.as_u64()).unwrap_or(0)
            });
            
            // 处理TLS设置
            if let Some(true) = clash_node.get("tls").and_then(|t| t.as_bool()) {
                let mut tls = json!({
                    "enabled": true
                });
                
                if let Some(sni) = clash_node.get("servername").and_then(|s| s.as_str()) {
                    tls["server_name"] = json!(sni);
                }
                
                if let Some(obj) = tls.as_object_mut() {
                    obj.insert("utls".to_string(), json!({
                        "enabled": true,
                        "fingerprint": "chrome"
                    }));
                }
                
                if let Some(obj) = node.as_object_mut() {
                    obj.insert("tls".to_string(), tls);
                }
            }
            
            // 处理传输方式
            if let Some(network) = clash_node.get("network").and_then(|n| n.as_str()) {
                match network {
                    "ws" => {
                        let mut transport = json!({
                            "type": "ws"
                        });
                        
                        if let Some(ws_opts) = clash_node.get("ws-opts") {
                            if let Some(path) = ws_opts.get("path").and_then(|p| p.as_str()) {
                                transport["path"] = json!(path);
                            }
                            
                            if let Some(headers) = ws_opts.get("headers") {
                                if let Some(obj) = headers.as_object() {
                                    transport["headers"] = json!(obj);
                                }
                            }
                        }
                        
                        if let Some(obj) = node.as_object_mut() {
                            obj.insert("transport".to_string(), transport);
                        }
                    },
                    // 其他传输方式如grpc, http等可以类似处理
                    _ => {}
                }
            }
            
            Some(node)
        },
        "vless" => {
            let uuid = clash_node.get("uuid").and_then(|u| u.as_str())?;
            let mut node = json!({
                "tag": name,
                "type": "vless",
                "server": server,
                "server_port": port,
                "uuid": uuid,
                "packet_encoding": "xudp"
            });
            
            // 处理TLS设置
            if let Some(true) = clash_node.get("tls").and_then(|t| t.as_bool()) {
                let mut tls = json!({
                    "enabled": true
                });
                
                if let Some(sni) = clash_node.get("servername").and_then(|s| s.as_str()) {
                    tls["server_name"] = json!(sni);
                }
                
                if let Some(obj) = tls.as_object_mut() {
                    obj.insert("utls".to_string(), json!({
                        "enabled": true,
                        "fingerprint": "chrome"
                    }));
                }
                
                if let Some(obj) = node.as_object_mut() {
                    obj.insert("tls".to_string(), tls);
                }
            }
            
            // 处理传输方式
            if let Some(network) = clash_node.get("network").and_then(|n| n.as_str()) {
                match network {
                    "ws" => {
                        let mut transport = json!({
                            "type": "ws"
                        });
                        
                        if let Some(ws_opts) = clash_node.get("ws-opts") {
                            if let Some(path) = ws_opts.get("path").and_then(|p| p.as_str()) {
                                transport["path"] = json!(path);
                            }
                            
                            if let Some(headers) = ws_opts.get("headers") {
                                if let Some(obj) = headers.as_object() {
                                    transport["headers"] = json!(obj);
                                }
                            }
                        }
                        
                        if let Some(obj) = node.as_object_mut() {
                            obj.insert("transport".to_string(), transport);
                        }
                    },
                    // 其他传输方式
                    _ => {}
                }
            }
            
            Some(node)
        },
        "trojan" => {
            let password = clash_node.get("password").and_then(|p| p.as_str())?;
            let mut node = json!({
                "tag": name,
                "type": "trojan",
                "server": server,
                "server_port": port,
                "password": password
            });
            
            // 添加TLS设置（Trojan必须启用TLS）
            let mut tls = json!({
                "enabled": true
            });
            
            if let Some(sni) = clash_node.get("sni").and_then(|s| s.as_str()) {
                tls["server_name"] = json!(sni);
            }
            
            if let Some(obj) = tls.as_object_mut() {
                obj.insert("utls".to_string(), json!({
                    "enabled": true,
                    "fingerprint": "chrome"
                }));
            }
            
            if let Some(obj) = node.as_object_mut() {
                obj.insert("tls".to_string(), tls);
            }
            
            Some(node)
        },
        "shadowsocks" => {
            let password = clash_node.get("password").and_then(|p| p.as_str())?;
            let method = clash_node.get("cipher").and_then(|c| c.as_str())?;
            
            Some(json!({
                "tag": name,
                "type": "shadowsocks",
                "server": server,
                "server_port": port,
                "method": method,
                "password": password
            }))
        },
        // 其他类型可以类似处理
        _ => None
    }
}

// 更新所有选择器的outbounds列表
fn update_selector_outbounds(outbounds_array: &mut Vec<Value>, nodes: &Vec<Value>) {
    let node_tags: Vec<String> = nodes.iter()
        .filter_map(|node| node.get("tag").and_then(|t| t.as_str()).map(|s| s.to_string()))
        .collect();
    
    // 处理所有selector类型的出站
    for outbound in outbounds_array.iter_mut() {
        if outbound.get("type").and_then(|t| t.as_str()) == Some("selector") {
            let tag = outbound.get("tag").and_then(|t| t.as_str());
            if tag == Some("手动切换") || tag == Some("自动选择") {
                continue; // 已在前面单独处理
            }
            
            if let Some(outbound_tags) = outbound.get_mut("outbounds") {
                if let Some(array) = outbound_tags.as_array_mut() {
                    // 保留前两个元素（通常是proxy和自动选择）
                    if array.len() >= 2 {
                        let first_two = array[0..2].to_vec();
                        array.clear();
                        array.extend(first_two);
                    }
                    
                    // 添加所有节点标签
                    for tag in &node_tags {
                        array.push(json!(tag));
                    }
                }
            }
        }
    }
}

// 处理订阅内容（手动添加）
fn process_subscription_content(content: String) -> Result<(), Box<dyn Error>> {
    // 提取节点信息
    let extracted_nodes = extract_nodes_from_subscription(&content)?;
    
    // 读取模板文件
    let template_path = get_template_path();
    let mut template_file = File::open(&template_path)?;
    let mut template_content = String::new();
    template_file.read_to_string(&mut template_content)?;
    
    // 将模板内容解析为JSON对象
    let mut config: Value = serde_json::from_str(&template_content)?;
    
    // 应用相同的节点合并逻辑
    if let Some(config_obj) = config.as_object_mut() {
        if let Some(outbounds) = config_obj.get_mut("outbounds") {
            if let Some(outbounds_array) = outbounds.as_array_mut() {
                // 更新自动选择和其他选择器
                if let Some(auto_select) = outbounds_array.iter_mut().find(|o| {
                    o.get("tag").and_then(|t| t.as_str()) == Some("自动选择")
                }) {
                    if let Some(outbound_tags) = auto_select.get_mut("outbounds") {
                        let node_tags: Vec<Value> = extracted_nodes.iter()
                            .map(|node| json!(node.get("tag").unwrap().as_str().unwrap()))
                            .collect();
                        *outbound_tags = json!(node_tags);
                    }
                }
                
                if let Some(proxy_select) = outbounds_array.iter_mut().find(|o| {
                    o.get("tag").and_then(|t| t.as_str()) == Some("手动切换")
                }) {
                    if let Some(outbound_tags) = proxy_select.get_mut("outbounds") {
                        let mut tags = vec![json!("自动选择")];
                        for node in &extracted_nodes {
                            tags.push(json!(node.get("tag").unwrap().as_str().unwrap()));
                        }
                        *outbound_tags = json!(tags);
                    }
                }
                
                // 更新其他选择器
                update_selector_outbounds(outbounds_array, &extracted_nodes);
                
                // 添加节点到outbounds
                for node in extracted_nodes {
                    outbounds_array.push(node);
                }
            }
        }
    }
    
    // 保存配置到文件
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let mut file = File::create(path)?;
    let config_str = serde_json::to_string_pretty(&config)?;
    file.write_all(config_str.as_bytes())?;

    info!("订阅内容已处理并应用到模板");
    Ok(())
}

// 改进base64检测逻辑
fn is_base64_encoded(text: &str) -> bool {
    // 先进行基本字符集检查
    let is_valid_charset = text.trim().chars().all(|c| 
        c.is_ascii_alphanumeric() || c == '+' || c == '/' || c == '=' || 
        c == '-' || c == '_' // 支持URL安全变体
    );
    
    if !is_valid_charset {
        return false;
    }
    
    let trimmed = text.trim();
    
    // 检查长度（标准base64长度应为4的倍数，可能有结尾的'='填充）
    if trimmed.len() % 4 != 0 && !trimmed.ends_with('=') {
        return false;
    }
    
    // 避免误判普通文本
    if trimmed.len() < 8 || trimmed.contains(" ") {
        return false;
    }
    
    // 尝试解码看是否成功（更准确但性能较低的方法）
    let standard_decode_ok = base64::decode(trimmed).is_ok();
    let url_safe_decode_ok = base64::decode_config(trimmed, base64::URL_SAFE).is_ok();
    
    // 如果能成功解码，再检查解码后内容是否合理（避免误判）
    if standard_decode_ok || url_safe_decode_ok {
        // 检查是否为常见订阅格式特征
        if trimmed.starts_with("ey") || trimmed.starts_with("dm") {
            return true; // 常见的JSON或YAML base64编码的开头特征
        }
    }
    
    false
}

// 获取当前代理模式
#[tauri::command]
pub fn get_current_proxy_mode() -> Result<String, String> {
    info!("正在获取当前代理模式");
    
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    
    // 检查配置文件是否存在
    if !path.exists() {
        return Ok("rule".to_string()); // 默认返回rule模式
    }
    
    // 读取配置文件
    match read_proxy_mode_from_config(&path) {
        Ok(mode) => {
            info!("当前代理模式为: {}", mode);
            Ok(mode)
        },
        Err(e) => {
            error!("获取代理模式失败: {}", e);
            Ok("rule".to_string()) // 出错时默认返回rule模式
        }
    }
}

// 从配置文件中读取代理模式
fn read_proxy_mode_from_config(config_path: &Path) -> Result<String, Box<dyn Error>> {
    // 读取配置文件
    let mut file = File::open(config_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;
    
    // 解析JSON
    let json: serde_json::Value = serde_json::from_str(&content)?;
    
    // 尝试读取experimental.clash_api.default_mode
    if let Some(experimental) = json.get("experimental") {
        if let Some(clash_api) = experimental.get("clash_api") {
            if let Some(default_mode) = clash_api.get("default_mode") {
                if let Some(mode) = default_mode.as_str() {
                    return Ok(mode.to_string());
                }
            }
        }
    }
    
    // 如果找不到，返回默认的rule模式
    Ok("rule".to_string())
} 