//! Windows 平台实现

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

use tracing::{info, warn};

/// 检测进程是否运行（Windows）
pub async fn platform_is_process_running(process_name: &str) -> Result<bool, String> {
    let process_lower = process_name.to_ascii_lowercase();

    // 方法1: 使用 tasklist
    let mut cmd = tokio::process::Command::new("tasklist");
    cmd.args(&[
        "/FI",
        &format!("IMAGENAME eq {}", process_name),
        "/FO",
        "CSV",
        "/NH",
    ]);
    platform_configure_process_command(&mut cmd);

    if let Ok(output) = cmd.output().await {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line
                .split('"')
                .filter(|s| !s.is_empty() && *s != ",")
                .collect();
            if let Some(image_name) = parts.get(0) {
                if image_name.to_ascii_lowercase() == process_lower {
                    info!("通过tasklist检测到进程: {}", image_name);
                    return Ok(true);
                }
            }
        }
    }

    // 方法2: 使用 PowerShell
    let mut cmd = tokio::process::Command::new("powershell");
    cmd.args(&[
        "-Command",
        &format!(
            "Get-Process | Where-Object {{ $_.ProcessName -eq '{}' }}",
            process_name.trim_end_matches(".exe")
        ),
    ]);
    platform_configure_process_command(&mut cmd);

    if let Ok(output) = cmd.output().await {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            info!("通过PowerShell检测到进程");
            return Ok(true);
        }
    }

    Ok(false)
}

/// 杀死指定名称的进程（Windows）
pub async fn platform_kill_processes_by_name(process_name: &str) -> Result<(), String> {
    let mut cmd = tokio::process::Command::new("taskkill");
    cmd.args(&["/F", "/IM", process_name]);
    platform_configure_process_command(&mut cmd);

    match cmd.output().await {
        Ok(output) => {
            if output.status.success() {
                info!("成功终止进程: {}", process_name);
                Ok(())
            } else {
                let stderr = String::from_utf8_lossy(&output.stderr);
                if stderr.contains("not found") || stderr.contains("没有找到") {
                    Ok(())
                } else {
                    Err(format!("终止进程失败: {}", stderr))
                }
            }
        }
        Err(e) => Err(format!("执行taskkill失败: {}", e)),
    }
}

/// 杀死指定 PID 的进程（Windows）
pub fn platform_kill_process_by_pid(pid: u32) -> Result<(), String> {
    let mut cmd = std::process::Command::new("taskkill");
    cmd.args(&["/F", "/PID", &pid.to_string()]);
    platform_configure_std_command(&mut cmd);
    
    let output = cmd.output()
        .map_err(|e| format!("执行taskkill失败: {}", e))?;

    if output.status.success() {
        info!("成功终止进程 PID: {}", pid);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("终止进程失败: {}", stderr))
    }
}

/// 获取系统架构（Windows）
pub fn platform_get_system_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "x86" => "386",
        "aarch64" => "arm64",
        _ => "amd64",
    }
}

/// 获取平台名称（Windows）
pub fn platform_get_platform_name() -> &'static str {
    "windows"
}

/// 获取内核可执行文件名（Windows）
pub fn platform_get_kernel_executable_name() -> &'static str {
    "sing-box.exe"
}

/// 获取系统运行时间（Windows）
pub async fn platform_get_system_uptime_ms() -> Result<u64, String> {
    let mut cmd = tokio::process::Command::new("powershell");
    cmd.args(&[
        "-Command",
        "(Get-Date) - (Get-CimInstance -ClassName Win32_OperatingSystem).LastBootUpTime | Select-Object -ExpandProperty TotalMilliseconds"
    ]);
    platform_configure_process_command(&mut cmd);

    match cmd.output().await {
        Ok(output) => {
            if output.status.success() {
                let uptime_str = String::from_utf8_lossy(&output.stdout);
                let uptime_ms: f64 = uptime_str.trim().parse().unwrap_or(0.0);
                Ok(uptime_ms as u64)
            } else {
                warn!("PowerShell获取系统时间失败，使用备用方法");
                Ok(std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_millis() as u64)
            }
        }
        Err(e) => {
            warn!("无法获取系统运行时间: {}", e);
            Ok(0)
        }
    }
}

/// 配置 tokio 进程命令（Windows）
pub fn platform_configure_process_command(command: &mut tokio::process::Command) {
    #[cfg(target_os = "windows")]
    {
        command.creation_flags(crate::app::constants::process::CREATE_NO_WINDOW);
    }
}

/// 配置标准库进程命令（Windows）
pub fn platform_configure_std_command(command: &mut std::process::Command) {
    #[cfg(target_os = "windows")]
    {
        command.creation_flags(crate::app::constants::process::CREATE_NO_WINDOW);
    }
}
