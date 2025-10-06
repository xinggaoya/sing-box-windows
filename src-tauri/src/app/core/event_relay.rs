use serde::Serialize;
use serde_json::Value;
use std::sync::Arc;
use tauri::{AppHandle, Emitter};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};
use url::Url;
use futures_util::StreamExt;

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

    /// 启动直接事件中继
    pub async fn start(&self) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let url = Url::parse(&self.endpoint)?;
        let (ws_stream, _) = connect_async(url).await?;
        let (mut _write, mut read) = ws_stream.split();

        let app_handle = self.app_handle.clone();
        let event_name = self.event_name.clone();
        let parser = self.parser.clone();
        let _event_name_for_logging = event_name.clone();
        // let _event_name_for_logging2 = event_name_for_logging.clone();
        // let _event_name_for_logging3 = event_name_for_logging.clone();

        // 处理接收到的消息
        let receive_task = tokio::task::spawn(async move {
            let mut message_count = 0u64;
            
            while let Some(msg) = read.next().await {
                match msg {
                    Ok(Message::Text(text)) => {
                        match serde_json::from_str::<Value>(&text) {
                            Ok(data) => {
                                let parsed_data = parser(data);
                                
                                // 直接发送Tauri事件到前端
                                if let Err(e) = app_handle.emit(&event_name, &parsed_data) {
                                    error!("发送{}事件失败: {}", event_name, e);
                                    break;
                                }

                                message_count += 1;
                                
                                // 每100条消息记录一次
                                if message_count % 100 == 0 {
                                    info!("已处理{}条数据", message_count);
                                }
                            }
                            Err(e) => {
                                warn!("解析{}数据失败: {}", event_name, e);
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        info!("连接已关闭");
                        break;
                    }
                    Err(e) => {
                        error!("连接错误: {}", e);
                        break;
                    }
                    _ => {}
                }
            }
        });

        // 发送初始消息以保持连接活跃
        let send_task = tokio::task::spawn(async move {
            // 这里可以发送初始握手消息
        });

        // 等待任务完成
        tokio::select! {
            _ = receive_task => {
                info!("接收任务结束");
            }
            _ = send_task => {
                info!("发送任务结束");
            }
        }

        Ok(())
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

/// 启动事件中继器的便捷函数
pub async fn start_event_relay_with_retry(
    relay: EventDirectRelay<Value>,
    relay_type: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut retry_count = 0;
    let max_retries = 5;
    let retry_delay = std::time::Duration::from_secs(2);

    loop {
        match relay.start().await {
            Ok(_) => {
                info!("{}事件中继器正常结束", relay_type);
                break Ok(());
            }
            Err(e) => {
                retry_count += 1;
                if retry_count >= max_retries {
                    error!("{}事件中继器重试{}次后仍然失败: {}", relay_type, max_retries, e);
                    break Err(e);
                }
                
                warn!("{}事件中继器失败，{}秒后重试 ({}/{}): {}", relay_type, retry_delay.as_secs(), retry_count, max_retries, e);
                tokio::time::sleep(retry_delay).await;
            }
        }
    }
}