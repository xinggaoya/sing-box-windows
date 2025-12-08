//! Linux 平台实现

use tracing::info;

/// 检测进程是否运行（Linux）
pub async fn platform_is_process_running(process_name: &str) -> Result<bool, String> {
    // 方法1: 使用 pidof
    if let Ok(output) = std::process::Command::new("pidof")
        .arg(process_name)
        .output()
    {
        if output.status.success() && !output.stdout.is_empty() {
            info!("通过pidof检测到进程");
            return Ok(true);
        }
    }

    // 方法2: 使用 pgrep
    if let Ok(output) = std::process::Command::new("pgrep")
        .arg(process_name)
        .output()
    {
        if output.status.success() && !output.stdout.is_empty() {
            info!("通过pgrep检测到进程");
            return Ok(true);
        }
    }

    // 方法3: 使用 ps
    if let Ok(output) = std::process::Command::new("ps")
        .args(&["-ef"])
        .output()
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains(process_name) {
                info!("通过ps检测到进程");
                return Ok(true);
            }
        }
    }

    Ok(false)
}

/// 杀死指定名称的进程（Linux）
pub async fn platform_kill_processes_by_name(process_name: &str) -> Result<(), String> {
    let output = std::process::Command::new("pkill")
        .arg("-9")
        .arg(process_name)
        .output()
        .map_err(|e| format!("执行pkill失败: {}", e))?;

    if output.status.success() || output.status.code() == Some(1) {
        info!("成功终止进程: {}", process_name);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("终止进程失败: {}", stderr))
    }
}

/// 杀死指定 PID 的进程（Linux）
pub fn platform_kill_process_by_pid(pid: u32) -> Result<(), String> {
    let output = std::process::Command::new("kill")
        .args(&["-9", &pid.to_string()])
        .output()
        .map_err(|e| format!("执行kill失败: {}", e))?;

    if output.status.success() {
        info!("成功终止进程 PID: {}", pid);
        Ok(())
    } else {
        let stderr = String::from_utf8_lossy(&output.stderr);
        Err(format!("终止进程失败: {}", stderr))
    }
}

/// 获取系统架构（Linux）
pub fn platform_get_system_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "x86" => "386",
        "aarch64" => "arm64",
        "arm" => "armv7",
        _ => "amd64",
    }
}

/// 获取平台名称（Linux）
pub fn platform_get_platform_name() -> &'static str {
    "linux"
}

/// 获取内核可执行文件名（Linux）
pub fn platform_get_kernel_executable_name() -> &'static str {
    "sing-box"
}

/// 获取系统运行时间（Linux）
pub async fn platform_get_system_uptime_ms() -> Result<u64, String> {
    match std::fs::read_to_string("/proc/uptime") {
        Ok(content) => {
            let uptime_seconds: f64 = content
                .split_whitespace()
                .next()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);
            Ok((uptime_seconds * 1000.0) as u64)
        }
        Err(_) => Ok(0),
    }
}

/// 配置 tokio 进程命令（Linux - 无操作）
pub fn platform_configure_process_command(_command: &mut tokio::process::Command) {
    // Linux 不需要特殊配置
}

/// 配置标准库进程命令（Linux - 无操作）
pub fn platform_configure_std_command(_command: &mut std::process::Command) {
    // Linux 不需要特殊配置
}
