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

#[test]
fn parse_uri_list_hysteria2() {
    let content =
        "hysteria2://password@example.com:443?peer=example.com&insecure=1&alpn=h3#Hysteria2";
    let nodes = extract_nodes_from_subscription(content).expect("should parse");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["type"].as_str().unwrap(), "hysteria2");
    assert_eq!(nodes[0]["tag"].as_str().unwrap(), "Hysteria2");
}
