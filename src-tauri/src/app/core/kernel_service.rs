use crate::app::constants::{common::messages, paths};
use crate::app::core::event_relay::{
    create_connection_event_relay, create_log_event_relay, create_memory_event_relay,
    create_traffic_event_relay, start_event_relay_with_retry,
};
use serde_json::json;
use tauri::Manager;
use std::process::Command;
use crate::process::manager::ProcessManager;
use crate::utils::http_client;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::{AppHandle, Emitter};
use tokio::sync::Notify;
use tokio::task::JoinHandle;
use tracing::{error, info, warn};

// 全局进程管理器
lazy_static::lazy_static! {
    pub(crate) static ref PROCESS_MANAGER: Arc<ProcessManager> = Arc::new(ProcessManager::new());
}

// 事件中继任务管理器
lazy_static::lazy_static! {
    static ref EVENT_RELAY_TASKS: Arc<tokio::sync::Mutex<Vec<JoinHandle<()>>>> = Arc::new(tokio::sync::Mutex::new(Vec::new()));
    static ref SHOULD_STOP_EVENTS: Arc<AtomicBool> = Arc::new(AtomicBool::new(false));
}

// 内核启动通知器
lazy_static::lazy_static! {
    static ref KERNEL_READY_NOTIFY: Arc<Notify> = Arc::new(Notify::new());
}

// 检查内核版本
#[tauri::command]
pub async fn check_kernel_version() -> Result<String, String> {
    let kernel_path = paths::get_kernel_path();

    if !kernel_path.exists() {
        return Err(messages::ERR_KERNEL_NOT_FOUND.to_string());
    }

    let mut cmd = tokio::process::Command::new(kernel_path);
    cmd.arg("version");

    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);

    let output = cmd.output()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_VERSION_CHECK_FAILED, e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{}: {}", messages::ERR_GET_VERSION_FAILED, error));
    }

    let version_info = String::from_utf8_lossy(&output.stdout);
    Ok(version_info.to_string())
}

// 检查配置是否正常
#[tauri::command]
pub async fn check_config_validity(config_path: String) -> Result<(), String> {
    let kernel_path = paths::get_kernel_path();

    if !kernel_path.exists() {
        return Err(messages::ERR_KERNEL_NOT_FOUND.to_string());
    }

    // 确保配置文件路径存在
    let path = if config_path.is_empty() {
        paths::get_config_path().to_string_lossy().to_string()
    } else {
        config_path
    };

    // 检查配置文件是否存在
    if !std::path::Path::new(&path).exists() {
        return Err(format!("配置文件不存在: {}", path));
    }

    let mut cmd = tokio::process::Command::new(kernel_path);
    cmd.arg("check")
        .arg("--config")
        .arg(path);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);

    let output = cmd.output()
        .await
        .map_err(|e| format!("执行配置检查命令失败: {}", e))?;

    // 检查命令是否成功执行
    if !output.status.success() {
        // 如果有错误输出，返回错误信息
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("配置检查失败: {}", error));
    }

    Ok(())
}

// 获取系统架构
fn get_system_arch() -> &'static str {
    // 首先检查是否手动指定了架构（用于特殊情况）
    if let Ok(force_arch) = std::env::var("SING_BOX_FORCE_ARCH") {
        info!("用户手动指定架构: {}", force_arch);
        return match force_arch.as_str() {
            "amd64" | "x86_64" => "amd64",
            "386" | "i386" => "386",
            "arm64" | "aarch64" => "arm64",
            "armv5" => "armv5",
            _ => "amd64", // 默认值
        };
    }

    // 添加更详细的调试信息
    info!("Rust ARCH 常量: {}", std::env::consts::ARCH);

    if cfg!(target_os = "windows") {
        // Windows 架构检测
        let arch = match std::env::consts::ARCH {
            "x86_64" => "amd64",
            "x86" => "386",
            "aarch64" => "arm64",
            _ => "amd64", // 默认值
        };
        info!("Windows 检测到架构: {}", arch);
        arch
    } else if cfg!(target_os = "linux") {
        // Linux 架构检测
        let mut detected_arch = "amd64"; // 默认值

        // 首先尝试通过 uname 命令获取准确架构
        if let Ok(output) = Command::new("uname").arg("-m").output() {
            if let Ok(arch_str) = String::from_utf8(output.stdout) {
                let arch = arch_str.trim();
                info!("uname -m 输出: '{}'", arch);

                detected_arch = match arch {
                    "x86_64" => "amd64",
                    "amd64" => "amd64",
                    "i386" | "i486" | "i586" | "i686" => "386",
                    "aarch64" | "arm64" => "arm64",
                    "armv7l" | "armv6l" => "armv5",
                    _ => {
                        info!("未知的 uname 架构，使用 Rust ARCH 常量");
                        match std::env::consts::ARCH {
                            "x86_64" => "amd64",
                            "x86" => "386",
                            "aarch64" => "arm64",
                            _ => "amd64",
                        }
                    }
                };
                info!("通过 uname 检测到的架构: {}", detected_arch);
            }
        } else {
            info!("uname 命令执行失败，使用 Rust ARCH 常量");
        }

        // 如果 uname 命令失败或结果不明确，使用 Rust 的 ARCH 常量作为备用
        if detected_arch == "amd64" && std::env::consts::ARCH != "x86_64" {
            detected_arch = match std::env::consts::ARCH {
                "x86_64" => "amd64",
                "x86" => "386",
                "aarch64" => "arm64",
                "arm" => "armv5",
                _ => "amd64",
            };
            info!("通过 Rust ARCH 常量检测到的架构: {}", detected_arch);
        }

        detected_arch
    } else {
        info!("其他平台，使用默认架构 amd64");
        "amd64" // 其他平台的默认值
    }
}

// 下载最新内核版本
#[tauri::command]
pub async fn download_latest_kernel(app_handle: tauri::AppHandle) -> Result<(), String> {
    info!("开始下载最新内核...");

    let window = app_handle.get_webview_window("main")
        .ok_or("无法获取主窗口")?;

    // 发送开始下载事件
    let _ = window.emit(
        "kernel-download-progress",
        json!({
            "status": "downloading",
            "progress": 0,
            "message": "开始下载内核..."
        }),
    );

    // 获取系统架构和平台信息
    let platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else {
        return Err("当前平台不支持".to_string());
    };

    let arch = get_system_arch();

    // 记录检测到的架构信息
    info!("检测到平台: {}, 架构: {}", platform, arch);

    // 构造下载 URL - 使用正确的 GitHub 资源命名格式
    let version = "1.12.10"; // 可以从 GitHub API 获取最新版本
    let filename = format!("sing-box-{}-{}-{}.tar.gz", version, platform, arch);

    // 使用多个下载源以提高成功率
    let download_urls = vec![
        // 使用 GitHub 快速加速镜像（优先）
        format!("https://ghfast.top/https://github.com/SagerNet/sing-box/releases/download/v{}/{}", version, filename),
        // 使用 GitHub 加速镜像（国内用户）
        format!("https://hub.fastgit.xyz/SagerNet/sing-box/releases/download/v{}/{}", version, filename),
        // 使用 GitLab 镜像
        format!("https://hub.fgit.cf/SagerNet/sing-box/releases/download/v{}/{}", version, filename),
        // 使用 jsdelivr CDN
        format!("https://cdn.jsdelivr.net/gh/SagerNet/sing-box@releases/download/v{}/{}", version, filename),
        // 使用 gh-proxy 镜像
        format!("https://ghproxy.com/https://github.com/SagerNet/sing-box/releases/download/v{}/{}", version, filename),
        // 原始 GitHub 链接（备用）
        format!("https://github.com/SagerNet/sing-box/releases/download/v{}/{}", version, filename),
    ];

    // 记录下载信息
    info!("文件名: {}", filename);
    info!("主要下载 URL (ghfast.top 加速): {}", download_urls[0]);
    info!("备用下载源 1 (hub.fastgit.xyz): {}", download_urls[1]);
    info!("备用下载源 2 (hub.fgit.cf): {}", download_urls[2]);
    info!("备用下载源 3 (jsdelivr CDN): {}", download_urls[3]);
    info!("备用下载源 4 (gh-proxy): {}", download_urls[4]);
    info!("备用下载源 5 (GitHub 原始): {}", download_urls[5]);
    info!("总共 {} 个下载源", download_urls.len());

    // 获取工作目录
    let work_dir = crate::utils::app_util::get_work_dir_sync();
    let kernel_dir = std::path::Path::new(&work_dir).join("sing-box");

    // 确保目录存在
    if let Err(e) = std::fs::create_dir_all(&kernel_dir) {
        return Err(format!("创建内核目录失败: {}", e));
    }

    let download_path = kernel_dir.join(&filename);

    // 发送下载进度更新
    let _ = window.emit(
        "kernel-download-progress",
        json!({
            "status": "downloading",
            "progress": 10,
            "message": "正在下载内核文件..."
        }),
    );

    // 实现下载逻辑 - 尝试多个下载源
    for (index, download_url) in download_urls.iter().enumerate() {
        info!("尝试第 {} 个下载源: {}", index + 1, download_url);

        // 发送尝试新下载源的事件
        let _ = window.emit(
            "kernel-download-progress",
            json!({
                "status": "downloading",
                "progress": 15 + (index * 5),
                "message": format!("尝试第 {} 个下载源...", index + 1)
            }),
        );

        match download_file(&download_url, &download_path, &window).await {
            Ok(_) => {
                info!("下载成功，使用下载源: {}", download_url);
                break; // 下载成功，退出循环
            }
            Err(e) => {
                let error_msg = format!("下载源 {} 失败: {}", index + 1, e);
                warn!("{}", error_msg);

                // 删除部分下载的文件
                let _ = std::fs::remove_file(&download_path);

                // 如果不是最后一个下载源，继续尝试
                if index < download_urls.len() - 1 {
                    continue;
                }

                // 所有下载源都失败
                let _ = window.emit(
                    "kernel-download-progress",
                    json!({
                        "status": "error",
                        "progress": 0,
                        "message": error_msg
                    }),
                );

                return Err(error_msg);
            }
        }
    }

    // 检查文件是否成功下载
    if !download_path.exists() {
        return Err("下载的文件不存在".to_string());
    }

    let _ = window.emit(
        "kernel-download-progress",
        json!({
            "status": "extracting",
            "progress": 80,
            "message": "正在解压内核文件..."
        }),
    );

    if let Err(e) = extract_archive(&download_path, &kernel_dir).await {
        let error_msg = format!("解压文件失败: {}", e);
        let _ = window.emit(
            "kernel-download-progress",
            json!({
                "status": "error",
                "progress": 0,
                "message": error_msg
            }),
        );
        return Err(error_msg);
    }

    // 清理下载的压缩文件
    let _ = std::fs::remove_file(&download_path);

    // 验证可执行文件是否存在
    let executable_name = if cfg!(target_os = "windows") {
        "sing-box.exe"
    } else {
        "sing-box"
    };

    // 查找可执行文件（可能在子目录中）
    let found_executable_path = find_executable_file(&kernel_dir, executable_name).await?;

    // 将可执行文件迁移到正确位置（kernel_dir/sing-box 或 kernel_dir/sing-box.exe）
    let target_executable_path = kernel_dir.join(executable_name);

    // 如果找到的文件不在目标位置，需要移动
    if found_executable_path != target_executable_path {
        info!("迁移内核文件从 {:?} 到 {:?}", found_executable_path, target_executable_path);

        // 确保目标位置的文件不存在
        if target_executable_path.exists() {
            if let Err(e) = std::fs::remove_file(&target_executable_path) {
                warn!("删除已存在的目标文件失败: {}, 将继续...", e);
            }
        }

        // 移动文件到正确位置
        if let Err(_e) = std::fs::rename(&found_executable_path, &target_executable_path) {
            // 如果跨设备移动失败，尝试复制后删除
            if let Err(copy_err) = std::fs::copy(&found_executable_path, &target_executable_path) {
                return Err(format!("复制内核文件失败: {}", copy_err));
            }
            if let Err(remove_err) = std::fs::remove_file(&found_executable_path) {
                warn!("删除原文件失败: {}, 将继续...", remove_err);
            }
            info!("成功复制内核文件到正确位置");
        } else {
            info!("成功移动内核文件到正确位置");
        }

        // 清理版本目录和其他不必要文件
        if let Some(parent_dir) = found_executable_path.parent() {
            info!("清理版本目录: {:?}", parent_dir);

            // 删除整个版本目录（包含所有文件）
            if let Err(e) = std::fs::remove_dir_all(parent_dir) {
                warn!("删除版本目录失败: {}, 将继续...", e);
            } else {
                info!("成功删除版本目录: {:?}", parent_dir);
            }
        }

        // 清理其他可能的解压文件（只保留可执行文件）
        if let Err(e) = cleanup_kernel_directory(&kernel_dir, executable_name) {
            warn!("清理内核目录失败: {}, 将继续...", e);
        }
    }

    // 在 Linux/macOS 下设置执行权限
    if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        if let Err(e) = set_executable_permission(&target_executable_path) {
            warn!("设置执行权限失败: {}, 将继续...", e);
        }
    }

    info!("内核文件已准备就绪: {:?}", target_executable_path);

    info!("内核下载并解压完成: {:?}", target_executable_path);

    let _ = window.emit(
        "kernel-download-progress",
        json!({
            "status": "completed",
            "progress": 100,
            "message": "内核下载完成！"
        }),
    );

    Ok(())
}

// 下载文件的辅助函数
async fn download_file(
    url: &str,
    path: &std::path::Path,
    window: &tauri::WebviewWindow,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    // 设置下载超时和更好的用户代理
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // 5分钟超时
        .user_agent("sing-box-windows/1.8.2")
        .build()?;

    info!("开始下载: {}", url);
    let response = client.get(url).send().await?;

    if !response.status().is_success() {
        return Err(format!("HTTP 错误: {}", response.status()).into());
    }

    let total_size = response.content_length().unwrap_or(0);
    let mut downloaded = 0u64;
    let mut file = File::create(path).await?;

    let mut stream = response.bytes_stream();
    use futures_util::StreamExt;

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        file.write_all(&chunk).await?;

        downloaded += chunk.len() as u64;

        if total_size > 0 {
            let progress = (downloaded * 100) / total_size;
            let _ = window.emit(
                "kernel-download-progress",
                json!({
                    "status": "downloading",
                    "progress": progress.min(70), // 最多到70%，留30%给解压
                    "message": format!("下载中... {}/{} bytes", downloaded, total_size)
                }),
            );
        }
    }

    file.flush().await?;
    Ok(())
}

// 解压文件的辅助函数
async fn extract_archive(
    archive_path: &std::path::Path,
    extract_to: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use std::fs::File;
    use flate2::read::GzDecoder;
    use tar::Archive;

    info!("开始解压文件: {:?}", archive_path);

    // 验证文件是否存在
    if !archive_path.exists() {
        return Err(format!("压缩文件不存在: {:?}", archive_path).into());
    }

    // 检查文件大小
    let metadata = std::fs::metadata(archive_path)?;
    let file_size = metadata.len();
    info!("压缩文件大小: {} bytes", file_size);

    if file_size == 0 {
        return Err("压缩文件为空".into());
    }

    // 打开压缩文件
    let file = File::open(archive_path)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    info!("解压到目录: {:?}", extract_to);

    // 确保解压目录存在
    if !extract_to.exists() {
        std::fs::create_dir_all(extract_to)?;
    }

    // 解压所有文件
    match archive.unpack(extract_to) {
        Ok(_) => {
            info!("文件解压完成");

            // 列出解压后的文件（用于调试）
            if let Ok(entries) = std::fs::read_dir(extract_to) {
                info!("解压后的文件:");
                for entry in entries.flatten() {
                    info!("  - {:?}", entry.path());
                }
            }
        }
        Err(e) => {
            return Err(format!("解压失败: {}", e).into());
        }
    }

    Ok(())
}

// 查找可执行文件的辅助函数
async fn find_executable_file(
    search_dir: &std::path::Path,
    executable_name: &str,
) -> Result<std::path::PathBuf, String> {
    info!("在目录 {:?} 中查找可执行文件: {}", search_dir, executable_name);

    // 首先直接在根目录查找
    let direct_path = search_dir.join(executable_name);
    if direct_path.exists() && direct_path.is_file() {
        info!("直接找到可执行文件: {:?}", direct_path);
        return Ok(direct_path);
    }

    // 递归搜索子目录
    let mut found_files = Vec::new();

    if let Ok(entries) = walkdir::WalkDir::new(search_dir)
        .into_iter()
        .collect::<Result<Vec<_>, _>>()
    {
        for entry in entries {
            let path = entry.path();
            if path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name == executable_name)
                .unwrap_or(false)
                && path.is_file() // 确保是文件而不是目录
            {
                info!("找到可执行文件: {:?}", path);
                found_files.push(path.to_path_buf());
            }
        }
    }

    if found_files.is_empty() {
        // 列出所有文件用于调试
        if let Ok(entries) = std::fs::read_dir(search_dir) {
            warn!("未找到可执行文件，目录内容:");
            for entry in entries.flatten() {
                warn!("  - {:?}", entry.path());
            }
        }
        return Err(format!("未找到可执行文件: {} 在目录 {:?} 中", executable_name, search_dir));
    }

    // 返回第一个找到的文件
    Ok(found_files[0].clone())
}

// 设置执行权限的辅助函数（跨平台兼容）
#[cfg(unix)]
fn set_executable_permission(file_path: &std::path::Path) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = std::fs::metadata(file_path)?.permissions();
    perms.set_mode(perms.mode() | 0o755); // rwxr-xr-x
    std::fs::set_permissions(file_path, perms)?;

    info!("已设置执行权限: {:?}", file_path);
    Ok(())
}

#[cfg(not(unix))]
fn set_executable_permission(_file_path: &std::path::Path) -> Result<(), std::io::Error> {
    // Windows 系统下不需要设置执行权限
    Ok(())
}

// 清理内核目录，只保留可执行文件
fn cleanup_kernel_directory(kernel_dir: &std::path::Path, executable_name: &str) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("清理内核目录，只保留可执行文件: {}", executable_name);

    if let Ok(entries) = std::fs::read_dir(kernel_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            // 跳过可执行文件本身
            if path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| name == executable_name)
                .unwrap_or(false)
            {
                continue;
            }

            // 删除其他所有文件和目录
            if path.is_file() {
                if let Err(e) = std::fs::remove_file(&path) {
                    warn!("删除文件失败 {:?}: {}", path, e);
                } else {
                    info!("删除文件: {:?}", path);
                }
            } else if path.is_dir() {
                if let Err(e) = std::fs::remove_dir_all(&path) {
                    warn!("删除目录失败 {:?}: {}", path, e);
                } else {
                    info!("删除目录: {:?}", path);
                }
            }
        }
    }

    info!("内核目录清理完成");
    Ok(())
}

// 安装内核
#[tauri::command]
pub async fn install_kernel() -> Result<(), String> {
    // 目前先返回成功，表示安装完成
    info!("内核安装完成");
    Ok(())
}

// 启动内核（带重试机制的完整版本）
#[tauri::command]
pub async fn start_kernel(app_handle: AppHandle, api_port: Option<u16>) -> Result<String, String> {
    let kernel_path = paths::get_kernel_path();
    let config_path = paths::get_config_path();

    if !kernel_path.exists() {
        return Err(messages::ERR_KERNEL_NOT_FOUND.to_string());
    }

    if !config_path.exists() {
        return Err("配置文件不存在".to_string());
    }

    // 检查内核是否已经在运行
    if is_kernel_running().await.unwrap_or(false) {
        warn!("内核已在运行中");
        
        // 如果内核已在运行，检查事件中继是否需要启动
        if let Some(port) = api_port {
            info!("内核已运行，检查并启动事件中继...");
            match start_websocket_relay(app_handle.clone(), Some(port)).await {
                Ok(_) => info!("✅ 事件中继启动成功"),
                Err(e) => warn!("⚠️ 事件中继启动失败: {}", e),
            }
        }
        
        return Ok("内核已在运行中".to_string());
    }

    // 带重试机制的内核启动
    let max_attempts = 3;
    let mut last_error = String::new();
    
    for attempt in 1..=max_attempts {
        info!("🚀 尝试启动内核，第 {}/{} 次", attempt, max_attempts);
        
        // 启动内核进程
        match PROCESS_MANAGER.start().await {
            Ok(_) => {
                info!("✅ 内核进程启动成功");
                
                // 等待内核启动并检查状态
                let mut kernel_ready = false;
                
                // 多次检查内核是否真正运行起来
                for check_attempt in 1..=5 {
                    tokio::time::sleep(Duration::from_secs(1)).await;
                    
                    if is_kernel_running().await.unwrap_or(false) {
                        info!("✅ 内核确认正在运行（第{}次检查）", check_attempt);
                        kernel_ready = true;
                        break;
                    } else {
                        warn!("⏳ 内核尚未就绪，第{}次检查", check_attempt);
                    }
                }
                
                if kernel_ready {
                    // 自动启动事件中继
                    if let Some(port) = api_port {
                        info!("🔌 自动启动事件中继服务...");
                        match start_websocket_relay(app_handle.clone(), Some(port)).await {
                            Ok(_) => {
                                info!("✅ 事件中继启动成功");
                                
                                // 发送内核就绪事件到前端
                                if let Err(e) = app_handle.emit("kernel-ready", true) {
                                    error!("发送内核就绪事件失败: {}", e);
                                }
                                
                                // 通知内核就绪
                                KERNEL_READY_NOTIFY.notify_waiters();
                                
                                return Ok("内核启动成功".to_string());
                            },
                            Err(e) => {
                                error!("❌ 事件中继启动失败: {}", e);
                                last_error = format!("内核启动成功，但事件中继启动失败: {}", e);
                                // 事件中继失败，尝试停止内核并重试
                                if let Err(stop_err) = PROCESS_MANAGER.stop().await {
                                    error!("停止内核失败: {}", stop_err);
                                }
                            }
                        }
                    } else {
                        // 没有API端口，但内核已启动
                        KERNEL_READY_NOTIFY.notify_waiters();
                        return Ok("内核启动成功（未启动事件中继）".to_string());
                    }
                } else {
                    last_error = "内核进程启动后未能稳定运行".to_string();
                    warn!("❌ 内核进程启动后未能稳定运行");
                    // 尝试停止可能损坏的进程
                    if let Err(stop_err) = PROCESS_MANAGER.stop().await {
                        error!("停止内核失败: {}", stop_err);
                    }
                }
            },
            Err(e) => {
                last_error = format!("{}: {}", messages::ERR_PROCESS_START_FAILED, e);
                error!("❌ 内核启动失败: {}", e);
            }
        }
        
        // 如果不是最后一次尝试，等待后重试
        if attempt < max_attempts {
            warn!("⏳ 第{}次启动失败，{}秒后重试...", attempt, 2 * attempt);
            tokio::time::sleep(Duration::from_secs(2 * attempt as u64)).await;
        }
    }
    
    error!("❌ 内核启动失败，已尝试{}次: {}", max_attempts, last_error);
    Err(last_error)
}

// 停止内核
#[tauri::command]
pub async fn stop_kernel() -> Result<String, String> {
    // 停止事件中继
    SHOULD_STOP_EVENTS.store(true, Ordering::Relaxed);
    cleanup_event_relay_tasks().await;

    // 停止内核进程
    PROCESS_MANAGER
        .stop()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_PROCESS_STOP_FAILED, e))?;

    // 等待进程完全停止
    tokio::time::sleep(Duration::from_secs(2)).await;

    if !is_kernel_running().await.unwrap_or(true) {
        info!("✅ 内核停止成功");
        Ok("内核停止成功".to_string())
    } else {
        Err(messages::ERR_PROCESS_STOP_FAILED.to_string())
    }
}

// 重启内核
#[tauri::command]
pub async fn restart_kernel(app_handle: AppHandle, api_port: Option<u16>) -> Result<String, String> {
    stop_kernel().await?;
    tokio::time::sleep(Duration::from_secs(3)).await;
    start_kernel(app_handle, api_port).await
}

/// 启动事件中继服务（增强版本，优化开机自启动场景）
#[tauri::command]
pub async fn start_websocket_relay(
    app_handle: AppHandle,
    api_port: Option<u16>,
) -> Result<(), String> {
    // 要求前端必须传递API端口，不使用硬编码默认值
    let port = api_port.ok_or("API端口参数是必需的，请从前端传递正确的端口配置")?;

    // 重置停止标志
    SHOULD_STOP_EVENTS.store(false, Ordering::Relaxed);

    // 清理旧任务
    cleanup_event_relay_tasks().await;

    info!("🔌 开始启动事件中继服务，端口: {}", port);

    // 增加更长的等待时间，特别是在开机自启动时
    let wait_time = if is_system_recently_started().await {
        info!("🕐 检测到系统刚启动，增加事件中继启动等待时间");
        Duration::from_secs(5)
    } else {
        Duration::from_secs(2)
    };
    
    tokio::time::sleep(wait_time).await;

    // 获取API token
    let token = crate::app::core::proxy_service::get_api_token();

    // 启动各种数据流的事件中继
    let traffic_relay = create_traffic_event_relay(app_handle.clone(), port, token.clone());
    let memory_relay = create_memory_event_relay(app_handle.clone(), port, token.clone());
    let log_relay = create_log_event_relay(app_handle.clone(), port, token.clone());
    let connection_relay = create_connection_event_relay(app_handle.clone(), port, token);

    // 启动事件中继任务（带增强的重试机制）
    let traffic_task = tokio::task::spawn(async move {
        if let Err(e) = start_event_relay_with_retry(traffic_relay, "traffic").await {
            error!("流量事件中继启动失败: {}", e);
        }
    });

    let memory_task = tokio::task::spawn(async move {
        if let Err(e) = start_event_relay_with_retry(memory_relay, "memory").await {
            error!("内存事件中继启动失败: {}", e);
        }
    });

    let log_task = tokio::task::spawn(async move {
        if let Err(e) = start_event_relay_with_retry(log_relay, "logs").await {
            error!("日志事件中继启动失败: {}", e);
        }
    });

    let connection_task = tokio::task::spawn(async move {
        if let Err(e) = start_event_relay_with_retry(connection_relay, "connections").await {
            error!("连接事件中继启动失败: {}", e);
        }
    });

    // 将任务添加到管理器
    {
        let mut tasks = EVENT_RELAY_TASKS.lock().await;
        tasks.push(traffic_task);
        tasks.push(memory_task);
        tasks.push(log_task);
        tasks.push(connection_task);
    }

    // 发送内核就绪事件
    let _ = app_handle.emit("kernel-ready", ());

    Ok(())
}

/// 检查系统是否最近启动（用于判断是否是开机自启动场景）
async fn is_system_recently_started() -> bool {
    // 简单的系统启动时间检查
    match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
        Ok(uptime) => {
            // 这是一个简化的检查，实际可能需要更精确的系统启动时间获取
            // 这里假设如果进程运行时间很短，可能是开机自启动
            uptime.as_secs() < 300 // 5分钟内认为是最近启动
        }
        Err(_) => false,
    }
}

// 检查内核是否正在运行 (跨平台实现)
#[tauri::command]
pub async fn is_kernel_running() -> Result<bool, String> {
    #[cfg(target_os = "windows")]
    {
        is_kernel_running_windows().await
    }

    #[cfg(target_os = "linux")]
    {
        is_kernel_running_linux().await
    }

    #[cfg(not(any(target_os = "windows", target_os = "linux")))]
    {
        Err("当前平台不支持内核状态检查".to_string())
    }
}

#[cfg(target_os = "windows")]
async fn is_kernel_running_windows() -> Result<bool, String> {
    // 获取我们的内核可执行文件路径
    let kernel_path = crate::app::constants::core::paths::get_kernel_path();

    info!("检查内核进程，可执行文件路径: {:?}", kernel_path);

    // 方法1: 通过tasklist命令检查精确的进程
    let kernel_filename = kernel_path.file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("sing-box.exe");

    let mut cmd = tokio::process::Command::new("tasklist");
    cmd.args(&["/FI", "IMAGENAME eq", kernel_filename, "/FO", "CSV", "/NH"]);
    cmd.creation_flags(crate::app::constants::process::CREATE_NO_WINDOW);

    if let Ok(output) = cmd.output().await {
        let stdout = String::from_utf8_lossy(&output.stdout);
        if stdout.contains(kernel_filename) {
            info!("内核进程正在运行 (tasklist检测): {}", kernel_filename);
            return Ok(true);
        }
    }

    // 方法2: 使用wmic检查进程
    if let Ok(output) = tokio::process::Command::new("wmic")
        .args(&["process", "where", "name='sing-box.exe'"])
        .output()
        .await
    {
        if !output.stdout.is_empty() {
            info!("内核进程正在运行 (wmic检测): true");
            return Ok(true);
        }
    }

    // 方法3: 使用PowerShell Get-Process
    if let Ok(output) = tokio::process::Command::new("powershell")
        .args(&["-Command", "Get-Process sing-box -ErrorAction SilentlyContinue"])
        .output()
        .await
    {
        if output.status.success() {
            info!("内核进程正在运行 (PowerShell检测): true");
            return Ok(true);
        }
    }

    info!("内核运行状态检查: false (未找到相关进程)");
    Ok(false)
}

#[cfg(target_os = "linux")]
async fn is_kernel_running_linux() -> Result<bool, String> {
    // 获取我们的内核工作目录
    let kernel_dir = crate::app::constants::core::paths::get_kernel_work_dir();
    let kernel_path = crate::app::constants::core::paths::get_kernel_path();

    info!("检查内核进程，可执行文件路径: {:?}", kernel_path);
    info!("内核工作目录: {:?}", kernel_dir);

    // 方法1: 检查我们的可执行文件是否被某个进程使用
    if let Ok(output) = tokio::process::Command::new("lsof")
        .arg(&kernel_path)
        .output()
        .await
    {
        if !output.stdout.is_empty() {
            info!("内核进程正在运行 (lsof检测): {}", output.status.success());
            return Ok(true);
        }
    }

    // 方法2: 使用 pgrep 检查特定路径的进程
    if let Ok(output) = tokio::process::Command::new("pgrep")
        .args(&["-f", &kernel_path.to_string_lossy()])
        .output()
        .await
    {
        if !output.stdout.is_empty() {
            info!("内核进程正在运行 (pgrep检测): {}", !output.stdout.is_empty());
            return Ok(true);
        }
    }

    // 方法3: 检查进程命令行是否包含我们的工作目录
    if let Ok(output) = tokio::process::Command::new("ps")
        .args(&["-ef", "-o", "args="])
        .output()
        .await
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let kernel_dir_str = kernel_dir.to_string_lossy();
        let kernel_path_str = kernel_path.to_string_lossy();

        if stdout.contains(&*kernel_dir_str) || stdout.contains(&*kernel_path_str) {
            info!("内核进程正在运行 (ps检测): true");
            return Ok(true);
        }
    }

    // 方法4: 最后用简单检查，但加上路径验证
    if let Ok(output) = tokio::process::Command::new("pgrep")
        .arg("sing-box")
        .output()
        .await
    {
        if !output.stdout.is_empty() {
            let stdout = String::from_utf8_lossy(&output.stdout);
            let pids: Vec<&str> = stdout.trim().split('\n').collect();

            let kernel_path_str = kernel_path.to_string_lossy();
            for pid in pids {
                if let Ok(cmdline_output) = tokio::process::Command::new("ps")
                    .args(&["-p", pid, "-o", "cmd="])
                    .output()
                    .await
                {
                    let cmdline = String::from_utf8_lossy(&cmdline_output.stdout);
                    if cmdline.contains(&*kernel_path_str) {
                        info!("内核进程正在运行 (精确匹配): PID {}, 命令: {}", pid, cmdline.trim());
                        return Ok(true);
                    }
                }
            }
        }
    }

    info!("内核运行状态检查: false (未找到相关进程)");
    Ok(false)
}

// 检查内核完整状态（进程 + API）
#[tauri::command]
pub async fn check_kernel_status(api_port: Option<u16>) -> Result<serde_json::Value, String> {
    // 要求前端必须传递API端口，不使用硬编码默认值
    let port = api_port.ok_or("API端口参数是必需的，请从前端传递正确的端口配置")?;

    let process_running = is_kernel_running().await.unwrap_or(false);

    let mut status = serde_json::json!({
        "process_running": process_running,
        "api_ready": false,
        "websocket_ready": false
    });

    if process_running {
        // 检查API是否可用
        let client = http_client::get_client();
        let api_url = format!("http://127.0.0.1:{}/version?token=", port);

        let api_ready = match client
            .get(&api_url)
            .timeout(Duration::from_secs(2))
            .send()
            .await
        {
            Ok(response) if response.status().is_success() => true,
            _ => false,
        };

        status["api_ready"] = serde_json::Value::Bool(api_ready);

        // 如果API可用，检查WebSocket
        if api_ready {
            let token = crate::app::core::proxy_service::get_api_token();
            let ws_ready = check_websocket_endpoints_ready(port, &token).await;
            status["websocket_ready"] = serde_json::Value::Bool(ws_ready);
        }
    }

    info!("内核完整状态: {}", status);
    Ok(status)
}

/// 清理事件中继任务
async fn cleanup_event_relay_tasks() {
    // 设置停止标志
    SHOULD_STOP_EVENTS.store(true, Ordering::Relaxed);

    // 等待所有任务完成
    let mut tasks = EVENT_RELAY_TASKS.lock().await;
    
    for task in tasks.drain(..) {
        task.abort();
    }
    
    info!("已清理所有事件中继任务");
}

/// 检查WebSocket端点是否就绪
async fn check_websocket_endpoints_ready(api_port: u16, token: &str) -> bool {
    use tokio_tungstenite::connect_async;
    use url::Url;

    let endpoints = ["traffic", "memory", "logs", "connections"];
    
    for endpoint in &endpoints {
        let url_str = format!("ws://127.0.0.1:{}/{}?token={}", api_port, endpoint, token);
        
        match Url::parse(&url_str) {
            Ok(url) => {
                match tokio::time::timeout(
                    Duration::from_secs(3),
                    connect_async(url)
                ).await {
                    Ok(Ok((ws_stream, _))) => {
                        // 连接成功，立即关闭
                        drop(ws_stream);
                        info!("✅ {} 端点就绪", endpoint);
                    }
                    _ => {
                        warn!("❌ {} 端点未就绪", endpoint);
                        return false;
                    }
                }
            }
            Err(_) => {
                warn!("❌ {} 端点URL解析失败", endpoint);
                return false;
            }
        }
    }
    
    true
}

/// 获取系统运行时间（毫秒）
#[tauri::command]
pub async fn get_system_uptime() -> Result<u64, String> {
    #[cfg(windows)]
    {
        // 使用Windows API获取系统运行时间
        let mut cmd = tokio::process::Command::new("powershell");
        cmd.args(&[
            "-Command",
            "(Get-Date) - (Get-CimInstance -ClassName Win32_OperatingSystem).LastBootUpTime | Select-Object -ExpandProperty TotalMilliseconds"
        ]);
        cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);

        match cmd.output().await
        {
            Ok(output) => {
                if output.status.success() {
                    let uptime_str = String::from_utf8_lossy(&output.stdout);
                    let uptime_ms: f64 = uptime_str.trim().parse().unwrap_or(0.0);
                    Ok(uptime_ms as u64)
                } else {
                    // 如果PowerShell失败，使用更简单的方法
                    warn!("PowerShell获取系统时间失败，使用备用方法");
                    // 使用性能计数器
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
    
    #[cfg(not(windows))]
    {
        // 对于非Windows系统，使用/proc/uptime
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
}

// ========== 新增的重构版本命令 ==========

/// 重构版本的启动命令 - 增强版
#[tauri::command]
pub async fn kernel_start_enhanced(app_handle: AppHandle, proxy_mode: Option<String>, api_port: Option<u16>) -> Result<serde_json::Value, String> {
    info!("🚀 启动内核增强版，代理模式: {:?}, API端口: {:?}", proxy_mode, api_port);

    // 检查内核是否已在运行
    if is_kernel_running().await.unwrap_or(false) {
        info!("内核已在运行中");
        return Ok(serde_json::json!({
            "success": true,
            "message": "内核已在运行中".to_string()
        }));
    }

    // 启动内核进程
    match PROCESS_MANAGER.start().await {
        Ok(_) => {
            info!("✅ 内核进程启动成功");

            // 如果提供了API端口，尝试启动事件中继
            if let Some(port) = api_port {
                info!("🔌 启动事件中继服务，端口: {}", port);
                match start_websocket_relay(app_handle.clone(), Some(port)).await {
                    Ok(_) => {
                        info!("✅ 事件中继启动成功");

                        // 发送内核就绪事件
                        let _ = app_handle.emit("kernel-ready", ());

                        Ok(serde_json::json!({
                            "success": true,
                            "message": "内核启动成功，事件中继已启动".to_string()
                        }))
                    }
                    Err(e) => {
                        warn!("⚠️ 事件中继启动失败: {}, 但内核进程已启动", e);

                        // 即使事件中继失败，内核也已经启动了
                        let _ = app_handle.emit("kernel-ready", ());

                        Ok(serde_json::json!({
                            "success": true,
                            "message": "内核启动成功，但事件中继启动失败".to_string()
                        }))
                    }
                }
            } else {
                // 没有提供API端口，只发送内核就绪事件
                let _ = app_handle.emit("kernel-ready", ());
                Ok(serde_json::json!({
                    "success": true,
                    "message": "内核启动成功".to_string()
                }))
            }
        }
        Err(e) => {
            error!("❌ 内核启动失败: {}", e);
            Ok(serde_json::json!({
                "success": false,
                "message": format!("内核启动失败: {}", e)
            }))
        }
    }
}

/// 重构版本的停止命令 - 增强版
#[tauri::command]
pub async fn kernel_stop_enhanced() -> Result<serde_json::Value, String> {
    info!("🛑 停止内核增强版");

    match stop_kernel().await {
        Ok(_) => Ok(serde_json::json!({
            "success": true,
            "message": "内核停止成功".to_string()
        })),
        Err(e) => Ok(serde_json::json!({
            "success": false,
            "message": format!("内核停止失败: {}", e)
        })),
    }
}

/// 重构版本的状态查询命令 - 增强版
#[tauri::command]
pub async fn kernel_get_status_enhanced(api_port: Option<u16>) -> Result<serde_json::Value, String> {
    // 使用传递的端口或默认端口12081（与AppStore默认值保持一致）
    let port = api_port.unwrap_or(12081);

    let process_running = is_kernel_running().await?;
    let mut api_ready = false;
    let mut websocket_ready = false;
    let mut error = None;

    if process_running {
        // 检查API状态
        let client = http_client::get_client();
        let api_url = format!("http://127.0.0.1:{}/version", port);

        api_ready = match client.get(&api_url).timeout(Duration::from_secs(2)).send().await {
            Ok(response) if response.status().is_success() => true,
            Ok(response) => {
                error = Some(format!("API返回错误状态码: {}", response.status()));
                false
            },
            Err(e) => {
                error = Some(format!("API连接失败: {}", e));
                false
            }
        };

        // 检查WebSocket状态（简化版）
        if api_ready {
            let token = crate::app::core::proxy_service::get_api_token();
            let url_str = format!("ws://127.0.0.1:{}/traffic?token={}", port, token);

            // 使用超时连接WebSocket
            websocket_ready = tokio::time::timeout(
                Duration::from_secs(3),
                tokio_tungstenite::connect_async(&url_str)
            ).await.is_ok();

            if !websocket_ready && error.is_none() {
                error = Some("WebSocket连接失败".to_string());
            }
        }

        // 如果进程运行但API不可用，提供详细错误信息
        if !api_ready && error.is_none() {
            error = Some("内核进程运行中但API服务不可用".to_string());
        }
    }

    Ok(serde_json::json!({
        "process_running": process_running,
        "api_ready": api_ready,
        "websocket_ready": websocket_ready,
        "uptime_ms": 0,
        "version": null,
        "error": error
    }))
}

/// 健康检查命令
#[tauri::command]
pub async fn kernel_check_health(api_port: Option<u16>) -> Result<serde_json::Value, String> {
    let mut issues = Vec::new();
    let mut healthy = true;

    // 检查内核文件
    let kernel_path = paths::get_kernel_path();
    if !kernel_path.exists() {
        issues.push("内核文件不存在".to_string());
        healthy = false;
    }

    // 检查配置文件
    let config_path = paths::get_config_path();
    if !config_path.exists() {
        issues.push("配置文件不存在".to_string());
        healthy = false;
    }

    // 检查进程状态
    let process_running = is_kernel_running().await.unwrap_or(false);
    if process_running {
        // 使用传递的端口或默认端口12081
        let port = api_port.unwrap_or(12081);
        let client = http_client::get_client();
        let api_url = format!("http://127.0.0.1:{}/version", port);
        
        let api_ready = match client.get(&api_url).timeout(Duration::from_secs(2)).send().await {
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