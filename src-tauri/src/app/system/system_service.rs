use crate::app::constants::{messages, process};
use std::os::windows::process::CommandExt;
use std::thread;
use std::time::Duration;
use tauri::Manager;

// 以管理员权限重启
#[tauri::command]
pub fn restart_as_admin(app_handle: tauri::AppHandle) -> Result<(), String> {
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("{}: {}", messages::ERR_GET_EXE_PATH_FAILED, e))?;

    // 创建批处理脚本来启动程序并关闭当前实例
    let temp_dir = std::env::temp_dir();
    let batch_path = temp_dir.join("restart_elevated.bat");
    
    // 获取当前进程ID
    let current_pid = std::process::id();
    
    // 创建批处理文件内容
    let batch_content = format!(
        "@echo off\n\
        timeout /t 1 /nobreak >nul\n\
        start \"\" /b \"{}\" \n\
        timeout /t 2 /nobreak >nul\n\
        taskkill /f /pid {} /t\n\
        del \"%~f0\"\n",
        current_exe.to_str().unwrap().replace("\\", "\\\\"),
        current_pid
    );
    
    // 写入批处理文件
    std::fs::write(&batch_path, batch_content)
        .map_err(|e| format!("无法创建重启脚本: {}", e))?;
    
    // 以管理员权限运行批处理文件
    let result = std::process::Command::new("powershell")
        .arg("Start-Process")
        .arg(batch_path.to_str().unwrap())
        .arg("-Verb")
        .arg("RunAs")
        .creation_flags(process::CREATE_NO_WINDOW)
        .spawn();
    
    match result {
        Ok(_) => {
            // 由于批处理文件会自动关闭当前进程，这里不需要额外操作
            Ok(())
        },
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

// 新增一个退出程序的命令
#[tauri::command]
pub fn exit_application(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}
