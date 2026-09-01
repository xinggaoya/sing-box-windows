use base64::Engine as _;
use serde_json::{json, Value};
use std::collections::HashMap;
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

                for outbound in outbounds.iter() {
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
                        Some(outbound_type) if is_supported_outbound_type(outbound_type) => {
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
                                        find_outbound_by_tag(outbounds, sub_tag)
                                    {
                                        let node_type =
                                            actual_node.get("type").and_then(|t| t.as_str());
                                        if let Some(type_str) = node_type {
                                            if is_supported_outbound_type(type_str) {
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
                                            if is_supported_outbound_type(t) {
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
            if nodes.is_empty()
                && (normalized_text.contains("vmess://")
                    || normalized_text.contains("ss://")
                    || normalized_text.contains("trojan://")
                    || normalized_text.contains("vless://")
                    || normalized_text.contains("hysteria2://")
                    || normalized_text.contains("tuic://")
                    || normalized_text.contains("anytls://"))
            {
                info!("检测到可能包含URI格式的节点，尝试逐行解析...");
                nodes.extend(extract_nodes_from_uri_list(&normalized_text));
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
        } else if c == '"' {
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

fn is_supported_outbound_type(node_type: &str) -> bool {
    matches!(
        node_type,
        "vless"
            | "vmess"
            | "trojan"
            | "shadowsocks"
            | "shadowsocksr"
            | "socks"
            | "http"
            | "hysteria2"
            | "tuic"
            | "anytls"
            | "snell"      // sing-box 1.14 新增
            | "wireguard"  // 1.13 已有但此前未识别
            | "tailscale"  // 1.14 endpoint（独立协议）
    )
}

/// 从 Clash 节点对象构造 TLS 配置（hysteria2/tuic/anytls 共用）。
///
/// 统一处理 sni/skip-cert-verify/alpn 字段映射，避免在多个分支重复样板代码。
fn build_tls_from_clash(clash_node: &Value, server: &str) -> Value {
    let sni = clash_node
        .get("sni")
        .and_then(|s| s.as_str())
        .unwrap_or("");
    let (insecure, alpn) = read_clash_tls_flags(clash_node);
    let mut tls = build_basic_tls_config(server, sni, insecure, None);
    if let Some(alpn_value) = alpn {
        tls["alpn"] = alpn_value;
    }
    tls
}

/// 从 URL query 中读取 insecure 开关。兼容 `allowInsecure` (clash/v2rayN 命名)
/// 和 `insecure` (sing-box 命名) 两种 query key。接受 `1` / `true` / `yes`
/// 这类"真值"字符串，统一通过 `parse_boolish` 解析；缺省为 `false`。
///
/// 机场的 vless+ws+tls 伪装节点 100% 需要 `insecure: true`（真实 SNI 域
/// 跟机场入口的 TLS 证书 SAN 不一致），如果 URI 里 `allowInsecure=1` 没传
/// 下去，sing-box 客户端会立刻 `CRYPTO_ERROR 0x12a` / SAN 不匹配。
fn read_insecure_from_query(query: &HashMap<String, String>) -> bool {
    query
        .get("allowInsecure")
        .or_else(|| query.get("insecure"))
        .and_then(|v| parse_boolish(v))
        .unwrap_or(false)
}

/// 从 URL query 中读取 `alpn`（CSV 字符串，如 `h2,h3`），并解析为 JSON 数组。
/// 缺省返回 `None`（调用方不应向 TLS 配置里塞 `alpn: null`）。
fn read_alpn_from_query(query: &HashMap<String, String>) -> Option<String> {
    query
        .get("alpn")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
        .map(|s| s.to_string())
}

/// 从 Clash 节点读取 `skip-cert-verify` + `alpn` 两个 TLS 标志，
/// 一次性返回 `(insecure, alpn_json)`。alpn 兼容数组 `[h3]` 和
/// 逗号字符串 `"h3,h2"` 两种 YAML 写法。
fn read_clash_tls_flags(clash_node: &Value) -> (bool, Option<Value>) {
    let insecure = clash_node
        .get("skip-cert-verify")
        .and_then(|v| v.as_bool())
        .unwrap_or(false);
    let alpn = if let Some(alpn_value) = clash_node.get("alpn") {
        if let Some(arr) = alpn_value.as_array() {
            let list: Vec<Value> = arr
                .iter()
                .filter_map(|v| v.as_str().map(|s| Value::String(s.to_string())))
                .collect();
            if list.is_empty() { None } else { Some(Value::Array(list)) }
        } else if let Some(s) = alpn_value.as_str() {
            parse_csv_string_array(Some(s))
        } else {
            None
        }
    } else {
        None
    };
    (insecure, alpn)
}

fn convert_clash_node_to_singbox(clash_node: &Value) -> Option<Value> {
    let node_type = clash_node.get("type").and_then(|t| t.as_str())?;
    let name = clash_node.get("name").and_then(|n| n.as_str())?;
    let server = clash_node.get("server").and_then(|s| s.as_str())?;
    // serde_yaml 可能把 "22892" 解析为字符串而非整数；同时兼容数值与字符串两种形式。
    let port = clash_node
        .get("port")
        .and_then(|p| p.as_u64().or_else(|| p.as_str().and_then(|s| s.parse::<u64>().ok())))?;

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
                let sni = clash_node.get("servername").and_then(|s| s.as_str()).unwrap_or("");
                let (insecure, alpn_value) = read_clash_tls_flags(clash_node);
                let alpn_csv = alpn_value.and_then(|v| {
                    v.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str())
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                });
                let tls = build_tls_config(server, sni, "chrome", insecure, alpn_csv.as_deref());
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
                let sni = clash_node.get("servername").and_then(|s| s.as_str()).unwrap_or("");
                let (insecure, alpn_value) = read_clash_tls_flags(clash_node);
                let alpn_csv = alpn_value.and_then(|v| {
                    v.as_array().map(|arr| {
                        arr.iter()
                            .filter_map(|x| x.as_str())
                            .collect::<Vec<_>>()
                            .join(",")
                    })
                });
                let tls = build_tls_config(server, sni, "chrome", insecure, alpn_csv.as_deref());
                if let Some(obj) = node.as_object_mut() {
                    obj.insert("tls".to_string(), tls);
                }
            }

            Some(node)
        }
        "trojan" => {
            let password = clash_node.get("password").and_then(|p| p.as_str())?;
            // 与 hysteria2 / tuic / anytls 统一走 build_basic_tls_config（不带 utls），
            // 完整透传 skip-cert-verify + alpn，避免漏 insecure。
            let sni = clash_node.get("sni").and_then(|s| s.as_str()).unwrap_or(server);
            let tls_enabled = clash_node.get("tls").and_then(|t| t.as_bool()).unwrap_or(true);
            let (insecure, alpn_value) = read_clash_tls_flags(clash_node);
            let alpn_csv = alpn_value.and_then(|v| {
                v.as_array().map(|arr| {
                    arr.iter()
                        .filter_map(|x| x.as_str())
                        .collect::<Vec<_>>()
                        .join(",")
                })
            });
            let mut tls = build_basic_tls_config(sni, sni, insecure, alpn_csv.as_deref());
            if let Some(obj) = tls.as_object_mut() {
                obj.insert("enabled".to_string(), json!(tls_enabled));
            }
            Some(json!({
                "tag": name,
                "type": "trojan",
                "server": server,
                "server_port": port,
                "password": password,
                "tls": tls
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
        "hysteria2" => {
            // sing-box hysteria2 用 password 字段；up/down 在 Clash 是 up/down（MBps），sing-box 用 up_mbps/down_mbps。
            let password = clash_node.get("password").and_then(|p| p.as_str())?;
            let mut node = json!({
                "tag": name,
                "type": "hysteria2",
                "server": server,
                "server_port": port,
                "password": password,
                "tls": build_tls_from_clash(clash_node, server)
            });

            if let Some(up) = clash_node.get("up").and_then(|v| {
                v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
            }) {
                node["up_mbps"] = json!(up);
            }
            if let Some(down) = clash_node.get("down").and_then(|v| {
                v.as_u64().or_else(|| v.as_str().and_then(|s| s.parse::<u64>().ok()))
            }) {
                node["down_mbps"] = json!(down);
            }

            // Clash hysteria2 的 salamander 混淆：obfs / obfs-password
            if let Some(obfs_type) = clash_node
                .get("obfs")
                .and_then(|o| o.as_str())
                .filter(|s| !s.is_empty())
            {
                let mut obfs = json!({ "type": obfs_type });
                if let Some(obfs_password) = clash_node
                    .get("obfs-password")
                    .and_then(|p| p.as_str())
                    .filter(|s| !s.is_empty())
                {
                    obfs["password"] = json!(obfs_password);
                }
                node["obfs"] = obfs;
            }

            Some(node)
        }
        "tuic" => {
            let uuid = clash_node.get("uuid").and_then(|u| u.as_str())?;
            let mut node = json!({
                "tag": name,
                "type": "tuic",
                "server": server,
                "server_port": port,
                "uuid": uuid,
                "tls": build_tls_from_clash(clash_node, server)
            });

            if let Some(password) = clash_node.get("password").and_then(|p| p.as_str()) {
                node["password"] = json!(password);
            }
            if let Some(congestion) = clash_node
                .get("congestion-controller")
                .and_then(|c| c.as_str())
                .filter(|s| !s.is_empty())
            {
                node["congestion_control"] = json!(congestion);
            }
            if let Some(udp_relay_mode) = clash_node
                .get("udp-relay-mode")
                .and_then(|c| c.as_str())
                .filter(|s| !s.is_empty())
            {
                node["udp_relay_mode"] = json!(udp_relay_mode);
            }
            if let Some(reduce_rtt) = clash_node.get("reduce-rtt").and_then(|v| v.as_bool()) {
                node["reduce_rtt"] = json!(reduce_rtt);
            }

            if let Some(tls_obj) = node.get_mut("tls").and_then(|t| t.as_object_mut()) {
                if let Some(disable_sni) = clash_node.get("disable-sni").and_then(|v| v.as_bool()) {
                    tls_obj.insert("disable_sni".to_string(), json!(disable_sni));
                }
            }

            Some(node)
        }
        "anytls" => {
            let password = clash_node.get("password").and_then(|p| p.as_str())?;
            let mut node = json!({
                "tag": name,
                "type": "anytls",
                "server": server,
                "server_port": port,
                "password": password,
                "tls": build_tls_from_clash(clash_node, server)
            });

            if let Some(interval) = clash_node
                .get("idle-session-check-interval")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                node["idle_session_check_interval"] = json!(interval);
            }
            if let Some(timeout) = clash_node
                .get("idle-session-timeout")
                .and_then(|v| v.as_str())
                .filter(|s| !s.is_empty())
            {
                node["idle_session_timeout"] = json!(timeout);
            }

            Some(node)
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
    if uri.starts_with("hysteria2://") {
        return parse_hysteria2_uri(uri);
    }
    if uri.starts_with("tuic://") {
        return parse_tuic_uri(uri);
    }
    if uri.starts_with("anytls://") {
        return parse_anytls_uri(uri);
    }
    if uri.starts_with("snell://") {
        // sing-box 1.14 新增：snell 协议
        return parse_snell_uri(uri);
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

fn normalize_fingerprint(fingerprint: Option<&str>) -> String {
    fingerprint
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("chrome")
        .to_string()
}

fn parse_query_map(url: &Url) -> HashMap<String, String> {
    let mut query = HashMap::<String, String>::new();
    for (k, v) in url.query_pairs() {
        query.insert(k.to_string(), v.to_string());
    }
    query
}

fn parse_boolish(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}

fn parse_csv_string_array(value: Option<&str>) -> Option<Value> {
    let list: Vec<Value> = value?
        .split(',')
        .map(|item| item.trim())
        .filter(|item| !item.is_empty())
        .map(|item| Value::String(item.to_string()))
        .collect();

    if list.is_empty() {
        None
    } else {
        Some(Value::Array(list))
    }
}

fn build_tls_config(
    server: &str,
    server_name: &str,
    fingerprint: &str,
    insecure: bool,
    alpn: Option<&str>,
) -> Value {
    let mut tls = json!({
        "enabled": true,
        "server_name": if server_name.is_empty() { server.to_string() } else { server_name.to_string() },
        "insecure": insecure,
        "utls": {
            "enabled": true,
            "fingerprint": normalize_fingerprint(Some(fingerprint))
        }
    });
    if let Some(csv) = alpn {
        if let Some(parsed) = parse_csv_string_array(Some(csv)) {
            tls["alpn"] = parsed;
        }
    }
    tls
}

fn build_basic_tls_config(
    server: &str,
    server_name: &str,
    insecure: bool,
    alpn: Option<&str>,
) -> Value {
    let mut tls = json!({
        "enabled": true,
        "server_name": if server_name.is_empty() { server.to_string() } else { server_name.to_string() },
        "insecure": insecure
    });
    if let Some(csv) = alpn {
        if let Some(parsed) = parse_csv_string_array(Some(csv)) {
            tls["alpn"] = parsed;
        }
    }
    tls
}

fn parse_vless_uri(uri: &str) -> Option<Value> {
    let url = Url::parse(uri).ok()?;
    let uuid = url.username().trim();
    if uuid.is_empty() {
        return None;
    }

    let server = url.host_str()?.to_string();
    let server_port = url.port().unwrap_or(443) as u64;

    let query = parse_query_map(&url);

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
    if let Some(flow) = query
        .get("flow")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        node["flow"] = json!(flow);
    }

    // TLS/REALITY：简化处理——只要显式声明或存在 SNI 就默认启用 TLS
    let security = query.get("security").map(|s| s.as_str()).unwrap_or("");
    let sni = query
        .get("sni")
        .or_else(|| query.get("servername"))
        .map(|s| s.trim())
        .unwrap_or("");
    let fingerprint = normalize_fingerprint(query.get("fp").map(|s| s.as_str()));

    if security == "tls" || security == "reality" || !sni.is_empty() {
        if security == "reality" {
            // REALITY 用 x25519 公钥做认证，不依赖 TLS 证书，因此不需要 insecure / alpn。
            // 单独构造，避免 `utls` 出现在 reality 分支下造成混淆。
            let mut tls = json!({
                "enabled": true,
                "server_name": if sni.is_empty() { server.clone() } else { sni.to_string() },
                "utls": {
                    "enabled": true,
                    "fingerprint": fingerprint.clone()
                }
            });
            let mut reality = json!({ "enabled": true });
            if let Some(public_key) = query.get("pbk").map(|s| s.trim()).filter(|s| !s.is_empty()) {
                reality["public_key"] = json!(public_key);
            }
            if let Some(short_id) = query.get("sid").map(|s| s.trim()).filter(|s| !s.is_empty()) {
                reality["short_id"] = json!(short_id);
            }
            tls["reality"] = reality;
            node["tls"] = tls;
        } else {
            // tls: 必须显式读 allowInsecure / alpn，机场伪装节点 100% 需要 insecure: true。
            let insecure = read_insecure_from_query(&query);
            let alpn = read_alpn_from_query(&query);
            node["tls"] = build_tls_config(
                &server,
                sni,
                &fingerprint,
                insecure,
                alpn.as_deref(),
            );
        }
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
        if let Some(path) = query
            .get("path")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            transport["path"] = json!(path);
        }
        if let Some(host) = query
            .get("host")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
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

    let query = parse_query_map(&url);

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

    let insecure = read_insecure_from_query(&query);
    let alpn = read_alpn_from_query(&query);

    let mut node = json!({
        "tag": tag,
        "type": "trojan",
        "server": server,
        "server_port": server_port,
        "password": password,
        "tls": build_tls_config(&server, sni, "chrome", insecure, alpn.as_deref())
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
        if let Some(path) = query
            .get("path")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            transport["path"] = json!(path);
        }
        if let Some(host) = query
            .get("host")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            transport["headers"] = json!({ "Host": host });
        }
        node["transport"] = transport;
    }

    Some(node)
}

fn parse_hysteria2_uri(uri: &str) -> Option<Value> {
    let url = Url::parse(uri).ok()?;
    let password = url.username().trim();
    if password.is_empty() {
        return None;
    }

    let server = url.host_str()?.to_string();
    let server_port = url.port().unwrap_or(443) as u64;

    let query = parse_query_map(&url);

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
        .get("insecure")
        .map(|v| v.as_str())
        .and_then(parse_boolish)
        .unwrap_or(false);

    let alpn = read_alpn_from_query(&query);
    let tls = build_basic_tls_config(&server, sni, insecure, alpn.as_deref());

    let mut node = json!({
        "tag": tag,
        "type": "hysteria2",
        "server": server,
        "server_port": server_port,
        "password": password,
        "tls": tls
    });

    // Parse optional hysteria2 obfuscation (salamander) parameters:
    //   obfs=<type>&obfs-password=<password>
    if let Some(obfs_type) = query
        .get("obfs")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let mut obfs = serde_json::Map::new();
        obfs.insert("type".to_string(), json!(obfs_type));
        if let Some(obfs_password) = query
            .get("obfs-password")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            obfs.insert("password".to_string(), json!(obfs_password));
        }
        node["obfs"] = json!(obfs);
    }

    // upmbps/downmbps 是 hysteria2 URI 的客户端带宽声明（MBps）
    if let Some(up) = query.get("upmbps").and_then(|s| s.trim().parse::<u64>().ok()) {
        node["up_mbps"] = json!(up);
    }
    if let Some(down) = query.get("downmbps").and_then(|s| s.trim().parse::<u64>().ok()) {
        node["down_mbps"] = json!(down);
    }

    // === sing-box 1.14 新增 hysteria2 字段 ===

    // bbr_profile（BBR 拥塞控制）
    if let Some(bbr_profile) = query
        .get("bbr_profile")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        node["bbr_profile"] = json!(bbr_profile);
    }

    // disable_chrome_parrot（Chrome QUIC 指纹伪装关闭，用于 Ed25519 服务器）
    if let Some(disable_chrome_parrot) = query
        .get("disable_chrome_parrot")
        .map(|s| s.as_str())
        .and_then(parse_boolish)
    {
        node["disable_chrome_parrot"] = json!(disable_chrome_parrot);
    }

    // hop_interval_max / hop_interval（跳跃间隔随机化）
    if let Some(hop_interval) = query
        .get("hop_interval_max")
        .or_else(|| query.get("hop_interval"))
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        node["hop_interval_max"] = json!(hop_interval);
    }

    // realm（Hysteria2 NAT traversal 配套字段，详见 1.14 changelog）
    if let Some(realm) = query
        .get("realm")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        node["realm"] = json!(realm);
    }

    // fastopen 参数（0/1）
    if let Some(fast_open) = query
        .get("fastopen")
        .map(|s| s.as_str())
        .and_then(parse_boolish)
    {
        node["tcp_fast_open"] = json!(fast_open);
    }

    // mport 多端口参数（逗号分隔），映射到 server_ports
    if let Some(mport) = query.get("mport").map(|s| s.trim()).filter(|s| !s.is_empty()) {
        let ports: Vec<Value> = mport
            .split(',')
            .map(|p| p.trim())
            .filter(|p| !p.is_empty())
            .map(|p| Value::String(p.to_string()))
            .collect();
        if !ports.is_empty() {
            node["server_ports"] = json!(ports);
        }
    }

    Some(node)
}

fn parse_tuic_uri(uri: &str) -> Option<Value> {
    let url = Url::parse(uri).ok()?;
    let uuid = url.username().trim();
    if uuid.is_empty() {
        return None;
    }

    let server = url.host_str()?.to_string();
    let server_port = url.port().unwrap_or(443) as u64;
    let query = parse_query_map(&url);
    let tag = {
        let decoded = decode_tag(url.fragment());
        if decoded.is_empty() {
            default_tag_for_url(&url)
        } else {
            decoded
        }
    };

    let password = url.password().map(str::trim).filter(|value| !value.is_empty());
    let sni = query
        .get("sni")
        .or_else(|| query.get("servername"))
        .map(|s| s.trim())
        .unwrap_or("");
    let insecure = query
        .get("insecure")
        .map(|s| s.as_str())
        .and_then(parse_boolish)
        .unwrap_or(false);

    let alpn = read_alpn_from_query(&query);

    let mut node = json!({
        "tag": tag,
        "type": "tuic",
        "server": server,
        "server_port": server_port,
        "uuid": uuid,
        "tls": build_basic_tls_config(&server, sni, insecure, alpn.as_deref())
    });

    if let Some(password) = password {
        node["password"] = json!(password);
    }

    if let Some(alpn) = parse_csv_string_array(query.get("alpn").map(|s| s.as_str())) {
        node["tls"]["alpn"] = alpn;
    }

    if let Some(congestion_control) = query
        .get("congestion_control")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        node["congestion_control"] = json!(congestion_control);
    }
    if let Some(udp_relay_mode) = query
        .get("udp_relay_mode")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        node["udp_relay_mode"] = json!(udp_relay_mode);
    }
    if let Some(heartbeat) = query
        .get("heartbeat")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        node["heartbeat"] = json!(heartbeat);
    }
    if let Some(network) = query
        .get("network")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        node["network"] = json!(network);
    }
    if let Some(udp_over_stream) = query
        .get("udp_over_stream")
        .map(|s| s.as_str())
        .and_then(parse_boolish)
    {
        node["udp_over_stream"] = json!(udp_over_stream);
    }
    if let Some(zero_rtt_handshake) = query
        .get("zero_rtt_handshake")
        .map(|s| s.as_str())
        .and_then(parse_boolish)
    {
        node["zero_rtt_handshake"] = json!(zero_rtt_handshake);
    }

    Some(node)
}

fn parse_anytls_uri(uri: &str) -> Option<Value> {
    let url = Url::parse(uri).ok()?;
    let password = url.username().trim();
    if password.is_empty() {
        return None;
    }

    let server = url.host_str()?.to_string();
    let server_port = url.port().unwrap_or(443) as u64;
    let query = parse_query_map(&url);
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
        .or_else(|| query.get("servername"))
        .map(|s| s.trim())
        .unwrap_or("");
    let insecure = query
        .get("insecure")
        .map(|s| s.as_str())
        .and_then(parse_boolish)
        .unwrap_or(false);

    let alpn = read_alpn_from_query(&query);

    let mut node = json!({
        "tag": tag,
        "type": "anytls",
        "server": server,
        "server_port": server_port,
        "password": password,
        "tls": build_basic_tls_config(&server, sni, insecure, alpn.as_deref())
    });
    if let Some(idle_session_check_interval) = query
        .get("idle_session_check_interval")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        node["idle_session_check_interval"] = json!(idle_session_check_interval);
    }
    if let Some(idle_session_timeout) = query
        .get("idle_session_timeout")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        node["idle_session_timeout"] = json!(idle_session_timeout);
    }
    if let Some(min_idle_session) = query
        .get("min_idle_session")
        .and_then(|s| s.trim().parse::<u64>().ok())
    {
        node["min_idle_session"] = json!(min_idle_session);
    }

    Some(node)
}

/// 解析 sing-box 1.14 新增的 snell 协议 URI：
///   snell://PSK@host:port?version=2&obfs=http#tag
fn parse_snell_uri(uri: &str) -> Option<Value> {
    let url = Url::parse(uri).ok()?;
    let psk = url.username().trim();
    if psk.is_empty() {
        return None;
    }
    let server = url.host_str()?.to_string();
    let server_port = url.port().unwrap_or(8388) as u64;
    let query = parse_query_map(&url);

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
        "type": "snell",
        "server": server,
        "server_port": server_port,
        "psk": psk,
    });

    // version（1/2/3/4/5，常见为 2）
    if let Some(version) = query
        .get("version")
        .and_then(|s| s.trim().parse::<u64>().ok())
    {
        node["version"] = json!(version);
    }

    // obfs: 可选 salamander 混淆（snell v3+ 才支持）
    if let Some(obfs_type) = query
        .get("obfs")
        .map(|s| s.trim())
        .filter(|s| !s.is_empty())
    {
        let mut obfs = json!({ "type": obfs_type });
        if let Some(obfs_host) = query
            .get("obfs-host")
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            obfs["host"] = json!(obfs_host);
        }
        node["obfs"] = obfs;
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
        // vmess JSON 里 `allowInsecure` 字段（v2rayN / v2rayNG 客户端）映射到 tls.insecure。
        // `alpn` 字段（数组 / CSV 字符串）一并透传；缺省为 None。
        let insecure = v
            .get("allowInsecure")
            .and_then(|x| x.as_bool())
            .or_else(|| v.get("insecure").and_then(|x| x.as_bool()))
            .unwrap_or(false);
        let alpn_csv: Option<String> = if let Some(arr) = v.get("alpn").and_then(|x| x.as_array()) {
            let csv: Vec<String> = arr
                .iter()
                .filter_map(|x| x.as_str().map(|s| s.to_string()))
                .collect();
            if csv.is_empty() { None } else { Some(csv.join(",")) }
        } else {
            v.get("alpn")
                .and_then(|x| x.as_str())
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
        };
        node["tls"] = build_tls_config(
            node["server"].as_str().unwrap_or(""),
            sni,
            "chrome",
            insecure,
            alpn_csv.as_deref(),
        );
    }

    let network = v.get("net").and_then(|s| s.as_str()).unwrap_or("");
    if network == "ws" {
        let mut transport = json!({
            "type": "ws"
        });
        if let Some(path) = v
            .get("path")
            .and_then(|s| s.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
            transport["path"] = json!(path);
        }
        if let Some(host) = v
            .get("host")
            .and_then(|s| s.as_str())
            .map(|s| s.trim())
            .filter(|s| !s.is_empty())
        {
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
        let decoded = fragment
            .and_then(|s| urlencoding::decode(s).ok())
            .map(|s| s.to_string());
        decoded
            .filter(|s| !s.trim().is_empty())
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
#[path = "parser.tests.rs"]
mod tests;
