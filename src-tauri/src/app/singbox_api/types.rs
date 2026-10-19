//! sing-box gRPC API 数据类型（手写，对应 `daemon.StartedService` proto 中我们关心的子集）
//!
//! ## ts-rs 导出
//!
//! 本文件的所有跨端类型都经 ts-rs 导出到 `src/types/generated/`，是前后端的**唯一事实来源**。
//! 前端不得再手写这些结构的 TypeScript 声明（详见 `docs/implementation-plan-2026-10.md` 阶段 0）。
//!
//! ### ⚠️ serde 与 ts-rs 的 rename：默认会传导，但仍要显式写 `#[ts(...)]`
//!
//! ts-rs 11 默认开启 `serde-compat` 特性，**会继承** serde 的 `rename` / `rename_all`，
//! 所以只写 `#[serde(rename_all = "camelCase")]` 也能生成出 camelCase 的 TS 字段。
//!
//! 但两者的**优先级**是 `#[ts(...)]` 覆盖 serde，且继承行为依赖 feature 开关——
//! 一旦两边写得不一样（历史代码里很容易发生），最终以 `#[ts(...)]` 为准。
//! 因此约定：**凡是带 serde rename 的结构体都显式写 `#[ts(rename_all = ...)]`**，
//! 让生成的 TS 字段名不依赖任何隐式行为。
//!
//! `src/types/generated/grpc-types.test.ts` 会对本文件做正则扫描来守护这条约定。

use serde::{Deserialize, Serialize};
use ts_rs::TS;

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/Groups.ts")]
pub struct Groups {
    pub group: Vec<Group>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/Group.ts")]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct Group {
    pub tag: String,
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub group_type: String,
    pub selectable: bool,
    pub selected: String,
    pub is_expand: bool,
    pub items: Vec<GroupItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/GroupItem.ts")]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct GroupItem {
    pub tag: String,
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub item_type: String,
    /// ts-rs 默认把 i64 映射成 `bigint`，但 JSON 里就是普通数字；
    /// 显式覆写成 `number` 才能和实际载荷、以及前端既有写法一致。
    #[ts(type = "number")]
    pub url_test_time: i64,
    pub url_test_delay: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/OutboundList.ts")]
pub struct OutboundList {
    pub outbounds: Vec<GroupItem>,
}

// === sing-box 1.14 gRPC API 新增类型 ===

/// sing-box 1.14 新增：路由规则（GetRules）
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/Rule.ts")]
pub struct Rule {
    pub rule_type: String,
    pub payload: String,
    pub outbound: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/RuleList.ts")]
pub struct RuleList {
    pub rules: Vec<Rule>,
}

/// sing-box 1.14 新增：服务列表（GetServices）
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/Service.ts")]
pub struct Service {
    pub name: String,
    pub r#type: String,
    pub running: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/ServiceList.ts")]
pub struct ServiceList {
    pub services: Vec<Service>,
}

/// sing-box 1.14 新增：网络质量测试结果（NetworkQualityTest）
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/NetworkQualityResult.ts"
)]
pub struct NetworkQualityResult {
    pub tcp_rtt_ms: i32,
    /// 带宽单位 bps；见 `GroupItem::url_test_time` 的 `#[ts(type = "number")]` 说明。
    #[ts(type = "number")]
    pub download_speed_bps: i64,
    pub download_latency_ms: i32,
    #[ts(type = "number")]
    pub upload_speed_bps: i64,
    pub upload_latency_ms: i32,
}

/// sing-box 1.14 新增：已弃用字段告警（GetDeprecatedWarnings）
/// 用于启动后提示用户哪些配置字段将在后续版本（1.16）被移除。
#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/DeprecatedWarning.ts")]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct DeprecatedWarning {
    pub message: String,
    pub impending: bool,
    pub migration_link: String,
    pub description: String,
    pub deprecated_version: String,
    pub scheduled_version: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/DeprecatedWarnings.ts")]
pub struct DeprecatedWarnings {
    pub warnings: Vec<DeprecatedWarning>,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/ServiceStatus.ts")]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
pub enum ServiceStatus {
    #[default]
    Idle,
    Starting,
    Started,
    Stopping,
    Fatal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/Status.ts")]
pub struct Status {
    /// i64/u64 在 JSON 里是普通数字，统一覆写成 `number`（见 GroupItem::url_test_time）。
    #[ts(type = "number")]
    pub memory: u64,
    pub goroutines: i32,
    pub connections_in: i32,
    pub connections_out: i32,
    pub traffic_available: bool,
    #[ts(type = "number")]
    pub uplink: i64,
    #[ts(type = "number")]
    pub downlink: i64,
    #[ts(type = "number")]
    pub uplink_total: i64,
    #[ts(type = "number")]
    pub downlink_total: i64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/LogLevel.ts")]
#[serde(rename_all = "lowercase")]
#[ts(rename_all = "lowercase")]
pub enum LogLevel {
    #[default]
    Info,
    Panic,
    Fatal,
    Error,
    Warn,
    Debug,
    Trace,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/LogEntry.ts")]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/Log.ts")]
pub struct Log {
    pub messages: Vec<LogEntry>,
    pub reset: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq, TS)]
#[ts(export, export_to = "../../src/types/generated/ConnectionEventType.ts")]
pub enum ConnectionEventType {
    #[default]
    New = 0,
    Update = 1,
    Closed = 2,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/ConnectionEvent.ts")]
pub struct ConnectionEvent {
    #[serde(rename = "type")]
    #[ts(rename = "type")]
    pub event_type: ConnectionEventType,
    pub id: String,
    pub connection: Option<Connection>,
    #[ts(type = "number")]
    pub uplink_delta: i64,
    #[ts(type = "number")]
    pub downlink_delta: i64,
    #[ts(type = "number")]
    pub closed_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/ConnectionEvents.ts")]
pub struct ConnectionEvents {
    pub events: Vec<ConnectionEvent>,
    pub reset: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/Connection.ts")]
pub struct Connection {
    pub id: String,
    pub inbound: String,
    pub inbound_type: String,
    pub ip_version: i32,
    pub network: String,
    pub source: String,
    pub destination: String,
    pub domain: String,
    #[serde(default)]
    pub protocol: String,
    #[serde(default)]
    pub user: String,
    #[serde(default)]
    pub from_outbound: String,
    #[ts(type = "number")]
    pub created_at: i64,
    #[ts(type = "number")]
    pub closed_at: i64,
    #[ts(type = "number")]
    pub uplink: i64,
    #[ts(type = "number")]
    pub downlink: i64,
    #[ts(type = "number")]
    pub uplink_total: i64,
    #[ts(type = "number")]
    pub downlink_total: i64,
    #[serde(default)]
    pub rule: String,
    #[serde(default)]
    pub outbound: String,
    #[serde(default)]
    pub outbound_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../../src/types/generated/ClashModeStatus.ts")]
#[serde(rename_all = "camelCase")]
#[ts(rename_all = "camelCase")]
pub struct ClashModeStatus {
    pub mode_list: Vec<String>,
    pub current_mode: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, TS)]
#[ts(
    export,
    export_to = "../../src/types/generated/ServiceStatusSnapshot.ts"
)]
pub struct ServiceStatusSnapshot {
    pub status: ServiceStatus,
    pub error_message: String,
}

impl From<i32> for ServiceStatus {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Starting,
            2 => Self::Started,
            3 => Self::Stopping,
            4 => Self::Fatal,
            _ => Self::Idle,
        }
    }
}

impl From<i32> for LogLevel {
    fn from(value: i32) -> Self {
        match value {
            0 => Self::Panic,
            1 => Self::Fatal,
            2 => Self::Error,
            3 => Self::Warn,
            4 => Self::Info,
            5 => Self::Debug,
            6 => Self::Trace,
            _ => Self::Info,
        }
    }
}

impl From<i32> for ConnectionEventType {
    fn from(value: i32) -> Self {
        match value {
            1 => Self::Update,
            2 => Self::Closed,
            _ => Self::New,
        }
    }
}
