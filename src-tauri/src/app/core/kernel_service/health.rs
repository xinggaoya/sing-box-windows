//! 内核存活状态的**唯一**判定入口
//!
//! ## 为什么需要这个模块
//!
//! 修复前「内核是否在跑」有三套彼此独立的实现，且结论可能互相矛盾：
//!
//! | 实现 | 位置 | 依据 |
//! |------|------|------|
//! | `status::is_kernel_running` | ProcessManager 句柄 → 平台进程扫描 | 进程是否存在 |
//! | `kernel_get_status_enhanced_v2` | 纯 TCP 端口探测 | 服务是否可达 |
//! | `KERNEL_STATE.readiness.process_alive` | 由调用方各自写入 | 谁写谁说了算 |
//!
//! 前端看的是第二套，托盘与守护用的是第一套，于是会出现
//! 「内核明明在跑、流量也正常，界面却显示已停止」这类无法定位的问题 ——
//! 原注释也承认了这点（"用户实测 is_kernel_running 因 ProcessManager 状态问题返回 false"）。
//!
//! ## 判定优先级
//!
//! `api_port` 能连 > 托管进程句柄 > 平台进程扫描。理由：端口可连才说明内核真的
//! 在提供服务；进程存在却端口不通（例如还在启动、或监听已被内核主动关闭）不该
//! 报成"已停止"。反之端口不通但进程在（启动中）也应如实报告进程存活。
//!
//! ## 可测试性
//!
//! 端口探测抽象为 [`PortProbe`] trait，生产实现走 `spawn_blocking`（TCP connect 是
//! 阻塞调用，直接放在 async 里会占用运行时工作线程），测试可注入假实现，
//! 从而能在没有真实 sing-box 进程的环境里覆盖优先级逻辑。

use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::time::Duration;

/// 单次 TCP 连接探测的超时。
const PROBE_TIMEOUT: Duration = Duration::from_millis(500);

/// 端口探测抽象：生产用 TCP 实现，测试可注入桩。
pub trait PortProbe: Send + Sync {
    /// 端口能否建立 TCP 连接。
    fn is_port_open(&self, port: u16, timeout: Duration) -> bool;
}

/// 生产实现：同步 `TcpStream::connect_timeout`。
///
/// 调用方负责放进 `spawn_blocking`（见 [`tcp_probe_within_blocking`]）。
#[derive(Debug, Default, Clone, Copy)]
pub struct TcpPortProbe;

impl PortProbe for TcpPortProbe {
    fn is_port_open(&self, port: u16, timeout: Duration) -> bool {
        let host = "127.0.0.1";
        let Ok(mut addrs) = (host, port).to_socket_addrs() else {
            return false;
        };
        addrs
            .by_ref()
            .find_map(|addr: SocketAddr| {
                let stream = TcpStream::connect_timeout(&addr, timeout).ok()?;
                // 主动关闭，避免探测连接在内核侧堆积成 TIME_WAIT
                let _ = stream.shutdown(std::net::Shutdown::Both);
                Some(stream)
            })
            .is_some()
    }
}

/// 在阻塞线程池里跑一次端口探测。
///
/// TCP connect 是阻塞 syscall；守护循环与状态查询都在 async 上下文里调用，
/// 不隔离会占住 tokio 工作线程。
pub async fn tcp_probe_within_blocking<P: PortProbe + 'static>(
    probe: P,
    port: u16,
    timeout: Duration,
) -> bool {
    match tokio::task::spawn_blocking(move || probe.is_port_open(port, timeout)).await {
        Ok(open) => open,
        Err(error) => {
            tracing::warn!("端口探测任务异常终止 (port={}): {}", port, error);
            false
        }
    }
}

/// 探测结果的来源，便于诊断"为什么认为是这个状态"。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HealthSource {
    /// gRPC API 端口可连：内核确实在提供服务
    ApiPortOpen,
    /// 代理入站端口可连（api_port 未开时的次要证据）
    ProxyPortOpen,
    /// 端口不通但托管进程句柄仍在：大概率正在启动
    ManagedProcess,
    /// 端口不通且无句柄，但系统里能扫到内核进程：外部启动的实例
    PlatformScan,
    /// 三种依据都否定
    None,
}

impl HealthSource {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::ApiPortOpen => "api-port",
            Self::ProxyPortOpen => "proxy-port",
            Self::ManagedProcess => "managed-process",
            Self::PlatformScan => "platform-scan",
            Self::None => "none",
        }
    }
}

/// 统一的内核健康快照。
#[derive(Debug, Clone, Copy)]
pub struct KernelHealth {
    /// 内核是否在运行（进程层面）
    pub process_running: bool,
    /// gRPC API 端口是否可连
    pub api_ready: bool,
    /// 代理入站端口是否可连
    pub proxy_port_open: bool,
    /// 上述结论的依据
    pub source: HealthSource,
}

impl KernelHealth {
    /// 进程层面未运行 —— 用于 `is_kernel_running` 这类只看"在不在跑"的场景。
    pub fn is_running(&self) -> bool {
        self.process_running
    }

    /// 服务层面可用 —— 前端展示"已就绪"应以此为准。
    pub fn is_service_ready(&self) -> bool {
        self.api_ready
    }
}

/// 统一探测：端口 > 托管句柄 > 平台扫描。
///
/// `managed_running` / `platform_scanned` 由调用方注入，以便：
/// - 生产路径复用 `PROCESS_MANAGER.is_running()` 与 `platform::is_process_running()`；
/// - 测试完全绕开真实进程。
pub async fn probe_kernel<P: PortProbe + Clone + 'static>(
    api_port: u16,
    proxy_port: u16,
    probe: P,
    managed_running: bool,
    platform_scanned: bool,
) -> KernelHealth {
    // 两个端口并发探测：串行会让总耗时翻倍（守护循环 8s 一次，仍应尽量短）
    let (api_ready, proxy_port_open) = tokio::join!(
        tcp_probe_within_blocking(probe.clone(), api_port, PROBE_TIMEOUT),
        tcp_probe_within_blocking(probe, proxy_port, PROBE_TIMEOUT),
    );

    let (process_running, source) = if api_ready {
        (true, HealthSource::ApiPortOpen)
    } else if proxy_port_open {
        // 代理端口通而 API 端口不通：进程在跑，但 API 可能还没起来（启动中）
        (true, HealthSource::ProxyPortOpen)
    } else if managed_running {
        (true, HealthSource::ManagedProcess)
    } else if platform_scanned {
        (true, HealthSource::PlatformScan)
    } else {
        (false, HealthSource::None)
    };

    KernelHealth {
        process_running,
        api_ready,
        proxy_port_open,
        source,
    }
}

#[cfg(test)]
#[path = "health.tests.rs"]
mod tests;
