use crate::app::constants::paths;
use crate::app::core::kernel_service::PROCESS_MANAGER;
use crate::utils::http_client;
use std::time::Duration;
use tracing::{info, warn};

#[tauri::command]
pub async fn is_kernel_running() -> Result<bool, String> {
    if PROCESS_MANAGER.is_running().await {
        return Ok(true);
    }

    #[cfg(target_os = "windows")]
    {
        is_kernel_running_windows().await
    }

    #[cfg(target_os = "linux")]
    {
        is_kernel_running_linux().await
    }

    #[cfg(target_os = "macos")]
    {
        is_kernel_running_macos().await
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
    {
        Err("当前平台不支持内核状态检查".to_string())
    }
}

#[cfg(target_os = "windows")]
async fn is_kernel_running_windows() -> Result<bool, String> {
    let kernel_path = crate::app::constants::core::paths::get_kernel_path();

    info!("检查内核进程，可执行文件路径: {:?}", kernel_path);

    let kernel_filename = kernel_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("sing-box.exe");
    let kernel_lower = kernel_filename.to_ascii_lowercase();

    let mut cmd = tokio::process::Command::new("tasklist");
    cmd.args(&[
        "/FI",
        &format!("IMAGENAME eq {}", kernel_filename),
        "/FO",
        "CSV",
        "/NH",
    ]);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::app::constants::process::CREATE_NO_WINDOW);

    if let Ok(output) = cmd.output().await {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            let parts: Vec<&str> = line
                .split('"')
                .filter(|s| !s.is_empty() && *s != ",")
                .collect();
            if let Some(image_name) = parts.get(0) {
                if image_name.to_ascii_lowercase() == kernel_lower {
                    info!("通过tasklist检测到内核进程: {}", image_name);
                    return Ok(true);
                }
            }
        }
    }

    let mut cmd = tokio::process::Command::new("powershell");
    cmd.args(&[
        "-Command",
        &format!(
            "Get-Process | Where-Object {{ $_.Path -like \"{}\" }}",
            kernel_path.to_string_lossy()
        ),
    ]);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::app::constants::process::CREATE_NO_WINDOW);

    if let Ok(output) = cmd.output().await {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if !stdout.trim().is_empty() {
            info!("通过PowerShell检测到内核进程");
            return Ok(true);
        }
    }

    info!("内核运行状态检查: false (未找到相关进程)");
    Ok(false)
}

#[cfg(target_os = "linux")]
async fn is_kernel_running_linux() -> Result<bool, String> {
    use std::process::Command;

    let kernel_path = crate::app::constants::core::paths::get_kernel_path();
    let kernel_filename = kernel_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("sing-box");

    if let Ok(output) = Command::new("pidof").arg(kernel_filename).output() {
        if output.status.success() && !output.stdout.is_empty() {
            info!("通过pidof检测到内核进程");
            return Ok(true);
        }
    }

    if let Ok(output) = Command::new("pgrep").arg(kernel_filename).output() {
        if output.status.success() && !output.stdout.is_empty() {
            info!("通过pgrep检测到内核进程");
            return Ok(true);
        }
    }

    if let Ok(output) = Command::new("ps").args(&["-ef"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains(kernel_filename) && line.contains("sing-box") {
                info!("通过ps检测到内核进程");
                return Ok(true);
            }
        }
    }

    info!("内核运行状态检查: false (未找到相关进程)");
    Ok(false)
}

#[cfg(target_os = "macos")]
async fn is_kernel_running_macos() -> Result<bool, String> {
    use std::process::Command;

    let kernel_path = crate::app::constants::core::paths::get_kernel_path();
    let kernel_filename = kernel_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("sing-box");

    if let Ok(output) = Command::new("pgrep").arg(kernel_filename).output() {
        if output.status.success() && !output.stdout.is_empty() {
            info!("通过pgrep检测到内核进程");
            return Ok(true);
        }
    }

    if let Ok(output) = Command::new("ps").args(&["-ef"]).output() {
        let stdout = String::from_utf8_lossy(&output.stdout);
        for line in stdout.lines() {
            if line.contains(kernel_filename) && line.contains("sing-box") {
                info!("通过ps检测到内核进程");
                return Ok(true);
            }
        }
    }

    info!("内核运行状态检查: false (未找到相关进程)");
    Ok(false)
}

#[tauri::command]
pub async fn get_system_uptime() -> Result<u64, String> {
    #[cfg(windows)]
    {
        let mut cmd = tokio::process::Command::new("powershell");
        cmd.args(&[
            "-Command",
            "(Get-Date) - (Get-CimInstance -ClassName Win32_OperatingSystem).LastBootUpTime | Select-Object -ExpandProperty TotalMilliseconds"
        ]);

        #[cfg(target_os = "windows")]
        cmd.creation_flags(crate::app::constants::process::CREATE_NO_WINDOW);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    let uptime_str = String::from_utf8_lossy(&output.stdout);
                    let uptime_ms: f64 = uptime_str.trim().parse().unwrap_or(0.0);
                    return Ok(uptime_ms as u64);
                } else {
                    warn!("PowerShell获取系统时间失败，使用备用方法");
                    return Ok(std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_millis() as u64);
                }
            }
            Err(e) => {
                warn!("无法获取系统运行时间: {}", e);
                return Ok(0);
            }
        }
    }

    #[cfg(target_os = "linux")]
    {
        match std::fs::read_to_string("/proc/uptime") {
            Ok(content) => {
                let uptime_seconds: f64 = content
                    .split_whitespace()
                    .next()
                    .unwrap_or("0")
                    .parse()
                    .unwrap_or(0.0);
                return Ok((uptime_seconds * 1000.0) as u64);
            }
            Err(_) => return Ok(0),
        }
    }

    #[cfg(target_os = "macos")]
    {
        let mut cmd = tokio::process::Command::new("sysctl");
        cmd.args(&["-n", "kern.boottime"]);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    let boottime_str = String::from_utf8_lossy(&output.stdout);
                    if let Some(sec_part) = boottime_str.split("sec = ").nth(1) {
                        if let Some(timestamp) = sec_part.split(',').next() {
                            if let Ok(boot_timestamp) = timestamp.trim().parse::<u64>() {
                                let current_timestamp = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();

                                let uptime_seconds =
                                    current_timestamp.saturating_sub(boot_timestamp);
                                return Ok(uptime_seconds * 1000);
                            }
                        }
                    }
                }
                match tokio::process::Command::new("uptime").output().await {
                    Ok(uptime_output) if uptime_output.status.success() => {
                        let uptime_str = String::from_utf8_lossy(&uptime_output.stdout);
                        info!("uptime输出: {}", uptime_str);
                        Ok(0)
                    }
                    _ => {
                        warn!("无法获取macOS系统运行时间");
                        Ok(0)
                    }
                }
            }
            Err(e) => {
                warn!("sysctl命令执行失败: {}", e);
                Ok(0)
            }
        }
    }
}

#[tauri::command]
pub async fn kernel_get_status_enhanced(
    app_handle: tauri::AppHandle,
    api_port: Option<u16>,
) -> Result<serde_json::Value, String> {
    let port = api_port.unwrap_or(12081);

    let process_running = is_kernel_running().await?;
    let mut api_ready = false;
    let mut websocket_ready = false;
    let mut error = None;

    if process_running {
        let client = http_client::get_client();
        let api_url = format!("http://127.0.0.1:{}/version", port);

        api_ready = match client
            .get(&api_url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => true,
            Ok(response) => {
                error = Some(format!("API返回错误状态码: {}", response.status()));
                false
            }
            Err(e) => {
                error = Some(format!("API连接失败: {}", e));
                false
            }
        };

        if api_ready {
            let token = crate::app::core::proxy_service::get_api_token();
            let url_str = format!("ws://127.0.0.1:{}/traffic?token={}", port, token);

            websocket_ready = tokio::time::timeout(
                Duration::from_secs(3),
                tokio_tungstenite::connect_async(&url_str),
            )
            .await
            .is_ok();

            if !websocket_ready && error.is_none() {
                error = Some("WebSocket连接失败".to_string());
            }
        }

        if !api_ready && error.is_none() {
            error = Some("内核进程运行中但API服务不可用".to_string());
        }
    }

    let mut version = if process_running {
        let client = http_client::get_client();
        let api_url = format!("http://127.0.0.1:{}/version", port);
        match client
            .get(&api_url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => match response.text().await {
                Ok(text) => Some(text.trim().to_string()),
                Err(_) => None,
            },
            _ => None,
        }
    } else {
        None
    };

    // 如果运行时并未获取到版本（未运行或API超时），回退到检查安装版本（DB/文件）
    if version.is_none() {
        if let Ok(v) = crate::app::core::kernel_service::versioning::check_kernel_version(app_handle).await {
            version = Some(v.trim().to_string());
        }
    }

    Ok(serde_json::json!({
        "process_running": process_running,
        "api_ready": api_ready,
        "websocket_ready": websocket_ready,
        "uptime_ms": 0,
        "version": version,
        "error": error
    }))
}

#[tauri::command]
pub async fn kernel_check_health(api_port: Option<u16>) -> Result<serde_json::Value, String> {
    let mut issues = Vec::new();
    let mut healthy = true;

    let kernel_path = paths::get_kernel_path();
    if !kernel_path.exists() {
        issues.push("内核文件不存在".to_string());
        healthy = false;
    }

    let config_path = paths::get_config_dir().join("config.json");
    if !config_path.exists() {
        issues.push("配置文件不存在".to_string());
        healthy = false;
    }

    let process_running = is_kernel_running().await.unwrap_or(false);
    if process_running {
        let port = api_port.unwrap_or(12081);
        let client = http_client::get_client();
        let api_url = format!("http://127.0.0.1:{}/version", port);

        let api_ready = match client
            .get(&api_url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => true,
            _ => false,
        };

        if !api_ready {
            issues.push(format!("内核进程运行但API不可用（端口: {}）", port));
            healthy = false;
        }
    }

    Ok(serde_json::json!({
        "healthy": healthy,
        "issues": issues
    }))
}
