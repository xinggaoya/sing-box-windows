use super::{
    extract_nodes_from_subscription, extract_subscription_userinfo,
    parse_subscription_userinfo, try_decode_base64_to_text,
};
use base64::{engine::general_purpose, Engine as _};
use reqwest::header::{HeaderMap, HeaderValue};

#[test]
fn base64_uri_list_should_extract_nodes_after_decode() {
    let uri_list = "trojan://password@example.com:443#test\nvless://uuid@example.com:443?security=tls&sni=example.com#vless\n";
    let b64 = general_purpose::STANDARD.encode(uri_list.as_bytes());

    let decoded = try_decode_base64_to_text(&b64).expect("decode should work");
    let nodes = extract_nodes_from_subscription(&decoded).expect("extract should work");
    assert_eq!(nodes.len(), 2);
}

#[test]
fn try_decode_base64_to_text_should_accept_whitespace_and_missing_padding() {
    let raw = "vmess://example\nvless://demo";
    let encoded = general_purpose::STANDARD
        .encode(raw.as_bytes())
        .trim_end_matches('=')
        .chars()
        .collect::<Vec<_>>();
    let formatted = format!("{} \n {}", encoded[..8].iter().collect::<String>(), encoded[8..].iter().collect::<String>());

    let decoded = try_decode_base64_to_text(&formatted).expect("decode should work");
    assert_eq!(decoded, raw);
}

#[test]
fn parse_subscription_userinfo_should_parse_known_fields() {
    let info = parse_subscription_userinfo("upload=1; download=2; total=3; expire=4; foo=bar")
        .expect("userinfo should be parsed");

    assert_eq!(info.upload, Some(1));
    assert_eq!(info.download, Some(2));
    assert_eq!(info.total, Some(3));
    assert_eq!(info.expire, Some(4));
}

#[test]
fn parse_subscription_userinfo_should_return_none_when_no_known_fields() {
    assert!(parse_subscription_userinfo("foo=1;bar=2").is_none());
    assert!(parse_subscription_userinfo("   ").is_none());
}

#[test]
fn extract_subscription_userinfo_should_support_case_variants() {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Subscription-Userinfo",
        HeaderValue::from_static("upload=10;download=20;total=30;expire=40"),
    );

    let info = extract_subscription_userinfo(&headers).expect("header should be parsed");
    assert_eq!(info.upload, Some(10));
    assert_eq!(info.download, Some(20));
    assert_eq!(info.total, Some(30));
    assert_eq!(info.expire, Some(40));
}
