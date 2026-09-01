//! sing-box 1.14+ `type: api` 客户端（HTTP/1.1 gRPC-Web 传输）
//!
//! sing-box 1.14.0 的官方 gRPC API 实际只接受 HTTP/1.1 + gRPC-Web 帧格式
//! （不支持 WebSocket 子协议 `grpc-websockets`；浏览器端 dashboard 通过
//!  Connect-RPC 的 fetch 传输访问，浏览器和 Rust 端协议一致）。
//!
//! 帧格式：
//! - 请求 body：1 字节 flag (0=未压缩) + 4 字节 BE 长度 + protobuf message
//! - 响应 body：1 字节 flag + 4 字节 BE 长度 + protobuf（可多次连续）
//! - 响应 trailers：HTTP trailer 头（grpc-status、grpc-message）
//!
//! server-streaming：HTTP body 多个 data frame 连续 + 末尾 trailer 帧（trailer
//! 头通过 chunked transfer 编码单独发送）。
//!
//! 参考：[gRPC-Web spec](https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md)

use futures::StreamExt;
use reqwest::Client;
use std::error::Error as _;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::warn;

use super::proto::{decode_error_to_string, Decoder};
use super::types::{ClashModeStatus, ConnectionEvents, Groups, Log, Status};

#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("HTTP error: {0}")]
    Http(String),
    #[error("gRPC status {status}: {message}")]
    Grpc { status: i32, message: String },
    #[error("decode error: {0}")]
    Decode(String),
    #[error("connection closed")]
    Closed,
}

pub type ApiResult<T> = Result<T, ApiError>;

#[derive(Debug, Clone)]
pub struct ApiClientConfig {
    pub host: String,
    pub port: u16,
    pub secret: String,
}

impl ApiClientConfig {
    pub fn localhost(port: u16) -> Self {
        Self {
            host: "127.0.0.1".to_string(),
            port,
            secret: String::new(),
        }
    }

    fn base_url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

/// 共享配置：跨任务 / 跨线程
#[derive(Clone)]
pub struct ApiClientHandle {
    config: Arc<ApiClientConfig>,
    /// 普通 RPC(unary + 短连接)用：keep-alive 提升高频调用的吞吐
    http: Client,
}

impl ApiClientHandle {
    pub fn new(config: ApiClientConfig) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            // 关键：禁用系统代理。Windows 上 TUN 客户端可能留下 127.0.0.1:12080
            // 之类的代理配置，会让 reqwest 走代理而不是直连内核。
            .no_proxy()
            // 长 server-streaming(SubscribeStatus/Connections)会通过设置
            // `Connection: close` 单独发请求,避开 keep-alive 池化
            // 把长 streaming 连接错误放回 idle 池导致 EOF 的问题。
            // 见 `build_streaming_request`。
            .build()
            .expect("reqwest client");
        Self {
            config: Arc::new(config),
            http,
        }
    }

    pub fn config(&self) -> &ApiClientConfig {
        &self.config
    }

    fn method_url(&self, method: &str) -> String {
        format!(
            "{}/daemon.StartedService/{}",
            self.config.base_url(),
            method
        )
    }

    fn build_request(&self, method: &str, body: Vec<u8>) -> reqwest::RequestBuilder {
        let mut req = self
            .http
            .post(self.method_url(method))
            .header("Content-Type", "application/grpc-web+proto")
            .header("X-Grpc-Web", "1")
            .header("Accept", "application/grpc-web+proto")
            .header("TE", "trailers")
            .body(encode_request_frame(&body));
        if !self.config.secret.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.config.secret));
        }
        req
    }

    /// server-streaming 专用:为每个长 streaming 调用**新建一个独立 reqwest::Client**,
    /// 不复用 `self.http`,不与 unary 调用共享任何内部状态(hyper pool / 连接句柄 /
    /// DNS cache / TLS state 等)。
    ///
    /// 根因(2026-09-01 实机诊断):Tauri runtime 下,3 个并发 streaming 共享同一 reqwest
    /// client 时,sing-box 1.14 gRPC server 会在某个 streaming 首次 send 后立即关闭连接
    /// (body EOF with 0 bytes)。隔离测试(1 个 client + 1 个 stream)无法复现。
    /// 改为独立 client 后,3 个 stream 互不干扰,每个走自己的连接池/资源。
    /// `pool_max_idle_per_host(0)` + 短 timeout 是更彻底的防御:hyper 不会把长 streaming
    /// 连接放回 idle 池导致 EOF 误判(虽然本场景下 server 端问题是主因)。
    fn build_streaming_request(&self, method: &str, body: Vec<u8>) -> reqwest::RequestBuilder {
        // ⚠️ reqwest 0.12 的 footgun:`Duration::from_secs(0)` 在 ClientBuilder::timeout
        // 里表示"立即超时"(is_timeout=true),不是"无限超时"。streaming 必须不传 timeout。
        // 同样:`Duration::from_secs(0)` 用在 `connect_timeout` 等其他 timeout 也会触发同样问题。
        // 这里**只**配置 `pool_max_idle_per_host(0)`,其他全部用 reqwest 默认,行为稳定。
        let streaming_http = Client::builder()
            .no_proxy()
            // 关键:禁用 keep-alive 池,杜绝 hyper 把长 streaming 连接错误放回 idle
            .pool_max_idle_per_host(0)
            .build()
            .expect("reqwest streaming client");
        // 排障时临时打开:打印请求全貌(method+url+headers+body hex),用于对比
        // Tauri runtime 下 reqwest 实际发出的 wire 字节与隔离测试的差异。
        let url = self.method_url(method);
        let body_hex: String = body
            .iter()
            .take(40)
            .map(|b| format!("{:02x}", b))
            .collect::<Vec<_>>()
            .join(" ");
        tracing::debug!(
            "REQ method={} url={} body={}B head=[{}]",
            method,
            url,
            body.len(),
            body_hex
        );
        let mut req = streaming_http
            .post(url)
            .header("Content-Type", "application/grpc-web+proto")
            .header("X-Grpc-Web", "1")
            .header("Accept", "application/grpc-web+proto")
            .header("TE", "trailers")
            .body(encode_request_frame(&body));
        if !self.config.secret.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", self.config.secret));
        }
        req
    }

    /// 发起一次 unary RPC
    pub async fn unary(&self, method: &str, body: &[u8]) -> ApiResult<Vec<u8>> {
        self.unary_with_timeout(method, body, None).await
    }

    /// 带独立超时的 unary RPC。共享 client 有 15s 总超时（`ApiClientHandle::new`）,
    /// 慢 RPC（如 NetworkQualityTest 要跑带宽压测）需要单独放宽。
    pub async fn unary_with_timeout(
        &self,
        method: &str,
        body: &[u8],
        timeout: Option<Duration>,
    ) -> ApiResult<Vec<u8>> {
        let mut req = self.build_request(method, body.to_vec());
        if let Some(t) = timeout {
            req = req.timeout(t);
        }
        let resp = req
            .send()
            .await
            .map_err(|e| ApiError::Http(format!("send: {}", e)))?;
        let resp = parse_response_headers(resp).await?;
        let status = resp.grpc_status;
        let message = resp.grpc_message;
        if status != 0 {
            return Err(ApiError::Grpc { status, message });
        }
        Ok(resp.body)
    }

    /// 一次性获取 Groups 快照
    pub async fn get_groups_snapshot(&self) -> ApiResult<Groups> {
        // SubscribeGroups 是 server-streaming,取首个 data frame 作为快照
        let mut stream = self.subscribe_groups().await?;
        stream.next().await?.ok_or(ApiError::Closed)
    }

    /// sing-box 1.14+：获取所有路由规则（GetRules）
    /// 用于恢复 RulesView.vue / 规则管理 UI
    pub async fn get_rules(&self) -> ApiResult<super::types::RuleList> {
        let resp = self.unary("GetRules", &[]).await?;
        super::proto::decode_rule_list(&resp)
            .map_err(|e| ApiError::Decode(decode_error_to_string(&e)))
    }

    /// sing-box 1.14+：获取所有服务（GetServices）
    /// 用于显示 Tailscale/USB/IP 等 service 状态
    pub async fn get_services(&self) -> ApiResult<super::types::ServiceList> {
        let resp = self.unary("GetServices", &[]).await?;
        super::proto::decode_service_list(&resp)
            .map_err(|e| ApiError::Decode(decode_error_to_string(&e)))
    }

    /// sing-box 1.14+：网络质量测试（unary，返回完整结果）
    /// 等价于 CLI `sing-box api network-quality`，含 TCP RTT + 上下行带宽。
    /// 带宽压测在慢链路上可能远超共享 client 的 15s 总超时,放宽到 90s。
    pub async fn network_quality_test(&self) -> ApiResult<super::types::NetworkQualityResult> {
        let resp = self
            .unary_with_timeout("NetworkQualityTest", &[], Some(Duration::from_secs(90)))
            .await?;
        super::proto::decode_network_quality_result(&resp)
            .map_err(|e| ApiError::Decode(decode_error_to_string(&e)))
    }

    /// GetVersion
    pub async fn get_version(&self) -> ApiResult<(String, i32)> {
        let resp = self.unary("GetVersion", &[]).await?;
        let mut dec = Decoder::new(&resp);
        let mut version = String::new();
        let mut api_version = 0i32;
        while !dec.eof() {
            let tag = dec
                .read_varint()
                .map_err(|e| ApiError::Decode(decode_error_to_string(&e)))?;
            let field = (tag >> 3) as u32;
            let wire = (tag & 0x07) as u8;
            match (field, wire) {
                (1, 2) => {
                    version = dec
                        .read_string()
                        .map_err(|e| ApiError::Decode(decode_error_to_string(&e)))?
                }
                (2, 0) => {
                    api_version = dec
                        .read_i32()
                        .map_err(|e| ApiError::Decode(decode_error_to_string(&e)))?
                }
                _ => dec
                    .skip_field(wire)
                    .map_err(|e| ApiError::Decode(decode_error_to_string(&e)))?,
            }
        }
        Ok((version, api_version))
    }

    /// GetClashModeStatus
    pub async fn get_clash_mode_status(&self) -> ApiResult<ClashModeStatus> {
        let resp = self.unary("GetClashModeStatus", &[]).await?;
        super::proto::decode_clash_mode_status(&resp)
            .map_err(|e| ApiError::Decode(decode_error_to_string(&e)))
    }

    /// GetStartedAt
    pub async fn get_started_at(&self) -> ApiResult<i64> {
        let resp = self.unary("GetStartedAt", &[]).await?;
        let mut dec = Decoder::new(&resp);
        while !dec.eof() {
            let tag = dec
                .read_varint()
                .map_err(|e| ApiError::Decode(decode_error_to_string(&e)))?;
            let field = (tag >> 3) as u32;
            let wire = (tag & 0x07) as u8;
            if field == 1 && wire == 0 {
                return dec
                    .read_i64()
                    .map_err(|e| ApiError::Decode(decode_error_to_string(&e)));
            }
            dec.skip_field(wire)
                .map_err(|e| ApiError::Decode(decode_error_to_string(&e)))?;
        }
        Ok(0)
    }

    /// SelectOutbound
    pub async fn select_outbound(&self, group_tag: &str, outbound_tag: &str) -> ApiResult<()> {
        // SelectOutboundRequest { groupTag=1, outboundTag=2 }
        let mut body = Vec::new();
        body.extend_from_slice(&encode_field_tag(1, 2));
        body.extend_from_slice(&encode_string(group_tag));
        body.extend_from_slice(&encode_field_tag(2, 2));
        body.extend_from_slice(&encode_string(outbound_tag));
        let _ = self.unary("SelectOutbound", &body).await?;
        Ok(())
    }

    /// URLTest
    pub async fn url_test(&self, outbound_tag: &str) -> ApiResult<()> {
        let mut body = Vec::new();
        body.extend_from_slice(&encode_field_tag(1, 2));
        body.extend_from_slice(&encode_string(outbound_tag));
        let _ = self.unary("URLTest", &body).await?;
        Ok(())
    }

    /// CloseConnection
    pub async fn close_connection(&self, id: &str) -> ApiResult<()> {
        let mut body = Vec::new();
        body.extend_from_slice(&encode_field_tag(1, 2));
        body.extend_from_slice(&encode_string(id));
        let _ = self.unary("CloseConnection", &body).await?;
        Ok(())
    }

    /// CloseAllConnections
    pub async fn close_all_connections(&self) -> ApiResult<()> {
        let _ = self.unary("CloseAllConnections", &[]).await?;
        Ok(())
    }

    /// SetClashMode（ClashMode message, mode field number = 3）
    pub async fn set_clash_mode(&self, mode: &str) -> ApiResult<()> {
        let mut body = Vec::new();
        body.extend_from_slice(&encode_field_tag(3, 2));
        body.extend_from_slice(&encode_string(mode));
        let _ = self.unary("SetClashMode", &body).await?;
        Ok(())
    }

    /// SetGroupExpand
    pub async fn set_group_expand(&self, group_tag: &str, is_expand: bool) -> ApiResult<()> {
        let mut body = Vec::new();
        body.extend_from_slice(&encode_field_tag(1, 2));
        body.extend_from_slice(&encode_string(group_tag));
        body.extend_from_slice(&encode_field_tag(2, 0));
        body.extend_from_slice(&encode_varint_i64(if is_expand { 1 } else { 0 }));
        let _ = self.unary("SetGroupExpand", &body).await?;
        Ok(())
    }

    /// SubscribeGroups（server-streaming）
    /// 返回的每帧是 proto Groups(含每个 group 当前 selected / 每个 item 的 url_test_delay)
    pub async fn subscribe_groups(&self) -> ApiResult<GroupsSubscription> {
        let stream = HttpStream::new(self.build_streaming_request("SubscribeGroups", Vec::new())).await?;
        let (tx, rx) = mpsc::channel::<Groups>(16);
        let task = tokio::spawn(spawn_groups_loop(stream, tx));
        Ok(GroupsSubscription { rx, _task: task })
    }

    /// SubscribeStatus（server-streaming）
    pub async fn subscribe_status(
        &self,
        interval_nanos: i64,
    ) -> ApiResult<StatusSubscription> {
        let mut body = Vec::new();
        body.extend_from_slice(&encode_field_tag(1, 0));
        body.extend_from_slice(&encode_varint_i64(interval_nanos));
        let stream = HttpStream::new(self.build_streaming_request("SubscribeStatus", body)).await?;
        let (tx, rx) = mpsc::channel::<Status>(32);
        let task = tokio::spawn(spawn_status_loop(stream, tx));
        Ok(StatusSubscription { rx, _task: task })
    }

    /// SubscribeServiceStatus（server-streaming）
    /// 返回 ServiceStatus{status: IDLE/STARTING/STARTED/STOPPING/FATAL, errorMessage}
    /// 用于探测内核运行状态(替代 1.13 的 experimental.clash_api HTTP /version)。
    pub async fn subscribe_service_status(&self) -> ApiResult<ServiceStatusSubscription> {
        let stream = HttpStream::new(
            self.build_streaming_request("SubscribeServiceStatus", Vec::new()),
        )
        .await?;
        let (tx, rx) = mpsc::channel::<super::types::ServiceStatusSnapshot>(8);
        let task = tokio::spawn(spawn_service_status_loop(stream, tx));
        Ok(ServiceStatusSubscription { rx, _task: task })
    }

    /// SubscribeLog（server-streaming）
    pub async fn subscribe_log(&self) -> ApiResult<LogSubscription> {
        let stream = HttpStream::new(self.build_streaming_request("SubscribeLog", Vec::new())).await?;
        let (tx, rx) = mpsc::channel::<Log>(64);
        let task = tokio::spawn(spawn_log_loop(stream, tx));
        Ok(LogSubscription { rx, _task: task })
    }

    /// SubscribeConnections（server-streaming）
    pub async fn subscribe_connections(
        &self,
        interval_nanos: i64,
    ) -> ApiResult<ConnectionsSubscription> {
        let mut body = Vec::new();
        body.extend_from_slice(&encode_field_tag(1, 0));
        body.extend_from_slice(&encode_varint_i64(interval_nanos));
        let stream = HttpStream::new(self.build_streaming_request("SubscribeConnections", body)).await?;
        let (tx, rx) = mpsc::channel::<ConnectionEvents>(128);
        let task = tokio::spawn(spawn_connections_loop(stream, tx));
        Ok(ConnectionsSubscription { rx, _task: task })
    }

    /// 探测服务状态：通过 SubscribeServiceStatus 拿 sing-box 1.14 真实状态。
    /// 这是 sing-box 1.14 替代 experimental.clash_api HTTP /version 的官方方式。
    /// 拿到任一帧即返回 ServiceStatus{status, errorMessage}。
    pub async fn probe_service_status(&self) -> ApiResult<super::types::ServiceStatusSnapshot> {
        let mut sub = self.subscribe_service_status().await?;
        let result = sub.next().await;
        sub.close().await;
        match result {
            Ok(Some(snap)) => Ok(snap),
            Ok(None) => Ok(super::types::ServiceStatusSnapshot {
                status: super::types::ServiceStatus::Idle,
                error_message: "service status stream returned no frames".to_string(),
            }),
            Err(e) => Err(e),
        }
    }
}

// ============ HTTP Stream（server-streaming via chunked） ============

/// 解析好的 HTTP/1.1 gRPC-Web 响应
struct ParsedResponse {
    /// protobuf body（unary 调用的返回数据，多个 data frame 拼接）
    body: Vec<u8>,
    /// trailer grpc-status（0 表示 OK）
    grpc_status: i32,
    /// trailer grpc-message
    grpc_message: String,
}

async fn parse_response_headers(resp: reqwest::Response) -> ApiResult<ParsedResponse> {
    let status = resp.status();
    if !status.is_success() {
        let text = resp.text().await.unwrap_or_default();
        return Err(ApiError::Http(format!("HTTP {}: {}", status, text)));
    }
    // 1.14 健壮性优化：边读边解析（与 HttpStream::new 一致），避免大响应 OOM。
    // 之前是 `body_stream.next().await` 一次性读到 buffer，对 SubscribeLog 等
    // 长生命周期 server-streaming 调用的 unary 版本（如 GetDeprecatedWarnings 大列表）会内存爆炸。
    let mut body_stream = resp.bytes_stream();
    let mut buffer: Vec<u8> = Vec::new();
    let mut parsed = ParsedResponse {
        body: Vec::new(),
        grpc_status: 0,
        grpc_message: String::new(),
    };
    while let Some(chunk) = body_stream.next().await {
        let chunk = chunk.map_err(|e| ApiError::Http(format!("body chunk: {}", e)))?;
        buffer.extend_from_slice(&chunk);
        // 尝试解析 buffer 中已完整的 frame（trailer 可能在中间，data frame 可能跨 chunk）
        let mut pos = 0;
        while buffer.len() - pos >= 5 {
            let len = u32::from_be_bytes([
                buffer[pos + 1], buffer[pos + 2], buffer[pos + 3], buffer[pos + 4],
            ]) as usize;
            if buffer.len() - pos < 5 + len {
                // frame 不完整，等下一个 chunk
                break;
            }
            let flag = buffer[pos];
            let frame_body = &buffer[pos + 5..pos + 5 + len];
            if (flag & 0x80) != 0 {
                // trailer 帧：解析 grpc-status / grpc-message
                if let Ok(text) = std::str::from_utf8(frame_body) {
                    for line in text.split("\r\n") {
                        if let Some((k, v)) = line.split_once(':') {
                            match k.trim().to_ascii_lowercase().as_str() {
                                "grpc-status" => {
                                    parsed.grpc_status = v.trim().parse().unwrap_or(0);
                                }
                                "grpc-message" => {
                                    parsed.grpc_message = v.trim().to_string();
                                }
                                _ => {}
                            }
                        }
                    }
                }
            } else {
                parsed.body.extend_from_slice(frame_body);
            }
            pos += 5 + len;
        }
        // 移除已解析的字节
        if pos > 0 {
            buffer.drain(..pos);
        }
    }
    // 兜底：剩余 buffer 用老逻辑解析一次（处理跨 chunk 的 frame）
    if !buffer.is_empty() {
        let legacy = parse_grpc_web_body(&buffer)?;
        parsed.body.extend_from_slice(&legacy.body);
        if parsed.grpc_status == 0 {
            parsed.grpc_status = legacy.grpc_status;
        }
        if parsed.grpc_message.is_empty() {
            parsed.grpc_message = legacy.grpc_message;
        }
    }
    Ok(parsed)
}

/// 解析 gRPC-Web over HTTP/1.1 响应 body：多个连续 frame，
/// - flag=0x00: data frame → protobuf message
/// - flag=0x80: trailer frame → "key: value\r\n" 头
fn parse_grpc_web_body(buffer: &[u8]) -> ApiResult<ParsedResponse> {
    let mut body = Vec::new();
    let mut grpc_status: Option<i32> = None;
    let mut grpc_message = String::new();
    let mut pos = 0;
    while pos < buffer.len() {
        if buffer.len() - pos < 5 {
            return Err(ApiError::Http(format!(
                "truncated frame at pos {pos}"
            )));
        }
        let flag = buffer[pos];
        let len = u32::from_be_bytes([
            buffer[pos + 1],
            buffer[pos + 2],
            buffer[pos + 3],
            buffer[pos + 4],
        ]) as usize;
        let frame_end = pos + 5 + len;
        if buffer.len() < frame_end {
            return Err(ApiError::Http(format!(
                "frame body truncated: pos={pos} len={len}"
            )));
        }
        let frame_body = &buffer[pos + 5..frame_end];
        if (flag & 0x80) != 0 {
            // trailer frame
            if let Ok(text) = std::str::from_utf8(frame_body) {
                for line in text.split("\r\n") {
                    if let Some((k, v)) = line.split_once(':') {
                        match k.trim().to_ascii_lowercase().as_str() {
                            "grpc-status" => {
                                grpc_status = v.trim().parse().ok();
                            }
                            "grpc-message" => {
                                grpc_message = v.trim().to_string();
                            }
                            _ => {}
                        }
                    }
                }
            }
        } else {
            body.extend_from_slice(frame_body);
        }
        pos = frame_end;
    }
    Ok(ParsedResponse {
        body,
        grpc_status: grpc_status.unwrap_or(0),
        grpc_message,
    })
}

/// server-streaming 句柄：返回响应中多个 data frame
pub struct HttpStream {
    rx: tokio::sync::mpsc::Receiver<ApiResult<Vec<u8>>>,
    /// 持有的 reqwest Response 任务 JoinHandle
    _task: tokio::task::JoinHandle<()>,
}

impl HttpStream {
    async fn new(req: reqwest::RequestBuilder) -> ApiResult<Self> {
        let resp = req.send().await.map_err(|e| {
            // 完整诊断:输出 reqwest 错误的所有结构化信息,定位 send 阶段失败根因
            ApiError::Http(format!(
                "send: {} (is_connect={} is_timeout={} is_request={} is_body={} is_decode={} src={:?})",
                e,
                e.is_connect(),
                e.is_timeout(),
                e.is_request(),
                e.is_body(),
                e.is_decode(),
                e.source()
            ))
        })?;
        let status = resp.status();
        // 排障时临时打开:打印响应 headers,看 sing-box 返回的 content-length /
        // transfer-encoding / grpc-status / trailers 头。
        let h = resp.headers();
        tracing::debug!(
            "RESP status={} content-length={:?} transfer-encoding={:?} content-type={:?} grpc-status={:?} grpc-message={:?} trailer={:?}",
            status,
            h.get("content-length"),
            h.get("transfer-encoding"),
            h.get("content-type"),
            h.get("grpc-status"),
            h.get("grpc-message"),
            h.get("trailer")
        );
        if !status.is_success() {
            return Err(ApiError::Http(format!("HTTP {}", status)));
        }
        let (tx, rx) = mpsc::channel::<ApiResult<Vec<u8>>>(32);
        let mut body_stream = resp.bytes_stream();
        let task = tokio::spawn(async move {
            let mut buffer: Vec<u8> = Vec::new();
            let mut total_bytes: usize = 0;
            let mut total_chunks: usize = 0;
            while let Some(chunk) = body_stream.next().await {
                let chunk = match chunk {
                    Ok(c) => c,
                    Err(e) => {
                        let _ = tx
                            .send(Err(ApiError::Http(format!(
                                "chunk: {} (is_timeout={} is_body={} src={:?}) after {}B/{} chunks",
                                e,
                                e.is_timeout(),
                                e.is_body(),
                                e.source(),
                                total_bytes,
                                total_chunks
                            ))))
                            .await;
                        break;
                    }
                };
                total_chunks += 1;
                total_bytes += chunk.len();
                if total_chunks == 1 {
                    // 排障时临时打开:首个 chunk hex,用于诊断 body 立即 EOF 的真实字节
                    let head: Vec<String> = chunk
                        .iter()
                        .take(40)
                        .map(|b| format!("{:02x}", b))
                        .collect();
                    tracing::debug!(
                        "HttpStream first chunk: {}B head=[{}]",
                        chunk.len(),
                        head.join(" ")
                    );
                }
                buffer.extend_from_slice(&chunk);
                // 从 buffer 中解析所有完整 data frame
                while buffer.len() >= 5 {
                    let len = u32::from_be_bytes([
                        buffer[1], buffer[2], buffer[3], buffer[4],
                    ]) as usize;
                    if buffer.len() < 5 + len {
                        break;
                    }
                    let frame: Vec<u8> = buffer.drain(..5 + len).collect();
                    let is_metadata = (frame[0] & 0x80) != 0;
                    if is_metadata {
                        if let Ok(text) = std::str::from_utf8(&frame[5..]) {
                            let mut status = 0i32;
                            let mut message = String::new();
                            for line in text.split("\r\n") {
                                if let Some((k, v)) = line.split_once(':') {
                                    match k.trim().to_ascii_lowercase().as_str() {
                                        "grpc-status" => {
                                            status = v.trim().parse().unwrap_or(2);
                                        }
                                        "grpc-message" => {
                                            message = v.trim().to_string();
                                        }
                                        _ => {}
                                    }
                                }
                            }
                            if status != 0 {
                                let _ = tx
                                    .send(Err(ApiError::Grpc { status, message }))
                                    .await;
                            }
                        }
                    } else {
                        let _ = tx.send(Ok(frame[5..].to_vec())).await;
                    }
                }
            }
            // body_stream 结束:打印最终统计,便于定位"send 成功但 0 帧就 EOF"
            if total_bytes == 0 {
                tracing::warn!(
                    "HttpStream body EOF with 0 bytes ({} chunks) — server closed immediately",
                    total_chunks
                );
            } else {
                tracing::debug!(
                    "HttpStream body EOF: {}B / {} chunks",
                    total_bytes,
                    total_chunks
                );
            }
        });
        Ok(Self { rx, _task: task })
    }

    pub async fn next(&mut self) -> ApiResult<Option<Vec<u8>>> {
        match self.rx.recv().await {
            Some(v) => match v {
                Ok(buf) => Ok(Some(buf)),
                Err(e) => Err(e),
            },
            None => Ok(None),
        }
    }
}

/// 占位句柄（保持 API 兼容；任务已 spawn，不需要用户保留）
pub struct HttpStreamHandle;

fn spawn_status_loop(
    mut stream: HttpStream,
    tx: mpsc::Sender<Status>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok(Some(buf)) = stream.next().await {
            match super::proto::decode_status(&buf) {
                Ok(v) => {
                    if tx.send(v).await.is_err() {
                        break;
                    }
                }
                Err(e) => warn!("Status decode error: {}", e),
            }
        }
    })
}

fn spawn_groups_loop(
    mut stream: HttpStream,
    tx: mpsc::Sender<Groups>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok(Some(buf)) = stream.next().await {
            match super::proto::decode_groups(&buf) {
                Ok(v) => {
                    if tx.send(v).await.is_err() {
                        break;
                    }
                }
                Err(e) => warn!("Groups decode error: {}", e),
            }
        }
    })
}

fn spawn_service_status_loop(
    mut stream: HttpStream,
    tx: mpsc::Sender<super::types::ServiceStatusSnapshot>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok(Some(buf)) = stream.next().await {
            match super::proto::decode_service_status(&buf) {
                Ok(v) => {
                    if tx.send(v).await.is_err() {
                        break;
                    }
                }
                Err(e) => warn!("ServiceStatus decode error: {}", e),
            }
        }
    })
}

fn spawn_log_loop(
    mut stream: HttpStream,
    tx: mpsc::Sender<Log>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok(Some(buf)) = stream.next().await {
            match super::proto::decode_log(&buf) {
                Ok(v) => {
                    if tx.send(v).await.is_err() {
                        break;
                    }
                }
                Err(e) => warn!("Log decode error: {}", e),
            }
        }
    })
}

fn spawn_connections_loop(
    mut stream: HttpStream,
    tx: mpsc::Sender<ConnectionEvents>,
) -> tokio::task::JoinHandle<()> {
    tokio::spawn(async move {
        while let Ok(Some(buf)) = stream.next().await {
            match super::proto::decode_connection_events(&buf) {
                Ok(v) => {
                    if tx.send(v).await.is_err() {
                        break;
                    }
                }
                Err(e) => warn!("ConnectionEvents decode error: {}", e),
            }
        }
    })
}

// ============ Subscription ============

pub struct StatusSubscription {
    rx: mpsc::Receiver<Status>,
    _task: tokio::task::JoinHandle<std::result::Result<(), tokio::task::JoinError>>,
}

impl StatusSubscription {
    pub async fn next(&mut self) -> ApiResult<Option<Status>> {
        Ok(self.rx.recv().await)
    }
    pub async fn close(mut self) {
        self.rx.close();
        let _ = self._task.await;
    }
}

pub struct ServiceStatusSubscription {
    rx: mpsc::Receiver<super::types::ServiceStatusSnapshot>,
    _task: tokio::task::JoinHandle<std::result::Result<(), tokio::task::JoinError>>,
}

impl ServiceStatusSubscription {
    pub async fn next(&mut self) -> ApiResult<Option<super::types::ServiceStatusSnapshot>> {
        Ok(self.rx.recv().await)
    }
    pub async fn close(mut self) {
        self.rx.close();
        let _ = self._task.await;
    }
}

pub struct GroupsSubscription {
    rx: mpsc::Receiver<Groups>,
    _task: tokio::task::JoinHandle<std::result::Result<(), tokio::task::JoinError>>,
}

impl GroupsSubscription {
    pub async fn next(&mut self) -> ApiResult<Option<Groups>> {
        Ok(self.rx.recv().await)
    }
    pub async fn close(mut self) {
        self.rx.close();
        let _ = self._task.await;
    }
}

pub struct LogSubscription {
    rx: mpsc::Receiver<Log>,
    _task: tokio::task::JoinHandle<std::result::Result<(), tokio::task::JoinError>>,
}

impl LogSubscription {
    pub async fn next(&mut self) -> ApiResult<Option<Log>> {
        Ok(self.rx.recv().await)
    }
    pub async fn close(mut self) {
        self.rx.close();
        let _ = self._task.await;
    }
}

pub struct ConnectionsSubscription {
    rx: mpsc::Receiver<ConnectionEvents>,
    _task: tokio::task::JoinHandle<std::result::Result<(), tokio::task::JoinError>>,
}

impl ConnectionsSubscription {
    pub async fn next(&mut self) -> ApiResult<Option<ConnectionEvents>> {
        Ok(self.rx.recv().await)
    }
    pub async fn close(mut self) {
        self.rx.close();
        let _ = self._task.await;
    }
}

// ============ Request body 编码 ============

/// protobuf 字段 tag = (field_number << 3) | wire_type
/// wire_type 0=varint(int32/int64/uint32/uint64/sint32/sint64/bool/enum)
/// wire_type 1=64-bit(fixed64/sfixed64/double)
/// wire_type 2=length-delimited(string/bytes/embedded message/packed repeated)
/// wire_type 5=32-bit(fixed32/sfixed32/float)
///
/// ⚠️ 2026-09-01 重大 bug:之前此函数硬编码 wire_type=2,导致 int64 字段(interval/is_expand)
/// 发出去 tag = 0x0A(被解读为 length-delimited),sing-box 1.14 protobuf 解码失败,
/// 返回 grpc-status=13 "cannot parse invalid wire-format data"。修复后必须显式传 wire_type。
pub fn encode_field_tag(field_number: u32, wire_type: u8) -> Vec<u8> {
    encode_varint(((field_number << 3) | (wire_type as u32 & 0x07)) as u64)
}

pub fn encode_string(s: &str) -> Vec<u8> {
    let mut out = Vec::with_capacity(s.len() + 8);
    out.extend_from_slice(&encode_varint(s.len() as u64));
    out.extend_from_slice(s.as_bytes());
    out
}

pub fn encode_varint(mut value: u64) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let mut byte = (value & 0x7f) as u8;
        value >>= 7;
        if value != 0 {
            byte |= 0x80;
        }
        out.push(byte);
        if value == 0 {
            break;
        }
    }
    out
}

pub fn encode_varint_i64(v: i64) -> Vec<u8> {
    encode_varint(v as u64)
}

/// 构造 gRPC-Web 请求 body：`[flag:1=0][len:4 BE][protobuf]`
pub fn encode_request_frame(protobuf_body: &[u8]) -> Vec<u8> {
    let mut out = Vec::with_capacity(5 + protobuf_body.len());
    out.push(0x00); // 1 = compressed flag; 0 means uncompressed
    let len = protobuf_body.len() as u32;
    out.extend_from_slice(&len.to_be_bytes());
    out.extend_from_slice(protobuf_body);
    out
}

// 重新导出 DecodeError 别名
pub use super::proto::DecodeError as DecoderError;