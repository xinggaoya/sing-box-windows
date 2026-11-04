//! Unix（Linux / macOS）提权状态探测。
//!
//! Unix 上内核进程可通过 `sudo` 单独提权（应用本身无需 root），
//! 因此这里的「已提权」只描述**应用自身**是否以 root 运行。

use super::state::ElevationState;

/// 通过真实/有效 UID 判定当前进程是否以 root 运行。
pub fn query_token_elevation() -> ElevationState {
    #[cfg(target_os = "linux")]
    {
        // Linux：检查真实 UID
        if nix::unistd::getuid().is_root() {
            ElevationState::Elevated
        } else {
            ElevationState::NotElevated
        }
    }

    #[cfg(target_os = "macos")]
    {
        // macOS：检查有效 UID（admin 组不等于 root）
        if nix::unistd::geteuid().is_root() {
            ElevationState::Elevated
        } else {
            ElevationState::NotElevated
        }
    }
}

/// Unix 无需回退检测：`getuid`/`geteuid` 不会失败。
pub fn query_fallback_elevation() -> ElevationState {
    ElevationState::Unknown
}

/// Unix 上的提权启动通道：内核可用 `sudo` 单独提权，因此**始终可用**
/// （应用本身无需 root）。
pub fn elevated_launch_available() -> bool {
    true
}
