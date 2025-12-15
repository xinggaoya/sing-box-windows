use serde_json::{json, Value};
use url::Url;
use tracing::info;

/// 将原始订阅内容解析为 JSON Value，支持 JSON/YAML。
/// 失败时返回 Err，调用方可根据需要选择继续尝试 Base64 等兜底。
pub fn parse_to_value(content: &str) -> Result<(Value, &'static str), Box<dyn std::error::Error>> {
    let cleaned_content = clean_json_content(content);
    match serde_json::from_str::<Value>(&cleaned_content) {
        Ok(v) => Ok((v, "json")),
        Err(json_err) => {
            let yaml_source = content.trim_start_matches('\u{FEFF}');
            match serde_yaml::from_str::<Value>(yaml_source) {
                Ok(v) => Ok((v, "yaml")),
                Err(yaml_err) => Err(format!(
                    "JSON 解析失败: {}; YAML 解析失败: {}",
                    json_err, yaml_err
                )
                .into()),
            }
        }
    }
}

pub fn extract_nodes_from_subscription(
    content: &str,
) -> Result<Vec<Value>, Box<dyn std::error::Error>> {
    let (content_json, format_label) = match parse_to_value(content) {
        Ok(v) => v,
        Err(e) => {
            info!("内容无法解析为 JSON/YAML，将尝试 URI 列表解析: {}", e);
            let uri_nodes = parse_uri_nodes(content);
            if !uri_nodes.is_empty() {
                return Ok(uri_nodes);
            }
            return Ok(Vec::new());
        }
    };

    let mut nodes = Vec::new();

    if let Some(outbounds) = content_json.get("outbounds").and_then(|o| o.as_array()) {
        info!(
            "检测到sing-box格式({})，outbounds数组长度: {}",
            format_label,
            outbounds.len()
        );

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
                | Some("hysteria2") | Some("tuic") => {
                    nodes.push(node_with_tag);
                }
                _ => {}
            }
        }

        if nodes.is_empty() {
            info!("在顶级outbounds中未找到支持的节点，尝试递归解析...");
            for outbound in outbounds {
                if let Some(sub_outbounds) = outbound.get("outbounds").and_then(|o| o.as_array()) {
                    for sub_outbound in sub_outbounds {
                        if let Some(sub_tag) = sub_outbound.as_str() {
                            if let Some(actual_node) = find_outbound_by_tag(&outbounds, sub_tag) {
                                let node_type = actual_node.get("type").and_then(|t| t.as_str());
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
                                        "tuic",
                                    ]
                                    .contains(&type_str)
                                    {
                                        let node_with_tag =
                                            if actual_node.get("tag").is_none() {
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
    } else if let Some(proxies) = content_json.get("proxies").and_then(|p| p.as_array()) {
        info!(
            "检测到Clash/Mihomo格式({})，proxies数组长度: {}",
            format_label,
            proxies.len()
        );

        for proxy in proxies {
            if let Some(converted_node) = convert_clash_node_to_singbox(proxy) {
                nodes.push(converted_node);
            }
        }
    } else {
        info!("未找到标准的outbounds或proxies数组，尝试解析其他位置...");

        if let Some(obj) = content_json.as_object() {
            let keys: Vec<&String> = obj.keys().collect();
            info!("顶级键: {:?}", keys);

            for (_key, value) in obj {
                if let Some(arr) = value.as_array() {
                    for item in arr {
                        if let Some(item_obj) = item.as_object() {
                            let has_type = item_obj.contains_key("type");
                            let has_tag = item_obj.contains_key("tag") || item_obj.contains_key("name");
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
                                        "tuic",
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
    if fixed_nodes.is_empty() {
        let uri_nodes = parse_uri_nodes(content);
        if !uri_nodes.is_empty() {
            info!("从 URI 列表提取到 {} 个节点", uri_nodes.len());
            return Ok(uri_nodes);
        }
    }
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

fn parse_uri_nodes(content: &str) -> Vec<Value> {
    let mut nodes = Vec::new();
    for raw_line in content.lines() {
        let line = raw_line.trim();
        if line.is_empty() {
            continue;
        }

        if line.starts_with("vmess://") {
            if let Some(node) = parse_vmess_uri(line) {
                nodes.push(node);
            }
            continue;
        }
        if line.starts_with("vless://") {
            if let Some(node) = parse_vless_uri(line) {
                nodes.push(node);
            }
            continue;
        }
        if line.starts_with("trojan://") {
            if let Some(node) = parse_trojan_uri(line) {
                nodes.push(node);
            }
            continue;
        }
        if line.starts_with("ss://") {
            // TODO: ss URI 支持可按需补充
            continue;
        }
    }
    nodes
}

fn parse_vmess_uri(uri: &str) -> Option<Value> {
    let b64 = uri.trim_start_matches("vmess://");
    let padded = pad_base64(b64);
    let decoded = base64::decode(padded).ok()?;
    let txt = String::from_utf8(decoded).ok()?;
    let v: Value = serde_json::from_str(&txt).ok()?;

    let server = v.get("add").and_then(|s| s.as_str())?;
    let port_str = v
        .get("port")
        .and_then(|p| p.as_str().map(|s| s.to_string()))
        .or_else(|| v.get("port").and_then(|p| p.as_i64().map(|n| n.to_string())))?;
    let port = port_str.parse::<u16>().ok()?;
    let uuid = v
        .get("id")
        .or_else(|| v.get("uuid"))
        .and_then(|u| u.as_str())?;
    let tag = v
        .get("ps")
        .and_then(|p| p.as_str())
        .filter(|s| !s.is_empty())
        .unwrap_or(server);

    let mut node = json!({
        "tag": tag,
        "type": "vmess",
        "server": server,
        "server_port": port,
        "uuid": uuid,
        "security": v.get("scy").and_then(|c| c.as_str()).unwrap_or("auto"),
        "alter_id": v.get("aid").and_then(|a| a.as_u64()).unwrap_or(0)
    });

    let net = v.get("net").and_then(|n| n.as_str()).unwrap_or("");
    if !net.is_empty() {
        apply_vmess_transport_from_map(&mut node, net, &v);
    }

    let tls_enabled = v
        .get("tls")
        .and_then(|t| t.as_str())
        .map(|s| s.eq_ignore_ascii_case("tls"))
        .unwrap_or(false);
    if tls_enabled {
        let sni = v
            .get("sni")
            .or_else(|| v.get("host"))
            .and_then(|s| s.as_str())
            .unwrap_or(server);
        node["tls"] = json!({
            "enabled": true,
            "server_name": sni,
            "utls": { "enabled": true, "fingerprint": "chrome" }
        });
    }

    Some(node)
}

fn apply_vmess_transport_from_map(node: &mut Value, net: &str, v: &Value) {
    match net {
        "ws" => {
            let mut t = json!({"type": "ws"});
            if let Some(path) = v.get("path").and_then(|p| p.as_str()) {
                t["path"] = json!(path);
            }
            if let Some(host) = v.get("host").and_then(|h| h.as_str()) {
                t["headers"] = json!({"Host": host});
            }
            if let Some(obj) = node.as_object_mut() {
                obj.insert("transport".to_string(), t);
            }
        }
        "grpc" => {
            let mut t = json!({"type": "grpc"});
            if let Some(service) = v.get("path").and_then(|p| p.as_str()) {
                t["service_name"] = json!(service.trim_start_matches('/'));
            }
            if let Some(obj) = node.as_object_mut() {
                obj.insert("transport".to_string(), t);
            }
        }
        "h2" | "http" => {
            let mut t = json!({"type": "http"});
            if let Some(path) = v.get("path").and_then(|p| p.as_str()) {
                t["path"] = json!(path);
            }
            if let Some(host) = v.get("host").and_then(|h| h.as_str()) {
                t["headers"] = json!({"Host": host});
            }
            if let Some(obj) = node.as_object_mut() {
                obj.insert("transport".to_string(), t);
            }
        }
        _ => {}
    }
}

fn parse_trojan_uri(uri: &str) -> Option<Value> {
    let url = Url::parse(uri).ok()?;
    let password = url.username();
    let host = url.host_str()?;
    let port = url.port().unwrap_or(443);
    let tag = url.fragment().map(decode_label).unwrap_or_else(|| host.to_string());

    let mut node = json!({
        "tag": tag,
        "type": "trojan",
        "server": host,
        "server_port": port,
        "password": password
    });

    let mut tls_obj = json!({ "enabled": true });
    if let Some(sni) = url
        .query_pairs()
        .find(|(k, _)| k == "sni" || k == "peer")
        .map(|(_, v)| v.to_string())
    {
        tls_obj["server_name"] = json!(sni);
    }
    if let Some(insecure) = url
        .query_pairs()
        .find(|(k, _)| k == "allowInsecure")
        .and_then(|(_, v)| v.parse::<i32>().ok())
    {
        tls_obj["insecure"] = json!(insecure == 1);
    }
    node["tls"] = tls_obj;

    if let Some(transport) = url.query_pairs().find(|(k, _)| k == "type") {
        if transport.1 == "ws" {
            let mut t = json!({"type": "ws"});
            if let Some(path) = url.query_pairs().find(|(k, _)| k == "path") {
                t["path"] = json!(path.1.to_string());
            }
            if let Some(host_h) = url.query_pairs().find(|(k, _)| k == "host") {
                t["headers"] = json!({"Host": host_h.1.to_string()});
            }
            node["transport"] = t;
        }
    }

    Some(node)
}

fn parse_vless_uri(uri: &str) -> Option<Value> {
    let url = Url::parse(uri).ok()?;
    let uuid = url.username();
    let host = url.host_str()?;
    let port = url.port().unwrap_or(443);
    let tag = url.fragment().map(decode_label).unwrap_or_else(|| host.to_string());

    let mut node = json!({
        "tag": tag,
        "type": "vless",
        "server": host,
        "server_port": port,
        "uuid": uuid
    });

    let mut tls_needed = false;
    let mut reality_obj = json!({});
    for (k, v) in url.query_pairs() {
        match k.as_ref() {
            "security" => {
                if v.eq_ignore_ascii_case("tls") || v.eq_ignore_ascii_case("reality") {
                    tls_needed = true;
                }
            }
            "sni" => {
                node["tls"]["server_name"] = json!(v.to_string());
            }
            "alpn" => {
                node["tls"]["alpn"] = json!([v.to_string()]);
            }
            "flow" => {
                node["flow"] = json!(v.to_string());
            }
            "pbk" | "publicKey" => {
                reality_obj["public_key"] = json!(v.to_string());
            }
            "sid" | "shortId" => {
                reality_obj["short_id"] = json!(v.to_string());
            }
            "fp" | "fingerprint" => {
                reality_obj["fingerprint"] = json!(v.to_string());
            }
            "spx" | "spiderX" => {
                reality_obj["spider_x"] = json!(v.to_string());
            }
            _ => {}
        }
    }

    if tls_needed {
        let mut tls_obj = json!({ "enabled": true });
        if let Some(sni) = node
            .get("tls")
            .and_then(|t| t.get("server_name"))
            .and_then(|s| s.as_str())
        {
            tls_obj["server_name"] = json!(sni);
        }
        node["tls"] = tls_obj;
    }

    if reality_obj.as_object().map(|m| !m.is_empty()).unwrap_or(false) {
        reality_obj["enabled"] = json!(true);
        node["reality"] = reality_obj;
    }

    if let Some((_, v)) = url.query_pairs().find(|(k, _)| k == "type") {
        match v.as_ref() {
            "ws" => {
                let mut t = json!({"type": "ws"});
                if let Some((_, path)) = url.query_pairs().find(|(k, _)| k == "path") {
                    t["path"] = json!(path.to_string());
                }
                if let Some((_, host_h)) = url.query_pairs().find(|(k, _)| k == "host") {
                    t["headers"] = json!({"Host": host_h.to_string()});
                }
                node["transport"] = t;
            }
            "grpc" => {
                let mut t = json!({"type": "grpc"});
                if let Some((_, service)) = url.query_pairs().find(|(k, _)| k == "serviceName") {
                    t["service_name"] = json!(service.to_string());
                }
                node["transport"] = t;
            }
            "http" | "h2" => {
                let mut t = json!({"type": "http"});
                if let Some((_, path)) = url.query_pairs().find(|(k, _)| k == "path") {
                    t["path"] = json!(path.to_string());
                }
                if let Some((_, host_h)) = url.query_pairs().find(|(k, _)| k == "host") {
                    t["headers"] = json!({"Host": host_h.to_string()});
                }
                node["transport"] = t;
            }
            _ => {}
        }
    }

    Some(node)
}

fn decode_label(raw: &str) -> String {
    urlencoding::decode(raw)
        .map(|cow| cow.to_string())
        .unwrap_or_else(|_| raw.to_string())
}

fn pad_base64(input: &str) -> String {
    let mut s = input.trim().to_string();
    let rem = s.len() % 4;
    if rem > 0 {
        s.extend(std::iter::repeat('=').take(4 - rem));
    }
    s
}

fn build_tls(
    clash_node: &Value,
    server: &str,
    fingerprint: Option<&str>,
) -> Option<Value> {
    let enable_tls = clash_node
        .get("tls")
        .and_then(|t| t.as_bool())
        .unwrap_or(true);
    if !enable_tls {
        return None;
    }

    let mut tls = json!({ "enabled": true });

    if let Some(sni) = clash_node
        .get("server-name")
        .or_else(|| clash_node.get("servername"))
        .or_else(|| clash_node.get("sni"))
        .and_then(|s| s.as_str())
    {
        tls["server_name"] = json!(sni);
    } else if !server.is_empty() {
        tls["server_name"] = json!(server);
    }

    if let Some(alpn) = clash_node.get("alpn") {
        tls["alpn"] = alpn.clone();
    }

    if let Some(skip) = clash_node
        .get("skip-cert-verify")
        .and_then(|v| v.as_bool())
    {
        tls["insecure"] = json!(skip);
    }

    if let Some(fp) = fingerprint {
        tls["utls"] = json!({ "enabled": true, "fingerprint": fp });
    }

    Some(tls)
}

fn apply_ws_transport(clash_node: &Value, transport: &mut Value) {
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
}

fn apply_grpc_transport(clash_node: &Value, transport: &mut Value) {
    if let Some(grpc_opts) = clash_node.get("grpc-opts") {
        if let Some(service) = grpc_opts.get("grpc-service-name").and_then(|p| p.as_str()) {
            transport["service_name"] = json!(service);
        }
        if let Some(host) = grpc_opts.get("grpc-headers") {
            transport["headers"] = host.clone();
        }
    }
}

fn apply_http_transport(clash_node: &Value, transport: &mut Value) {
    if let Some(http_opts) = clash_node.get("h2-opts").or_else(|| clash_node.get("http-opts")) {
        if let Some(path) = http_opts.get("path").and_then(|p| p.as_array()) {
            // 取首个 path，避免过度复杂化
            if let Some(first) = path.first().and_then(|p| p.as_str()) {
                transport["path"] = json!(first);
            }
        }
        if let Some(host) = http_opts.get("host") {
            transport["headers"] = json!({ "Host": host.clone() });
        }
    }
}

fn apply_transport(clash_node: &Value, node: &mut Value) {
    if let Some(network) = clash_node.get("network").and_then(|n| n.as_str()) {
        let mut transport = match network {
            "ws" => json!({ "type": "ws" }),
            "grpc" => json!({ "type": "grpc" }),
            "http" | "h2" => json!({ "type": "http" }),
            _ => json!({}),
        };

        match network {
            "ws" => apply_ws_transport(clash_node, &mut transport),
            "grpc" => apply_grpc_transport(clash_node, &mut transport),
            "http" | "h2" => apply_http_transport(clash_node, &mut transport),
            _ => {}
        }

        if let Some(obj) = node.as_object_mut() {
            obj.insert("transport".to_string(), transport);
        }
    }
}

fn convert_clash_node_to_singbox(clash_node: &Value) -> Option<Value> {
    let node_type = clash_node.get("type").and_then(|t| t.as_str())?;
    let name = clash_node.get("name").and_then(|n| n.as_str())?;
    let server = clash_node.get("server").and_then(|s| s.as_str())?;
    let port = clash_node.get("port").and_then(|p| p.as_u64())?;
    let fingerprint = clash_node
        .get("client-fingerprint")
        .and_then(|f| f.as_str())
        .or_else(|| clash_node.get("fingerprint").and_then(|f| f.as_str()));

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

            if let Some(tls) = build_tls(clash_node, server, fingerprint) {
                if let Some(obj) = node.as_object_mut() {
                    obj.insert("tls".to_string(), tls);
                }
            }

            apply_transport(clash_node, &mut node);

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

            if let Some(flow) = clash_node.get("flow").and_then(|f| f.as_str()) {
                if let Some(obj) = node.as_object_mut() {
                    obj.insert("flow".to_string(), json!(flow));
                }
            }

            // reality-opts 兼容
            if let Some(reality) = clash_node.get("reality-opts") {
                let mut reality_obj = json!({ "enabled": true });
                if let Some(pk) = reality.get("public-key").and_then(|v| v.as_str()) {
                    reality_obj["public_key"] = json!(pk);
                }
                if let Some(sid) = reality.get("short-id").and_then(|v| v.as_str()) {
                    reality_obj["short_id"] = json!(sid);
                }
                if let Some(spider) = reality.get("spider-x").and_then(|v| v.as_str()) {
                    reality_obj["spider_x"] = json!(spider);
                }
                if let Some(fp) = reality.get("fingerprint").and_then(|v| v.as_str()) {
                    reality_obj["fingerprint"] = json!(fp);
                }
                if let Some(sni) = reality.get("server-name").and_then(|v| v.as_str()) {
                    reality_obj["server_name"] = json!(sni);
                }
                if let Some(obj) = node.as_object_mut() {
                    obj.insert("reality".to_string(), reality_obj);
                }
            }

            if let Some(tls) = build_tls(clash_node, server, fingerprint) {
                if let Some(obj) = node.as_object_mut() {
                    obj.insert("tls".to_string(), tls);
                }
            }

            apply_transport(clash_node, &mut node);

            Some(node)
        }
        "trojan" => {
            let password = clash_node.get("password").and_then(|p| p.as_str())?;
            let mut node = json!({
                "tag": name,
                "type": "trojan",
                "server": server,
                "server_port": port,
                "password": password,
            });

            if let Some(sni) = clash_node
                .get("sni")
                .or_else(|| clash_node.get("servername"))
                .and_then(|s| s.as_str())
            {
                node["tls"] = json!({
                    "enabled": clash_node.get("tls").and_then(|t| t.as_bool()).unwrap_or(true),
                    "server_name": sni
                });
            } else {
                if let Some(tls) = build_tls(clash_node, server, fingerprint) {
                    if let Some(obj) = node.as_object_mut() {
                        obj.insert("tls".to_string(), tls);
                    }
                }
            }

            apply_transport(clash_node, &mut node);

            Some(node)
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
            let pwd = clash_node
                .get("password")
                .or_else(|| clash_node.get("auth"))
                .and_then(|p| p.as_str())
                .unwrap_or_default();
            let mut node = json!({
                "tag": name,
                "type": "hysteria2",
                "server": server,
                "server_port": port,
                "password": pwd
            });

            let mut obfs_obj = json!({});
            if let Some(obfs) = clash_node.get("obfs").and_then(|o| o.as_str()) {
                obfs_obj["type"] = json!(obfs);
            }
            if let Some(obfs_pwd) = clash_node
                .get("obfs-password")
                .and_then(|p| p.as_str())
            {
                obfs_obj["password"] = json!(obfs_pwd);
            }
            if obfs_obj.as_object().map(|m| !m.is_empty()).unwrap_or(false) {
                node["obfs"] = obfs_obj;
            }

            if let Some(tls) = build_tls(clash_node, server, fingerprint) {
                if let Some(obj) = node.as_object_mut() {
                    obj.insert("tls".to_string(), tls);
                }
            }

            Some(node)
        }
        "tuic" => {
            let uuid = clash_node.get("uuid").and_then(|u| u.as_str())?;
            let password = clash_node
                .get("password")
                .or_else(|| clash_node.get("token"))
                .and_then(|u| u.as_str())
                .unwrap_or("");

            let mut node = json!({
                "tag": name,
                "type": "tuic",
                "server": server,
                "server_port": port,
                "uuid": uuid,
                "password": password
            });

            if let Some(cc) = clash_node
                .get("congestion-controller")
                .and_then(|c| c.as_str())
            {
                node["congestion_controller"] = json!(cc);
            }

            if let Some(mode) = clash_node
                .get("udp-relay-mode")
                .and_then(|c| c.as_str())
            {
                node["udp_relay_mode"] = json!(mode);
            }

            if let Some(alpn) = clash_node.get("alpn") {
                node["alpn"] = alpn.clone();
            }

            if let Some(reduce_rtt) = clash_node
                .get("reduce-rtt")
                .and_then(|c| c.as_bool())
            {
                node["reduce_rtt"] = json!(reduce_rtt);
            }

            if let Some(tls) = build_tls(clash_node, server, fingerprint) {
                if let Some(obj) = node.as_object_mut() {
                    obj.insert("tls".to_string(), tls);
                }
            }

            Some(node)
        }
        _ => None,
    }
}

// `selector` 分组的更新逻辑已迁移至 `app::singbox::config_generator`：
// - 订阅模块只负责“提取节点”，不再耦合模板替换与特定分组名字。
