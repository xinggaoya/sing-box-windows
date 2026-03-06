use super::paths;

#[test]
fn get_config_dir_should_point_to_sing_box_subdir() {
    let config_dir = paths::get_config_dir();

    assert!(config_dir.ends_with("sing-box"));
    assert!(config_dir.parent().is_some());
}

#[test]
fn get_kernel_path_should_use_platform_specific_file_name() {
    let kernel_path = paths::get_kernel_path();

    #[cfg(target_os = "windows")]
    assert!(kernel_path.ends_with(r"sing-box\sing-box.exe"));

    #[cfg(not(target_os = "windows"))]
    assert!(kernel_path.ends_with("sing-box/sing-box"));
}

#[test]
fn get_kernel_work_dir_should_match_config_dir() {
    assert_eq!(paths::get_kernel_work_dir(), paths::get_config_dir());
}
