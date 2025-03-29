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

    let response = client.get(url.trim()).headers(headers).send().await?;
    let response_text = response.text().await?;
    
    // 直接尝试从原始内容提取节点
    let mut extracted_nodes = extract_nodes_from_subscription(&response_text)?;
    info!("从原始内容提取到 {} 个节点", extracted_nodes.len());
    
    // 如果没有提取到节点，尝试base64解码后重新提取
    if extracted_nodes.is_empty() {
        info!("未从原始内容提取到节点，尝试base64解码...");
        
        // 尝试标准base64解码
        let decoded_result = base64::decode(&response_text.trim());
        if let Ok(decoded) = decoded_result {
            if let Ok(decoded_text) = String::from_utf8(decoded.clone()) {
                info!("base64标准解码成功，重新从解码内容提取节点...");
                extracted_nodes = extract_nodes_from_subscription(&decoded_text)?;
                info!("从标准base64解码内容提取到 {} 个节点", extracted_nodes.len());
            } else {
                info!("base64解码成功但无法转换为UTF-8文本");
            }
        } else {
            info!("标准base64解码失败，尝试URL安全base64解码...");
            
            // 尝试URL安全的base64变体
            let url_safe_decoded = base64::decode_config(&response_text.trim(), base64::URL_SAFE);
            if let Ok(decoded) = url_safe_decoded {
                if let Ok(decoded_text) = String::from_utf8(decoded.clone()) {
                    info!("URL安全base64解码成功，重新从解码内容提取节点...");
                    extracted_nodes = extract_nodes_from_subscription(&decoded_text)?;
                    info!("从URL安全base64解码内容提取到 {} 个节点", extracted_nodes.len());
                } else {
                    info!("URL安全base64解码成功但无法转换为UTF-8文本");
                }
            } else {
                info!("URL安全base64解码也失败");
            }
        }
    }
    
    // 如果依然没有提取到节点，再尝试移除可能的前缀后再解码
    if extracted_nodes.is_empty() {
        info!("标准解码方法均未提取到节点，尝试移除前缀后再解码...");
        
        // 移除可能的前缀 (例如: "ss://", "vmess://")
        let stripped_text = response_text.trim().replace("vmess://", "").replace("ss://", "")
            .replace("trojan://", "").replace("vless://", "");
        
        if let Ok(decoded) = base64::decode(&stripped_text) {
            if let Ok(decoded_text) = String::from_utf8(decoded) {
                extracted_nodes = extract_nodes_from_subscription(&decoded_text)?;
                info!("从移除前缀后解码内容提取到 {} 个节点", extracted_nodes.len());
            }
        }
    }
    
    // 如果依然没有提取到节点，返回错误
    if extracted_nodes.is_empty() {
        error!("无法从订阅内容提取节点信息，已尝试所有解码方式");
        return Err("无法从订阅内容提取节点信息，请检查订阅链接或内容格式".into());
    }

    info!("成功提取到 {} 个节点，准备应用到配置", extracted_nodes.len());
    
    // 使用模板和提取的节点信息创建新的配置
    let work_dir = get_work_dir();
    let config_path = Path::new(&work_dir).join("sing-box/config.json");
    
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

    info!("订阅已更新并应用到模板，配置已保存");
    Ok(())
}

// 从订阅内容中提取节点信息
fn extract_nodes_from_subscription(content: &str) -> Result<Vec<Value>, Box<dyn Error>> {
    // 清理内容中的非法字符
    let cleaned_content = clean_json_content(content);
    
    // 解析内容为JSON（如果是JSON格式）
    let content_json: Result<Value, _> = serde_json::from_str(&cleaned_content);
    
    let mut nodes = Vec::new();
    
    match content_json {
        Ok(json) => {
            info!("成功解析内容为JSON格式");
            
            // 如果是JSON格式，尝试从中提取outbounds或proxies
            if let Some(outbounds) = json.get("outbounds").and_then(|o| o.as_array()) {
                info!("检测到sing-box格式，outbounds数组长度: {}", outbounds.len());
                
                // 从sing-box格式的配置中提取节点
                for (_i, outbound) in outbounds.iter().enumerate() {
                    let outbound_type = outbound.get("type").and_then(|t| t.as_str());
                    
                    // 确保每个节点都有tag属性，如果没有则创建一个
                    let node_with_tag = if outbound.get("tag").is_none() {
                        // 如果没有tag，创建一个包含tag的节点副本
                        let server = outbound.get("server").and_then(|s| s.as_str()).unwrap_or("unknown");
                        let node_type = outbound_type.unwrap_or("unknown");
                        let tag = format!("{}-{}", node_type, server);
                        
                        // 创建新的节点对象，添加tag属性
                        let mut node_obj = outbound.clone();
                        if let Some(obj) = node_obj.as_object_mut() {
                            obj.insert("tag".to_string(), json!(tag));
                        }
                        node_obj
                    } else {
                        // 已有tag，直接使用
                        outbound.clone()
                    };
                    
                    match outbound_type {
                        Some("vless") | Some("vmess") | Some("trojan") | Some("shadowsocks") | 
                        Some("shadowsocksr") | Some("socks") | Some("http") => {
                            nodes.push(node_with_tag);
                        },
                        _ => {} // 忽略其他类型的出站
                    }
                }
                
                // 如果仍然没找到节点，尝试递归解析所有outbound
                if nodes.is_empty() {
                    info!("在顶级outbounds中未找到支持的节点，尝试递归解析...");
                    for outbound in outbounds {
                        // 检查是否有子outbounds
                        if let Some(sub_outbounds) = outbound.get("outbounds").and_then(|o| o.as_array()) {
                            for sub_outbound in sub_outbounds {
                                if let Some(sub_tag) = sub_outbound.as_str() {
                                    // 这是一个引用，尝试在主outbounds中找到对应的节点
                                    if let Some(actual_node) = find_outbound_by_tag(&outbounds, sub_tag) {
                                        let node_type = actual_node.get("type").and_then(|t| t.as_str());
                                        if let Some(type_str) = node_type {
                                            if ["vless", "vmess", "trojan", "shadowsocks", "shadowsocksr", "socks", "http"].contains(&type_str) {
                                                // 确保节点有tag
                                                let node_with_tag = if actual_node.get("tag").is_none() {
                                                    let mut node_obj = actual_node.clone();
                                                    if let Some(obj) = node_obj.as_object_mut() {
                                                        obj.insert("tag".to_string(), json!(sub_tag));
                                                    }
                                                    node_obj
                                                } else {
                                                    actual_node.clone()
                                                };
                                                nodes.push(node_with_tag);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            } else if let Some(proxies) = json.get("proxies").and_then(|p| p.as_array()) {
                info!("检测到Clash格式，proxies数组长度: {}", proxies.len());
                
                // 从Clash格式的配置中提取节点并转换为sing-box格式
                for proxy in proxies {
                    if let Some(converted_node) = convert_clash_node_to_singbox(proxy) {
                        nodes.push(converted_node);
                    }
                }
            } else {
                // 尝试查找其他可能的位置
                info!("未找到标准的outbounds或proxies数组，尝试解析其他位置...");
                
                // 输出JSON的顶级键，帮助诊断
                if let Some(obj) = json.as_object() {
                    let keys: Vec<&String> = obj.keys().collect();
                    info!("JSON顶级键: {:?}", keys);
                    
                    // 如果是sing-box配置但outbounds在不同位置
                    for (_key, value) in obj {
                        if let Some(arr) = value.as_array() {
                            // 检查数组中的每个元素是否可能是节点
                            for item in arr {
                                if let Some(item_obj) = item.as_object() {
                                    // 如果对象有type和tag/name字段，可能是节点
                                    let has_type = item_obj.contains_key("type");
                                    let has_tag = item_obj.contains_key("tag") || item_obj.contains_key("name");
                                    let has_server = item_obj.contains_key("server");
                                    
                                    if has_type && (has_tag || has_server) {
                                        let item_type = item.get("type").and_then(|t| t.as_str());
                                        
                                        if let Some(t) = item_type {
                                            if ["vless", "vmess", "trojan", "shadowsocks", "shadowsocksr", "socks", "http"].contains(&t) {
                                                // 确保节点有tag
                                                let node_with_tag = if !has_tag {
                                                    let server = item.get("server").and_then(|s| s.as_str()).unwrap_or("unknown");
                                                    let tag = format!("{}-{}", t, server);
                                                    
                                                    let mut node_obj = item.clone();
                                                    if let Some(obj) = node_obj.as_object_mut() {
                                                        obj.insert("tag".to_string(), json!(tag));
                                                    }
                                                    node_obj
                                                } else {
                                                    item.clone()
                                                };
                                                nodes.push(node_with_tag);
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        },
        Err(e) => {
            info!("内容不是有效的JSON格式: {}", e);
            
            // 尝试解析为Clash YAML格式（简化处理，实际中可能需要更复杂的YAML解析）
            if cleaned_content.contains("proxies:") {
                info!("检测到可能的Clash YAML格式");
                // 这里应该添加YAML格式解析逻辑，简化实现
                // 实际中需要使用yaml解析库提取节点并转换
            }
            
            // 检查是否包含URI格式的节点
            if cleaned_content.contains("vmess://") || cleaned_content.contains("ss://") || 
               cleaned_content.contains("trojan://") || cleaned_content.contains("vless://") {
                info!("检测到可能包含URI格式的节点");
                // TODO: 解析URI格式的节点
            }
        }
    }
    
    // 确保所有节点都有有效的tag
    let mut fixed_nodes = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        let tag = node.get("tag").and_then(|t| t.as_str());
        if tag.is_none() || tag.unwrap().is_empty() {
            // 没有tag或tag为空，创建一个新的
            let node_type = node.get("type").and_then(|t| t.as_str()).unwrap_or("unknown");
            let server = node.get("server").and_then(|s| s.as_str()).unwrap_or("unknown");
            let new_tag = format!("{}-{}-{}", node_type, server, i);
            
            let mut node_obj = node.clone();
            if let Some(obj) = node_obj.as_object_mut() {
                obj.insert("tag".to_string(), json!(new_tag));
            }
            fixed_nodes.push(node_obj);
        } else {
            fixed_nodes.push(node.clone());
        }
    }
    
    info!("从订阅中提取了 {} 个节点", fixed_nodes.len());
    Ok(fixed_nodes)
}

// 清理JSON内容中的非法字符
fn clean_json_content(content: &str) -> String {
    let mut cleaned = String::with_capacity(content.len());
    let mut in_string = false;
    let mut escape_next = false;
    let mut last_char: Option<char> = None;
    
    // 首先移除BOM标记
    let content = content.trim_start_matches('\u{FEFF}');
    
    for c in content.chars() {
        // 跳过零宽字符和其他控制字符
        if c == '\u{200B}' || c == '\u{200C}' || c == '\u{200D}' || 
           (c.is_control() && c != '\n' && c != '\r' && c != '\t') {
            continue;
        }
        
        if in_string {
            if escape_next {
                // 在转义状态下，只允许JSON规范的转义字符
                match c {
                    '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u' => {
                        cleaned.push('\\');
                        cleaned.push(c);
                    },
                    _ => {
                        // 无效的转义序列，添加空格替代
                        cleaned.push(' ');
                    }
                }
                escape_next = false;
            } else if c == '\\' {
                // 遇到反斜杠，进入转义状态
                escape_next = true;
            } else if c == '"' {
                // 非转义的引号表示字符串结束
                in_string = false;
                cleaned.push(c);
            } else if c.is_ascii_graphic() || c == ' ' || c.is_ascii_whitespace() || !c.is_ascii() {
                // 保留ASCII可见字符、空白字符和所有非ASCII字符（包括中文等Unicode字符）
                cleaned.push(c);
            } else {
                // 不可见或不可打印的ASCII控制字符替换为空格
                cleaned.push(' ');
            }
        } else {
            // 字符串外部
            if c == '"' {
                // 开始一个新的字符串
                in_string = true;
                cleaned.push(c);
            } else if c == '{' || c == '}' || c == '[' || c == ']' || 
                      c == ':' || c == ',' || c == '.' || c == '-' || 
                      c == '+' || c.is_ascii_digit() {
                // 保留JSON结构字符和数字
                cleaned.push(c);
            } else if c.is_ascii_whitespace() {
                // 保留空白字符，但避免连续的空白字符
                if let Some(last) = last_char {
                    if !last.is_ascii_whitespace() {
                        cleaned.push(c);
                    }
                } else {
                    cleaned.push(c);
                }
            } else if c.is_ascii_alphabetic() || !c.is_ascii() {
                // 保留字母字符（对于JSON里的true/false/null很重要）和所有非ASCII字符
                cleaned.push(c);
            } else {
                // 其他不相关的字符替换为空格，但避免连续的空白字符
                if let Some(last) = last_char {
                    if !last.is_ascii_whitespace() {
                        cleaned.push(' ');
                    }
                } else {
                    cleaned.push(' ');
                }
            }
        }
        last_char = Some(c);
    }
    
    // 如果字符串还没结束但已到达内容末尾，强制闭合
    if in_string {
        cleaned.push('"');
    }
    
    // 移除开头和结尾的空白字符，并确保没有连续的空白字符
    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

// 根据标签查找outbound
fn find_outbound_by_tag<'a>(outbounds: &'a [Value], tag: &str) -> Option<&'a Value> {
    for outbound in outbounds {
        if let Some(outbound_tag) = outbound.get("tag").and_then(|t| t.as_str()) {
            if outbound_tag == tag {
                return Some(outbound);
            }
        }
    }
    None
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