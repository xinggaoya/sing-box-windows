//! 提权状态探测。
//!
//! 分层：
//! - [`state`]：平台无关的状态模型与合并规则（纯函数，可跨平台单测）；
//! - [`windows`] / [`unix`]：各平台的探测实现。
//!
//! 对外只暴露 [`current_elevation`] 与 [`is_elevated`]，
//! 调用方不需要知道平台差异。

pub mod state;

#[cfg(unix)]
pub mod unix;
#[cfg(target_os = "windows")]
pub mod windows;

/// 非 unix / 非 windows 平台的兜底（保持与旧 `check_admin` 一致的可编译性）。
#[cfg(not(any(unix, target_os = "windows")))]
mod fallback {
    use super::state::ElevationState;

    pub fn query_token_elevation() -> ElevationState {
        ElevationState::Unknown
    }

    pub fn query_fallback_elevation() -> ElevationState {
        ElevationState::Unknown
    }

    pub fn elevated_launch_available() -> bool {
        false
    }
}

pub use state::{merge_elevation, ElevationState};

#[cfg(not(any(unix, target_os = "windows")))]
use self::fallback as platform;
#[cfg(unix)]
use self::unix as platform;
#[cfg(target_os = "windows")]
use self::windows as platform;

/// 当前进程的提权状态。
///
/// 优先使用平台提供的**权威检测**（Windows: 进程 token；Unix: UID）；
/// 仅当权威检测无法判定时才执行回退检测，且回退检测结果只在权威检测
/// 缺失时生效 —— 见 [`merge_elevation`]。
pub fn current_elevation() -> ElevationState {
    let token = platform::query_token_elevation();

    // 回退检测可能需要拉起子进程，只在权威检测不可用时才执行。
    let fallback = if token == ElevationState::Unknown {
        platform::query_fallback_elevation()
    } else {
        ElevationState::Unknown
    };

    merge_elevation(token, fallback)
}

/// 当前进程是否已提权。
///
/// 无法判定时返回 `false`（安全侧默认：不得据此跳过必要的提权流程）。
pub fn is_elevated() -> bool {
    current_elevation().is_elevated()
}

/// 当前环境能否以 TUN 所需的权限运行内核。
///
/// - **Unix**：内核可通过 `sudo` 单独提权，应用本身无需 root → 恒为 `true`；
/// - **Windows**：TUN 需要 Administrator token，因此要求「进程已提权」
///   或「已就绪的提权启动通道可用」。
///
/// 供 `proxy_service::platform_policy` 判定运行态是否能真的启用 TUN。
pub fn can_run_tun_kernel() -> bool {
    // Unix：内核可用 `sudo` 单独提权 → 提权通道始终可用（应用本身无需 root）
    // Windows：需要进程已提权，或已就绪的提权启动通道可用
    is_elevated() || platform::elevated_launch_available()
}
