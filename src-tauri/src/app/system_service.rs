use crate::app::constants::{messages, process};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use tracing::{error, info};

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

// 使用计划任务设置开机自启
#[tauri::command]
pub fn set_autostart(enabled: bool) -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("{}: {}", messages::ERR_GET_EXE_PATH_FAILED, e))?;
    let exe_path = current_exe.to_str().unwrap();
    let task_name = "sing-box-windows";

    if enabled {
        // 创建计划任务
        let result = std::process::Command::new("schtasks")
            .arg("/create")
            .arg("/tn")
            .arg(task_name)
            .arg("/tr")
            .arg(format!("\"{}\" --hide", exe_path))
            .arg("/sc")
            .arg("onlogon")
            .arg("/rl")
            .arg("highest") // 以最高权限运行
            .arg("/f") // 强制创建
            .creation_flags(process::CREATE_NO_WINDOW)
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    info!("开机自启已启用");
                    Ok(())
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    error!("启用开机自启失败: {}", error);
                    Err(format!("启用开机自启失败: {}", error))
                }
            }
            Err(e) => {
                error!("启用开机自启失败: {}", e);
                Err(format!("启用开机自启失败: {}", e))
            }
        }
    } else {
        // 删除计划任务
        let result = std::process::Command::new("schtasks")
            .arg("/delete")
            .arg("/tn")
            .arg(task_name)
            .arg("/f") // 强制删除
            .creation_flags(process::CREATE_NO_WINDOW)
            .output();

        match result {
            Ok(output) => {
                if output.status.success() {
                    info!("开机自启已禁用");
                    Ok(())
                } else {
                    let error = String::from_utf8_lossy(&output.stderr);
                    error!("禁用开机自启失败: {}", error);
                    Err(format!("禁用开机自启失败: {}", error))
                }
            }
            Err(e) => {
                error!("禁用开机自启失败: {}", e);
                Err(format!("禁用开机自启失败: {}", e))
            }
        }
    }
}

// 检查开机自启是否已启用
#[tauri::command]
pub fn is_autostart_enabled() -> bool {
    let task_name = "sing-box-windows";
    let result = std::process::Command::new("schtasks")
        .arg("/query")
        .arg("/tn")
        .arg(task_name)
        .arg("/fo")
        .arg("list")
        .creation_flags(process::CREATE_NO_WINDOW)
        .output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
} 