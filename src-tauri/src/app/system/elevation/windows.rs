//! Windows 提权状态探测。
//!
//! 两个信号，权威性不同：
//! - [`query_token_elevation`]：直接读进程 token 的 `TokenElevation`，**权威、无外部依赖**；
//! - [`query_fallback_elevation`]：`net session`，依赖 Server 服务（lanmanServer）可用性，
//!   服务被禁用时会**假阴性**，因此只作兜底。

use super::state::ElevationState;
use std::os::windows::process::CommandExt;

/// 通过进程 token 判定当前进程是否已提权。
///
/// 直接调用 `GetTokenInformation(TokenElevation)`，不依赖任何系统服务或外部命令，
/// 因此不受「Server 服务被禁用」「精简镜像缺少 `net`」等环境因素影响。
pub fn query_token_elevation() -> ElevationState {
    use windows::Win32::Foundation::{CloseHandle, HANDLE};
    use windows::Win32::Security::{
        GetTokenInformation, TokenElevation, TOKEN_ELEVATION, TOKEN_QUERY,
    };
    use windows::Win32::System::Threading::{GetCurrentProcess, OpenProcessToken};

    // SAFETY:
    // - `OpenProcessToken` 传入 `GetCurrentProcess()` 的伪句柄，仅申请 TOKEN_QUERY 权限，
    //   不会修改进程状态；失败时返回错误，函数按 Unknown 处理。
    // - `GetTokenInformation` 的输出缓冲区指向栈上的 `TOKEN_ELEVATION`，长度与类型匹配。
    // - token 句柄在读取后立即 `CloseHandle`，无泄漏。
    unsafe {
        let mut token = HANDLE::default();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_QUERY, &mut token).is_err() {
            return ElevationState::Unknown;
        }

        let mut elevation = TOKEN_ELEVATION::default();
        let mut returned = 0u32;
        let queried = GetTokenInformation(
            token,
            TokenElevation,
            Some(&mut elevation as *mut TOKEN_ELEVATION as *mut core::ffi::c_void),
            std::mem::size_of::<TOKEN_ELEVATION>() as u32,
            &mut returned,
        );
        let _ = CloseHandle(token);

        if queried.is_err() {
            return ElevationState::Unknown;
        }

        if elevation.TokenIsElevated != 0 {
            ElevationState::Elevated
        } else {
            ElevationState::NotElevated
        }
    }
}

/// 回退检测：`net session`。
///
/// 成功说明具备管理员权限。该命令依赖 Server 服务，服务被禁用时会失败 →
/// 假阴性。仅在 [`query_token_elevation`] 不可用时使用。
pub fn query_fallback_elevation() -> ElevationState {
    let result = std::process::Command::new("net")
        .arg("session")
        .creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW)
        .output();

    match result {
        Ok(output) if output.status.success() => ElevationState::Elevated,
        Ok(_) => ElevationState::NotElevated,
        Err(_) => ElevationState::Unknown,
    }
}

/// 提权启动通道是否已就绪。
///
/// TUN-11 落地前恒为 `false` —— 此时只能靠整个应用以管理员身份运行，
/// 内核作为普通子进程继承应用 token。
pub fn elevated_launch_available() -> bool {
    false
}

#[cfg(test)]
#[path = "windows.tests.rs"]
mod tests;
