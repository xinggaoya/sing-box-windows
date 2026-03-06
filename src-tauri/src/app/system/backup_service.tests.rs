use super::*;
use crate::app::storage::state_model::Subscription;

fn build_subscription(path: &str) -> Subscription {
    Subscription {
        name: "test-sub".to_string(),
        url: "https://example.com/sub".to_string(),
        is_loading: false,
        last_update: None,
        is_manual: false,
        manual_content: None,
        use_original_config: false,
        config_path: Some(path.to_string()),
        backup_path: None,
        auto_update_interval_minutes: Some(720),
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

fn legacy_absolute_path(file_name: &str) -> String {
    #[cfg(target_os = "windows")]
    {
        format!(
            "C:\\Users\\legacy-user\\AppData\\Local\\sing-box-windows\\sing-box\\configs\\{}",
            file_name
        )
    }

    #[cfg(not(target_os = "windows"))]
    {
        format!(
            "/tmp/legacy-user/sing-box-windows/sing-box/configs/{}",
            file_name
        )
    }
}

#[test]
fn encode_absolute_path_for_snapshot_should_return_relative_path() {
    let local_abs = paths::get_config_dir().join("configs").join("sample.json");
    let encoded = encode_path_for_snapshot(
        &local_abs.to_string_lossy(),
        SnapshotPathKind::SubscriptionConfig,
    );
    assert_eq!(encoded, "configs/sample.json");
}

#[test]
fn rewrite_paths_for_snapshot_should_migrate_legacy_absolute_paths() {
    let snapshot = BackupSnapshot {
        format_version: 1,
        app_config: crate::app::storage::state_model::AppConfig {
            active_config_path: Some(legacy_absolute_path("active.json")),
            ..Default::default()
        },
        subscriptions: vec![build_subscription(&legacy_absolute_path("sub.json"))],
        ..Default::default()
    };

    let (app_config, subscriptions, stats) = rewrite_paths_for_snapshot(&snapshot);
    let active_path = PathBuf::from(app_config.active_config_path.unwrap_or_default());
    let sub_path = PathBuf::from(subscriptions[0].config_path.clone().unwrap_or_default());

    assert!(active_path.starts_with(paths::get_config_dir()));
    assert!(sub_path.starts_with(paths::get_config_dir()));
    assert!(stats.absolute_rewrites >= 2);
    assert!(stats.active_path_rewritten);
}
