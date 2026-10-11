//! `DatabaseService` 的回归测试。
//!
//! 覆盖两条历史上出过事故的路径：
//! 1. `save_app_config` 的全列手工 positional INSERT —— 曾因调整列顺序导致写错列；
//! 2. 增量列迁移 —— 早期实现对每条 `ALTER` 直接 `.ok()` 丢弃错误，
//!    迁移失败在日志里完全不可见，线上仅表现为“某项设置不生效”。

use super::{read_schema_version, DatabaseService, SCHEMA_VERSION};
use crate::app::storage::state_model::AppConfig;
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use std::path::PathBuf;

/// 每个测试用独立临时目录，避免相互干扰。
fn temp_db_path(name: &str) -> PathBuf {
    let mut dir = std::env::temp_dir();
    dir.push(format!("singbox-db-test-{}-{}", name, std::process::id()));
    std::fs::create_dir_all(&dir).expect("创建临时目录");
    dir.join("test.db")
}

async fn temp_service(name: &str) -> (PathBuf, DatabaseService) {
    let path = temp_db_path(name);
    // 确保从干净状态开始
    let _ = std::fs::remove_file(&path);
    let service = DatabaseService::new(&path.to_string_lossy())
        .await
        .expect("初始化测试数据库");
    (path, service)
}

/// 全新库必须建齐全部表。
#[tokio::test]
async fn creates_all_tables_on_fresh_database() {
    let (path, _service) = temp_service("fresh").await;

    let pool = SqlitePool::connect(&format!("sqlite:{}", path.to_string_lossy()))
        .await
        .expect("连接测试库");

    for table in [
        "app_config",
        "theme_config",
        "locale_config",
        "window_config",
        "update_config",
        "generic_config",
    ] {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name=?1",
        )
        .bind(table)
        .fetch_one(&pool)
        .await
        .expect("查询 sqlite_master");
        assert_eq!(row.0, 1, "表 {} 缺失", table);
    }

    let _ = std::fs::remove_file(&path);
}

/// `app_config` 必须包含所有 1.14 升级新增字段 + 1.13 基础字段。
#[tokio::test]
async fn app_config_contains_all_migrated_columns() {
    let (path, service) = temp_service("columns").await;

    let rows = sqlx::query("PRAGMA table_info(app_config)")
        .fetch_all(&service.pool)
        .await
        .expect("读取 app_config 列信息");
    let names: Vec<String> = rows
        .iter()
        .map(|row| row.try_get::<String, _>("name").expect("读取列名"))
        .collect();

    for expected in [
        // 1.14 新增
        "kernel_update_track",
        "singbox_dns_optimistic_cache",
        "singbox_dns_timeout",
        "singbox_dns_use_mdns",
        "singbox_enable_tls_spoof",
        "tun_dns_mode",
        "tun_include_macs",
        "tun_exclude_macs",
        "hysteria2_disable_chrome_parrot",
        "hysteria2_obfs_type",
        "clash_mode",
        "enable_web_dashboard",
        "enable_tailscale_endpoint",
        "tailscale_run_ssh_server",
        "tailscale_taildrop_directory",
        // 1.13 及更早的基础字段
        "proxy_mode",
        "auto_start_kernel",
        "prefer_ipv6",
        "proxy_port",
        "api_port",
        "tun_self_heal_enabled",
        "tun_self_heal_cooldown_secs",
        "active_config_path",
        "created_at",
        "updated_at",
    ] {
        assert!(names.iter().any(|n| n == expected), "缺列: {}", expected);
    }

    let _ = std::fs::remove_file(&path);
}

/// 迁移是幂等的：重复对同一个库执行 `DatabaseService::new` 不得报错，
/// 且不得因“列已存在”而失败（旧实现靠 `.ok()` 掩盖，新实现靠显式查列）。
#[tokio::test]
async fn schema_initialization_is_idempotent() {
    let (path, _first) = temp_service("idempotent").await;

    // 第二次初始化同一个文件
    let second = DatabaseService::new(&path.to_string_lossy())
        .await
        .expect("重复初始化必须成功");

    let version = read_schema_version(&second.pool)
        .await
        .expect("读取 schema 版本");
    assert_eq!(version, SCHEMA_VERSION);

    let _ = std::fs::remove_file(&path);
}

/// 关键回归：每个独立数据库路径都必须各自建表。
///
/// 旧实现用进程级 `static SCHEMA_INIT: OnceCell<()>`，导致第二个不同路径的库
/// 静默不建表。
#[tokio::test]
async fn separate_databases_each_get_schema() {
    let (path_a, service_a) = temp_service("multi-a").await;
    let (path_b, service_b) = temp_service("multi-b").await;

    assert_ne!(path_a, path_b);

    for service in [&service_a, &service_b] {
        let row: (i64,) = sqlx::query_as(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='app_config'",
        )
        .fetch_one(&service.pool)
        .await
        .expect("查询第二个库的表清单");
        assert_eq!(row.0, 1, "独立数据库必须各自完成 schema 初始化");
    }

    let _ = std::fs::remove_file(&path_a);
    let _ = std::fs::remove_file(&path_b);
}

/// `save_app_config` / `load_app_config` 全字段往返。
///
/// `save_app_config` 是全列的手工 positional INSERT，历史上曾因调整列顺序
/// 把值写进错误的列。这个往返测试能捕获绝大多数列错位 / 漏列问题。
#[tokio::test]
async fn app_config_round_trips_every_field() {
    let (path, service) = temp_service("roundtrip").await;

    // 每一项都刻意偏离默认值，确保“写错列”不会被默认值掩盖
    let config = AppConfig {
        auto_start_kernel: false,
        auto_start_app: true,
        auto_hide_to_tray_on_autostart: false,
        tray_close_behavior: "quit".to_string(),
        prefer_ipv6: true,
        allow_lan_access: true,
        proxy_port: 13080,
        api_port: 13081,
        proxy_mode: "global".to_string(),
        system_proxy_enabled: true,
        tun_enabled: true,
        tray_instance_id: Some("tray-1".to_string()),
        system_proxy_bypass: "192.168.*;10.*".to_string(),
        tun_auto_route: false,
        tun_strict_route: false,
        tun_mtu: 1420,
        tun_ipv4: "10.10.0.1/30".to_string(),
        tun_ipv6: "fd00::1/126".to_string(),
        tun_stack: "gvisor".to_string(),
        tun_enable_ipv6: true,
        tun_route_exclude_address: Some(vec!["10.0.0.0/8".to_string()]),
        active_config_path: Some("C:/tmp/config-a.json".to_string()),
        installed_kernel_version: Some("1.14.0".to_string()),
        singbox_dns_proxy: "https://dns.google/dns-query".to_string(),
        singbox_dns_cn: "h3://dns.alidns.com/dns-query".to_string(),
        singbox_dns_resolver: "8.8.8.8".to_string(),
        singbox_urltest_url: "http://www.gstatic.com/generate_204".to_string(),
        singbox_default_proxy_outbound: "direct".to_string(),
        singbox_block_ads: false,
        singbox_download_detour: "proxy".to_string(),
        singbox_dns_hijack: false,
        singbox_fake_dns_enabled: true,
        singbox_fake_dns_ipv4_range: "198.19.0.0/15".to_string(),
        singbox_fake_dns_ipv6_range: "fc00::/18".to_string(),
        singbox_fake_dns_filter_mode: "proxy_only".to_string(),
        singbox_enable_app_groups: false,
        tun_self_heal_enabled: false,
        tun_self_heal_cooldown_secs: 120,
        kernel_update_track: "beta".to_string(),
        singbox_dns_optimistic_cache: false,
        singbox_dns_timeout: "9s".to_string(),
        singbox_dns_use_mdns: false,
        singbox_enable_tls_spoof: true,
        tun_dns_mode: "dhcp".to_string(),
        tun_include_macs: vec!["aa:bb:cc:dd:ee:ff".to_string()],
        tun_exclude_macs: vec!["11:22:33:44:55:66".to_string()],
        hysteria2_disable_chrome_parrot: true,
        hysteria2_obfs_type: "salamander".to_string(),
        clash_mode: "global".to_string(),
        enable_web_dashboard: true,
        enable_tailscale_endpoint: true,
        tailscale_run_ssh_server: true,
        tailscale_taildrop_directory: "Drop".to_string(),
    };

    service
        .save_app_config(&config)
        .await
        .expect("保存 app_config 失败");

    let loaded = service
        .load_app_config()
        .await
        .expect("读取 app_config 失败")
        .expect("app_config 记录应存在");

    // 全字段比对：任一列写错位置 / 漏写都会在这里暴露
    // （差异输出会直接打印两个AppConfig 的完整Debug）
    assert_eq!(loaded, config);

    let _ = std::fs::remove_file(&path);
}

/// 空库读取 app_config 应返回 `None` 而非报错。
#[tokio::test]
async fn load_app_config_returns_none_when_empty() {
    let (path, service) = temp_service("empty-load").await;

    let loaded = service
        .load_app_config()
        .await
        .expect("空库读取不应报错");
    assert!(loaded.is_none(), "空库应返回 None");

    let _ = std::fs::remove_file(&path);
}

/// 空字符串数组列必须往返为空 Vec，而不是解析失败。
#[tokio::test]
async fn string_array_columns_round_trip() {
    let (path, service) = temp_service("string-array").await;

    let config = AppConfig {
        tun_include_macs: vec!["aa:bb:cc:dd:ee:ff".to_string()],
        tun_exclude_macs: Vec::new(),
        ..Default::default()
    };

    service.save_app_config(&config).await.expect("保存失败");
    let loaded = service
        .load_app_config()
        .await
        .expect("读取失败")
        .expect("记录应存在");

    assert_eq!(
        loaded.tun_include_macs,
        vec!["aa:bb:cc:dd:ee:ff".to_string()]
    );
    assert!(loaded.tun_exclude_macs.is_empty());

    let _ = std::fs::remove_file(&path);
}