use futures_util::StreamExt;
use serde::Serialize;
use serde_json::Value;
use std::cmp::min;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};

/// 直接的事件发送器，不再使用WebSocket中继
/// 后端直接连接到sing-box API，然后将数据作为Tauri事件发送到前端
pub struct EventDirectRelay<R> {
    app_handle: AppHandle,
    endpoint: String,
    event_name: String,
    parser: Arc<dyn Fn(Value) -> R + Send + Sync>,
    // API connection details (for future use)
    // api_port: u16,
    // token: String,
}

impl<R: Send + Sync + 'static + Serialize> EventDirectRelay<R> {
    pub fn new<F>(
        app_handle: AppHandle,
        endpoint: &str,
        event_name: &str,
        parser: F,
        api_port: u16,
        token: String,
    ) -> Self
    where
        F: Fn(Value) -> R + Send + Sync + 'static,
    {
        Self {
            app_handle,
            endpoint: format!("ws://127.0.0.1:{}{}?token={}", api_port, endpoint, token),
            event_name: event_name.to_string(),
            parser: Arc::new(parser),
            // api_port,
            // token,
        }
    }

    /// 启动直接事件中继。
    ///
    /// 该 future 的生命周期必须跟随 websocket 读取循环，不能被一个空的发送任务提前结束；
    /// 否则前端会在内核仍运行时失去日志/连接/流量事件。
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = self.endpoint.as_str();
        let (ws_stream, _) = connect_async(url).await?;
        let (_write, mut read) = ws_stream.split();

        let mut message_count = 0u64;

        while let Some(msg) = read.next().await {
            match msg {
                Ok(Message::Text(text)) => match serde_json::from_str::<Value>(&text) {
                    Ok(data) => {
                        let parsed_data = (self.parser.as_ref())(data);

                        // 直接发送 Tauri 事件到前端
                        self.app_handle
                            .emit(&self.event_name, &parsed_data)
                            .map_err(|e| {
                                let message = format!("发送{}事件失败: {}", self.event_name, e);
                                error!("{}", message);
                                std::io::Error::new(std::io::ErrorKind::BrokenPipe, message)
                            })?;

                        message_count += 1;

                        // 每100条消息记录一次
                        if message_count % 100 == 0 {
                            info!("{} 已处理{}条数据", self.event_name, message_count);
                        }
                    }
                    Err(e) => {
                        warn!("解析{}数据失败: {}", self.event_name, e);
                    }
                },
                Ok(Message::Close(frame)) => {
                    let message = format!("{} websocket 连接已关闭: {:?}", self.event_name, frame);
                    warn!("{}", message);
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::ConnectionAborted,
                        message,
                    )
                    .into());
                }
                Err(e) => {
                    error!("{} websocket 连接错误: {}", self.event_name, e);
                    return Err(e.into());
                }
                _ => {}
            }
        }

        let message = format!("{} websocket 数据流结束", self.event_name);
        warn!("{}", message);
        Err(std::io::Error::new(std::io::ErrorKind::UnexpectedEof, message).into())
    }
}

/// 创建流量数据事件发送器
pub fn create_traffic_event_relay(
    app_handle: AppHandle,
    api_port: u16,
    token: String,
) -> EventDirectRelay<Value> {
    EventDirectRelay::new(
        app_handle,
        "/traffic",
        "traffic-data",
        |data| data,
        api_port,
        token,
    )
}

/// 创建内存数据事件发送器
pub fn create_memory_event_relay(
    app_handle: AppHandle,
    api_port: u16,
    token: String,
) -> EventDirectRelay<Value> {
    EventDirectRelay::new(
        app_handle,
        "/memory",
        "memory-data",
        |data| data,
        api_port,
        token,
    )
}

/// 创建日志事件发送器
pub fn create_log_event_relay(
    app_handle: AppHandle,
    api_port: u16,
    token: String,
) -> EventDirectRelay<Value> {
    EventDirectRelay::new(
        app_handle,
        "/logs",
        "log-data",
        |data| data,
        api_port,
        token,
    )
}

/// 创建连接事件发送器
pub fn create_connection_event_relay(
    app_handle: AppHandle,
    api_port: u16,
    token: String,
) -> EventDirectRelay<Value> {
    EventDirectRelay::new(
        app_handle,
        "/connections",
        "connections-data",
        |data| data,
        api_port,
        token,
    )
}

/// 启动事件中继器并在失败时按退避策略重试
pub async fn start_event_relay_with_retry(
    relay: EventDirectRelay<Value>,
    relay_type: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut retry_count = 0;
    let max_retries = 8; // 增加重试次数
    let mut retry_delay = std::time::Duration::from_secs(1);
    let max_retry_delay = std::time::Duration::from_secs(10);

    info!(
        "🔌 开始启动{}事件中继器，最大重试次数: {}",
        relay_type, max_retries
    );

    loop {
        match relay.start().await {
            Ok(_) => {
                info!("✅ {}事件中继器启动成功并正常结束", relay_type);
                break Ok(());
            }
            Err(e) => {
                retry_count += 1;

                if retry_count >= max_retries {
                    error!(
                        "❌ {}事件中继器重试{}次后仍然失败: {}",
                        relay_type, max_retries, e
                    );
                    break Err(e);
                }

                // 根据重试次数调整延迟时间，但不超过最大延迟
                if retry_count <= 3 {
                    retry_delay = std::time::Duration::from_secs(retry_count as u64);
                } else {
                    retry_delay = min(retry_delay * 2, max_retry_delay);
                }

                warn!(
                    "⚠️ {}事件中继器失败，{}秒后重试 ({}/{}): {}",
                    relay_type,
                    retry_delay.as_secs(),
                    retry_count,
                    max_retries,
                    e
                );

                tokio::time::sleep(retry_delay).await;
            }
        }
    }
}
