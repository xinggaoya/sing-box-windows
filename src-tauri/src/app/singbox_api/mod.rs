//! sing-box 1.14+ 官方 gRPC API 客户端
//!
//! 通过 **HTTP/1.1 + gRPC-Web 帧格式** 与内核通信，
//! 完全替代 `experimental.clash_api` 的 HTTP RESTful 调用。
//!
//! sing-box 1.14.0 的 `type: api` 实际**只支持 HTTP/1.1 gRPC-Web 传输**
//! （不支持 WebSocket `grpc-websockets` 子协议；浏览器端 dashboard 通过
//! Connect-RPC 的 fetch 传输走相同 HTTP/1.1 路径）。
//!
//! ## 协议细节
//!
//! - 端点：`POST http://127.0.0.1:<api_port>/daemon.StartedService/<Method>`
//! - 必填 header：`Content-Type: application/grpc-web+proto`、`X-Grpc-Web: 1`、`TE: trailers`
//! - 请求 body：`[flag:u8=0][length:u32 BE][protobuf:N]`
//! - 响应 body：连续多个 frame，格式同上
//!   - flag `0x00` = 数据帧（protobuf message）
//!   - flag `0x80` = trailers 帧（每行 `key: value\r\n`，最后包含 `grpc-status: 0`）
//!
//! 关键：reqwest 客户端必须 `.no_proxy()` 禁用 Windows 系统代理
//! （TUN 客户端可能残留 `127.0.0.1:12080` 代理配置）。

pub mod client;
pub mod proto;
pub mod types;

pub use client::{ApiClientConfig, ApiClientHandle, GroupsSubscription, HttpStream};
pub use types::{
    ClashModeStatus, Connection, ConnectionEvents, Group, Groups, GroupItem, Log, LogEntry,
    LogLevel, NetworkQualityResult, OutboundList, Rule, RuleList, Service, ServiceList,
    ServiceStatus, Status,
};