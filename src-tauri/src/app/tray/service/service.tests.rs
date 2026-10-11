//! 托盘运行态快照合并的单元测试
//!
//! 拆分 service.rs 时随实现一同迁出（保持「测试与实现分离为 *.tests.rs」的约定）。

use super::runtime_state::apply_backend_runtime_snapshot;
use crate::app::storage::state_model::AppConfig;
use crate::app::tray::model::TrayCloseBehavior;
use crate::app::tray::state::TrayRuntimeState;

fn sample_app_config() -> AppConfig {
    AppConfig {
        system_proxy_enabled: true,
        tun_enabled: false,
        tray_close_behavior: "lightweight".to_string(),
        ..AppConfig::default()
    }
}

#[test]
fn backend_runtime_sync_updates_proxy_related_fields() {
    let mut state = TrayRuntimeState::default();
    let config = sample_app_config();

    let changed = apply_backend_runtime_snapshot(&mut state, &config, true);

    assert!(changed);
    assert!(state.system_proxy_enabled);
    assert!(!state.tun_enabled);
    assert!(state.kernel_running);
    assert_eq!(state.close_behavior, TrayCloseBehavior::Lightweight);
}

#[test]
fn backend_runtime_sync_preserves_window_lifecycle_fields() {
    let mut state = TrayRuntimeState {
        window_visible: false,
        last_visible_route: "/settings".to_string(),
        pending_restore_route: Some("/settings".to_string()),
        keep_alive_without_windows: true,
        allow_app_exit: true,
        ..TrayRuntimeState::default()
    };
    let config = AppConfig {
        system_proxy_enabled: false,
        tun_enabled: true,
        tray_close_behavior: "hide".to_string(),
        ..AppConfig::default()
    };

    let changed = apply_backend_runtime_snapshot(&mut state, &config, false);

    assert!(changed);
    assert!(!state.window_visible);
    assert_eq!(state.last_visible_route, "/settings");
    assert_eq!(state.pending_restore_route.as_deref(), Some("/settings"));
    assert!(state.keep_alive_without_windows);
    assert!(state.allow_app_exit);
    assert!(state.tun_enabled);
    assert_eq!(state.close_behavior, TrayCloseBehavior::Hide);
}
