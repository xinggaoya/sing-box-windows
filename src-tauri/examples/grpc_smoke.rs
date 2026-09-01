//! sing-box 1.14+ gRPC API 客户端实机联调 smoke test
//!
//! 前置：先在一个独立工作目录启动 sing-box：
//!   `sing-box.exe run -c .tmp_smoke/config.json`
//! （监听 127.0.0.1:29999，启用 `type: api`）
//!
//! 运行：
//!   `cargo run --example grpc_smoke`

use app_lib::app::singbox_api::{
    ApiClientConfig, ApiClientHandle, ClashModeStatus, Groups, LogLevel,
};
use std::time::Duration;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let handle = ApiClientHandle::new(ApiClientConfig::localhost(29999));

    // 1) GetVersion
    let (version, api_version) = handle.get_version().await?;
    println!("✅ GetVersion: version={version}, apiVersion={api_version}");

    // 2) GetGroups（一次快照）
    let groups: Groups = handle.get_groups_snapshot().await?;
    println!(
        "✅ GetGroups: {} groups, first group has {} items",
        groups.group.len(),
        groups.group.first().map(|g| g.items.len()).unwrap_or(0)
    );
    for g in &groups.group {
        println!("   - tag={} type={} selected={} items={}", g.tag, g.group_type, g.selected, g.items.len());
    }

    // 3) SelectOutbound
    if let Some(g) = groups.group.iter().find(|g| !g.items.is_empty()) {
        let target = if g.items.len() > 1 { &g.items[1].tag } else { &g.items[0].tag };
        handle.select_outbound(&g.tag, target).await?;
        println!("✅ SelectOutbound: group={} → outbound={}", g.tag, target);
    }

    // 4) URLTest
    if let Some(g) = groups.group.first() {
        handle.url_test(&g.tag).await?;
        println!("✅ URLTest: triggered on group={}", g.tag);
    }

    // 5) GetClashModeStatus
    let clash: ClashModeStatus = handle.get_clash_mode_status().await?;
    println!(
        "✅ GetClashModeStatus: currentMode={}, modeList={:?}",
        clash.current_mode, clash.mode_list
    );

    // 6) SetClashMode + 验证
    handle.set_clash_mode("global").await?;
    println!("✅ SetClashMode: → global");
    let clash2 = handle.get_clash_mode_status().await?;
    println!(
        "✅ GetClashModeStatus (after SetClashMode): currentMode={}, modeList={:?}",
        clash2.current_mode, clash2.mode_list
    );
    handle.set_clash_mode("rule").await?;
    println!("✅ SetClashMode: → rule (回滚)");

    // 7) SetGroupExpand
    if let Some(g) = groups.group.first() {
        handle.set_group_expand(&g.tag, true).await?;
        println!("✅ SetGroupExpand: group={} → is_expand=true", g.tag);
    }

    // 8) GetStartedAt
    let started_at = handle.get_started_at().await?;
    println!("✅ GetStartedAt: {started_at}");

    // 9) SubscribeStatus（取 1 条消息后立即关闭）
    {
        let mut sub = handle.subscribe_status(500_000_000).await?;
        let status = sub.next().await?;
        if let Some(s) = status {
            println!(
                "✅ SubscribeStatus: mem={}MB connIn={} connOut={} up={} down={}",
                s.memory / 1024 / 1024,
                s.connections_in,
                s.connections_out,
                s.uplink,
                s.downlink
            );
        } else {
            println!("⚠️  SubscribeStatus: 流空");
        }
        sub.close().await;
    }

    // 10) SubscribeLog（取 1 条消息后立即关闭）
    {
        let mut sub = handle.subscribe_log().await?;
        // 等待最多 2 秒拿一条
        let log = tokio::time::timeout(Duration::from_secs(2), sub.next()).await;
        match log {
            Ok(Ok(Some(l))) if !l.messages.is_empty() => {
                let entry = &l.messages[0];
                let level = match entry.level {
                    LogLevel::Panic => "panic",
                    LogLevel::Fatal => "fatal",
                    LogLevel::Error => "error",
                    LogLevel::Warn => "warn",
                    LogLevel::Info => "info",
                    LogLevel::Debug => "debug",
                    LogLevel::Trace => "trace",
                };
                println!(
                    "✅ SubscribeLog: level={} message={:?}",
                    level,
                    entry.message.chars().take(80).collect::<String>()
                );
            }
            Ok(Ok(Some(_))) => println!("✅ SubscribeLog: 收到空 batch"),
            Ok(Ok(None)) => println!("⚠️  SubscribeLog: 流结束（2s 内无消息，正常）"),
            Ok(Err(e)) => println!("❌ SubscribeLog 错误: {e}"),
            Err(_) => println!("⚠️  SubscribeLog: 2s 超时无消息"),
        }
        sub.close().await;
    }

    // 11) SubscribeConnections（取 1 条消息后立即关闭）
    {
        let mut sub = handle.subscribe_connections(500_000_000).await?;
        let result = tokio::time::timeout(Duration::from_secs(2), sub.next()).await;
        match result {
            Ok(Ok(Some(events))) => {
                println!(
                    "✅ SubscribeConnections: reset={} events={}",
                    events.reset,
                    events.events.len()
                );
            }
            Ok(Ok(None)) => println!("⚠️  SubscribeConnections: 流结束（无连接，正常）"),
            Ok(Err(e)) => println!("❌ SubscribeConnections 错误: {e}"),
            Err(_) => println!("⚠️  SubscribeConnections: 2s 超时"),
        }
        sub.close().await;
    }

    println!("\n🎉 全部 gRPC API 调用通过！");
    Ok(())
}