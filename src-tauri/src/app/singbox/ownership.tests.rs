//! 配置归属（Ownership）判定测试
//!
//! 这些用例守护的是"**不误伤用户导入的原始配置**"这条底线：
//! 一旦归属判定放宽，补丁路径就会去重建用户的入口、改写用户的路由终点，
//! 表现为"能连但走错线路"，而且不报错。

use super::*;
use serde_json::json;

/// 一份典型的机场原始配置：有自己的入口、DNS server、出站和 rule-set，
/// 但完全不是本程序生成的（没有 `mixed-in`，也没有本程序的 DNS server tag）。
fn foreign_config() -> Value {
    json!({
        "inbounds": [
            { "type": "socks", "tag": "socks-in", "listen": "127.0.0.1", "listen_port": 1080 }
        ],
        "dns": {
            "servers": [
                { "tag": "remote-dns", "type": "https", "server": "1.1.1.1" }
            ],
            "rules": []
        },
        "outbounds": [
            { "type": "selector", "tag": "proxy", "outbounds": ["node-a"] },
            { "type": "vless", "tag": "node-a", "server": "1.2.3.4" }
        ],
        "route": {
            "rule_set": [
                { "type": "remote", "tag": "geosite-cn", "format": "binary", "url": "https://example.com/cn.srs" }
            ],
            "rules": [],
            "final": "proxy"
        }
    })
}

/// 一份本程序生成的配置（骨架裁剪版）
fn owned_config() -> Value {
    json!({
        "inbounds": [
            { "type": "mixed", "tag": "mixed-in", "listen": "127.0.0.1", "listen_port": 7890 }
        ],
        "dns": {
            "servers": [
                { "tag": "dns_proxy", "type": "https", "server": "1.1.1.1" },
                { "tag": "dns_cn", "type": "h3", "server": "dns.alidns.com" }
            ],
            "rules": []
        },
        "outbounds": [
            { "type": "urltest", "tag": TAG_AUTO, "outbounds": [] },
            { "type": "selector", "tag": TAG_MANUAL, "outbounds": [TAG_AUTO] }
        ],
        "route": {
            "rule_set": [
                { "type": "remote", "tag": "geosite-cn", "format": "binary", "url": "https://example.com/cn.srs" }
            ],
            "rules": []
        }
    })
}

#[test]
fn generated_config_should_be_recognized_as_owned() {
    let ownership = Ownership::inferred(&owned_config());
    assert!(ownership.managed, "本程序生成的配置应被识别为已接管");
    assert!(ownership.owns_inbound("mixed-in"));
    assert!(ownership.owns_dns_server("dns_proxy"));
    assert!(ownership.owns_dns_server("dns_cn"));
    assert!(ownership.owns_group(TAG_AUTO));
    assert!(ownership.owns_group(TAG_MANUAL));
    assert!(ownership.owns_rule_set("geosite-cn"));
}

#[test]
fn foreign_config_should_not_be_claimed() {
    let ownership = Ownership::inferred(&foreign_config());
    assert!(!ownership.managed, "外来配置不应被接管");
    assert!(ownership.inbounds.is_empty());
    assert!(ownership.dns_servers.is_empty());
    assert!(ownership.groups.is_empty());
    assert!(ownership.rule_sets.is_empty());
}

#[test]
fn foreign_config_with_same_named_group_should_not_be_claimed() {
    // 同名分组（自动选择/手动切换）在用户自己的配置里也可能存在，
    // 不能拿它们当"接管标记"，否则会误改用户的分组
    let mut config = foreign_config();
    config["outbounds"] = json!([
        { "type": "urltest", "tag": TAG_AUTO, "outbounds": [] },
        { "type": "selector", "tag": TAG_MANUAL, "outbounds": [TAG_AUTO] }
    ]);

    let ownership = Ownership::inferred(&config);
    assert!(!ownership.managed, "只靠同名分组不足以判定为本程序的配置");
}

#[test]
fn empty_shell_config_should_be_treated_as_owned() {
    // 本程序刚生成、还没注入节点的中间态：没有可管理的结构，交给补丁路径补齐是安全的
    let config = json!({
        "inbounds": [],
        "dns": { "servers": [], "rules": [] },
        "route": { "rule_set": [], "rules": [] }
    });
    assert!(looks_managed(&config));
    assert!(Ownership::inferred(&config).managed);
}

#[test]
fn empty_shell_ownership_should_claim_nothing() {
    // 空壳归属（一个 tag 都没认领）没有信息量，落库后会变成"用空清单过滤"，
    // 让 DNS 地址 / 测速 URL 等设置静默不生效——所以这种记录不落库。
    let config = json!({
        "inbounds": [],
        "dns": { "servers": [], "rules": [] },
        "route": { "rule_set": [], "rules": [] }
    });
    let ownership = Ownership::inferred(&config);
    assert!(ownership.managed);
    assert!(ownership.claims_nothing(), "空壳配置不应认领任何 tag");
}

#[test]
fn real_ownership_should_claim_something() {
    let ownership = Ownership::inferred(&owned_config());
    assert!(!ownership.claims_nothing());
}

#[test]
fn storage_key_should_be_scoped_by_config_path() {
    assert_eq!(
        storage_key("C:/app/sing-box/config.json"),
        "managed_config_tags:C:/app/sing-box/config.json"
    );
    assert_ne!(
        storage_key("a.json"),
        storage_key("b.json"),
        "不同配置文件的归属必须隔离"
    );
}
