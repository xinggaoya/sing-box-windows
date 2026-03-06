use super::*;

#[test]
fn test_kernel_state_transitions() {
    let manager = KernelStateManager::new();

    assert_eq!(manager.get_state(), KernelState::Stopped);
    assert!(manager.get_state().can_start());

    assert!(manager.try_transition_to_starting());
    assert_eq!(manager.get_state(), KernelState::Starting);
    assert!(!manager.get_state().can_start());

    manager.mark_running(12081);
    assert_eq!(manager.get_state(), KernelState::Running);
    assert!(manager.get_state().is_running());

    assert!(manager.try_transition_to_stopping());
    assert_eq!(manager.get_state(), KernelState::Stopping);

    manager.mark_stopped();
    assert_eq!(manager.get_state(), KernelState::Stopped);
}

#[test]
fn test_kernel_runtime_config_merge() {
    let mut base = KernelRuntimeConfig {
        api_port: Some(12081),
        proxy_port: Some(12080),
        ..Default::default()
    };

    let overrides = KernelRuntimeConfig {
        api_port: Some(9090),
        prefer_ipv6: Some(true),
        ..Default::default()
    };

    base.merge(&overrides);

    assert_eq!(base.api_port, Some(9090));
    assert_eq!(base.proxy_port, Some(12080)); // 未被覆盖
    assert_eq!(base.prefer_ipv6, Some(true));
}
