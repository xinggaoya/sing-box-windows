use super::build_dns_server_config;

#[test]
fn should_convert_https_legacy_address_to_new_dns_server() {
    let server = build_dns_server_config(
        "dns_proxy",
        "https://1.1.1.1/dns-query",
        Some("ipv4_only"),
        Some("manual"),
        Some("dns_resolver"),
    )
    .expect("https 地址应可转换");

    assert_eq!(server.server_type.as_deref(), Some("https"));
    assert_eq!(server.server.as_deref(), Some("1.1.1.1"));
    assert_eq!(server.server_port, Some(443));
    assert_eq!(server.path.as_deref(), Some("/dns-query"));
}

#[test]
fn should_set_domain_resolver_for_domain_server() {
    let server = build_dns_server_config(
        "dns_proxy",
        "https://dns.google/dns-query",
        Some("prefer_ipv4"),
        Some("manual"),
        Some("dns_resolver"),
    )
    .expect("域名 DNS 地址应可转换");

    assert_eq!(
        server
            .domain_resolver
            .as_ref()
            .and_then(|v| v.get("server"))
            .and_then(|v| v.as_str()),
        Some("dns_resolver")
    );
    assert_eq!(
        server
            .domain_resolver
            .as_ref()
            .and_then(|v| v.get("strategy"))
            .and_then(|v| v.as_str()),
        Some("prefer_ipv4")
    );
}

#[test]
fn should_omit_direct_detour_for_dns_server() {
    let server = build_dns_server_config(
        "dns_resolver",
        "223.5.5.5",
        Some("prefer_ipv4"),
        Some("direct"),
        None,
    )
    .expect("dns_resolver 构建应成功");

    assert!(server.detour.is_none());
}
