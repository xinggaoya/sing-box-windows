use super::*;

#[test]
fn test_kernel_status_payload_running() {
    let payload = KernelStatusPayload::running();
    assert!(payload.process_running);
    assert!(payload.api_ready);
    assert!(payload.websocket_ready);
    assert!(payload.readiness.relay_ready);
}

#[test]
fn test_kernel_status_payload_stopped() {
    let payload = KernelStatusPayload::stopped();
    assert!(!payload.process_running);
    assert!(!payload.api_ready);
    assert!(!payload.websocket_ready);
    assert!(!payload.readiness.process_alive);
}

#[test]
fn test_kernel_status_payload_to_json() {
    let payload = KernelStatusPayload::new(
        true,
        false,
        true,
        KernelReadinessSnapshot {
            config_validated: Some(true),
            process_spawned: Some(true),
            process_alive: true,
            api_ready: false,
            relay_ready: true,
        },
        None,
    );
    let json = payload.to_json();
    assert_eq!(json["process_running"], true);
    assert_eq!(json["api_ready"], false);
    assert_eq!(json["websocket_ready"], true);
    assert_eq!(json["readiness"]["process_alive"], true);
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
    assert_eq!(payload["startup_diagnosis"]["kind"], "config_invalid");
    // 兼容老前端字段
    assert_eq!(payload["error"], "内核启动失败");
    assert!(payload["timestamp"].as_u64().is_some());
}

#[test]
fn test_build_kernel_error_payload_should_include_structured_api_http_error_diagnosis() {
    let payload = build_kernel_error_payload(
        "KERNEL_API_HTTP_ERROR",
        "内核 API 返回了错误状态码",
        Some("稳定性检查第4次失败：API状态码 400 Bad Request"),
        Some("kernel.runtime.startup_stability"),
        true,
    );

    assert_eq!(payload["startup_diagnosis"]["kind"], "api_http_error");
    assert_eq!(payload["startup_diagnosis"]["stage"], "readiness");
    assert_eq!(payload["startup_diagnosis"]["http_status"], 400);
}

// ── TUN-05：权限类失败的识别 ───────────────────────────────────────────────

#[test]
fn permission_failure_is_detected_from_os_level_denial() {
    // Windows 上 taskkill / OpenProcess 失败时的典型文案
    assert!(is_permission_failure_detail("终止进程失败: 拒绝访问。\r\n"));
    assert!(is_permission_failure_detail("Access is denied."));
    assert!(is_permission_failure_detail(
        "error: operation not permitted"
    ));
}

#[test]
fn permission_failure_is_detected_from_privilege_requirement() {
    assert!(is_permission_failure_detail(
        "The requested operation requires elevation."
    ));
    assert!(is_permission_failure_detail(
        "create tun device: administrator privileges are required"
    ));
    assert!(is_permission_failure_detail("需要管理员权限"));
}

#[test]
fn permission_failure_is_detected_from_windows_error_code() {
    // ERROR_ACCESS_DENIED
    assert!(is_permission_failure_detail("failed: 0x80070005"));
}

#[test]
fn unrelated_failures_are_not_treated_as_permission_issues() {
    assert!(!is_permission_failure_detail("配置文件不存在"));
    assert!(!is_permission_failure_detail("内核文件不存在"));
    assert!(!is_permission_failure_detail("配置校验失败"));
    assert!(!is_permission_failure_detail(
        "kernel API not ready within stability window"
    ));
    assert!(!is_permission_failure_detail("API status 500"));
}

#[test]
fn permission_detection_is_case_insensitive() {
    assert!(is_permission_failure_detail("ACCESS IS DENIED"));
    assert!(is_permission_failure_detail("access is denied"));
}
