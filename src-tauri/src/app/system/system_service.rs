use crate::app::constants::{messages, process};
use std::os::windows::process::CommandExt;
use tauri::Manager;

// 以管理员权限重启
#[tauri::command]
pub fn restart_as_admin(app_handle: tauri::AppHandle) -> Result<(), String> {
    // 检查当前是否已经有管理员权限
    if check_admin() {
        return Ok(());
    }

    // 获取当前可执行文件路径
    let current_exe = std::env::current_exe()
        .map_err(|e| format!("{}: {}", messages::ERR_GET_EXE_PATH_FAILED, e))?;

    // 确保文件存在
    if !current_exe.exists() {
        return Err(format!("找不到程序可执行文件: {}", current_exe.display()));
    }

    // 创建VBS脚本实现UAC提权
    let temp_dir = std::env::temp_dir();
    let vbs_path = temp_dir.join("elevate.vbs");

    // 确保路径正确格式化
    let exe_path = current_exe.to_string_lossy().replace("\\", "\\\\");

    let vbs_content = format!(
        "Set UAC = CreateObject(\"Shell.Application\")\n\
        UAC.ShellExecute \"{}\", \"\", \"\", \"runas\", 1\n",
        exe_path
    );

    // 写入VBS脚本
    std::fs::write(&vbs_path, vbs_content).map_err(|e| format!("无法创建提权脚本: {}", e))?;

    // 运行VBS脚本
    let result = std::process::Command::new("wscript")
        .arg(vbs_path.to_str().unwrap())
        .creation_flags(process::CREATE_NO_WINDOW)
        .spawn();

    match result {
        Ok(_) => {
            // 启动成功，退出当前进程
            app_handle.exit(0);
            Ok(())
        }
        Err(e) => {
            // 尝试备用方法 - 使用cmd的start命令
            let result = std::process::Command::new("cmd")
                .args(&[
                    "/C",
                    "start",
                    "",
                    "/B",
                    "runas",
                    current_exe.to_str().unwrap(),
                ])
                .creation_flags(process::CREATE_NO_WINDOW)
                .spawn();

            match result {
                Ok(_) => {
                    app_handle.exit(0);
                    Ok(())
                }
                Err(e2) => Err(format!(
                    "{}: {} 然后尝试备用方法失败: {}",
                    messages::ERR_RESTART_FAILED,
                    e,
                    e2
                )),
            }
        }
    }
}

// 检查是否有管理员权限 - 使用Windows API的方式
#[tauri::command]
pub fn check_admin() -> bool {
    // 尝试执行一个需要管理员权限的操作，例如查询系统会话
    let result = std::process::Command::new("net")
        .arg("session")
        .creation_flags(process::CREATE_NO_WINDOW)
        .output();

    match result {
        Ok(output) => output.status.success(),
        Err(_) => false,
    }
}

// 切换开发者工具
#[tauri::command]
pub fn toggle_devtools(app_handle: tauri::AppHandle) -> Result<(), String> {
    let main_window = app_handle
        .get_webview_window("main")
        .ok_or("无法获取主窗口".to_string())?;

    // 检查devtools是否已经打开
    if main_window.is_devtools_open() {
        main_window.close_devtools();
    } else {
        main_window.open_devtools();
    }

    Ok(())
}

// 打开开发者工具
#[tauri::command]
pub fn open_devtools(app_handle: tauri::AppHandle) -> Result<(), String> {
    let main_window = app_handle
        .get_webview_window("main")
        .ok_or("无法获取主窗口".to_string())?;

    main_window.open_devtools();
    Ok(())
}

// 关闭开发者工具
#[tauri::command]
pub fn close_devtools(app_handle: tauri::AppHandle) -> Result<(), String> {
    let main_window = app_handle
        .get_webview_window("main")
        .ok_or("无法获取主窗口".to_string())?;

    main_window.close_devtools();
    Ok(())
}

// 检查开发者工具是否已打开
#[tauri::command]
pub fn is_devtools_open(app_handle: tauri::AppHandle) -> Result<bool, String> {
    let main_window = app_handle
        .get_webview_window("main")
        .ok_or("无法获取主窗口".to_string())?;

    Ok(main_window.is_devtools_open())
}
