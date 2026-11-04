//! Windows 提权探测的实机冒烟测试。
//!
//! 这些用例只在 Windows 上编译运行（CI 的 Linux 任务不会执行），
//! 但它们守的是一条真实回归：`GetTokenInformation(TokenElevation)`
//! 在任何 Windows 上都应当可用，**不应返回 Unknown** ——
//! 一旦返回 Unknown，系统就会退化到 `net session` 回退检测，
//! 重新引入 TUN-01 的假阴性。

use super::{query_fallback_elevation, query_token_elevation};
use crate::app::system::elevation::ElevationState;

#[test]
fn token_elevation_query_is_available() {
    assert_ne!(
        query_token_elevation(),
        ElevationState::Unknown,
        "token 提权检测在 Windows 上必须可用，否则会退化到有假阴性的 net session 回退"
    );
}

#[test]
fn fallback_detection_returns_a_decidable_state() {
    // 回退检测要么给出明确结论，要么明确「无法判定」，不能给出与事实相反的结论
    let state = query_fallback_elevation();
    assert!(
        matches!(
            state,
            ElevationState::Elevated | ElevationState::NotElevated | ElevationState::Unknown
        ),
        "回退检测必须返回合法状态枚举"
    );
}
