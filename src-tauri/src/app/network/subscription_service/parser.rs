use serde_json::{json, Value};
use tracing::info;

pub fn extract_nodes_from_subscription(
    content: &str,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let cleaned_content = clean_json_content(content);
    let content_json: Result<Value, _> = serde_json::from_str(&cleaned_content);

    let mut nodes = Vec::new();

    match content_json {
        Ok(json) => {
            info!("成功解析内容为JSON格式");

            if let Some(outbounds) = json.get("outbounds").and_then(|o| o.as_array()) {
                info!("检测到sing-box格式，outbounds数组长度: {}", outbounds.len());

                for (_i, outbound) in outbounds.iter().enumerate() {
                    let outbound_type = outbound.get("type").and_then(|t| t.as_str());

                    let node_with_tag = if outbound.get("tag").is_none() {
                        let server = outbound
                            .get("server")
                            .and_then(|s| s.as_str())
                            .unwrap_or("unknown");
                        let node_type = outbound_type.unwrap_or("unknown");
                        let tag = format!("{}-{}", node_type, server);

                        let mut node_obj = outbound.clone();
                        if let Some(obj) = node_obj.as_object_mut() {
                            obj.insert("tag".to_string(), json!(tag));
                        }
                        node_obj
                    } else {
                        outbound.clone()
                    };

                    match outbound_type {
                        Some("vless") | Some("vmess") | Some("trojan") | Some("shadowsocks")
                        | Some("shadowsocksr") | Some("socks") | Some("http")
                        | Some("hysteria2") => {
                            nodes.push(node_with_tag);
                        }
                        _ => {}
                    }
                }

                if nodes.is_empty() {
                    info!("在顶级outbounds中未找到支持的节点，尝试递归解析...");
                    for outbound in outbounds {
                        if let Some(sub_outbounds) =
                            outbound.get("outbounds").and_then(|o| o.as_array())
                        {
                            for sub_outbound in sub_outbounds {
                                if let Some(sub_tag) = sub_outbound.as_str() {
                                    if let Some(actual_node) =
                                        find_outbound_by_tag(&outbounds, sub_tag)
                                    {
                                        let node_type =
                                            actual_node.get("type").and_then(|t| t.as_str());
                                        if let Some(type_str) = node_type {
                                            if [
                                                "vless",
                                                "vmess",
                                                "trojan",
                                                "shadowsocks",
                                                "shadowsocksr",
                                                "socks",
                                                "http",
                                                "hysteria2",
                                            ]
                                            .contains(&type_str)
                                            {
                                                let node_with_tag =
                                                    if actual_node.get("tag").is_none() {
                                                        let mut node_obj = actual_node.clone();
                                                        if let Some(obj) = node_obj.as_object_mut()
                                                        {
                                                            obj.insert(
                                                                "tag".to_string(),
                                                                json!(sub_tag),
                                                            );
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

                for proxy in proxies {
                    if let Some(converted_node) = convert_clash_node_to_singbox(proxy) {
                        nodes.push(converted_node);
                    }
                }
            } else {
                info!("未找到标准的outbounds或proxies数组，尝试解析其他位置...");

                if let Some(obj) = json.as_object() {
                    let keys: Vec<&String> = obj.keys().collect();
                    info!("JSON顶级键: {:?}", keys);

                    for (_key, value) in obj {
                        if let Some(arr) = value.as_array() {
                            for item in arr {
                                if let Some(item_obj) = item.as_object() {
                                    let has_type = item_obj.contains_key("type");
                                    let has_tag = item_obj.contains_key("tag")
                                        || item_obj.contains_key("name");
                                    let has_server = item_obj.contains_key("server");

                                    if has_type && (has_tag || has_server) {
                                        let item_type = item.get("type").and_then(|t| t.as_str());

                                        if let Some(t) = item_type {
                                            if [
                                                "vless",
                                                "vmess",
                                                "trojan",
                                                "shadowsocks",
                                                "shadowsocksr",
                                                "socks",
                                                "http",
                                                "hysteria2",
                                            ]
                                            .contains(&t)
                                            {
                                                let node_with_tag = if !has_tag {
                                                    let server = item
                                                        .get("server")
                                                        .and_then(|s| s.as_str())
                                                        .unwrap_or("unknown");
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
        }
        Err(e) => {
            info!("内容不是有效的JSON格式: {}", e);

            if cleaned_content.contains("proxies:") {
                info!("检测到可能的Clash YAML格式");
            }

            if cleaned_content.contains("vmess://")
                || cleaned_content.contains("ss://")
                || cleaned_content.contains("trojan://")
                || cleaned_content.contains("vless://")
            {
                info!("检测到可能包含URI格式的节点");
            }
        }
    }

    let mut fixed_nodes = Vec::new();
    for (i, node) in nodes.iter().enumerate() {
        let tag = node.get("tag").and_then(|t| t.as_str());
        if tag.is_none() || tag.unwrap().is_empty() {
            let node_type = node
                .get("type")
                .and_then(|t| t.as_str())
                .unwrap_or("unknown");
            let server = node
                .get("server")
                .and_then(|s| s.as_str())
                .unwrap_or("unknown");
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

pub fn clean_json_content(content: &str) -> String {
    let mut cleaned = String::with_capacity(content.len());
    let mut in_string = false;
    let mut escape_next = false;
    let mut last_char: Option<char> = None;

    let content = content.trim_start_matches('\u{FEFF}');

    for c in content.chars() {
        if c == '\u{200B}'
            || c == '\u{200C}'
            || c == '\u{200D}'
            || (c.is_control() && c != '\n' && c != '\r' && c != '\t')
        {
            continue;
        }

        if in_string {
            if escape_next {
                match c {
                    '"' | '\\' | '/' | 'b' | 'f' | 'n' | 'r' | 't' | 'u' => {
                        cleaned.push('\\');
                        cleaned.push(c);
                    }
                    _ => {
                        cleaned.push(' ');
                    }
                }
                escape_next = false;
            } else if c == '\\' {
                escape_next = true;
            } else if c == '"' {
                in_string = false;
                cleaned.push(c);
            } else if c.is_ascii_graphic() || c == ' ' || c.is_ascii_whitespace() || !c.is_ascii() {
                cleaned.push(c);
            } else {
                cleaned.push(' ');
            }
        } else {
            if c == '"' {
                in_string = true;
                cleaned.push(c);
            } else if c == '{'
                || c == '}'
                || c == '['
                || c == ']'
                || c == ':'
                || c == ','
                || c == '.'
                || c == '-'
                || c == '+'
                || c.is_ascii_digit()
            {
                cleaned.push(c);
            } else if c.is_ascii_whitespace() {
                if let Some(last) = last_char {
                    if !last.is_ascii_whitespace() {
                        cleaned.push(c);
                    }
                } else {
                    cleaned.push(c);
                }
            } else if c.is_ascii_alphabetic() || !c.is_ascii() {
                cleaned.push(c);
            } else if let Some(last) = last_char {
                if !last.is_ascii_whitespace() {
                    cleaned.push(' ');
                }
            } else {
                cleaned.push(' ');
            }
        }
        last_char = Some(c);
    }

    if in_string {
        cleaned.push('"');
    }

    cleaned.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn find_outbound_by_tag<'a>(outbounds: &'a [Value], tag: &str) -> Option<&'a Value> {
    outbounds
        .iter()
        .find(|outbound| outbound.get("tag").and_then(|t| t.as_str()) == Some(tag))
}

fn convert_clash_node_to_singbox(clash_node: &Value) -> Option<Value> {
    let node_type = clash_node.get("type").and_then(|t| t.as_str())?;
    let name = clash_node.get("name").and_then(|n| n.as_str())?;
    let server = clash_node.get("server").and_then(|s| s.as_str())?;
    let port = clash_node.get("port").and_then(|p| p.as_u64())?;

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

            if let Some(true) = clash_node.get("tls").and_then(|t| t.as_bool()) {
                let mut tls = json!({
                    "enabled": true
                });

                if let Some(sni) = clash_node.get("servername").and_then(|s| s.as_str()) {
                    tls["server_name"] = json!(sni);
                }

                if let Some(obj) = tls.as_object_mut() {
                    obj.insert(
                        "utls".to_string(),
                        json!({
                            "enabled": true,
                            "fingerprint": "chrome"
                        }),
                    );
                }

                if let Some(obj) = node.as_object_mut() {
                    obj.insert("tls".to_string(), tls);
                }
            }

            if let Some(network) = clash_node.get("network").and_then(|n| n.as_str()) {
                if network == "ws" {
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
                }
            }

            Some(node)
        }
        "vless" => {
            let uuid = clash_node.get("uuid").and_then(|u| u.as_str())?;
            let mut node = json!({
                "tag": name,
                "type": "vless",
                "server": server,
                "server_port": port,
                "uuid": uuid
            });

            if let Some(true) = clash_node.get("tls").and_then(|t| t.as_bool()) {
                let mut tls = json!({
                    "enabled": true
                });

                if let Some(sni) = clash_node.get("servername").and_then(|s| s.as_str()) {
                    tls["server_name"] = json!(sni);
                }

                if let Some(obj) = tls.as_object_mut() {
                    obj.insert(
                        "utls".to_string(),
                        json!({
                            "enabled": true,
                            "fingerprint": "chrome"
                        }),
                    );
                }

                if let Some(obj) = node.as_object_mut() {
                    obj.insert("tls".to_string(), tls);
                }
            }

            Some(node)
        }
        "trojan" => {
            let password = clash_node.get("password").and_then(|p| p.as_str())?;
            Some(json!({
                "tag": name,
                "type": "trojan",
                "server": server,
                "server_port": port,
                "password": password,
                "tls": {
                    "enabled": clash_node.get("tls").and_then(|t| t.as_bool()).unwrap_or(true),
                    "server_name": clash_node.get("sni").and_then(|s| s.as_str()).unwrap_or(server)
                }
            }))
        }
        "ss" => {
            let method = clash_node.get("cipher").and_then(|c| c.as_str())?;
            let password = clash_node.get("password").and_then(|p| p.as_str())?;
            Some(json!({
                "tag": name,
                "type": "shadowsocks",
                "server": server,
                "server_port": port,
                "method": method,
                "password": password
            }))
        }
        _ => None,
    }
}

pub fn update_selector_outbounds(outbounds_array: &mut Vec<Value>, nodes: &Vec<Value>) {
    let selector_tags = vec![
        "手动切换",
        "自动选择",
        "TikTok节点",
        "OpenAI节点",
        "Netflix节点",
        "Spotify节点",
        "YouTube节点",
        "Disney节点",
    ];

    let node_tags: Vec<Value> = nodes
        .iter()
        .filter_map(|node| node.get("tag").cloned())
        .collect();

    for tag in selector_tags {
        if let Some(selector) = outbounds_array
            .iter_mut()
            .find(|o| o.get("tag").and_then(|t| t.as_str()) == Some(tag))
        {
            if let Some(selector_obj) = selector.as_object_mut() {
                selector_obj.insert("outbounds".to_string(), json!(node_tags));
            }
        }
    }
}
