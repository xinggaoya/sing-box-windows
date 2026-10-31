//! `AppConfig` 历史遗留默认值迁移的回归测试。

use super::*;

fn default_config() -> AppConfig {
    AppConfig::default()
}

#[test]
fn legacy_dns_cn_default_should_be_migrated_to_doh_over_tcp() {
    let mut config = default_config();
    config.singbox_dns_cn = LEGACY_DNS_CN_DEFAULT.to_string();

    assert!(config.apply_legacy_migrations(), "应触发迁移");
    assert_eq!(config.singbox_dns_cn, MIGRATED_DNS_CN_DEFAULT);
    assert!(
        !config.singbox_dns_cn.starts_with("h3:"),
        "国内 DNS 不应继续使用 DoH3（QUIC/UDP 443 易被限速阻断）"
    );
}

#[test]
fn user_defined_dns_cn_should_not_be_overwritten() {
    let mut config = default_config();
    config.singbox_dns_cn = "https://doh.pub/dns-query".to_string();

    config.apply_legacy_migrations();

    assert_eq!(config.singbox_dns_cn, "https://doh.pub/dns-query");
}

#[test]
fn legacy_dns_timeout_should_be_migrated_to_official_default() {
    let mut config = default_config();
    config.singbox_dns_timeout = LEGACY_DNS_TIMEOUT_DEFAULT.to_string();

    assert!(config.apply_legacy_migrations());
    assert_eq!(config.singbox_dns_timeout, MIGRATED_DNS_TIMEOUT_DEFAULT);
}

#[test]
fn empty_optimistic_timeout_should_get_a_bounded_default() {
    let mut config = default_config();
    config.singbox_dns_optimistic_timeout = String::new();

    assert!(config.apply_legacy_migrations());
    assert_eq!(
        config.singbox_dns_optimistic_timeout, "30m",
        "乐观缓存必须带上显式的陈旧窗口上限，不能落到内核隐含的 3 天"
    );
}

#[test]
fn migration_should_be_idempotent() {
    let mut config = default_config();
    config.singbox_dns_cn = LEGACY_DNS_CN_DEFAULT.to_string();
    config.singbox_dns_timeout = LEGACY_DNS_TIMEOUT_DEFAULT.to_string();
    config.singbox_dns_optimistic_timeout = String::new();

    assert!(config.apply_legacy_migrations());
    assert!(
        !config.apply_legacy_migrations(),
        "第二次迁移不应再产生变更"
    );
}
