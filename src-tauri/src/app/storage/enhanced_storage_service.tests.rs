//! `EnhancedStorageService` 的单元测试。
//!
//! 重点覆盖 `AppConfig` 内存快照：`db_get_app_config` 全仓有 58 处调用点且守护循环
//! 每 8 秒读 2 次，因此读取必须走快照；而快照一旦与数据库不同步，就会出现
//! 「界面改了但重启后回退」或「写入了但读不到」这类难排查的问题。

use super::EnhancedStorageService;
use crate::app::storage::database::DatabaseService;
use crate::app::storage::state_model::AppConfig;
use std::path::PathBuf;

fn temp_db_path(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!(
        "singbox-storage-test-{}-{}",
        name,
        std::process::id()
    ));
    std::fs::create_dir_all(&dir).expect("创建临时目录");
    let path = dir.join("test.db");
    let _ = std::fs::remove_file(&path);
    path
}

async fn build_service(name: &str) -> (PathBuf, EnhancedStorageService) {
    let path = temp_db_path(name);
    let database = DatabaseService::new(&path.to_string_lossy())
        .await
        .expect("初始化数据库");
    // 直接构造：避免依赖 Tauri AppHandle
    let service = EnhancedStorageService::from_database(std::sync::Arc::new(database)).await;
    (path, service)
}

#[tokio::test]
async fn empty_database_returns_default_config() {
    let (path, service) = build_service("default").await;

    let config = service.get_app_config().await.expect("读取配置");
    assert_eq!(config, AppConfig::default());

    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn saved_config_is_readable_via_snapshot() {
    let (path, service) = build_service("snapshot").await;

    let expected = AppConfig {
        proxy_port: 23456,
        api_port: 23457,
        prefer_ipv6: true,
        clash_mode: "global".to_string(),
        ..AppConfig::default()
    };

    service.save_app_config(&expected).await.expect("保存配置");

    // 关键断言：不经过数据库往返，直接从快照读到刚写入的值
    let loaded = service.get_app_config().await.expect("读取配置");
    assert_eq!(loaded, expected);

    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn snapshot_reflects_latest_write_not_first() {
    let (path, service) = build_service("latest-wins").await;

    service
        .save_app_config(&AppConfig {
            proxy_port: 10000,
            ..AppConfig::default()
        })
        .await
        .expect("第一次保存");

    service
        .save_app_config(&AppConfig {
            proxy_port: 20000,
            ..AppConfig::default()
        })
        .await
        .expect("第二次保存");

    // 回归：若快照只在初始化时填充，这里会读到第一次写入的旧值
    let loaded = service.get_app_config().await.expect("读取配置");
    assert_eq!(loaded.proxy_port, 20000, "快照必须是最后一次写入的值");

    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn setting_snapshot_without_persisting() {
    let (path, service) = build_service("set-snapshot").await;

    let original = AppConfig {
        tun_enabled: true,
        ..AppConfig::default()
    };
    service.save_app_config(&original).await.expect("保存配置");

    // 模拟 `db_get_app_config_internal` 的平台修正：只改快照、不落库
    let mut corrected = original.clone();
    corrected.tun_enabled = false;
    service.set_app_config_snapshot(corrected.clone());

    let loaded = service.get_app_config().await.expect("读取配置");
    assert!(!loaded.tun_enabled, "应返回修正后的快照值");
    assert_eq!(loaded, corrected);

    let _ = std::fs::remove_file(&path);
}

#[tokio::test]
async fn generic_config_round_trip() {
    let (path, service) = build_service("generic").await;

    #[derive(Debug, Clone, PartialEq, serde::Serialize, serde::Deserialize)]
    struct Payload {
        value: String,
        count: u32,
    }

    let payload = Payload {
        value: "hello".to_string(),
        count: 42,
    };

    service
        .save_generic_config("unit-test-key", &payload)
        .await
        .expect("保存通用配置");

    let loaded: Option<Payload> = service
        .load_generic_config("unit-test-key")
        .await
        .expect("读取通用配置");

    assert_eq!(loaded, Some(payload));

    let _ = std::fs::remove_file(&path);
}

use super::{
    normalize_app_config_for_persistence, read_config_value, resolve_patch_mode_for_subscription,
    resolve_patch_mode_with_hint, sync_settings_to_config_file, ConfigPatchMode,
};
use crate::app::singbox::ownership::Ownership;
use crate::app::storage::state_model::Subscription;

fn build_subscription(use_original_config: bool) -> Subscription {
    Subscription {
        name: "test".to_string(),
        url: "https://example.com/sub".to_string(),
        is_loading: false,
        last_update: None,
        is_manual: false,
        manual_content: None,
        use_original_config,
        config_path: Some("/tmp/sub.json".to_string()),
        backup_path: None,
        auto_update_interval_minutes: Some(60),
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
fn should_use_ports_only_for_original_subscription_without_hint() {
    let subscription = build_subscription(true);
    assert_eq!(
        resolve_patch_mode_for_subscription(Some(&subscription)),
        ConfigPatchMode::PortsOnly
    );
}

#[test]
fn should_trust_explicit_original_hint_when_subscription_missing() {
    assert_eq!(
        resolve_patch_mode_with_hint(None, Some(true)),
        ConfigPatchMode::PortsOnly
    );
}

#[test]
fn should_trust_explicit_non_original_hint_over_subscription_state() {
    let subscription = build_subscription(true);
    assert_eq!(
        resolve_patch_mode_with_hint(Some(&subscription), Some(false)),
        ConfigPatchMode::Full
    );
}

#[test]
fn should_normalize_blank_tun_route_exclude_address_to_none() {
    let normalized =
        normalize_app_config_for_persistence(crate::app::storage::state_model::AppConfig {
            tun_route_exclude_address: Some(vec!["  ".to_string(), "".to_string()]),
            ..crate::app::storage::state_model::AppConfig::default()
        })
        .expect("blank route exclude address should normalize");

    assert_eq!(normalized.tun_route_exclude_address, None);
}

#[test]
fn should_reject_invalid_tun_route_exclude_address_on_save() {
    let error = normalize_app_config_for_persistence(crate::app::storage::state_model::AppConfig {
        tun_route_exclude_address: Some(vec!["invalid".to_string()]),
        ..crate::app::storage::state_model::AppConfig::default()
    })
    .expect_err("invalid route exclude address should be rejected");

    assert!(
        error.contains("invalid"),
        "error should mention invalid CIDR, got: {}",
        error
    );
}

#[test]
fn sync_settings_to_config_file_should_migrate_stale_dns_bootstrap_strategy() {
    // 模拟旧版本落盘的激活配置：prefer_ipv6 泄漏进 DNS bootstrap 解析策略
    let stale_config = r#"{
  "dns": {
"final": "dns_proxy",
"servers": [
  {
    "tag": "dns_cn",
    "type": "h3",
    "server": "dns.alidns.com",
    "server_port": 443,
    "domain_resolver": {
      "server": "dns_resolver",
      "strategy": "prefer_ipv6"
    }
  }
],
"rules": []
  },
  "inbounds": [],
  "outbounds": [],
  "route": {
"rules": [],
"final": "自动选择",
"default_domain_resolver": {
  "server": "dns_resolver",
  "strategy": "prefer_ipv6"
}
  }
}"#;

    let dir = std::env::temp_dir().join(format!(
        "sbw-sync-test-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).expect("应能创建临时目录");
    let config_path = dir.join("config.json");
    std::fs::write(&config_path, stale_config).expect("应能写入临时配置");

    let app_config = crate::app::storage::state_model::AppConfig {
        prefer_ipv6: true,
        ..crate::app::storage::state_model::AppConfig::default()
    };

    // 这份配置带本程序的 DNS server tag → 归属为本程序，Full patch 才允许改写
    let ownership = Ownership::inferred(&read_config_value(&config_path));
    assert!(
        ownership.managed,
        "带 dns_cn 的存量配置应被识别为本程序的配置"
    );

    let result =
        sync_settings_to_config_file(&config_path, &app_config, ConfigPatchMode::Full, &ownership);
    assert!(result.is_ok(), "Full patch 应成功: {:?}", result);

    let synced = std::fs::read_to_string(&config_path).expect("应能读回配置");
    let value: serde_json::Value = serde_json::from_str(&synced).expect("patch 后应为合法 JSON");

    // DNS bootstrap 策略应被迁移为 IPv4 优先，而全局偏好仍保留在 route 层
    let cn_resolver = value
        .pointer("/dns/servers/0/domain_resolver")
        .expect("dns_cn 应被重建并带 domain_resolver");
    assert_eq!(
        cn_resolver.get("strategy").and_then(|v| v.as_str()),
        Some("prefer_ipv4"),
        "存量配置的 DNS bootstrap 策略应迁移为 prefer_ipv4: {}",
        synced
    );
    assert_eq!(
        value
            .pointer("/route/default_domain_resolver/strategy")
            .and_then(|v| v.as_str()),
        Some("prefer_ipv6")
    );

    // 幂等：再次同步内容无变化时不应改写文件
    let before = std::fs::read_to_string(&config_path).expect("应能读取当前配置");
    let second =
        sync_settings_to_config_file(&config_path, &app_config, ConfigPatchMode::Full, &ownership);
    assert!(second.is_ok(), "重复 patch 应成功: {:?}", second);
    let after = std::fs::read_to_string(&config_path).expect("应能读取当前配置");
    assert_eq!(before, after, "重复同步不应改变配置内容");

    let _ = std::fs::remove_dir_all(&dir);
}
