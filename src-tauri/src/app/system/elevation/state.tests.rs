//! 提权状态合并逻辑的单元测试。
//!
//! 背景：Windows 上判定「当前进程是否已提权」有两类信号：
//! - **token 检测**（`GetTokenInformation(TokenElevation)`）：直接读进程 token，权威、无副作用；
//! - **回退检测**（`net session`）：依赖 Server 服务可用性，会假阴性。
//!
//! 合并规则必须是「token 优先、未知才回退」——否则 token 已明确说提权，
//! 仍会被 `net session` 的假阴性覆盖成「未提权」，进而触发多余的 UAC 重启
//! 与「静默关闭 TUN」副作用。

use super::{merge_elevation, ElevationState};

#[test]
fn token_elevated_wins_over_fallback_not_elevated() {
    // 回归：token 明确提权时，不能被回退检测的假阴性覆盖
    assert_eq!(
        merge_elevation(ElevationState::Elevated, ElevationState::NotElevated),
        ElevationState::Elevated
    );
}

#[test]
fn token_not_elevated_wins_over_fallback_elevated() {
    // token 明确未提权时，回退检测的「提权」结论不可信（同样可能是环境异常）
    assert_eq!(
        merge_elevation(ElevationState::NotElevated, ElevationState::Elevated),
        ElevationState::NotElevated
    );
}

#[test]
fn unknown_token_falls_back_to_fallback_result() {
    assert_eq!(
        merge_elevation(ElevationState::Unknown, ElevationState::Elevated),
        ElevationState::Elevated
    );
    assert_eq!(
        merge_elevation(ElevationState::Unknown, ElevationState::NotElevated),
        ElevationState::NotElevated
    );
}

#[test]
fn unknown_token_and_unknown_fallback_stays_unknown() {
    assert_eq!(
        merge_elevation(ElevationState::Unknown, ElevationState::Unknown),
        ElevationState::Unknown
    );
}

#[test]
fn unknown_is_not_treated_as_elevated() {
    // 安全侧默认：无法判定时绝不能当作「已提权」，否则会跳过必要的提权流程
    assert_ne!(
        merge_elevation(ElevationState::Unknown, ElevationState::Unknown),
        ElevationState::Elevated
    );
    assert_ne!(
        merge_elevation(ElevationState::Unknown, ElevationState::NotElevated),
        ElevationState::Elevated
    );
}

#[test]
fn is_elevated_only_reports_elevated_state() {
    assert!(ElevationState::Elevated.is_elevated());
    assert!(!ElevationState::NotElevated.is_elevated());
    assert!(!ElevationState::Unknown.is_elevated());
}
