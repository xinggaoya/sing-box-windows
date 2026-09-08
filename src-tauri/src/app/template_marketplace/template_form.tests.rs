use super::*;
use crate::app::storage::custom_rule::{CustomRule, CustomRuleAction, CustomRuleMatchType};
use crate::app::storage::state_model::AppConfig;

fn rule(match_type: CustomRuleMatchType, action: CustomRuleAction, payload: &str) -> CustomRule {
    CustomRule {
        id: "r1".to_string(),
        enabled: true,
        match_type,
        payload: payload.to_string(),
        action,
        outbound: None,
        note: None,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    }
}

fn base_form() -> TemplateFormOptions {
    TemplateFormOptions {
        default_outbound: "manual".to_string(),
        urltest_url: Some("http://cp.cloudflare.com/generate_204".to_string()),
        block_ads: true,
        dns_hijack: true,
        fake_dns_enabled: false,
        dns_use_mdns: true,
        app_groups: vec![TAG_TELEGRAM.to_string(), TAG_GOOGLE.to_string()],
        dns_proxy: Some("https://dns.example/dns-query".to_string()),
        dns_cn: None,
        dns_resolver: None,
        custom_rules: vec![rule(
            CustomRuleMatchType::DomainSuffix,
            CustomRuleAction::Direct,
            "example.com",
        )],
    }
}

#[test]
fn form_generates_skeleton_with_selected_groups_only() {
    let config = generate_config_from_form(&AppConfig::default(), &base_form()).unwrap();
    let tags: Vec<&str> = config["outbounds"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|o| o.get("tag").and_then(|v| v.as_str()))
        .collect();
    assert!(tags.contains(&TAG_TELEGRAM));
    assert!(tags.contains(&TAG_GOOGLE));
    assert!(!tags.contains(&TAG_YOUTUBE));
    assert!(!tags.contains(&TAG_NETFLIX));
    assert!(!tags.contains(&TAG_OPENAI));

    let rule_sets: Vec<String> = config["route"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .filter_map(|r| {
            r.get("rule_set")
                .and_then(|v| v.as_str())
                .map(str::to_string)
        })
        .collect();
    assert!(rule_sets.iter().any(|rs| rs == RS_GEOSITE_TELEGRAM));
    assert!(rule_sets.iter().all(|rs| rs != RS_GEOSITE_YOUTUBE));
}

#[test]
fn custom_rule_injected_before_builtin_segment() {
    let config = generate_config_from_form(&AppConfig::default(), &base_form()).unwrap();
    let rules = config["route"]["rules"].as_array().unwrap();
    let custom_idx = rules
        .iter()
        .position(|r| r.get("domain_suffix").is_some())
        .expect("自定义规则应被注入");
    let cn_idx = rules
        .iter()
        .position(|r| {
            r.get("rule_set")
                .and_then(|v| v.as_str())
                .map(|rs| rs == RS_GEOSITE_PRIVATE)
                .unwrap_or(false)
        })
        .expect("内置私网直连段应存在");
    assert!(custom_idx < cn_idx, "自定义规则应优先于内置分流");
}

#[test]
fn form_parses_back_from_generated_skeleton() {
    let content = generate_config_from_form(&AppConfig::default(), &base_form()).unwrap();
    let content_str = serde_json::to_string(&content).unwrap();
    let parsed = parse_form_from_template(&content_str).unwrap();

    assert_eq!(parsed.default_outbound, "manual");
    assert!(parsed.block_ads);
    assert!(parsed.dns_hijack);
    assert!(parsed.dns_use_mdns);
    assert!(!parsed.fake_dns_enabled);
    assert_eq!(parsed.app_groups, base_form().app_groups);
    assert_eq!(
        parsed.dns_proxy.as_deref(),
        Some("https://dns.example/dns-query")
    );
    assert_eq!(
        parsed.urltest_url.as_deref(),
        Some("http://cp.cloudflare.com/generate_204")
    );
    assert_eq!(parsed.custom_rules.len(), 1);
    assert_eq!(
        parsed.custom_rules[0].match_type,
        CustomRuleMatchType::DomainSuffix
    );
    assert_eq!(parsed.custom_rules[0].action, CustomRuleAction::Direct);
    assert_eq!(parsed.custom_rules[0].payload, "example.com");
}

#[test]
fn parse_rejects_unknown_final_outbound() {
    let content =
        r#"{"route": {"final": "my-proxy"}, "outbounds": [{"type": "direct", "tag": "direct"}]}"#;
    assert!(parse_form_from_template(content).is_err());
}

#[test]
fn parse_rejects_custom_rule_with_group_outbound() {
    // 自定义规则指定了业务分流组出站（表单不支持），解析应失败以回退 JSON 模式
    let content = r#"{
        "route": {"final": "手动切换", "rules": [{"domain_suffix": ["a.com"], "outbound": "Telegram"}]},
        "outbounds": [
            {"type": "selector", "tag": "Telegram", "outbounds": []},
            {"type": "direct", "tag": "direct"}
        ]
    }"#;
    assert!(parse_form_from_template(content).is_err());
}

#[test]
fn empty_fields_fall_back_to_app_settings() {
    let mut form = base_form();
    form.dns_proxy = None;
    form.urltest_url = None;
    form.block_ads = false;
    let config = generate_config_from_form(&AppConfig::default(), &form).unwrap();
    // 未启用广告拦截时不应出现 ads reject 规则
    let has_ads = config["route"]["rules"]
        .as_array()
        .unwrap()
        .iter()
        .any(|r| r.get("rule_set").and_then(|v| v.as_str()) == Some(RS_GEOSITE_ADS));
    assert!(!has_ads);
    // urltest 跟随应用设置默认值
    let auto = config["outbounds"]
        .as_array()
        .unwrap()
        .iter()
        .find(|o| o.get("tag").and_then(|v| v.as_str()) == Some(TAG_AUTO))
        .unwrap();
    assert!(auto.get("url").is_some());
}

#[test]
fn fakeip_toggle_affects_dns_servers() {
    let mut form = base_form();
    form.fake_dns_enabled = true;
    let config = generate_config_from_form(&AppConfig::default(), &form).unwrap();
    let servers = config["dns"]["servers"].as_array().unwrap();
    assert!(servers
        .iter()
        .any(|s| s.get("tag").and_then(|v| v.as_str()) == Some(DNS_FAKEIP)));
}
