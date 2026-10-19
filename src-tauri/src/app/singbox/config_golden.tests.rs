//! 配置生成黄金样本测试
//!
//! ## 为什么需要它
//!
//! 配置生成逻辑此前分散在两处：`config_generator.rs`（从零生成）与
//! `settings_patch.rs`（在已有配置上打补丁）。两处对同一批语义各写一遍，
//! 重构时极易出现"改了生成侧、忘了补丁侧"的不一致——而这种不一致**不会报错**，
//! 只会让用户拿到一份与设置项不匹配的配置。
//!
//! 本测试把「生成 + 补丁」的最终产物固化为 JSON 快照：
//! 后续任何重构都必须让快照**逐字段不变**，否则测试失败并打印差异。
//!
//! ## 重新生成快照
//!
//! ```powershell
//! $env:SINGBOX_BLESS_GOLDEN = '1'; cd src-tauri; cargo test config_golden
//! ```
//!
//! 快照里对环境相关字段做了归一化（见 [`normalize_for_snapshot`]），
//! 因此在不同机器 / 不同工作目录下结果一致。

use super::*;
use crate::app::singbox::ownership::Ownership;
use crate::app::singbox::settings_patch::apply_intent_to_config;
use crate::app::storage::state_model::AppConfig;
use serde_json::Value;
use std::path::PathBuf;

/// 快照目录：与本文件同级的 `testdata/config_golden/`
fn snapshot_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("src")
        .join("app")
        .join("singbox")
        .join("testdata")
        .join("config_golden")
}

/// 环境相关字段归一化：保证快照跨机器可复现
fn normalize_for_snapshot(mut config: Value) -> Value {
    // 日志输出路径依赖运行机器的工作目录
    if let Some(log) = config.get_mut("log").and_then(|v| v.as_object_mut()) {
        if log.contains_key("output") {
            log.insert("output".to_string(), json!("<LOG_PATH>"));
        }
    }
    config
}

/// 四组代表性配置：覆盖默认、TUN、FakeDNS、局域网监听四条主要分支
fn golden_variants() -> Vec<(&'static str, AppConfig)> {
    let tun = AppConfig {
        tun_enabled: true,
        tun_stack: "gvisor".to_string(),
        tun_mtu: 9000,
        tun_dns_mode: "disabled".to_string(),
        ..AppConfig::default()
    };

    let fake_dns = AppConfig {
        singbox_fake_dns_enabled: true,
        singbox_dns_optimistic_cache: true,
        singbox_dns_timeout: "3s".to_string(),
        singbox_dns_use_mdns: true,
        singbox_block_ads: true,
        ..AppConfig::default()
    };

    let lan = AppConfig {
        allow_lan_access: true,
        proxy_port: 10808,
        api_port: 9090,
        singbox_enable_app_groups: false,
        ..AppConfig::default()
    };

    vec![
        ("default", AppConfig::default()),
        ("tun", tun),
        ("fake_dns", fake_dns),
        ("lan_no_app_groups", lan),
    ]
}

fn diff_json(expected: &Value, actual: &Value, path: &str, out: &mut Vec<String>) {
    match (expected, actual) {
        (Value::Object(a), Value::Object(b)) => {
            for (key, value) in a {
                match b.get(key) {
                    Some(other) => diff_json(value, other, &format!("{path}.{key}"), out),
                    None => out.push(format!("{path}.{key}: 期望存在，实际缺失")),
                }
            }
            for key in b.keys() {
                if !a.contains_key(key) {
                    out.push(format!("{path}.{key}: 实际多出字段"));
                }
            }
        }
        (Value::Array(a), Value::Array(b)) => {
            if a.len() != b.len() {
                out.push(format!(
                    "{path}: 数组长度不同，期望 {} 实际 {}",
                    a.len(),
                    b.len()
                ));
                return;
            }
            for (idx, (x, y)) in a.iter().zip(b.iter()).enumerate() {
                diff_json(x, y, &format!("{path}[{idx}]"), out);
            }
        }
        (x, y) => {
            if x != y {
                out.push(format!("{path}: 期望 {x}，实际 {y}"));
            }
        }
    }
}

#[test]
fn generated_config_matches_golden_snapshots() {
    let bless = std::env::var("SINGBOX_BLESS_GOLDEN").is_ok();
    let dir = snapshot_dir();

    for (name, app_config) in golden_variants() {
        let generated = normalize_for_snapshot(generate_base_config(&app_config));
        let snapshot_path = dir.join(format!("{name}.json"));

        if bless {
            std::fs::create_dir_all(&dir).expect("创建快照目录失败");
            let pretty = serde_json::to_string_pretty(&generated).expect("序列化快照失败");
            std::fs::write(&snapshot_path, pretty).expect("写入快照失败");
            continue;
        }

        let expected_raw = std::fs::read_to_string(&snapshot_path).unwrap_or_else(|err| {
            panic!(
                "读取黄金快照失败: {err}\n路径: {}\n如需重新生成请设置 SINGBOX_BLESS_GOLDEN=1",
                snapshot_path.display()
            )
        });
        let expected: Value =
            serde_json::from_str(&expected_raw).expect("解析黄金快照失败（快照文件已损坏？）");

        let mut diffs = Vec::new();
        diff_json(&expected, &generated, "", &mut diffs);
        assert!(
            diffs.is_empty(),
            "配置生成结果与黄金快照不一致（{}）：\n{}\n如确认变更正确，请设置 SINGBOX_BLESS_GOLDEN=1 重新生成快照",
            name,
            diffs.join("\n")
        );
    }
}

#[test]
fn generated_config_is_stable_across_calls() {
    // 同一份 AppConfig 连续生成两次必须完全一致（防止生成过程里混入随机/时间因素）
    for (_, app_config) in golden_variants() {
        let first = generate_base_config(&app_config);
        let second = generate_base_config(&app_config);
        assert_eq!(
            first, second,
            "相同 AppConfig 连续生成结果不一致，说明生成过程存在非确定性"
        );
    }
}

#[test]
fn generated_config_should_be_stable_under_patch() {
    // 「双模型收敛」的核心断言：**生成路径与补丁路径对同一份 intent 必须产出同一份配置**。
    //
    // 收敛前两者会分叉：补丁路径会重排 dns.servers（假 DNS server 被挪到末尾）、
    // 改写假 DNS 规则的形态、覆盖 route.final……
    // 表现是"订阅刚刷新出来的配置"与"改一次设置后的配置"不是一回事，
    // 用户会看到同一套设置在不同时间点生成出不同的线路。
    //
    // 收敛后：生成路径的产物再走一遍补丁路径，必须**逐字段不变**。
    for (name, app_config) in golden_variants() {
        let intent = ConfigIntent::from_app_config(&app_config);
        let expected = render_skeleton(&intent);

        let mut patched = render_skeleton(&intent);
        let ownership = Ownership::claimed(&patched);
        apply_intent_to_config(&mut patched, &intent, &ownership);

        let mut diffs = Vec::new();
        diff_json(&expected, &patched, "", &mut diffs);
        assert!(
            diffs.is_empty(),
            "生成路径与补丁路径的产物不一致（{}）：\n{}",
            name,
            diffs.join("\n")
        );
    }
}
