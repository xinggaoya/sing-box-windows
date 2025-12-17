use base64::Engine as _;
use serde_json::{json, Value};
use tracing::info;
use url::Url;

pub fn extract_nodes_from_subscription(
    content: &str,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    // JSON 场景需要更激进的清洗（避免控制字符/不可见字符导致解析失败）。
    let cleaned_json = clean_json_content(content);
    let content_json: Result<Value, _> = serde_json::from_str(&cleaned_json);

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

            // 对于非 JSON 内容，尽量保留换行：很多订阅是“URI 一行一个节点”。
            let normalized_text = normalize_text_content(content);

            // 1) Clash YAML（最常见的 YAML 订阅）
            if normalized_text.contains("proxies:") || normalized_text.contains("proxy-groups:") {
                info!("检测到可能的Clash YAML格式，尝试解析...");
                nodes.extend(extract_nodes_from_clash_yaml(&normalized_text));
            }

            // 2) URI 列表（如：vmess:// / ss:// / trojan:// / vless:// 一行一个）
            // 说明：这里不直接返回错误，避免“部分节点格式不标准”导致整个订阅无法导入。
            if nodes.is_empty() {
                if normalized_text.contains("vmess://")
                    || normalized_text.contains("ss://")
                    || normalized_text.contains("trojan://")
                    || normalized_text.contains("vless://")
                {
                    info!("检测到可能包含URI格式的节点，尝试逐行解析...");
                    nodes.extend(extract_nodes_from_uri_list(&normalized_text));
                }
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

/// 非 JSON 内容的“温和清洗”：
/// - 去掉 BOM / 零宽字符 / 不必要的控制字符
/// - 保留换行，便于解析 “URI 一行一个节点” 的订阅格式
fn normalize_text_content(content: &str) -> String {
    let mut cleaned = String::with_capacity(content.len());
    let content = content.trim_start_matches('\u{FEFF}');

    for c in content.chars() {
        if c == '\u{200B}' || c == '\u{200C}' || c == '\u{200D}' {
            continue;
        }
        // 保留换行与制表符，其他控制字符剔除
        if c.is_control() && c != '\n' && c != '\r' && c != '\t' {
            continue;
        }
        cleaned.push(c);
    }
    cleaned
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

fn extract_nodes_from_clash_yaml(content: &str) -> Vec<Value> {
    // serde_yaml -> serde_json::Value，再复用现有的 Clash JSON 转换逻辑
    let yaml_value: serde_yaml::Value = match serde_yaml::from_str(content) {
        Ok(v) => v,
        Err(e) => {
            info!("Clash YAML 解析失败: {}", e);
            return Vec::new();
        }
    };

    let json_value: Value = match serde_json::to_value(yaml_value) {
        Ok(v) => v,
        Err(e) => {
            info!("Clash YAML 转 JSON 失败: {}", e);
            return Vec::new();
        }
    };

    let mut nodes = Vec::new();
    if let Some(proxies) = json_value.get("proxies").and_then(|p| p.as_array()) {
        info!("Clash YAML 解析成功，proxies 数量: {}", proxies.len());
        for proxy in proxies {
            if let Some(converted_node) = convert_clash_node_to_singbox(proxy) {
                nodes.push(converted_node);
            }
        }
    }
    nodes
}

fn extract_nodes_from_uri_list(content: &str) -> Vec<Value> {
    let mut nodes = Vec::new();

    for (idx, raw_line) in content.lines().enumerate() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        // 常见订阅会在末尾附带 “# remark”，或混入注释行
        if line.starts_with('#') {
            continue;
        }

        if let Some(node) = convert_uri_node_to_singbox(line) {
            nodes.push(node);
        } else {
            // 不要刷屏：仅在可疑情况下输出一次提示
            if idx < 3 {
                info!("URI 节点解析失败（将跳过该行）: {}", line);
            }
        }
    }

    nodes
}

fn convert_uri_node_to_singbox(uri: &str) -> Option<Value> {
    if uri.starts_with("vmess://") {
        return parse_vmess_uri(uri);
    }
    if uri.starts_with("vless://") {
        return parse_vless_uri(uri);
    }
    if uri.starts_with("trojan://") {
        return parse_trojan_uri(uri);
    }
    if uri.starts_with("ss://") {
        return parse_ss_uri(uri);
    }
    None
}

fn decode_tag(raw: Option<&str>) -> String {
    raw.and_then(|s| urlencoding::decode(s).ok())
        .map(|s| s.to_string())
        .unwrap_or_default()
}

fn default_tag_for_url(url: &Url) -> String {
    let host = url.host_str().unwrap_or("unknown");
    let port = url.port().unwrap_or(0);
    format!("{}-{}:{}", url.scheme(), host, port)
}

fn parse_vless_uri(uri: &str) -> Option<Value> {
    let url = Url::parse(uri).ok()?;
    let uuid = url.username().trim();
    if uuid.is_empty() {
        return None;
    }

    let server = url.host_str()?.to_string();
    let server_port = url.port().unwrap_or(443) as u64;

    let mut query = std::collections::HashMap::<String, String>::new();
    for (k, v) in url.query_pairs() {
        query.insert(k.to_string(), v.to_string());
    }

    let tag = {
        let decoded = decode_tag(url.fragment());
        if decoded.is_empty() {
            default_tag_for_url(&url)
        } else {
            decoded
        }
    };

    let mut node = json!({
        "tag": tag,
        "type": "vless",
        "server": server,
        "server_port": server_port,
        "uuid": uuid,
    });

    // flow（如 xtls-rprx-vision）对部分节点是必要字段
    if let Some(flow) = query.get("flow").map(|s| s.trim()).filter(|s| !s.is_empty()) {
        node["flow"] = json!(flow);
    }

    // TLS/REALITY：简化处理——只要显式声明或存在 SNI 就默认启用 TLS
    let security = query.get("security").map(|s| s.as_str()).unwrap_or("");
    let sni = query
        .get("sni")
        .or_else(|| query.get("servername"))
        .map(|s| s.trim())
        .unwrap_or("");

    if security == "tls" || security == "reality" || !sni.is_empty() {
        node["tls"] = json!({
            "enabled": true,
            "server_name": if sni.is_empty() { server.clone() } else { sni.to_string() },
            "utls": { "enabled": true, "fingerprint": "chrome" }
        });
    }

    // 传输层（最常见：ws）
    let network = query
        .get("type")
        .or_else(|| query.get("network"))
        .map(|s| s.as_str())
        .unwrap_or("");

    if network == "ws" {
        let mut transport = json!({
            "type": "ws"
        });
        if let Some(path) = query.get("path").map(|s| s.trim()).filter(|s| !s.is_empty()) {
            transport["path"] = json!(path);
        }
        if let Some(host) = query.get("host").map(|s| s.trim()).filter(|s| !s.is_empty()) {
            transport["headers"] = json!({ "Host": host });
        }
        node["transport"] = transport;
    }

    Some(node)
}

fn parse_trojan_uri(uri: &str) -> Option<Value> {
    let url = Url::parse(uri).ok()?;
    let password = url.username().trim();
    if password.is_empty() {
        return None;
    }

    let server = url.host_str()?.to_string();
    let server_port = url.port().unwrap_or(443) as u64;

    let mut query = std::collections::HashMap::<String, String>::new();
    for (k, v) in url.query_pairs() {
        query.insert(k.to_string(), v.to_string());
    }

    let tag = {
        let decoded = decode_tag(url.fragment());
        if decoded.is_empty() {
            default_tag_for_url(&url)
        } else {
            decoded
        }
    };

    let sni = query
        .get("sni")
        .or_else(|| query.get("peer"))
        .or_else(|| query.get("servername"))
        .map(|s| s.trim())
        .unwrap_or("");

    let insecure = query
        .get("allowInsecure")
        .or_else(|| query.get("insecure"))
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false);

    let mut node = json!({
        "tag": tag,
        "type": "trojan",
        "server": server,
        "server_port": server_port,
        "password": password,
        "tls": {
            "enabled": true,
            "server_name": if sni.is_empty() { server.clone() } else { sni.to_string() },
            "insecure": insecure,
            "utls": { "enabled": true, "fingerprint": "chrome" }
        }
    });

    // 传输层（最常见：ws）
    let network = query
        .get("type")
        .or_else(|| query.get("network"))
        .map(|s| s.as_str())
        .unwrap_or("");
    if network == "ws" {
        let mut transport = json!({
            "type": "ws"
        });
        if let Some(path) = query.get("path").map(|s| s.trim()).filter(|s| !s.is_empty()) {
            transport["path"] = json!(path);
        }
        if let Some(host) = query.get("host").map(|s| s.trim()).filter(|s| !s.is_empty()) {
            transport["headers"] = json!({ "Host": host });
        }
        node["transport"] = transport;
    }

    Some(node)
}

fn parse_vmess_uri(uri: &str) -> Option<Value> {
    let payload = uri.trim().strip_prefix("vmess://")?.trim();
    if payload.is_empty() {
        return None;
    }

    // vmess:// 通常是 base64(json)
    let decoded = base64_decode_relaxed(payload)?;
    let text = String::from_utf8(decoded).ok()?;
    let v: Value = serde_json::from_str(&text).ok()?;

    let server = v.get("add").and_then(|s| s.as_str())?.to_string();
    let port = match v.get("port") {
        Some(Value::String(s)) => s.parse::<u64>().ok(),
        Some(Value::Number(n)) => n.as_u64(),
        _ => None,
    }
    .unwrap_or(443);
    let uuid = v.get("id").and_then(|s| s.as_str())?.to_string();

    let tag = v
        .get("ps")
        .and_then(|s| s.as_str())
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
        .unwrap_or_else(|| format!("vmess-{}:{}", server, port));

    let alter_id = match v.get("aid") {
        Some(Value::String(s)) => s.parse::<u64>().ok(),
        Some(Value::Number(n)) => n.as_u64(),
        _ => None,
    }
    .unwrap_or(0);

    let mut node = json!({
        "tag": tag,
        "type": "vmess",
        "server": server,
        "server_port": port,
        "uuid": uuid,
        "security": v.get("scy").and_then(|s| s.as_str()).unwrap_or("auto"),
        "alter_id": alter_id,
    });

    let tls = v.get("tls").and_then(|s| s.as_str()).unwrap_or("");
    let sni = v
        .get("sni")
        .and_then(|s| s.as_str())
        .or_else(|| v.get("host").and_then(|s| s.as_str()))
        .unwrap_or("");

    if tls.eq_ignore_ascii_case("tls") {
        node["tls"] = json!({
            "enabled": true,
            "server_name": if sni.is_empty() { node["server"].as_str().unwrap_or("").to_string() } else { sni.to_string() },
            "utls": { "enabled": true, "fingerprint": "chrome" }
        });
    }

    let network = v.get("net").and_then(|s| s.as_str()).unwrap_or("");
    if network == "ws" {
        let mut transport = json!({
            "type": "ws"
        });
        if let Some(path) = v.get("path").and_then(|s| s.as_str()).map(|s| s.trim()).filter(|s| !s.is_empty()) {
            transport["path"] = json!(path);
        }
        if let Some(host) = v.get("host").and_then(|s| s.as_str()).map(|s| s.trim()).filter(|s| !s.is_empty()) {
            transport["headers"] = json!({ "Host": host });
        }
        node["transport"] = transport;
    }

    Some(node)
}

fn parse_ss_uri(uri: &str) -> Option<Value> {
    // ss:// 支持多种格式（这里尽量覆盖常见形式）：
    // 1) ss://method:password@host:port#name
    // 2) ss://BASE64(method:password)@host:port#name
    // 3) ss://BASE64(method:password@host:port)#name
    let raw = uri.trim().strip_prefix("ss://")?.trim();
    if raw.is_empty() {
        return None;
    }

    let (before_fragment, fragment) = match raw.split_once('#') {
        Some((a, b)) => (a, Some(b)),
        None => (raw, None),
    };

    // 忽略 plugin 等参数（目前只生成基础 SS 节点）
    let before_query = before_fragment.split('?').next().unwrap_or(before_fragment);
    let tag = {
        let decoded = fragment.and_then(|s| urlencoding::decode(s).ok()).map(|s| s.to_string());
        decoded.filter(|s| !s.trim().is_empty())
            .unwrap_or_else(|| "shadowsocks".to_string())
    };

    // 情况 A：包含 @，host:port 在 @ 之后
    if let Some((left, hostport)) = before_query.rsplit_once('@') {
        // left 可能是 method:password 或 base64(method:password)
        let (method, password) = parse_ss_method_password(left)?;
        let (server, server_port) = parse_host_port(hostport)?;
        return Some(json!({
            "tag": tag,
            "type": "shadowsocks",
            "server": server,
            "server_port": server_port,
            "method": method,
            "password": password
        }));
    }

    // 情况 B：整体是 base64(method:password@host:port)
    let decoded = base64_decode_relaxed(before_query)?;
    let decoded_text = String::from_utf8(decoded).ok()?;
    let (left, hostport) = decoded_text.rsplit_once('@')?;
    let (method, password) = parse_ss_method_password(left)?;
    let (server, server_port) = parse_host_port(hostport)?;

    Some(json!({
        "tag": tag,
        "type": "shadowsocks",
        "server": server,
        "server_port": server_port,
        "method": method,
        "password": password
    }))
}

fn base64_decode_relaxed(input: &str) -> Option<Vec<u8>> {
    // 有些订阅会省略 padding，或者使用 URL_SAFE。
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return None;
    }

    let mut s = trimmed.to_string();
    let rem = s.len() % 4;
    if rem != 0 {
        s.push_str(&"=".repeat(4 - rem));
    }

    base64::engine::general_purpose::STANDARD
        .decode(&s)
        .or_else(|_| base64::engine::general_purpose::URL_SAFE.decode(&s))
        .ok()
}

fn parse_host_port(hostport: &str) -> Option<(String, u64)> {
    let mut it = hostport.splitn(2, ':');
    let server = it.next()?.trim().to_string();
    let port = it.next()?.trim().parse::<u64>().ok()?;
    if server.is_empty() {
        return None;
    }
    Some((server, port))
}

fn parse_ss_method_password(input: &str) -> Option<(String, String)> {
    // input 可能是明文 method:password，也可能是 base64(method:password)
    if let Some((method, password)) = input.split_once(':') {
        let m = method.trim();
        let p = password.trim();
        if !m.is_empty() && !p.is_empty() {
            return Some((m.to_string(), p.to_string()));
        }
    }

    let decoded = base64_decode_relaxed(input.trim())?;
    let decoded_text = String::from_utf8(decoded).ok()?;
    let (method, password) = decoded_text.split_once(':')?;
    let m = method.trim();
    let p = password.trim();
    if m.is_empty() || p.is_empty() {
        return None;
    }
    Some((m.to_string(), p.to_string()))
}

// `selector` 分组的更新逻辑已迁移至 `app::singbox::config_generator`：
// - 订阅模块只负责“提取节点”，不再耦合模板替换与特定分组名字。

#[cfg(test)]
mod tests {
    use super::extract_nodes_from_subscription;

    #[test]
    fn parse_uri_list_vless_trojan() {
        let content = r#"
trojan://password@example.com:443?allowInsecure=1&type=ws&sni=example.com#Trojan%20Node
vless://26a1d547-b031-4139-9fc5-6671e1d0408a@example.com:443?type=tcp&encryption=none&security=tls&flow=xtls-rprx-vision&sni=example.com#VLESS%20Node
"#;
        let nodes = extract_nodes_from_subscription(content).expect("should parse");
        assert_eq!(nodes.len(), 2);
        assert_eq!(nodes[0]["type"].as_str().unwrap(), "trojan");
        assert_eq!(nodes[1]["type"].as_str().unwrap(), "vless");
    }

    #[test]
    fn parse_clash_yaml_ss() {
        let yaml = r#"
proxies:
  - name: "ss-test"
    type: ss
    server: 1.1.1.1
    port: 8388
    cipher: aes-128-gcm
    password: "pass"
"#;
        let nodes = extract_nodes_from_subscription(yaml).expect("should parse");
        assert_eq!(nodes.len(), 1);
        assert_eq!(nodes[0]["type"].as_str().unwrap(), "shadowsocks");
        assert_eq!(nodes[0]["tag"].as_str().unwrap(), "ss-test");
    }
}
