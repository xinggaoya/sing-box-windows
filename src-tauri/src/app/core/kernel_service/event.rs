//! sing-box 1.14+ 官方 gRPC API 事件中继
//!
//! 通过 4 个 gRPC server-streaming 订阅替代之前 4 个独立 WebSocket：
//! - `SubscribeStatus`     → 流量（traffic-data）+ 内存（合并到 status）+ goroutines
//! - `SubscribeLog`        → 日志（log-data，含 level）
//! - `SubscribeConnections`→ 连接事件（connections-data）
//! - `SubscribeGroups`     → 代理组快照（groups-data）
//!
//! 启动时由 `start_websocket_relay` 创建后台 task，停止内核时由 `cleanup_event_relay_tasks` 终止。

use futures::FutureExt;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tracing::{debug, error, info, warn};

use crate::app::singbox_api::{ApiClientConfig, ApiClientHandle, ConnectionEvents, Groups, Log, Status};

/// 全局通知：清理事件中继任务时通过此 Notify 让所有 task 退出
pub(super) static SHOULD_STOP_EVENTS: tokio::sync::OnceCell<Notify> =
    tokio::sync::OnceCell::const_new();

/// 当前活跃的事件中继 task 句柄
pub(super) static RELAY_TASKS: std::sync::OnceLock<Mutex<Vec<JoinHandle<()>>>> =
    std::sync::OnceLock::new();

/// 清理纪元：`cleanup_event_relay_tasks` 每执行一次 +1。
/// `start_websocket_relay` 在 spawn 与注册句柄之间存在窗口，若并发的 cleanup
/// 恰好落在该窗口内，新 task 既收不到 notify 也不在句柄表里，会泄漏成重复事件流。
/// start 通过在注册完成后复查纪元来堵住这个窗口（见 start_websocket_relay）。
static RELAY_EPOCH: AtomicU64 = AtomicU64::new(0);

fn tasks_vec() -> &'static Mutex<Vec<JoinHandle<()>>> {
    RELAY_TASKS.get_or_init(|| Mutex::new(Vec::new()))
}

pub(super) async fn cleanup_event_relay_tasks() {
    // 先递增纪元再清理，保证 start 侧"注册后复查纪元"能观察到本次清理。
    RELAY_EPOCH.fetch_add(1, Ordering::Relaxed);
    if let Some(notify) = SHOULD_STOP_EVENTS.get() {
        notify.notify_waiters();
    }
    if let Some(tasks) = RELAY_TASKS.get() {
        if let Ok(mut tasks) = tasks.lock() {
            for task in tasks.drain(..) {
                task.abort();
            }
        }
    }
}

/// 启动 4 个 gRPC 订阅并把消息作为 Tauri 事件转发给前端
pub(super) async fn start_websocket_relay(
    app_handle: AppHandle,
    port: Option<u16>,
) -> Result<(), String> {
    cleanup_event_relay_tasks().await;
    let epoch_after_cleanup = RELAY_EPOCH.load(Ordering::Relaxed);

    let port = match port {
        Some(p) => p,
        None => {
            warn!("未提供 API 端口，跳过事件中继");
            return Ok(());
        }
    };

    let config = ApiClientConfig::localhost(port);
    let handle = ApiClientHandle::new(config);
    let notify = SHOULD_STOP_EVENTS
        .get_or_init(|| async { Notify::new() })
        .await;

    let status_handle = handle.clone();
    let status_app = app_handle.clone();
    let status_task = tokio::spawn(async move {
        run_status_relay(status_app, status_handle, notify).await;
    });
    let status_abort = status_task.abort_handle();

    let log_handle = handle.clone();
    let log_app = app_handle.clone();
    let log_task = tokio::spawn(async move {
        run_log_relay(log_app, log_handle, notify).await;
    });
    let log_abort = log_task.abort_handle();

    let conn_handle = handle.clone();
    let conn_app = app_handle.clone();
    let conn_task = tokio::spawn(async move {
        run_connections_relay(conn_app, conn_handle, notify).await;
    });
    let conn_abort = conn_task.abort_handle();

    // 4) SubscribeGroups 持续订阅:URLTest 测速完成后 / SelectOutbound 切换节点 /
    // SetGroupExpand 折叠展开时,sing-box 都会推一帧新的 Groups(含 url_test_delay),
    // 转 `groups-data` 事件给前端,代理页延迟才会实时刷新。
    let groups_handle = handle.clone();
    let groups_app = app_handle.clone();
    let groups_task = tokio::spawn(async move {
        run_groups_relay(groups_app, groups_handle, notify).await;
    });
    let groups_abort = groups_task.abort_handle();

    if let Ok(mut tasks) = tasks_vec().lock() {
        tasks.push(status_task);
        tasks.push(log_task);
        tasks.push(conn_task);
        tasks.push(groups_task);
    }

    // 并发 cleanup 可能在"spawn 完 → 句柄注册完"的窗口内执行：那批 task 当时
    // 尚未注册，cleanup 的 drain 拿不到句柄，notify_waiters 也唤醒不了还在
    // connect 阶段的 task。注册完成后复查纪元，若期间发生过 cleanup，
    // 由这里 abort 自己刚 spawn 的 4 个 task（只动本地句柄，不碰句柄表，
    // 避免误杀并发启动的另一代任务），避免同一内核出现两代中继重复 emit 事件。
    if RELAY_EPOCH.load(Ordering::Relaxed) != epoch_after_cleanup {
        warn!("事件中继启动期间发生并发清理，中止本次启动的 4 个中继 task");
        status_abort.abort();
        log_abort.abort();
        conn_abort.abort();
        groups_abort.abort();
        return Ok(());
    }

    info!("gRPC 事件中继已启动（4 个 server-streaming）");
    Ok(())
}

async fn run_status_relay(app: AppHandle, handle: ApiClientHandle, notify: &Notify) {
    let mut backoff = RelayBackoff::new();
    loop {
        if notify.notified().now_or_never().is_some() {
            break;
        }
        match handle.subscribe_status(1_000_000_000).await {
            Ok(mut sub) => loop {
                tokio::select! {
                    _ = notify.notified() => {
                        sub.close().await;
                        return;
                    }
                    msg = sub.next() => match msg {
                        Ok(Some(status)) => {
                            backoff.reset();
                            emit_traffic_and_memory(&app, &status)
                        }
                        Ok(None) => {
                            warn!(
                                "SubscribeStatus 流结束 (累计成功 {}/失败 {}), 退避 {}ms 后重连",
                                backoff.successes, backoff.consecutive_failures, backoff.current_ms
                            );
                            sub.close().await;
                            backoff.sleep().await;
                            break;
                        }
                        Err(e) => {
                            error!(
                                "SubscribeStatus 流错误: {}, 退避 {}ms 后重连 (累计成功 {}/失败 {})",
                                e, backoff.current_ms, backoff.successes, backoff.consecutive_failures
                            );
                            sub.close().await;
                            backoff.sleep().await;
                            break;
                        }
                    }
                }
            },
            Err(e) => {
                error!(
                    "SubscribeStatus 连接失败: {}, 退避 {}ms 后重连 (累计成功 {}/失败 {})",
                    e, backoff.current_ms, backoff.successes, backoff.consecutive_failures
                );
                backoff.sleep().await;
            }
        }
    }
}

async fn run_log_relay(app: AppHandle, handle: ApiClientHandle, notify: &Notify) {
    let mut backoff = RelayBackoff::new();
    loop {
        if notify.notified().now_or_never().is_some() {
            break;
        }
        match handle.subscribe_log().await {
            Ok(mut sub) => loop {
                tokio::select! {
                    _ = notify.notified() => {
                        sub.close().await;
                        return;
                    }
                    msg = sub.next() => match msg {
                        Ok(Some(log)) => {
                            backoff.reset();
                            emit_log(&app, &log)
                        }
                        Ok(None) => {
                            warn!(
                                "SubscribeLog 流结束 (累计成功 {}/失败 {}), 退避 {}ms 后重连",
                                backoff.successes, backoff.consecutive_failures, backoff.current_ms
                            );
                            sub.close().await;
                            backoff.sleep().await;
                            break;
                        }
                        Err(e) => {
                            error!(
                                "SubscribeLog 流错误: {}, 退避 {}ms 后重连 (累计成功 {}/失败 {})",
                                e, backoff.current_ms, backoff.successes, backoff.consecutive_failures
                            );
                            sub.close().await;
                            backoff.sleep().await;
                            break;
                        }
                    }
                }
            },
            Err(e) => {
                error!(
                    "SubscribeLog 连接失败: {}, 退避 {}ms 后重连 (累计成功 {}/失败 {})",
                    e, backoff.current_ms, backoff.successes, backoff.consecutive_failures
                );
                backoff.sleep().await;
            }
        }
    }
}

async fn run_connections_relay(app: AppHandle, handle: ApiClientHandle, notify: &Notify) {
    let mut backoff = RelayBackoff::new();
    loop {
        if notify.notified().now_or_never().is_some() {
            break;
        }
        match handle.subscribe_connections(1_000_000_000).await {
            Ok(mut sub) => loop {
                tokio::select! {
                    _ = notify.notified() => {
                        sub.close().await;
                        return;
                    }
                    msg = sub.next() => match msg {
                        Ok(Some(events)) => {
                            backoff.reset();
                            emit_connections(&app, &events)
                        }
                        Ok(None) => {
                            warn!(
                                "SubscribeConnections 流结束 (累计成功 {}/失败 {}), 退避 {}ms 后重连",
                                backoff.successes, backoff.consecutive_failures, backoff.current_ms
                            );
                            sub.close().await;
                            backoff.sleep().await;
                            break;
                        }
                        Err(e) => {
                            error!(
                                "SubscribeConnections 流错误: {}, 退避 {}ms 后重连 (累计成功 {}/失败 {})",
                                e, backoff.current_ms, backoff.successes, backoff.consecutive_failures
                            );
                            sub.close().await;
                            backoff.sleep().await;
                            break;
                        }
                    }
                }
            },
            Err(e) => {
                error!(
                    "SubscribeConnections 连接失败: {}, 退避 {}ms 后重连 (累计成功 {}/失败 {})",
                    e, backoff.current_ms, backoff.successes, backoff.consecutive_failures
                );
                backoff.sleep().await;
            }
        }
    }
}

/// server-streaming 重连退避:连续失败时指数增加(2s → 4s → 8s → ... → 30s),
/// 成功接收一条消息后重置。避免内核启动 / 端口未就绪时日志刷屏。
struct RelayBackoff {
    current_ms: u64,
    consecutive_failures: u64,
    successes: u64,
}

impl RelayBackoff {
    fn new() -> Self {
        Self { current_ms: 2_000, consecutive_failures: 0, successes: 0 }
    }
    /// 失败一次:sleep 当前延迟后翻倍(最大 30s),累计失败 +1
    async fn sleep(&mut self) {
        tokio::time::sleep(Duration::from_millis(self.current_ms)).await;
        self.consecutive_failures += 1;
        self.current_ms = (self.current_ms * 2).min(30_000);
    }
    /// 成功接收一条消息:重置延迟到 2s,累计成功 +1
    fn reset(&mut self) {
        self.current_ms = 2_000;
        self.consecutive_failures = 0;
        self.successes += 1;
    }
}

fn emit_traffic_and_memory(app: &AppHandle, status: &Status) {
    if status.traffic_available {
        let payload = serde_json::json!({
            "up": status.uplink,
            "down": status.downlink,
        });
        if let Err(e) = app.emit("traffic-data", payload) {
            warn!("emit traffic-data failed: {}", e);
        }
    }
    let mem_payload = serde_json::json!({
        "inuse": status.memory,
        "oslimit": 0u64,
    });
    if let Err(e) = app.emit("memory-data", mem_payload) {
        warn!("emit memory-data failed: {}", e);
    }
}

fn emit_log(app: &AppHandle, log: &Log) {
    for entry in &log.messages {
        let payload = serde_json::json!({
            "type": format!("{:?}", entry.level).to_lowercase(),
            "payload": entry.message,
        });
        if let Err(e) = app.emit("log-data", payload) {
            warn!("emit log-data failed: {}", e);
        }
    }
}

async fn run_groups_relay(app: AppHandle, handle: ApiClientHandle, notify: &Notify) {
    let mut backoff = RelayBackoff::new();
    loop {
        if notify.notified().now_or_never().is_some() {
            break;
        }
        match handle.subscribe_groups().await {
            Ok(mut sub) => loop {
                tokio::select! {
                    _ = notify.notified() => {
                        sub.close().await;
                        return;
                    }
                    msg = sub.next() => match msg {
                        Ok(Some(groups)) => {
                            backoff.reset();
                            emit_groups(&app, &groups)
                        }
                        Ok(None) => {
                            warn!(
                                "SubscribeGroups 流结束 (累计成功 {}/失败 {}), 退避 {}ms 后重连",
                                backoff.successes, backoff.consecutive_failures, backoff.current_ms
                            );
                            sub.close().await;
                            backoff.sleep().await;
                            break;
                        }
                        Err(e) => {
                            error!(
                                "SubscribeGroups 流错误: {}, 退避 {}ms 后重连 (累计成功 {}/失败 {})",
                                e, backoff.current_ms, backoff.successes, backoff.consecutive_failures
                            );
                            sub.close().await;
                            backoff.sleep().await;
                            break;
                        }
                    }
                }
            },
            Err(e) => {
                error!("SubscribeGroups 连接失败: {}", e);
                backoff.sleep().await;
            }
        }
    }
}

fn emit_groups(app: &AppHandle, groups: &Groups) {
    // 直接 emit 后端解出来的 Groups 结构(snake_case 字段名),前端负责按需转换。
    // SubscribeGroups 在以下情况都会推新 frame:URLTest 测速完成 / SelectOutbound 切换 /
    // SetGroupExpand 折叠展开,前端实时收到。
    //
    // debug 级诊断:每帧的 group 数 + items 总数 + 分类计数(成功 / 失败 / 未测),
    // 排查"测速后没刷新"或"全部失败"时能直接看出 relay 链路是否在推送,以及测速
    // 结果的分布。失败 = urlTestTime > 0 但 urlTestDelay == 0(sing-box 1.14 URLTest
    // 失败时调 DeleteURLTestHistory,history 不存在时 proto 默认 0)。
    if tracing::enabled!(tracing::Level::DEBUG) {
        let total_items: usize = groups.group.iter().map(|g| g.items.len()).sum();
        let with_delay: usize = groups
            .group
            .iter()
            .flat_map(|g| g.items.iter())
            .filter(|it| it.url_test_delay > 0)
            .count();
        let with_failure: usize = groups
            .group
            .iter()
            .flat_map(|g| g.items.iter())
            .filter(|it| it.url_test_time > 0 && it.url_test_delay == 0)
            .count();
        let untested = total_items.saturating_sub(with_delay + with_failure);
        debug!(
            "groups-data emit: groups={} items={} (ok={} failed={} untested={})",
            groups.group.len(),
            total_items,
            with_delay,
            with_failure,
            untested
        );
    }
    if let Err(e) = app.emit("groups-data", groups) {
        warn!("emit groups-data failed: {}", e);
    }
}

fn emit_connections(app: &AppHandle, events: &ConnectionEvents) {
    // sing-box 1.14 gRPC SubscribeConnections 返回的是 ConnectionEvent 列表(每次推送可能包含
    // NEW/UPDATE/CLOSED 三种事件),字段名是 snake_case(inbound_type/uplink_total/from_outbound 等),
    // 与前端 ConnectionStore 期望的 `connections: ConnectionItem[]` 格式(字段名 chains/download/start
    // 等)不一致。
    //
    // 这里把 ConnectionEvent 列表展开成"当前活跃连接"列表,字段重命名为前端期望的格式,
    // 然后以 `{connections: [...], uploadTotal, downloadTotal, memory}` emit。
    // CLOSED 事件通过 `id` 隐式删除(前端在 activeConnections 中找不到就移到 closedConnections)。
    //
    // serde 字段名必须与 src/types/events.ts 的 ConnectionsDataPayload /
    // ConnectionItem / ConnectionMetadata 一致(camelCase,IP 字段为 sourceIP /
    // destinationIP 大写 IP)。
    use serde::Serialize;
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Metadata {
        network: String,
        #[serde(rename = "type")]
        ty: String,
        #[serde(rename = "sourceIP")]
        source_ip: String,
        #[serde(rename = "destinationIP")]
        destination_ip: String,
        source_port: String,
        destination_port: String,
        host: String,
        dns_mode: String,
        process: String,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct ConnItem {
        id: String,
        chains: Vec<String>,
        rule: String,
        rule_payload: String,
        start: String,
        upload: i64,
        download: i64,
        metadata: Metadata,
    }
    #[derive(Serialize)]
    #[serde(rename_all = "camelCase")]
    struct Payload {
        connections: Vec<ConnItem>,
        upload_total: i64,
        download_total: i64,
        memory: u64,
    }

    let mut active: Vec<ConnItem> = Vec::with_capacity(events.events.len());
    let mut upload_total: i64 = 0;
    let mut download_total: i64 = 0;
    for ev in &events.events {
        // CLOSED 事件:connection 字段为 None,只通过 id 删除(前端自然处理)
        if let Some(conn) = &ev.connection {
            // sing-box 1.14 gRPC Connection.from_outbound 是单个 string(可能逗号分隔多跳),
            // 这里简单拆成 chains: [outbound, ...from_outbound 拆分]。
            // 已知限制:from_outbound 字段在 1.14 中是 single string(不是 Vec),
            // 复杂拆分留给未来按需细化。
            let mut chains: Vec<String> = Vec::new();
            if !conn.outbound.is_empty() {
                chains.push(conn.outbound.clone());
            }
            if !conn.from_outbound.is_empty() {
                // 尝试按逗号拆分(可能格式 "hop1,hop2");如果拆分后只有一个元素就是原字符串
                for hop in conn.from_outbound.split(',') {
                    let h = hop.trim();
                    if !h.is_empty() {
                        chains.push(h.to_string());
                    }
                }
            }
            upload_total = upload_total.saturating_add(ev.uplink_delta);
            download_total = download_total.saturating_add(ev.downlink_delta);
            let metadata = Metadata {
                network: conn.network.clone(),
                ty: conn.inbound_type.clone(),
                source_ip: conn.source.clone(),
                destination_ip: conn.destination.clone(),
                source_port: String::new(),
                destination_port: String::new(),
                host: conn.domain.clone(),
                dns_mode: String::new(),
                process: String::new(),
            };
            active.push(ConnItem {
                id: conn.id.clone(),
                chains,
                rule: conn.rule.clone(),
                rule_payload: conn.rule.clone(),
                start: chrono::DateTime::from_timestamp_millis(conn.created_at)
                    .map(|t| t.to_rfc3339())
                    .unwrap_or_default(),
                upload: conn.uplink,
                download: conn.downlink,
                metadata,
            });
        }
    }

    let payload = Payload {
        connections: active,
        upload_total,
        download_total,
        memory: 0,
    };
    if let Err(e) = app.emit("connections-data", &payload) {
        warn!("emit connections-data failed: {}", e);
    }
}