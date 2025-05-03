use crate::app::constants::{messages, process};
use crate::utils::app_util::{get_service_path, get_work_dir};
use serde_json::json;
use std::fs;
use std::io::{Read};
use std::os::windows::process::CommandExt;
use std::path::PathBuf;
use std::process::Command;
use tracing::{info, error, warn};
use sha2::{Sha256, Digest};

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

/// 将服务文件复制到缓存目录
fn copy_service_to_cache() -> Result<PathBuf, String> {
    let source_path = get_service_path();
    let cache_dir = get_user_cache_dir();
    let target_path = cache_dir.join("sing-box-service.exe");

    info!("从 {:?} 复制服务到 {:?}", source_path, target_path);

    // 确保源文件存在
    if !source_path.exists() {
        return Err(format!("服务文件不存在: {:?}", source_path));
    }

    // 确保目标目录存在
    if let Err(e) = std::fs::create_dir_all(&cache_dir) {
        return Err(format!("创建缓存目录失败: {}", e));
    }

    // 复制文件
    match fs::copy(&source_path, &target_path) {
        Ok(_) => {
            info!("服务文件复制成功");
            Ok(target_path)
        }
        Err(e) => Err(format!("复制服务文件失败: {}", e)),
    }
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

    // 复制服务文件到缓存目录
    let service_path = match copy_service_to_cache() {
        Ok(path) => path,
        Err(e) => return Err(e),
    };

    // 执行服务安装命令
    let output = Command::new(service_path)
        .arg("install")
        .arg(cache_dir.to_string_lossy().to_string())
        .creation_flags(process::CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行服务安装命令失败: {}", e))?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    info!(
        "服务安装结果: 退出码={}, stdout={}, stderr={}",
        exit_code, stdout, stderr
    );

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

    // 尝试从缓存目录获取服务文件
    let cache_dir = get_user_cache_dir();
    let cached_service_path = cache_dir.join("sing-box-service.exe");

    // 如果缓存中有服务文件，使用它；否则使用原始路径
    let service_path = if cached_service_path.exists() {
        cached_service_path
    } else {
        get_service_path()
    };

    // 执行服务卸载命令
    let output = Command::new(service_path)
        .arg("uninstall")
        .creation_flags(process::CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行服务卸载命令失败: {}", e))?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    info!(
        "服务卸载结果: 退出码={}, stdout={}, stderr={}",
        exit_code, stdout, stderr
    );

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

/// 获取服务状态
#[tauri::command]
pub async fn check_service_status() -> Result<serde_json::Value, String> {
    info!("检查服务状态");

    // 检查服务是否存在的命令
    let output = Command::new("sc")
        .args(&["query", "SingBoxService"])
        .creation_flags(process::CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行服务状态检查命令失败: {}", e))?;

    let exit_code = output.status.code().unwrap_or(-1);
    let stdout = String::from_utf8_lossy(&output.stdout).to_string();

    let is_installed =
        exit_code == 0 && !stdout.contains("不存在") && !stdout.contains("DOES NOT EXIST");
    let is_running = is_installed && stdout.contains("RUNNING");

    info!(
        "服务状态检查结果: 已安装={}, 运行中={}",
        is_installed, is_running
    );

    Ok(json!({
        "installed": is_installed,
        "running": is_running
    }))
}

/// 计算文件的SHA256哈希值
fn calculate_file_hash(path: &PathBuf) -> Result<String, String> {
    let mut file = fs::File::open(path)
        .map_err(|e| format!("无法打开文件进行哈希计算: {}", e))?;
    
    let mut hasher = Sha256::new();
    let mut buffer = [0; 4096];
    
    loop {
        let bytes_read = file.read(&mut buffer)
            .map_err(|e| format!("读取文件失败: {}", e))?;
        
        if bytes_read == 0 {
            break;
        }
        
        hasher.update(&buffer[..bytes_read]);
    }
    
    let hash = hasher.finalize();
    let hash_str = format!("{:x}", hash);
    
    Ok(hash_str)
}

/// 更新服务
#[tauri::command]
pub async fn update_service() -> Result<serde_json::Value, String> {
    info!("开始检查服务更新");
    
    // 检查是否有管理员权限
    if !check_admin() {
        return Err("更新服务需要管理员权限".to_string());
    }
    
    // 首先检查服务是否已安装
    let service_status = check_service_status().await?;
    let is_installed = service_status["installed"].as_bool().unwrap_or(false);
    
    if !is_installed {
        info!("服务未安装，无需更新");
        return Ok(json!({
            "success": true,
            "updated": false,
            "message": "服务未安装，无需更新"
        }));
    }
    
    // 获取原始服务文件路径和缓存目录中的服务文件路径
    let original_service_path = get_service_path();
    let cache_dir = get_user_cache_dir();
    let cached_service_path = cache_dir.join("sing-box-service.exe");
    
    // 检查两个文件是否存在
    if !original_service_path.exists() {
        return Err(format!("原始服务文件不存在: {:?}", original_service_path));
    }
    
    if !cached_service_path.exists() {
        info!("缓存目录中的服务文件不存在，将进行首次复制");
        
        // 如果缓存文件不存在，直接复制原始文件到缓存目录
        return Ok(json!({
            "success": true,
            "updated": true,
            "message": "服务文件已更新"
        }));
    }
    
    // 计算两个文件的哈希值
    let original_hash = calculate_file_hash(&original_service_path)?;
    let cached_hash = calculate_file_hash(&cached_service_path)?;
    
    // 如果哈希值相同，说明文件相同，无需更新
    if original_hash == cached_hash {
        info!("服务文件哈希值相同，无需更新");
        return Ok(json!({
            "success": true,
            "updated": false,
            "message": "服务文件没有变化，无需更新"
        }));
    }
    
    info!("检测到服务文件已更新，准备更新服务");
    
    // 服务是否在运行中
    let is_running = service_status["running"].as_bool().unwrap_or(false);
    
    // 如果服务正在运行，先停止服务
    if is_running {
        info!("停止服务...");
        let stop_result = Command::new("sc")
            .args(&["stop", "SingBoxService"])
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("停止服务失败: {}", e))?;
        
        if !stop_result.status.success() {
            let error = String::from_utf8_lossy(&stop_result.stderr);
            warn!("停止服务时出现警告: {}", error);
            // 继续执行，因为有些情况下即使命令返回非零状态码，服务也可能已经停止
        }
        
        // 等待服务完全停止
        std::thread::sleep(std::time::Duration::from_secs(2));
    }
    
    // 复制新的服务文件到缓存目录
    info!("复制新的服务文件到缓存目录");
    match fs::copy(&original_service_path, &cached_service_path) {
        Ok(_) => {
            info!("服务文件复制成功");
        }
        Err(e) => {
            error!("复制服务文件失败: {}", e);
            return Err(format!("复制服务文件失败: {}", e));
        }
    }
    
    // 如果服务之前在运行，则重新启动服务
    if is_running {
        info!("重新启动服务...");
        let start_result = Command::new("sc")
            .args(&["start", "SingBoxService"])
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()
            .map_err(|e| format!("启动服务失败: {}", e))?;
        
        if !start_result.status.success() {
            let error = String::from_utf8_lossy(&start_result.stderr);
            return Err(format!("启动服务失败: {}", error));
        }
    }
    
    info!("服务更新完成");
    Ok(json!({
        "success": true,
        "updated": true,
        "message": "服务文件已更新"
    }))
}

/// 检查服务是否需要更新
#[tauri::command]
pub async fn check_service_update_needed() -> Result<serde_json::Value, String> {
    info!("检查服务是否需要更新");
    
    // 首先检查服务是否已安装
    let service_status = check_service_status().await?;
    let is_installed = service_status["installed"].as_bool().unwrap_or(false);
    
    if !is_installed {
        info!("服务未安装，无需检查更新");
        return Ok(json!({
            "success": true,
            "need_update": false,
            "message": "服务未安装，无需更新"
        }));
    }
    
    // 获取原始服务文件路径和缓存目录中的服务文件路径
    let original_service_path = get_service_path();
    let cache_dir = get_user_cache_dir();
    let cached_service_path = cache_dir.join("sing-box-service.exe");
    
    // 检查两个文件是否存在
    if !original_service_path.exists() {
        return Err(format!("原始服务文件不存在: {:?}", original_service_path));
    }
    
    if !cached_service_path.exists() {
        info!("缓存目录中的服务文件不存在，需要更新");
        return Ok(json!({
            "success": true,
            "need_update": true,
            "message": "需要更新服务文件"
        }));
    }
    
    // 计算两个文件的哈希值
    let original_hash = calculate_file_hash(&original_service_path)?;
    let cached_hash = calculate_file_hash(&cached_service_path)?;
    
    // 如果哈希值相同，说明文件相同，无需更新
    if original_hash == cached_hash {
        info!("服务文件哈希值相同，无需更新");
        return Ok(json!({
            "success": true,
            "need_update": false,
            "message": "服务文件没有变化，无需更新"
        }));
    }
    
    info!("检测到服务文件已更新，需要更新服务");
    Ok(json!({
        "success": true,
        "need_update": true,
        "message": "服务文件有更新，需要更新服务"
    }))
}
