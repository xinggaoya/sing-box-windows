//! sing-box gRPC API 数据类型（手写，对应 `daemon.StartedService` proto 中我们关心的子集）

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Groups {
    pub group: Vec<Group>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Group {
    pub tag: String,
    #[serde(rename = "type")]
    pub group_type: String,
    pub selectable: bool,
    pub selected: String,
    pub is_expand: bool,
    pub items: Vec<GroupItem>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct GroupItem {
    pub tag: String,
    #[serde(rename = "type")]
    pub item_type: String,
    pub url_test_time: i64,
    pub url_test_delay: i32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct OutboundList {
    pub outbounds: Vec<GroupItem>,
}

// === sing-box 1.14 gRPC API 新增类型 ===

/// sing-box 1.14 新增：路由规则（GetRules）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Rule {
    pub rule_type: String,
    pub payload: String,
    pub outbound: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct RuleList {
    pub rules: Vec<Rule>,
}

/// sing-box 1.14 新增：服务列表（GetServices）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Service {
    pub name: String,
    pub r#type: String,
    pub running: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ServiceList {
    pub services: Vec<Service>,
}

/// sing-box 1.14 新增：网络质量测试结果（NetworkQualityTest）
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NetworkQualityResult {
    pub tcp_rtt_ms: i32,
    pub download_speed_bps: i64,
    pub download_latency_ms: i32,
    pub upload_speed_bps: i64,
    pub upload_latency_ms: i32,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ServiceStatus {
    #[default]
    Idle,
    Starting,
    Started,
    Stopping,
    Fatal,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Status {
    pub memory: u64,
    pub goroutines: i32,
    pub connections_in: i32,
    pub connections_out: i32,
    pub traffic_available: bool,
    pub uplink: i64,
    pub downlink: i64,
    pub uplink_total: i64,
    pub downlink_total: i64,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
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

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Log {
    pub messages: Vec<LogEntry>,
    pub reset: bool,
}

#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize, PartialEq, Eq)]
#[repr(i32)]
pub enum ConnectionEventType {
    #[default]
    New = 0,
    Update = 1,
    Closed = 2,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionEvent {
    #[serde(rename = "type")]
    pub event_type: ConnectionEventType,
    pub id: String,
    pub connection: Option<Connection>,
    pub uplink_delta: i64,
    pub downlink_delta: i64,
    pub closed_at: i64,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ConnectionEvents {
    pub events: Vec<ConnectionEvent>,
    pub reset: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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
    pub created_at: i64,
    pub closed_at: i64,
    pub uplink: i64,
    pub downlink: i64,
    pub uplink_total: i64,
    pub downlink_total: i64,
    #[serde(default)]
    pub rule: String,
    #[serde(default)]
    pub outbound: String,
    #[serde(default)]
    pub outbound_type: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ClashModeStatus {
    pub mode_list: Vec<String>,
    pub current_mode: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
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