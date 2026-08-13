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
fn parse_vless_reality_uri_preserves_reality_fields() {
    let content = "vless://26a1d547-b031-4139-9fc5-6671e1d0408a@example.com:443?type=tcp&encryption=none&security=reality&pbk=PUBLIC_KEY&sid=SHORT_ID&fp=firefox&sni=www.example.com&flow=xtls-rprx-vision#VLESS%20Reality";
    let nodes = extract_nodes_from_subscription(content).expect("should parse");

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["type"].as_str().unwrap(), "vless");
    assert_eq!(nodes[0]["flow"].as_str().unwrap(), "xtls-rprx-vision");
    assert_eq!(
        nodes[0]["tls"]["server_name"].as_str().unwrap(),
        "www.example.com"
    );
    assert_eq!(
        nodes[0]["tls"]["utls"]["fingerprint"].as_str().unwrap(),
        "firefox"
    );
    assert!(nodes[0]["tls"]["reality"]["enabled"].as_bool().unwrap());
    assert_eq!(
        nodes[0]["tls"]["reality"]["public_key"].as_str().unwrap(),
        "PUBLIC_KEY"
    );
    assert_eq!(
        nodes[0]["tls"]["reality"]["short_id"].as_str().unwrap(),
        "SHORT_ID"
    );
}

#[test]
fn parse_vless_reality_uri_defaults_fingerprint_to_chrome() {
    let content = "vless://26a1d547-b031-4139-9fc5-6671e1d0408a@example.com:443?type=tcp&encryption=none&security=reality&pbk=PUBLIC_KEY&sid=SHORT_ID&sni=www.example.com#VLESS%20Reality";
    let nodes = extract_nodes_from_subscription(content).expect("should parse");

    assert_eq!(nodes.len(), 1);
    assert_eq!(
        nodes[0]["tls"]["utls"]["fingerprint"].as_str().unwrap(),
        "chrome"
    );
    assert_eq!(
        nodes[0]["tls"]["reality"]["public_key"].as_str().unwrap(),
        "PUBLIC_KEY"
    );
    assert_eq!(
        nodes[0]["tls"]["reality"]["short_id"].as_str().unwrap(),
        "SHORT_ID"
    );
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

#[test]
fn parse_uri_list_hysteria2_with_obfs() {
    let content = concat!(
        "hysteria2://password123@example.com:443",
        "?obfs=salamander&obfs-password=obfs-secret",
        "&sni=example.com&alpn=h3,h2,http/1.1",
        "&insecure=0&upmbps=100&downmbps=200",
        "&fastopen=1&mport=443,8443#hy2-obfs"
    );
    let nodes = extract_nodes_from_subscription(content).expect("should parse");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["type"].as_str().unwrap(), "hysteria2");
    assert_eq!(nodes[0]["password"].as_str().unwrap(), "password123");
    assert_eq!(nodes[0]["server"].as_str().unwrap(), "example.com");
    assert_eq!(nodes[0]["server_port"].as_u64().unwrap(), 443);
    assert_eq!(nodes[0]["tls"]["server_name"].as_str().unwrap(), "example.com");
    assert_eq!(nodes[0]["tls"]["alpn"][0].as_str().unwrap(), "h3");
    assert_eq!(nodes[0]["obfs"]["type"].as_str().unwrap(), "salamander");
    assert_eq!(nodes[0]["obfs"]["password"].as_str().unwrap(), "obfs-secret");
    assert_eq!(nodes[0]["up_mbps"].as_u64().unwrap(), 100);
    assert_eq!(nodes[0]["down_mbps"].as_u64().unwrap(), 200);
    assert!(nodes[0]["tcp_fast_open"].as_bool().unwrap());
    assert_eq!(nodes[0]["server_ports"][0].as_str().unwrap(), "443");
    assert_eq!(nodes[0]["server_ports"][1].as_str().unwrap(), "8443");
}

#[test]
fn parse_uri_list_tuic_basic() {
    let content = "tuic://2DD61D93-75D8-4DA4-AC0E-6AECE7EAC365:hello@example.com:10443#TUIC";
    let nodes = extract_nodes_from_subscription(content).expect("should parse");

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["type"].as_str().unwrap(), "tuic");
    assert_eq!(nodes[0]["server"].as_str().unwrap(), "example.com");
    assert_eq!(nodes[0]["server_port"].as_u64().unwrap(), 10443);
    assert_eq!(
        nodes[0]["uuid"].as_str().unwrap(),
        "2DD61D93-75D8-4DA4-AC0E-6AECE7EAC365"
    );
    assert_eq!(nodes[0]["password"].as_str().unwrap(), "hello");
    assert!(nodes[0]["tls"]["enabled"].as_bool().unwrap());
}

#[test]
fn parse_uri_list_tuic_with_options() {
    let content = concat!(
        "tuic://2DD61D93-75D8-4DA4-AC0E-6AECE7EAC365:hello@example.com:10443",
        "?congestion_control=bbr",
        "&udp_relay_mode=native",
        "&udp_over_stream=1",
        "&zero_rtt_handshake=true",
        "&heartbeat=10s",
        "&network=tcp",
        "&alpn=h3,hq-29",
        "&sni=edge.example.com",
        "&insecure=1",
        "#TUIC%20Advanced"
    );
    let nodes = extract_nodes_from_subscription(content).expect("should parse");

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["tag"].as_str().unwrap(), "TUIC Advanced");
    assert_eq!(nodes[0]["congestion_control"].as_str().unwrap(), "bbr");
    assert_eq!(nodes[0]["udp_relay_mode"].as_str().unwrap(), "native");
    assert!(nodes[0]["udp_over_stream"].as_bool().unwrap());
    assert!(nodes[0]["zero_rtt_handshake"].as_bool().unwrap());
    assert_eq!(nodes[0]["heartbeat"].as_str().unwrap(), "10s");
    assert_eq!(nodes[0]["network"].as_str().unwrap(), "tcp");
    assert_eq!(nodes[0]["tls"]["server_name"].as_str().unwrap(), "edge.example.com");
    assert!(nodes[0]["tls"]["insecure"].as_bool().unwrap());
    assert_eq!(nodes[0]["tls"]["alpn"][0].as_str().unwrap(), "h3");
    assert_eq!(nodes[0]["tls"]["alpn"][1].as_str().unwrap(), "hq-29");
}

#[test]
fn parse_uri_list_anytls_basic() {
    let content = "anytls://secret@example.com:443?sni=cdn.example.com#AnyTLS";
    let nodes = extract_nodes_from_subscription(content).expect("should parse");

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["type"].as_str().unwrap(), "anytls");
    assert_eq!(nodes[0]["server"].as_str().unwrap(), "example.com");
    assert_eq!(nodes[0]["server_port"].as_u64().unwrap(), 443);
    assert_eq!(nodes[0]["password"].as_str().unwrap(), "secret");
    assert_eq!(nodes[0]["tls"]["server_name"].as_str().unwrap(), "cdn.example.com");
}

#[test]
fn parse_uri_list_anytls_with_idle_options() {
    let content = concat!(
        "anytls://secret@example.com:8443",
        "?sni=cdn.example.com",
        "&alpn=h2,http/1.1",
        "&insecure=true",
        "&idle_session_check_interval=45s",
        "&idle_session_timeout=90s",
        "&min_idle_session=5"
    );
    let nodes = extract_nodes_from_subscription(content).expect("should parse");

    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["tag"].as_str().unwrap(), "anytls-example.com:8443");
    assert!(nodes[0]["tls"]["insecure"].as_bool().unwrap());
    assert_eq!(nodes[0]["tls"]["alpn"][0].as_str().unwrap(), "h2");
    assert_eq!(nodes[0]["tls"]["alpn"][1].as_str().unwrap(), "http/1.1");
    assert_eq!(
        nodes[0]["idle_session_check_interval"].as_str().unwrap(),
        "45s"
    );
    assert_eq!(nodes[0]["idle_session_timeout"].as_str().unwrap(), "90s");
    assert_eq!(nodes[0]["min_idle_session"].as_u64().unwrap(), 5);
}

#[test]
fn extract_json_outbounds_should_include_tuic_and_anytls() {
    let content = r#"{
  "outbounds": [
    {
      "type": "tuic",
      "tag": "tuic-node",
      "server": "tuic.example.com",
      "server_port": 10443,
      "uuid": "2DD61D93-75D8-4DA4-AC0E-6AECE7EAC365",
      "password": "hello",
      "tls": {
        "enabled": true
      }
    },
    {
      "type": "anytls",
      "tag": "anytls-node",
      "server": "anytls.example.com",
      "server_port": 443,
      "password": "secret",
      "tls": {
        "enabled": true
      }
    }
  ]
}"#;
    let nodes = extract_nodes_from_subscription(content).expect("should parse");

    assert_eq!(nodes.len(), 2);
    assert_eq!(nodes[0]["type"].as_str().unwrap(), "tuic");
    assert_eq!(nodes[1]["type"].as_str().unwrap(), "anytls");
}

#[test]
fn mixed_uri_list_should_parse_old_and_new_protocols() {
    let content = r#"
vless://26a1d547-b031-4139-9fc5-6671e1d0408a@example.com:443?security=tls&sni=example.com#VLESS
tuic://2DD61D93-75D8-4DA4-AC0E-6AECE7EAC365:hello@tuic.example.com:10443#TUIC
anytls://secret@anytls.example.com:443?sni=cdn.example.com#AnyTLS
hysteria2://password@hy2.example.com:443?peer=example.com#Hysteria2
"#;
    let nodes = extract_nodes_from_subscription(content).expect("should parse");

    assert_eq!(nodes.len(), 4);
    assert_eq!(nodes[0]["type"].as_str().unwrap(), "vless");
    assert_eq!(nodes[1]["type"].as_str().unwrap(), "tuic");
    assert_eq!(nodes[2]["type"].as_str().unwrap(), "anytls");
    assert_eq!(nodes[3]["type"].as_str().unwrap(), "hysteria2");
}

// --- issue #61：Clash YAML 路径下 hysteria2 / tuic / anytls 转换 ---

#[test]
fn parse_clash_yaml_hysteria2() {
    let yaml = r#"
proxies:
  - name: "hy2-test"
    type: hysteria2
    server: 203.10.99.66
    port: 26892
    sni: v3-web-prime.douyinvod.com
    up: 50
    down: 50
    skip-cert-verify: true
    password: "secret-pass"
    obfs: salamander
    obfs-password: "obfs-secret-pass"
"#;
    let nodes = extract_nodes_from_subscription(yaml).expect("should parse");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["type"].as_str().unwrap(), "hysteria2");
    assert_eq!(nodes[0]["tag"].as_str().unwrap(), "hy2-test");
    assert_eq!(nodes[0]["server"].as_str().unwrap(), "203.10.99.66");
    assert_eq!(nodes[0]["server_port"].as_u64().unwrap(), 26892);
    assert_eq!(nodes[0]["password"].as_str().unwrap(), "secret-pass");
    assert_eq!(
        nodes[0]["tls"]["server_name"].as_str().unwrap(),
        "v3-web-prime.douyinvod.com"
    );
    assert!(nodes[0]["tls"]["insecure"].as_bool().unwrap());
    assert_eq!(nodes[0]["up_mbps"].as_u64().unwrap(), 50);
    assert_eq!(nodes[0]["down_mbps"].as_u64().unwrap(), 50);
    assert_eq!(nodes[0]["obfs"]["type"].as_str().unwrap(), "salamander");
    assert_eq!(nodes[0]["obfs"]["password"].as_str().unwrap(), "obfs-secret-pass");
}

#[test]
fn parse_clash_yaml_tuic() {
    let yaml = r#"
proxies:
  - name: "tuic-test"
    type: tuic
    server: tuic.example.com
    port: 22892
    uuid: 4c4e0c2e-645d-4b9d-a479-697e94629d71
    password: "secret-pass"
    alpn: [h3]
    udp-relay-mode: native
    congestion-controller: bbr
    disable-sni: false
    reduce-rtt: false
    skip-cert-verify: true
    sni: www.bing.com
"#;
    let nodes = extract_nodes_from_subscription(yaml).expect("should parse");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["type"].as_str().unwrap(), "tuic");
    assert_eq!(nodes[0]["tag"].as_str().unwrap(), "tuic-test");
    assert_eq!(nodes[0]["server_port"].as_u64().unwrap(), 22892);
    assert_eq!(
        nodes[0]["uuid"].as_str().unwrap(),
        "4c4e0c2e-645d-4b9d-a479-697e94629d71"
    );
    assert_eq!(nodes[0]["password"].as_str().unwrap(), "secret-pass");
    assert_eq!(nodes[0]["tls"]["alpn"][0].as_str().unwrap(), "h3");
    assert_eq!(
        nodes[0]["tls"]["server_name"].as_str().unwrap(),
        "www.bing.com"
    );
    assert!(nodes[0]["tls"]["insecure"].as_bool().unwrap());
    assert_eq!(nodes[0]["congestion_control"].as_str().unwrap(), "bbr");
    assert_eq!(nodes[0]["udp_relay_mode"].as_str().unwrap(), "native");
}

#[test]
fn parse_clash_yaml_port_as_string() {
    // serde_yaml 可能把 "22892" 解析为字符串，必须兼容，否则整条节点被丢弃。
    let yaml = r#"
proxies:
  - name: "str-port"
    type: hysteria2
    server: example.com
    port: "443"
    password: "p"
"#;
    let nodes = extract_nodes_from_subscription(yaml).expect("should parse");
    assert_eq!(nodes.len(), 1);
    assert_eq!(nodes[0]["type"].as_str().unwrap(), "hysteria2");
    assert_eq!(nodes[0]["server_port"].as_u64().unwrap(), 443);
}

#[test]
fn parse_clash_yaml_multiple_protocols() {
    // 综合：确认同一份订阅里 vmess/ss/hysteria2/tuic 都能被转换（不再静默丢弃）。
    let yaml = r#"
proxies:
  - {name: vm, type: vmess, server: a.com, port: 443, uuid: u1, cipher: auto}
  - {name: hy2, type: hysteria2, server: b.com, port: 443, password: p1}
  - {name: tu, type: tuic, server: c.com, port: 443, uuid: u2}
  - {name: ss, type: ss, server: d.com, port: 8388, cipher: aes-128-gcm, password: p2}
"#;
    let nodes = extract_nodes_from_subscription(yaml).expect("should parse");
    assert_eq!(nodes.len(), 4);
    let types: Vec<&str> = nodes
        .iter()
        .map(|n| n["type"].as_str().unwrap())
        .collect();
    assert!(types.contains(&"vmess"));
    assert!(types.contains(&"hysteria2"));
    assert!(types.contains(&"tuic"));
    assert!(types.contains(&"shadowsocks"));
}
