use super::{
    active_config_change_requires_restart,
    extract_nodes_from_subscription, extract_subscription_userinfo,
    merge_subscription_fetch_result, parse_subscription_userinfo,
    should_retry_subscription_userinfo, try_decode_base64_to_text, SubscriptionFetchResult,
    SubscriptionUserInfo,
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
fn active_config_change_should_request_runtime_restart() {
    assert!(active_config_change_requires_restart(
        &Some("D:/configs/old.json".to_string()),
        &Some("D:/configs/new.json".to_string()),
    ));
    assert!(active_config_change_requires_restart(
        &Some("D:/configs/old.json".to_string()),
        &None,
    ));
}

#[test]
fn unchanged_active_config_should_not_request_runtime_restart() {
    assert!(!active_config_change_requires_restart(
        &Some("D:/configs/current.json".to_string()),
        &Some("D:/configs/current.json".to_string()),
    ));
}

#[test]
fn try_decode_base64_to_text_should_accept_whitespace_and_missing_padding() {
    let raw = "vmess://example\nvless://demo";
    let encoded = general_purpose::STANDARD
        .encode(raw.as_bytes())
        .trim_end_matches('=')
        .chars()
        .collect::<Vec<_>>();
    let formatted = format!(
        "{} \n {}",
        encoded[..8].iter().collect::<String>(),
        encoded[8..].iter().collect::<String>()
    );

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

#[test]
fn should_retry_subscription_userinfo_when_body_exists_but_header_missing() {
    let result = SubscriptionFetchResult {
        body: "vmess://demo".to_string(),
        userinfo: None,
    };

    assert!(should_retry_subscription_userinfo(&result));
}

#[test]
fn should_not_retry_subscription_userinfo_when_body_is_empty() {
    let result = SubscriptionFetchResult {
        body: "   ".to_string(),
        userinfo: None,
    };

    assert!(!should_retry_subscription_userinfo(&result));
}

#[test]
fn merge_subscription_fetch_result_should_preserve_primary_body_and_use_fallback_userinfo() {
    let primary = SubscriptionFetchResult {
        body: "primary-body".to_string(),
        userinfo: None,
    };
    let fallback_userinfo = Some(SubscriptionUserInfo {
        upload: Some(1),
        download: Some(2),
        total: Some(3),
        expire: Some(4),
    });

    let merged = merge_subscription_fetch_result(primary, fallback_userinfo);

    assert_eq!(merged.body, "primary-body");
    assert_eq!(
        merged.userinfo.as_ref().and_then(|info| info.upload),
        Some(1)
    );
    assert_eq!(
        merged.userinfo.as_ref().and_then(|info| info.download),
        Some(2)
    );
    assert_eq!(
        merged.userinfo.as_ref().and_then(|info| info.total),
        Some(3)
    );
    assert_eq!(
        merged.userinfo.as_ref().and_then(|info| info.expire),
        Some(4)
    );
}

#[test]
fn merge_subscription_fetch_result_should_not_override_existing_userinfo() {
    let primary = SubscriptionFetchResult {
        body: "primary-body".to_string(),
        userinfo: Some(SubscriptionUserInfo {
            upload: Some(10),
            download: Some(20),
            total: Some(30),
            expire: Some(40),
        }),
    };
    let fallback_userinfo = Some(SubscriptionUserInfo {
        upload: Some(1),
        download: Some(2),
        total: Some(3),
        expire: Some(4),
    });

    let merged = merge_subscription_fetch_result(primary, fallback_userinfo);

    assert_eq!(merged.body, "primary-body");
    assert_eq!(
        merged.userinfo.as_ref().and_then(|info| info.upload),
        Some(10)
    );
    assert_eq!(
        merged.userinfo.as_ref().and_then(|info| info.download),
        Some(20)
    );
    assert_eq!(
        merged.userinfo.as_ref().and_then(|info| info.total),
        Some(30)
    );
    assert_eq!(
        merged.userinfo.as_ref().and_then(|info| info.expire),
        Some(40)
    );
}

#[test]
fn merge_subscription_fetch_result_should_keep_none_when_fallback_missing() {
    let primary = SubscriptionFetchResult {
        body: "primary-body".to_string(),
        userinfo: None,
    };

    let merged = merge_subscription_fetch_result(primary, None);

    assert_eq!(merged.body, "primary-body");
    assert!(merged.userinfo.is_none());
}
