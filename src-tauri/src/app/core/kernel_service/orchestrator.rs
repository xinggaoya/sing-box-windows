//! 内核生命周期编排器
//!
//! 通过单队列串行执行变更型操作，避免 start/stop/restart 并发竞态。

use futures::future::BoxFuture;
use serde_json::{json, Value};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use tauri::{AppHandle, Emitter};
use tokio::sync::{mpsc, oneshot, OnceCell};
use tracing::{error, info, warn};

type OperationResult = Result<Value, String>;
type OperationFuture = BoxFuture<'static, OperationResult>;

struct OperationRequest {
    op_id: String,
    op_name: &'static str,
    app_handle: AppHandle,
    task: OperationFuture,
    response_tx: oneshot::Sender<OperationResult>,
}

const QUEUE_CAPACITY: usize = 32;

static OP_COUNTER: AtomicU64 = AtomicU64::new(1);
static STATE_VERSION: AtomicU64 = AtomicU64::new(0);
static ORCHESTRATOR_TX: OnceCell<mpsc::Sender<OperationRequest>> = OnceCell::const_new();

fn now_millis() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn next_op_id() -> String {
    let ts = now_millis();
    let seq = OP_COUNTER.fetch_add(1, Ordering::SeqCst);
    format!("op-{}-{}", ts, seq)
}

fn bump_state_version() -> u64 {
    STATE_VERSION.fetch_add(1, Ordering::SeqCst) + 1
}

pub fn current_state_version() -> u64 {
    STATE_VERSION.load(Ordering::SeqCst)
}

fn with_operation_meta(
    mut value: Value,
    op_id: &str,
    op_name: &'static str,
    state_version: u64,
) -> Value {
    if let Some(obj) = value.as_object_mut() {
        obj.insert("op_id".to_string(), json!(op_id));
        obj.insert("operation".to_string(), json!(op_name));
        obj.insert("state_version".to_string(), json!(state_version));
        value
    } else {
        json!({
            "success": true,
            "data": value,
            "op_id": op_id,
            "operation": op_name,
            "state_version": state_version
        })
    }
}

async fn emit_operation_event(
    app_handle: &AppHandle,
    event: &str,
    op_id: &str,
    op_name: &'static str,
    state_version: u64,
    error: Option<&str>,
) {
    let payload = json!({
        "op_id": op_id,
        "operation": op_name,
        "state_version": state_version,
        "timestamp": now_millis(),
        "error": error
    });
    let _ = app_handle.emit(event, payload);
}

async fn run_worker(mut rx: mpsc::Receiver<OperationRequest>) {
    while let Some(req) = rx.recv().await {
        let state_version = bump_state_version();
        let queued_at = now_millis();

        emit_operation_event(
            &req.app_handle,
            "kernel-operation-started",
            &req.op_id,
            req.op_name,
            state_version,
            None,
        )
        .await;

        info!(
            "内核编排器开始执行: {} (op_id={}, state_version={}, queued_at={})",
            req.op_name, req.op_id, state_version, queued_at
        );

        let result = req.task.await;
        let final_result = match result {
            Ok(value) => {
                emit_operation_event(
                    &req.app_handle,
                    "kernel-operation-finished",
                    &req.op_id,
                    req.op_name,
                    state_version,
                    None,
                )
                .await;
                Ok(with_operation_meta(
                    value,
                    &req.op_id,
                    req.op_name,
                    state_version,
                ))
            }
            Err(err) => {
                warn!(
                    "内核编排器执行失败: {} (op_id={}, err={})",
                    req.op_name, req.op_id, err
                );
                emit_operation_event(
                    &req.app_handle,
                    "kernel-operation-failed",
                    &req.op_id,
                    req.op_name,
                    state_version,
                    Some(&err),
                )
                .await;
                Err(err)
            }
        };

        if req.response_tx.send(final_result).is_err() {
            warn!("内核编排器响应发送失败: {}", req.op_id);
        }
    }
}

async fn get_sender() -> mpsc::Sender<OperationRequest> {
    ORCHESTRATOR_TX
        .get_or_init(|| async {
            let (tx, rx) = mpsc::channel::<OperationRequest>(QUEUE_CAPACITY);
            tokio::spawn(run_worker(rx));
            tx
        })
        .await
        .clone()
}

/// 串行执行内核生命周期操作。
pub async fn execute_kernel_operation(
    app_handle: AppHandle,
    op_name: &'static str,
    task: OperationFuture,
) -> OperationResult {
    let tx = get_sender().await;
    let op_id = next_op_id();
    let (resp_tx, resp_rx) = oneshot::channel();

    let request = OperationRequest {
        op_id: op_id.clone(),
        op_name,
        app_handle,
        task,
        response_tx: resp_tx,
    };

    tx.send(request)
        .await
        .map_err(|e| format!("提交编排任务失败: {}", e))?;

    match resp_rx.await {
        Ok(result) => result,
        Err(e) => {
            error!("内核编排器响应异常 (op_id={}): {}", op_id, e);
            Err(format!("编排任务异常中断: {}", e))
        }
    }
}
