//! `health` 模块单元测试。
//!
//! 这段逻辑原先散落三处、结论互相矛盾（前端看端口探测、托盘看进程句柄），
//! 出现过"内核在跑但界面显示已停止"。端口探测已抽象为 trait，
//! 因此可以在没有真实 sing-box 进程的环境里完整覆盖优先级规则。

use super::{probe_kernel, HealthSource, KernelHealth, PortProbe};

/// 桩探测：按端口号返回预设结果
#[derive(Clone)]
struct StubProbe {
    open_ports: Vec<u16>,
}

impl PortProbe for StubProbe {
    fn is_port_open(&self, port: u16, _timeout: std::time::Duration) -> bool {
        self.open_ports.contains(&port)
    }
}

/// 全端口打开
fn probe_open_all() -> StubProbe {
    StubProbe {
        open_ports: vec![12081, 12080],
    }
}

/// 全端口关闭
fn probe_closed() -> StubProbe {
    StubProbe { open_ports: vec![] }
}

/// 只开 API 端口
fn probe_api_only() -> StubProbe {
    StubProbe {
        open_ports: vec![12081],
    }
}

async fn run(probe: StubProbe, managed: bool, platform: bool) -> KernelHealth {
    probe_kernel(12081, 12080, probe, managed, platform).await
}

#[tokio::test]
async fn api_port_open_wins() {
    let health = run(probe_api_only(), false, false).await;
    assert!(health.is_running());
    assert!(health.is_service_ready());
    assert_eq!(health.source, HealthSource::ApiPortOpen);
}

#[tokio::test]
async fn proxy_port_open_counts_as_running_but_not_service_ready() {
    // 仅代理端口通：进程在跑但 API 还没起来（启动中）
    let health = run(
        StubProbe {
            open_ports: vec![12080],
        },
        false,
        false,
    )
    .await;
    assert!(health.is_running(), "代理端口可连即认为进程在跑");
    assert!(!health.is_service_ready(), "API 未就绪");
    assert_eq!(health.source, HealthSource::ProxyPortOpen);
}

#[tokio::test]
async fn managed_process_used_when_ports_closed() {
    // 端口不通但句柄在：启动中，不应报成"已停止"
    let health = run(probe_closed(), true, false).await;
    assert!(health.is_running());
    assert!(!health.is_service_ready());
    assert_eq!(health.source, HealthSource::ManagedProcess);
}

#[tokio::test]
async fn platform_scan_used_as_last_resort() {
    let health = run(probe_closed(), false, true).await;
    assert!(health.is_running());
    assert_eq!(health.source, HealthSource::PlatformScan);
}

#[tokio::test]
async fn all_negative_reports_not_running() {
    let health = run(probe_closed(), false, false).await;
    assert!(!health.is_running());
    assert!(!health.is_service_ready());
    assert_eq!(health.source, HealthSource::None);
}

#[tokio::test]
async fn priority_is_strict_even_when_lower_signals_disagree() {
    // 三种依据同时为真时，端口必须赢
    let health = run(probe_open_all(), true, true).await;
    assert_eq!(health.source, HealthSource::ApiPortOpen);

    // 端口不通时，托管句柄优先于平台扫描
    let health = run(probe_closed(), true, true).await;
    assert_eq!(health.source, HealthSource::ManagedProcess);
}

#[tokio::test]
async fn captures_both_port_states() {
    let health = run(probe_open_all(), false, false).await;
    assert!(health.api_ready);
    assert!(health.proxy_port_open);

    let health = run(probe_api_only(), false, false).await;
    assert!(health.api_ready);
    assert!(!health.proxy_port_open);
}

#[tokio::test]
async fn health_source_has_stable_string_representation() {
    // 该字符串会进日志/事件载荷，改动会让历史日志无法比对
    assert_eq!(HealthSource::ApiPortOpen.as_str(), "api-port");
    assert_eq!(HealthSource::ProxyPortOpen.as_str(), "proxy-port");
    assert_eq!(HealthSource::ManagedProcess.as_str(), "managed-process");
    assert_eq!(HealthSource::PlatformScan.as_str(), "platform-scan");
    assert_eq!(HealthSource::None.as_str(), "none");
}

#[tokio::test]
async fn uses_configured_ports_not_hardcoded() {
    #[derive(Clone)]
    struct RecorderProbe;

    impl PortProbe for RecorderProbe {
        fn is_port_open(&self, port: u16, _timeout: std::time::Duration) -> bool {
            // 只认自定义端口，确保调用方传入的端口真的被使用
            port == 34567
        }
    }

    let health = probe_kernel(34567, 34568, RecorderProbe, false, false).await;
    assert!(health.api_ready, "自定义 api_port 应被探测");
    assert!(!health.proxy_port_open);
}

#[tokio::test]
async fn two_ports_probed_concurrently_within_timeout() {
    // 桩实现不 sleep，因此总耗时应远小于 2 * 500ms；这里只断言结果正确，
    // 避免在慢 CI 上产生时间相关的脆弱断言。
    let health = run(probe_open_all(), false, false).await;
    assert!(health.api_ready && health.proxy_port_open);
}
