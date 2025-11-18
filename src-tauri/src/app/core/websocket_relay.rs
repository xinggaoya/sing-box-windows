use futures_util::StreamExt;
use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use tauri::Emitter;
use tauri::Window;
use tokio::task;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};
use url::Url;

/// 通用的WebSocket中继器
/// 用于处理不同类型的WebSocket数据流（流量、内存、日志、连接等）
pub struct WebSocketRelay<R> {
    window: Window,
    endpoint: String,
    event_name: String,
    parser: Arc<dyn Fn(Value) -> R + Send + Sync>,
    // API connection details (for future use)
    // api_port: u16,
    // token: String,
}

impl<R: Send + Sync + 'static + Serialize> WebSocketRelay<R> {
    pub fn new<F>(
        window: Window,
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
            window,
            endpoint: format!("ws://127.0.0.1:{}{}?token={}", api_port, endpoint, token),
            event_name: event_name.to_string(),
            parser: Arc::new(parser),
            // api_port,
            // token,
        }
    }

    /// 启动WebSocket中继
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(&self.endpoint)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (_write, mut read) = ws_stream.split();

        let window = self.window.clone();
        let event_name = self.event_name.clone();
        let parser = self.parser.clone();
        let event_name_clone = event_name.clone();

        // 处理接收到的消息
        let receive_task = task::spawn(async move {
            let mut message_count = 0u64;

            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<Value>(&text) {
                            Ok(data) => {
                                let parsed_data = parser(data);

                                // 发送到前端
                                if let Err(e) = window.emit(&event_name_clone, &parsed_data) {
                                    error!("发送{}事件失败: {}", event_name, e);
                                    break;
                                }

                                message_count += 1;

                                // 每100条消息或缓冲区快满时记录一次
                                if message_count % 100 == 0 {
                                    info!("已处理{}条{}数据", message_count, event_name);
                                }
                            }
                            Err(e) => {
                                warn!("解析{}数据失败: {}", event_name, e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("{}连接已关闭", event_name);
                        break;
                    }
                    Err(e) => {
                        error!("{}连接错误: {}", event_name, e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // 发送初始消息以保持连接活跃
        let send_task = task::spawn(async move {
            // 这里可以发送初始握手消息
        });

        // 等待任务完成
        tokio::select! {
            _ = receive_task => {
                info!("{}接收任务结束", "websocket");
            }
            _ = send_task => {
                info!("{}发送任务结束", "websocket");
            }
        }

        Ok(())
    }
}

/// 创建流量数据中继器
pub fn create_traffic_relay(window: Window, api_port: u16, token: String) -> WebSocketRelay<Value> {
    WebSocketRelay::new(
        window,
        "/traffic",
        "traffic-data",
        |data| data,
        api_port,
        token,
    )
}

/// 创建内存数据中继器
pub fn create_memory_relay(window: Window, api_port: u16, token: String) -> WebSocketRelay<Value> {
    WebSocketRelay::new(
        window,
        "/memory",
        "memory-data",
        |data| data,
        api_port,
        token,
    )
}

/// 创建日志中继器
pub fn create_log_relay(window: Window, api_port: u16, token: String) -> WebSocketRelay<Value> {
    WebSocketRelay::new(window, "/logs", "log-data", |data| data, api_port, token)
}

/// 创建连接中继器
pub fn create_connection_relay(
    window: Window,
    api_port: u16,
    token: String,
) -> WebSocketRelay<Value> {
    WebSocketRelay::new(
        window,
        "/connections",
        "connection-data",
        |data| data,
        api_port,
        token,
    )
}

/// 启动通用中继器的便捷函数
pub async fn start_relay_with_retry<R: Send + Sync + 'static + Serialize>(
    _relay: WebSocketRelay<R>,
    _relay_type: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // TODO: 重新实现WebSocket中继
    println!("WebSocket中继功能暂时禁用");
    Ok(())
}
