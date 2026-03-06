use super::*;

#[test]
fn get_work_dir_sync_should_return_existing_sing_box_windows_dir() {
    let work_dir = get_work_dir_sync();
    let work_dir_path = PathBuf::from(&work_dir);

    assert!(work_dir_path.exists());
    assert!(work_dir_path.ends_with("sing-box-windows"));
}

#[tokio::test]
async fn get_work_dir_should_return_existing_sing_box_windows_dir() {
    let work_dir = get_work_dir().await;
    let work_dir_path = PathBuf::from(&work_dir);

    assert!(work_dir_path.exists());
    assert!(work_dir_path.ends_with("sing-box-windows"));
}

#[test]
fn get_service_path_should_point_to_expected_binary_name() {
    let service_path = get_service_path();

    #[cfg(target_os = "windows")]
    assert!(service_path.ends_with(r"src\config\sing-box-service.exe"));

    #[cfg(not(target_os = "windows"))]
    assert!(service_path.ends_with("src/config/sing-box-service"));
}
