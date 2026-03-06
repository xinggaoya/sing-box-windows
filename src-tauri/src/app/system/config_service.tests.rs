use super::*;
use serde_json::json;

fn outside_config_path(file_name: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!(r"C:\external\{}", file_name)
    }

    #[cfg(not(target_os = "windows"))]
    {
        format!("/tmp/{}", file_name)
    }
}

fn has_private_ip_rule(rules: &[Value]) -> bool {
    rules.iter().any(|rule| {
        rule.get("ip_cidr")
            .and_then(|v| v.as_array())
            .is_some_and(|cidrs| {
                PRIVATE_IP_CIDRS
                    .iter()
                    .all(|cidr| cidrs.iter().any(|value| value.as_str() == Some(*cidr)))
            })
    })
}

#[test]
fn sanitize_file_name_should_replace_invalid_characters_and_fallback() {
    assert_eq!(sanitize_file_name("my config?.json", "config.json"), "my-config-.json");
    assert_eq!(sanitize_file_name(".", "config.json"), "config.json");
    assert_eq!(sanitize_file_name("..", "config.json"), "config.json");
}

#[test]
fn normalize_active_config_local_path_should_use_default_for_missing_path() {
    let (path, migrated) = normalize_active_config_local_path(None);

    assert_eq!(path, paths::get_config_dir().join("config.json"));
    assert!(!migrated);
}

#[test]
fn normalize_active_config_local_path_should_keep_local_absolute_path() {
    let local_path = paths::get_config_dir().join("configs").join("manual.json");
    let local_path_str = local_path.to_string_lossy().to_string();

    let (normalized, migrated) =
        normalize_active_config_local_path(Some(local_path_str.as_str()));

    assert_eq!(normalized, local_path);
    assert!(!migrated);
}

#[test]
fn normalize_active_config_local_path_should_rebase_external_absolute_path() {
    let (normalized, migrated) =
        normalize_active_config_local_path(Some(&outside_config_path("custom?.json")));

    assert_eq!(
        normalized,
        paths::get_config_dir().join("configs").join("custom-.json")
    );
    assert!(migrated);
}

#[test]
fn sanitize_geoip_private_rule_sets_should_rewrite_route_entries() {
    let mut config_obj = json!({
        "route": {
            "rule_set": [
                { "tag": RS_GEOIP_PRIVATE },
                { "tag": "keep-me" }
            ],
            "rules": [
                { "rule_set": RS_GEOIP_PRIVATE, "outbound": "proxy" },
                { "rule_set": [RS_GEOIP_PRIVATE, "keep-tag"], "outbound": "proxy" }
            ]
        }
    })
    .as_object()
    .cloned()
    .expect("config json should be an object");

    sanitize_geoip_private_rule_sets(&mut config_obj);

    let route = config_obj
        .get("route")
        .and_then(|value| value.as_object())
        .expect("route should remain an object");
    let rule_sets = route
        .get("rule_set")
        .and_then(|value| value.as_array())
        .expect("rule_set should remain an array");
    let rules = route
        .get("rules")
        .and_then(|value| value.as_array())
        .expect("rules should remain an array");

    assert_eq!(rule_sets.len(), 1);
    assert_eq!(rule_sets[0]["tag"].as_str(), Some("keep-me"));
    assert_eq!(rules[0]["rule_set"].as_str(), Some(RS_GEOSITE_PRIVATE));

    let second_rule_sets = rules[1]["rule_set"]
        .as_array()
        .expect("rule_set array should remain after filtering");
    assert_eq!(second_rule_sets.len(), 1);
    assert_eq!(second_rule_sets[0].as_str(), Some("keep-tag"));
    assert!(has_private_ip_rule(rules));
}

#[test]
fn ensure_private_ip_rule_should_not_duplicate_existing_rule() {
    let mut rules = vec![json!({
        "ip_cidr": PRIVATE_IP_CIDRS,
        "outbound": TAG_DIRECT
    })];

    ensure_private_ip_rule(&mut rules);

    assert_eq!(rules.len(), 1);
}

#[test]
fn ensure_private_ip_rule_should_append_rule_when_missing() {
    let mut rules = vec![json!({
        "domain_suffix": ["example.com"],
        "outbound": "proxy"
    })];

    ensure_private_ip_rule(&mut rules);

    assert_eq!(rules.len(), 2);
    assert!(has_private_ip_rule(&rules));
}
