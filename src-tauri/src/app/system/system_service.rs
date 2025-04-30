use crate::app::constants::{messages, process};
use std::os::windows::process::CommandExt;
use serde_json::json;
use std::path::PathBuf;
use std::process::Command;
use tracing::info;
use crate::utils::app_util::{get_service_path, get_work_dir};

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
    std::fs::write(&vbs_path, vbs_content)
        .map_err(|e| format!("无法创建提权脚本: {}", e))?;
    
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
        },
        Err(e) => {
            // 尝试备用方法 - 使用cmd的start命令
            let result = std::process::Command::new("cmd")
                .args(&["/C", "start", "", "/B", "runas", current_exe.to_str().unwrap()])
                .creation_flags(process::CREATE_NO_WINDOW)
                .spawn();
                
            match result {
                Ok(_) => {
                    app_handle.exit(0);
                    Ok(())
                },
                Err(e2) => Err(format!("{}: {} 然后尝试备用方法失败: {}", 
                                       messages::ERR_RESTART_FAILED, e, e2))
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

// 退出程序的命令
#[tauri::command]
pub fn exit_application(app_handle: tauri::AppHandle) {
    app_handle.exit(0);
}

/// 获取用户缓存目录
fn get_user_cache_dir() -> PathBuf {
    let work_dir = get_work_dir();
    PathBuf::from(&work_dir)
}

/// 安装服务
#[tauri::command]
pub async fn install_service() -> Result<serde_json::Value, String> {
    info!("安装服务");
    
    // 检查是否有管理员权限
    if !check_admin() {
        return Err("安装服务需要管理员权限".to_string());
    }
    
    let cache_dir = get_user_cache_dir();
    info!("用户缓存目录: {:?}", cache_dir);
    
    // 确保缓存目录存在
    if let Err(e) = std::fs::create_dir_all(&cache_dir) {
        return Err(format!("创建缓存目录失败: {}", e));
    }
    
    // 执行服务安装命令
    let output = Command::new(get_service_path())
        .arg("install")
        .arg(cache_dir.to_string_lossy().to_string())
        .creation_flags(process::CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行服务安装命令失败: {}", e))?;
    
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    info!("服务安装结果: 退出码={}, stdout={}, stderr={}", exit_code, stdout, stderr);
    
    if exit_code == 0 {
        Ok(json!({
            "success": true,
            "message": "服务安装成功",
            "output": stdout
        }))
    } else {
        Err(format!("服务安装失败: {}", stderr))
    }
}

/// 卸载服务
#[tauri::command]
pub async fn uninstall_service() -> Result<serde_json::Value, String> {
    info!("卸载服务");
    
    // 检查是否有管理员权限
    if !check_admin() {
        return Err("卸载服务需要管理员权限".to_string());
    }
    
    // 执行服务卸载命令
    let output = Command::new(get_service_path())
        .arg("uninstall")
        .creation_flags(process::CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行服务卸载命令失败: {}", e))?;
    
    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    
    info!("服务卸载结果: 退出码={}, stdout={}, stderr={}", exit_code, stdout, stderr);
    
    if exit_code == 0 {
        Ok(json!({
            "success": true,
            "message": "服务卸载成功",
            "output": stdout
        }))
    } else {
        Err(format!("服务卸载失败: {}", stderr))
    }
}
