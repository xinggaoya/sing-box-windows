use serde_json::json;
use std::os::windows::process::CommandExt;
use crate::app::constants::{process, messages};

// 获取流量数据
#[tauri::command]
pub async fn get_traffic_data() -> Result<String, String> {
    // 这里实现获取流量数据的逻辑
    // 由于需要实际的网络接口数据，这里返回一个示例数据
    Ok(json!({
        "upload": 0,
        "download": 0
    })
    .to_string())
}

// 以管理员权限重启
#[tauri::command]
pub fn restart_as_admin() -> Result<(), String> {
    let current_exe =
        std::env::current_exe().map_err(|e| format!("{}: {}", messages::ERR_GET_EXE_PATH_FAILED, e))?;

    let result = std::process::Command::new("powershell")
        .arg("Start-Process")
        .arg(current_exe.to_str().unwrap())
        .arg("-Verb")
        .arg("RunAs")
        .creation_flags(process::CREATE_NO_WINDOW)
        .spawn();

    match result {
        Ok(_) => Ok(()),
        Err(e) => Err(format!("{}: {}", messages::ERR_RESTART_FAILED, e)),
    }
}

// 检查是否有管理员权限
#[tauri::command]
pub fn check_admin() -> bool {
    let result = std::process::Command::new("net")
        .arg("session")
        .creation_flags(process::CREATE_NO_WINDOW)
        .output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
} 