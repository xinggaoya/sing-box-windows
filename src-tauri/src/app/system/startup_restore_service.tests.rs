use super::*;
use crate::app::storage::state_model::{AppConfig, Subscription};

fn make_subscription(name: &str, config_path: Option<&str>) -> Subscription {
    Subscription {
        name: name.to_string(),
        url: String::new(),
        is_loading: false,
        last_update: None,
        is_manual: false,
        manual_content: None,
        use_original_config: false,
        config_from_template: false,
        config_path: config_path.map(|value| value.to_string()),
        backup_path: None,
        auto_update_interval_minutes: None,
        subscription_upload: None,
        subscription_download: None,
        subscription_total: None,
        subscription_expire: None,
        auto_update_fail_count: None,
        last_auto_update_attempt: None,
        last_auto_update_error: None,
        last_auto_update_error_type: None,
        last_auto_update_backoff_until: None,
    }
}

#[test]
fn resolve_startup_active_config_path_prefers_existing_app_config_path() {
    let app_config = AppConfig {
        active_config_path: Some("D:/configs/current.json".to_string()),
        ..AppConfig::default()
    };
    let subscriptions = vec![make_subscription("alpha", Some("D:/configs/alpha.json"))];

    let resolved = resolve_startup_active_config_path(&app_config, &subscriptions, Some(0));

    assert_eq!(resolved.as_deref(), Some("D:/configs/current.json"));
}

#[test]
fn resolve_startup_active_config_path_falls_back_to_saved_active_subscription_index() {
    let app_config = AppConfig {
        active_config_path: None,
        ..AppConfig::default()
    };
    let subscriptions = vec![
        make_subscription("alpha", Some("D:/configs/alpha.json")),
        make_subscription("beta", Some("D:/configs/beta.json")),
    ];

    let resolved = resolve_startup_active_config_path(&app_config, &subscriptions, Some(1));

    assert_eq!(resolved.as_deref(), Some("D:/configs/beta.json"));
}

#[test]
fn resolve_startup_active_config_path_ignores_invalid_saved_index() {
    let app_config = AppConfig {
        active_config_path: None,
        ..AppConfig::default()
    };
    let subscriptions = vec![
        make_subscription("alpha", Some("D:/configs/alpha.json")),
        make_subscription("beta", Some("D:/configs/beta.json")),
    ];

    let resolved = resolve_startup_active_config_path(&app_config, &subscriptions, Some(4));

    assert_eq!(resolved, None);
}

#[test]
fn resolve_startup_active_config_path_falls_back_to_only_subscription_when_index_is_unusable() {
    let app_config = AppConfig {
        active_config_path: None,
        ..AppConfig::default()
    };
    let subscriptions = vec![make_subscription("alpha", Some("D:/configs/alpha.json"))];

    let resolved = resolve_startup_active_config_path(&app_config, &subscriptions, Some(4));

    assert_eq!(resolved.as_deref(), Some("D:/configs/alpha.json"));
}
