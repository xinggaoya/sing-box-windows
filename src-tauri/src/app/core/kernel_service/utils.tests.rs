use super::*;

#[test]
fn test_kernel_status_payload_running() {
    let payload = KernelStatusPayload::running();
    assert!(payload.process_running);
    assert!(payload.api_ready);
    assert!(payload.websocket_ready);
}

#[test]
fn test_kernel_status_payload_stopped() {
    let payload = KernelStatusPayload::stopped();
    assert!(!payload.process_running);
    assert!(!payload.api_ready);
    assert!(!payload.websocket_ready);
}

#[test]
fn test_kernel_status_payload_to_json() {
    let payload = KernelStatusPayload::new(true, false, true);
    let json = payload.to_json();
    assert_eq!(json["process_running"], true);
    assert_eq!(json["api_ready"], false);
    assert_eq!(json["websocket_ready"], true);
}

#[test]
fn test_kernel_error_payload_contains_compat_and_structured_fields() {
    let payload = build_kernel_error_payload(
        "KERNEL_START_FAILED",
        "内核启动失败",
        Some("配置校验失败"),
        Some("kernel.runtime.start"),
        true,
    );

    assert_eq!(payload["code"], "KERNEL_START_FAILED");
    assert_eq!(payload["message"], "内核启动失败");
    assert_eq!(payload["details"], "配置校验失败");
    assert_eq!(payload["source"], "kernel.runtime.start");
    assert_eq!(payload["recoverable"], true);
    // 兼容老前端字段
    assert_eq!(payload["error"], "内核启动失败");
    assert!(payload["timestamp"].as_u64().is_some());
}
