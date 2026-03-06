use super::*;

#[test]
fn resolve_target_config_path_should_rebase_absolute_path() {
    let path = if cfg!(target_os = "windows") {
        r"C:\Users\legacy-user\AppData\Local\sing-box-windows\sing-box\configs\legacy.json"
            .to_string()
    } else {
        "/tmp/legacy-user/sing-box-windows/sing-box/configs/legacy.json".to_string()
    };

    let resolved = resolve_target_config_path(None, Some(path)).expect("should resolve path");
    assert!(resolved.starts_with(managed_config_dir()));
    assert_eq!(
        resolved.file_name().and_then(|v| v.to_str()),
        Some("legacy.json")
    );
}

#[test]
fn resolve_target_config_path_should_rebase_relative_path() {
    let resolved = resolve_target_config_path(None, Some("configs/original.json".to_string()))
        .expect("should resolve path");

    assert!(resolved.starts_with(managed_config_dir()));
    assert_eq!(
        resolved.file_name().and_then(|v| v.to_str()),
        Some("original.json")
    );
}
