use crate::app::constants::{common::messages, paths};
use crate::app::core::event_relay::{
    create_connection_event_relay, create_log_event_relay, create_memory_event_relay,
    create_traffic_event_relay, start_event_relay_with_retry,
};
use crate::app::core::proxy_service::{
    apply_proxy_runtime_state, update_dns_strategy, ProxyRuntimeState,
};
use crate::app::core::tun_profile::TunProxyOptions;
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use crate::app::storage::state_model::AppConfig;
use crate::process::manager::ProcessManager;
use crate::utils::http_client;
use serde::Serialize;
use serde_json::json;
use std::process::Command;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::sync::Arc;
use std::time::Duration;
use tauri::Manager;
use tauri::{AppHandle, Emitter};
use tokio::sync::{Mutex, Notify};
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

static KEEP_ALIVE_ENABLED: AtomicBool = AtomicBool::new(false);
static GUARDED_API_PORT: AtomicU16 = AtomicU16::new(0);

lazy_static::lazy_static! {
    static ref KERNEL_GUARD_HANDLE: Mutex<Option<JoinHandle<()>>> = Mutex::new(None);
}

async fn enable_kernel_guard(app_handle: AppHandle, api_port: u16) {
    GUARDED_API_PORT.store(api_port, Ordering::Relaxed);
    if KEEP_ALIVE_ENABLED.swap(true, Ordering::Relaxed) {
        return;
    }

    let mut handle_slot = KERNEL_GUARD_HANDLE.lock().await;
    let guard_handle = tokio::spawn(async move {
        info!("内核守护已启动");
        loop {
            if !KEEP_ALIVE_ENABLED.load(Ordering::Relaxed) {
                break;
            }

            tokio::time::sleep(Duration::from_secs(8)).await;

            if !KEEP_ALIVE_ENABLED.load(Ordering::Relaxed) {
                break;
            }

            match is_kernel_running().await {
                Ok(true) => {
                    continue;
                }
                _ => {
                    info!("守护检测到内核停止，尝试自动重启...");
                    
                    // 发送内核停止事件
                    let _ = app_handle.emit("kernel-stopped", json!({
                        "process_running": false,
                        "api_ready": false,
                        "websocket_ready": false
                    }));
                    let _ = app_handle.emit("kernel-status-changed", json!({
                        "process_running": false,
                        "api_ready": false,
                        "websocket_ready": false
                    }));
                    
                    if let Err(err) = PROCESS_MANAGER.start().await {
                        warn!("守护重启内核失败: {}", err);
                        continue;
                    }

                    let port_value = GUARDED_API_PORT.load(Ordering::Relaxed);
                    if port_value > 0 {
                        if let Err(e) =
                            start_websocket_relay(app_handle.clone(), Some(port_value)).await
                        {
                            warn!("守护启动事件中继失败: {}", e);
                        }
                    }

                    // 发送内核已启动事件
                    let _ = app_handle.emit("kernel-started", json!({
                        "process_running": true,
                        "api_ready": true,
                        "auto_restarted": true
                    }));
                    let _ = app_handle.emit("kernel-status-changed", json!({
                        "process_running": true,
                        "api_ready": true,
                        "websocket_ready": true
                    }));
                    let _ = app_handle.emit("kernel-ready", ());
                }
            }
        }

        info!("内核守护任务结束");
    });

    *handle_slot = Some(guard_handle);
}

async fn disable_kernel_guard() {
    if !KEEP_ALIVE_ENABLED.swap(false, Ordering::Relaxed) {
        return;
    }

    GUARDED_API_PORT.store(0, Ordering::Relaxed);
    let mut handle_slot = KERNEL_GUARD_HANDLE.lock().await;
    if let Some(handle) = handle_slot.take() {
        handle.abort();
    }
}

// 获取最新内核版本号
async fn get_latest_kernel_version() -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    use serde::Deserialize;

    // GitHub Release API 响应结构
    #[derive(Deserialize)]
    struct GitHubRelease {
        tag_name: String,
    }

    // 设置下载超时和更好的用户代理
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10)) // 10秒超时
        .user_agent("sing-box-windows/1.8.2")
        .build()?;

    // 使用多个 API 源获取版本信息
    let api_urls = vec![
        // 使用 GitHub API（原始）
        "https://api.github.com/repos/SagerNet/sing-box/releases/latest",
        // 使用 gh-proxy 加速的 GitHub API
        "https://v6.gh-proxy.com/https://api.github.com/repos/SagerNet/sing-box/releases/latest",
        // 使用 gh-proxy 镜像的 GitHub API
        "https://gh-proxy.com/https://api.github.com/repos/SagerNet/sing-box/releases/latest",
    ];

    for (index, api_url) in api_urls.iter().enumerate() {
        info!("尝试第 {} 个 API 源获取版本: {}", index + 1, api_url);

        match client.get(*api_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let release: GitHubRelease = response.json().await?;
                    let tag_name = release.tag_name;

                    // 去掉 'v' 前缀，只保留版本号
                    let version = if tag_name.starts_with('v') {
                        tag_name[1..].to_string()
                    } else {
                        tag_name
                    };

                    info!("成功获取版本号: {} (来源: {})", version, api_url);
                    return Ok(version);
                } else {
                    warn!(
                        "API 返回错误状态: {} (来源: {})",
                        response.status(),
                        api_url
                    );
                }
            }
            Err(e) => {
                warn!("API 请求失败: {} (来源: {})", e, api_url);
            }
        }
    }

    Err("所有 API 源都获取版本失败".into())
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

    let output = cmd
        .output()
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
    cmd.arg("check").arg("--config").arg(path);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);

    let output = cmd
        .output()
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
    } else if cfg!(target_os = "macos") {
        // macOS 架构检测
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

    let window = app_handle
        .get_webview_window("main")
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
    } else if cfg!(target_os = "macos") {
        "darwin"
    } else {
        return Err("当前平台不支持".to_string());
    };

    let arch = get_system_arch();

    // 记录检测到的架构信息
    info!("检测到平台: {}, 架构: {}", platform, arch);

    // 获取最新版本号
    let version = match get_latest_kernel_version().await {
        Ok(v) => {
            info!("获取到最新版本号: {}", v);
            v
        }
        Err(e) => {
            warn!("获取最新版本失败: {}, 使用默认版本 1.12.10", e);
            "1.12.10".to_string()
        }
    };

    // 构造下载 URL - 使用 sing-box 官方的文件命名格式
    // 根据官方文件列表，格式为：sing-box-{version}-{platform}-{arch}.tar.gz 或 .zip
    let filename = if cfg!(target_os = "windows") {
        format!("sing-box-{}-windows-{}.zip", version, arch) // Windows 使用 .zip
    } else if cfg!(target_os = "macos") {
        format!("sing-box-{}-darwin-{}.tar.gz", version, arch) // macOS 使用 .tar.gz
    } else {
        format!("sing-box-{}-linux-{}.tar.gz", version, arch) // Linux 使用 .tar.gz
    };

    // 使用多个下载源以提高成功率
    let download_urls = vec![
        // 使用 v6.gh-proxy 镜像（新增）
        format!(
            "https://v6.gh-proxy.com/https://github.com/SagerNet/sing-box/releases/download/v{}/{}",
            version, filename
        ),
        // 使用 gh-proxy 镜像
        format!(
            "https://gh-proxy.com/https://github.com/SagerNet/sing-box/releases/download/v{}/{}",
            version, filename
        ),
        // 使用 GitHub 快速加速镜像（优先）
        format!(
            "https://ghfast.top/https://github.com/SagerNet/sing-box/releases/download/v{}/{}",
            version, filename
        ),
        // 使用 GitHub 加速镜像（国内用户）
        format!(
            "https://hub.fastgit.xyz/SagerNet/sing-box/releases/download/v{}/{}",
            version, filename
        ),
        // 使用 GitLab 镜像
        format!(
            "https://hub.fgit.cf/SagerNet/sing-box/releases/download/v{}/{}",
            version, filename
        ),
        // 使用 jsdelivr CDN
        format!(
            "https://cdn.jsdelivr.net/gh/SagerNet/sing-box@releases/download/v{}/{}",
            version, filename
        ),
        // 原始 GitHub 链接（备用）
        format!(
            "https://github.com/SagerNet/sing-box/releases/download/v{}/{}",
            version, filename
        ),
    ];

    // 记录下载信息
    info!("内核版本: {}", version);
    info!("平台: {}, 架构: {}", platform, arch);
    info!("文件名: {}", filename);
    info!("主要下载 URL (v6.gh-proxy 加速): {}", download_urls[0]);
    info!("备用下载源 1 (gh-proxy): {}", download_urls[1]);
    info!("备用下载源 2 (ghfast.top): {}", download_urls[2]);
    info!("备用下载源 3 (hub.fastgit.xyz): {}", download_urls[3]);
    info!("备用下载源 4 (hub.fgit.cf): {}", download_urls[4]);
    info!("备用下载源 5 (jsdelivr CDN): {}", download_urls[5]);
    info!("备用下载源 6 (GitHub 原始): {}", download_urls[6]);
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
                let source_name = match index {
                    0 => "v6.gh-proxy 镜像",
                    1 => "gh-proxy 镜像",
                    2 => "ghfast.top 加速",
                    3 => "hub.fastgit.xyz",
                    4 => "hub.fgit.cf",
                    5 => "jsdelivr CDN",
                    6 => "GitHub 原始",
                    _ => "未知源",
                };

                let error_details = format!("{} 失败: {}", source_name, e);
                let error_msg = format!("下载源 {} 失败: {}", source_name, e);
                warn!("{}", error_msg);

                // 发送详细的失败信息到前端
                let _ = window.emit(
                    "kernel-download-progress",
                    json!({
                        "status": "downloading",
                        "progress": 15 + (index * 5),
                        "message": format!("⚠️ {} - 尝试下一个下载源...", error_details)
                    }),
                );

                // 删除部分下载的文件
                let _ = std::fs::remove_file(&download_path);

                // 如果不是最后一个下载源，继续尝试
                if index < download_urls.len() - 1 {
                    continue;
                }

                // 所有下载源都失败，汇总所有错误信息
                let final_error = format!(
                    "所有下载源都已失败。最后尝试的 {} 也失败了。请检查网络连接或稍后重试。",
                    source_name
                );

                let _ = window.emit(
                    "kernel-download-progress",
                    json!({
                        "status": "error",
                        "progress": 0,
                        "message": final_error
                    }),
                );

                return Err(final_error);
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

    info!("开始查找可执行文件: {}", executable_name);

    // 查找可执行文件（可能在子目录中）
    let found_executable_path = find_executable_file(&kernel_dir, executable_name).await?;

    // 将可执行文件迁移到正确位置（kernel_dir/sing-box 或 kernel_dir/sing-box.exe）
    let target_executable_path = kernel_dir.join(executable_name);

    // 如果找到的文件不在目标位置，需要移动
    if found_executable_path != target_executable_path {
        info!(
            "迁移内核文件从 {:?} 到 {:?}",
            found_executable_path, target_executable_path
        );

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

    auto_manage_with_saved_config(&app_handle, true, "kernel-download").await;

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

    // 根据文件扩展名决定解压方式
    let file_extension = archive_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("");

    info!("文件扩展名: {}", file_extension);

    if file_extension == "zip" {
        // Windows ZIP 格式解压
        extract_zip_archive(archive_path, extract_to).await?;
    } else if file_extension == "gz" || archive_path.to_string_lossy().ends_with(".tar.gz") {
        // Linux TAR.GZ 格式解压
        extract_tar_gz_archive(archive_path, extract_to).await?;
    } else {
        return Err(format!("不支持的压缩格式: {}", file_extension).into());
    }

    // 列出解压后的文件（用于调试）
    if let Ok(entries) = std::fs::read_dir(extract_to) {
        info!("解压后的文件:");
        for entry in entries.flatten() {
            info!("  - {:?}", entry.path());
        }
    }

    Ok(())
}

// 解压 ZIP 文件（Windows）
async fn extract_zip_archive(
    archive_path: &std::path::Path,
    extract_to: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use zip::ZipArchive;

    info!("解压 ZIP 文件: {:?}", archive_path);

    let file = std::fs::File::open(archive_path)?;
    let mut zip = ZipArchive::new(file)?;

    // 确保解压目录存在
    if !extract_to.exists() {
        std::fs::create_dir_all(extract_to)?;
    }

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let file_path = extract_to.join(file.name());

        // 跳过目录条目
        if file.name().ends_with('/') {
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            continue;
        }

        // 确保父目录存在
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }

        let mut output_file = std::fs::File::create(&file_path)?;
        std::io::copy(&mut file, &mut output_file)?;
    }

    info!("ZIP 文件解压完成");
    Ok(())
}

// 解压 TAR.GZ 文件（Linux）
async fn extract_tar_gz_archive(
    archive_path: &std::path::Path,
    extract_to: &std::path::Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use flate2::read::GzDecoder;
    use std::fs::File;
    use tar::Archive;

    info!("解压 TAR.GZ 文件: {:?}", archive_path);

    let file = File::open(archive_path)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    // 确保解压目录存在
    if !extract_to.exists() {
        std::fs::create_dir_all(extract_to)?;
    }

    // 解压所有文件
    match archive.unpack(extract_to) {
        Ok(_) => {
            info!("TAR.GZ 文件解压完成");
        }
        Err(e) => {
            return Err(format!("TAR.GZ 解压失败: {}", e).into());
        }
    }

    Ok(())
}

// 查找可执行文件的辅助函数
async fn find_executable_file(
    search_dir: &std::path::Path,
    executable_name: &str,
) -> Result<std::path::PathBuf, String> {
    info!(
        "在目录 {:?} 中查找可执行文件: {}",
        search_dir, executable_name
    );

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
            if path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name == executable_name)
                .unwrap_or(false)
                && path.is_file()
            // 确保是文件而不是目录
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
        return Err(format!(
            "未找到可执行文件: {} 在目录 {:?} 中",
            executable_name, search_dir
        ));
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
fn cleanup_kernel_directory(
    kernel_dir: &std::path::Path,
    executable_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("清理内核目录，只保留可执行文件: {}", executable_name);

    if let Ok(entries) = std::fs::read_dir(kernel_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            // 跳过可执行文件本身
            if path
                .file_name()
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
                            }
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
            }
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
    disable_kernel_guard().await;
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
pub async fn restart_kernel(
    app_handle: AppHandle,
    api_port: Option<u16>,
) -> Result<String, String> {
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
    // 首先检查内部进程管理器，这是最准确的
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
    // 获取我们的内核可执行文件路径
    let kernel_path = crate::app::constants::core::paths::get_kernel_path();

    info!("检查内核进程，可执行文件路径: {:?}", kernel_path);

    // 方法1: 通过tasklist命令检查精确的进程
    let kernel_filename = kernel_path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("sing-box.exe");

    let mut cmd = tokio::process::Command::new("tasklist");
    // 修复参数格式：/FI 后面的过滤器必须作为一个完整的字符串参数
    cmd.args(&["/FI", &format!("IMAGENAME eq {}", kernel_filename), "/FO", "CSV", "/NH"]);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::app::constants::process::CREATE_NO_WINDOW);

    if let Ok(output) = cmd.output().await {
        let stdout = String::from_utf8_lossy(&output.stdout);
        // 解析 CSV 格式: "Image Name","PID","Session Name","Session#","Mem Usage"
        // "sing-box.exe","1234","Console","1","12,345 K"
        for line in stdout.lines() {
            if line.contains(kernel_filename) {
                // 简单的包含检查可能不够，最好是分割 CSV
                let parts: Vec<&str> = line.split(',').collect();
                if let Some(name) = parts.first() {
                    // 去除引号
                    let clean_name = name.trim_matches('"');
                    if clean_name == kernel_filename {
                        info!("内核进程正在运行 (tasklist检测): {}", kernel_filename);
                        return Ok(true);
                    }
                }
            }
        }
    }

    // 方法2: 使用wmic检查进程
    {
        let mut cmd = tokio::process::Command::new("wmic");
        cmd.args(&["process", "where", "name='sing-box.exe'"]);

        #[cfg(target_os = "windows")]
        cmd.creation_flags(crate::app::constants::process::CREATE_NO_WINDOW);

        if let Ok(output) = cmd.output().await {
            if !output.stdout.is_empty() {
                info!("内核进程正在运行 (wmic检测): true");
                return Ok(true);
            }
        }
    }

    // 方法3: 移除 PowerShell 检测，因为可能在某些 Windows 版本上不可用或被禁用
    // 如果前两种方法都失败了，通常意味着进程确实没有运行，或者系统环境受到严重限制

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
            info!(
                "内核进程正在运行 (pgrep检测): {}",
                !output.stdout.is_empty()
            );
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
                        info!(
                            "内核进程正在运行 (精确匹配): PID {}, 命令: {}",
                            pid,
                            cmdline.trim()
                        );
                        return Ok(true);
                    }
                }
            }
        }
    }

    info!("内核运行状态检查: false (未找到相关进程)");
    Ok(false)
}

#[cfg(target_os = "macos")]
async fn is_kernel_running_macos() -> Result<bool, String> {
    // 获取我们的内核工作目录
    let kernel_dir = crate::app::constants::core::paths::get_kernel_work_dir();
    let kernel_path = crate::app::constants::core::paths::get_kernel_path();

    info!("检查内核进程，可执行文件路径: {:?}", kernel_path);
    info!("内核工作目录: {:?}", kernel_dir);

    // 方法1: 使用 lsof 检查我们的可执行文件是否被某个进程使用
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
            info!(
                "内核进程正在运行 (pgrep检测): {}",
                !output.stdout.is_empty()
            );
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

    // 方法4: 使用 ps aux 检查 sing-box 进程
    if let Ok(output) = tokio::process::Command::new("ps")
        .args(&["aux"])
        .output()
        .await
    {
        let stdout = String::from_utf8_lossy(&output.stdout);
        let kernel_path_str = kernel_path.to_string_lossy();

        if stdout.contains("sing-box") && stdout.contains(&*kernel_path_str) {
            info!("内核进程正在运行 (ps aux检测): true");
            return Ok(true);
        }
    }

    // 方法5: 最后用简单检查，但加上路径验证
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
                    .args(&["-p", pid, "-o", "command="])
                    .output()
                    .await
                {
                    let cmdline = String::from_utf8_lossy(&cmdline_output.stdout);
                    if cmdline.contains(&*kernel_path_str) {
                        info!(
                            "内核进程正在运行 (精确匹配): PID {}, 命令: {}",
                            pid,
                            cmdline.trim()
                        );
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
                match tokio::time::timeout(Duration::from_secs(3), connect_async(url)).await {
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

        #[cfg(target_os = "windows")]
        cmd.creation_flags(crate::app::constants::process::CREATE_NO_WINDOW);

        match cmd.output().await {
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

    #[cfg(target_os = "linux")]
    {
        // 对于Linux系统，使用/proc/uptime
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

    #[cfg(target_os = "macos")]
    {
        // 对于macOS系统，使用sysctl命令获取系统运行时间
        let mut cmd = tokio::process::Command::new("sysctl");
        cmd.args(&["-n", "kern.boottime"]);

        match cmd.output().await {
            Ok(output) => {
                if output.status.success() {
                    let boottime_str = String::from_utf8_lossy(&output.stdout);
                    // 输出格式类似: { sec = 1699123456, usec = 123456 }
                    if let Some(sec_part) = boottime_str.split("sec = ").nth(1) {
                        if let Some(timestamp) = sec_part.split(',').next() {
                            if let Ok(boot_timestamp) = timestamp.trim().parse::<u64>() {
                                // 获取当前时间戳
                                let current_timestamp = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();

                                // 计算运行时间（毫秒）
                                let uptime_seconds =
                                    current_timestamp.saturating_sub(boot_timestamp);
                                return Ok(uptime_seconds * 1000);
                            }
                        }
                    }
                }
                // 如果sysctl失败，尝试使用uptime命令
                match tokio::process::Command::new("uptime").output().await {
                    Ok(uptime_output) if uptime_output.status.success() => {
                        let uptime_str = String::from_utf8_lossy(&uptime_output.stdout);
                        // 解析uptime输出，提取运行时间
                        info!("uptime输出: {}", uptime_str);
                        Ok(0) // 简化处理，返回0
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

// ========== 新增的重构版本命令 ==========

#[derive(Debug, Clone, Default)]
struct ProxyOverrides {
    proxy_mode: Option<String>,
    api_port: Option<u16>,
    proxy_port: Option<u16>,
    prefer_ipv6: Option<bool>,
    system_proxy_bypass: Option<String>,
    tun_options: Option<TunProxyOptions>,
    system_proxy_enabled: Option<bool>,
    tun_enabled: Option<bool>,
    keep_alive: Option<bool>,
}

#[derive(Debug, Clone)]
struct ResolvedProxyState {
    proxy: ProxyRuntimeState,
    api_port: u16,
    prefer_ipv6: bool,
    auto_start_kernel: bool,
}

impl ResolvedProxyState {
    fn derived_mode(&self) -> String {
        self.proxy.derived_mode()
    }
}

async fn resolve_proxy_runtime_state(
    app_handle: &AppHandle,
    overrides: ProxyOverrides,
) -> Result<ResolvedProxyState, String> {
    let mut app_config = db_get_app_config(app_handle.clone()).await?;

    if let Some(api_port) = overrides.api_port {
        app_config.api_port = api_port;
    }
    if let Some(proxy_port) = overrides.proxy_port {
        app_config.proxy_port = proxy_port;
    }
    if let Some(prefer_ipv6) = overrides.prefer_ipv6 {
        app_config.prefer_ipv6 = prefer_ipv6;
    }

    if let Some(proxy_mode) = overrides.proxy_mode {
        match proxy_mode.as_str() {
            "system" => {
                app_config.system_proxy_enabled = true;
                app_config.tun_enabled = false;
            }
            "tun" => {
                app_config.system_proxy_enabled = false;
                app_config.tun_enabled = true;
            }
            _ => {
                app_config.system_proxy_enabled = false;
                app_config.tun_enabled = false;
            }
        }
    }

    if let Some(enabled) = overrides.system_proxy_enabled {
        app_config.system_proxy_enabled = enabled;
    }
    if let Some(enabled) = overrides.tun_enabled {
        app_config.tun_enabled = enabled;
    }

    let tun_options = overrides.tun_options.unwrap_or_else(|| TunProxyOptions {
        ipv4_address: app_config.tun_ipv4.clone(),
        ipv6_address: app_config.tun_ipv6.clone(),
        mtu: app_config.tun_mtu,
        auto_route: app_config.tun_auto_route,
        strict_route: app_config.tun_strict_route,
        stack: app_config.tun_stack.clone(),
        enable_ipv6: app_config.tun_enable_ipv6,
        interface_name: None,
    });

    let proxy_state = ProxyRuntimeState {
        proxy_port: app_config.proxy_port,
        system_proxy_enabled: app_config.system_proxy_enabled,
        tun_enabled: app_config.tun_enabled,
        system_proxy_bypass: overrides
            .system_proxy_bypass
            .unwrap_or_else(|| app_config.system_proxy_bypass.clone()),
        tun_options,
    };

    Ok(ResolvedProxyState {
        proxy: proxy_state,
        api_port: app_config.api_port,
        prefer_ipv6: app_config.prefer_ipv6,
        auto_start_kernel: app_config.auto_start_kernel,
    })
}

async fn start_kernel_with_state(
    app_handle: AppHandle,
    resolved: &ResolvedProxyState,
    keep_alive_enabled: bool,
) -> Result<serde_json::Value, String> {
    info!(
        "🚀 启动内核增强版，代理模式: {}, API端口: {}, 代理端口: {}",
        resolved.derived_mode(),
        resolved.api_port,
        resolved.proxy.proxy_port
    );

    let _ = app_handle.emit("kernel-starting", json!({
        "proxy_mode": resolved.derived_mode(),
        "api_port": resolved.api_port,
        "proxy_port": resolved.proxy.proxy_port
    }));

    crate::app::system::config_service::ensure_singbox_config()
        .map_err(|e| format!("准备内核配置失败: {}", e))?;
    if let Err(e) = crate::app::system::config_service::update_singbox_ports(
        resolved.proxy.proxy_port,
        resolved.api_port,
    ) {
        warn!("更新端口配置失败: {}", e);
    }

    if let Err(e) = apply_proxy_runtime_state(&resolved.proxy) {
        return Ok(json!({
            "success": false,
            "message": format!("应用代理配置失败: {}", e)
        }));
    }

    if let Err(e) = update_dns_strategy(resolved.prefer_ipv6) {
        warn!("更新DNS策略失败: {}", e);
    }

    if is_kernel_running().await.unwrap_or(false) {
        if keep_alive_enabled {
            enable_kernel_guard(app_handle.clone(), resolved.api_port).await;
        } else {
            disable_kernel_guard().await;
        }
        info!("内核已在运行中");
        return Ok(serde_json::json!({
            "success": true,
            "message": "内核已在运行中".to_string()
        }));
    }

    match PROCESS_MANAGER.start().await {
        Ok(_) => {
            info!("✅ 内核进程启动成功");

            info!("🔌 启动事件中继服务，端口: {}", resolved.api_port);
            match start_websocket_relay(app_handle.clone(), Some(resolved.api_port)).await {
                Ok(_) => {
                    info!("✅ 事件中继启动成功");

                    if keep_alive_enabled {
                        enable_kernel_guard(app_handle.clone(), resolved.api_port).await;
                    } else {
                        disable_kernel_guard().await;
                    }

                    let _ = app_handle.emit("kernel-ready", ());
                    let _ = app_handle.emit("kernel-started", json!({
                        "proxy_mode": resolved.derived_mode(),
                        "api_port": resolved.api_port,
                        "proxy_port": resolved.proxy.proxy_port,
                        "process_running": true,
                        "api_ready": true
                    }));
                    let _ = app_handle.emit("kernel-status-changed", json!({
                        "process_running": true,
                        "api_ready": true,
                        "websocket_ready": true
                    }));

                    Ok(serde_json::json!({
                        "success": true,
                        "message": "内核启动成功，事件中继已启动".to_string()
                    }))
                }
                Err(e) => {
                    warn!("⚠️ 事件中继启动失败: {}, 但内核进程已启动", e);

                    if keep_alive_enabled {
                        enable_kernel_guard(app_handle.clone(), resolved.api_port).await;
                    } else {
                        disable_kernel_guard().await;
                    }

                    let _ = app_handle.emit("kernel-ready", ());

                    Ok(serde_json::json!({
                        "success": true,
                        "message": "内核启动成功，但事件中继启动失败".to_string()
                    }))
                }
            }
        }
        Err(e) => {
            error!("❌ 内核启动失败: {}", e);

            let _ = app_handle.emit("kernel-error", json!({
                "error": format!("启动失败: {}", e)
            }));

            Ok(serde_json::json!({
                "success": false,
                "message": format!("内核启动失败: {}", e)
            }))
        }
    }
}

/// 重构版本的启动命令 - 增强版
#[tauri::command]
pub async fn kernel_start_enhanced(
    app_handle: AppHandle,
    proxy_mode: Option<String>,
    api_port: Option<u16>,
    proxy_port: Option<u16>,
    prefer_ipv6: Option<bool>,
    system_proxy_bypass: Option<String>,
    tun_options: Option<TunProxyOptions>,
    keep_alive: Option<bool>,
    system_proxy_enabled: Option<bool>,
    tun_enabled: Option<bool>,
) -> Result<serde_json::Value, String> {
    let overrides = ProxyOverrides {
        proxy_mode,
        api_port,
        proxy_port,
        prefer_ipv6,
        system_proxy_bypass,
        tun_options,
        system_proxy_enabled,
        tun_enabled,
        keep_alive,
    };

    let resolved = resolve_proxy_runtime_state(&app_handle, overrides.clone()).await?;
    let keep_alive_enabled = overrides.keep_alive.unwrap_or(resolved.auto_start_kernel);

    start_kernel_with_state(app_handle, &resolved, keep_alive_enabled).await
}

/// 仅应用代理配置，不进行内核重启
#[tauri::command]
pub async fn apply_proxy_settings(
    app_handle: AppHandle,
    system_proxy_enabled: Option<bool>,
    tun_enabled: Option<bool>,
) -> Result<serde_json::Value, String> {
    let overrides = ProxyOverrides {
        system_proxy_enabled,
        tun_enabled,
        ..Default::default()
    };

    let resolved = resolve_proxy_runtime_state(&app_handle, overrides).await?;

    if let Err(e) = apply_proxy_runtime_state(&resolved.proxy) {
        return Ok(json!({
            "success": false,
            "message": format!("应用代理配置失败: {}", e)
        }));
    }

    if let Err(e) = update_dns_strategy(resolved.prefer_ipv6) {
        warn!("更新DNS策略失败: {}", e);
    }

    Ok(json!({
        "success": true,
        "mode": resolved.derived_mode(),
        "system_proxy_enabled": resolved.proxy.system_proxy_enabled,
        "tun_enabled": resolved.proxy.tun_enabled
    }))
}

/// 重构版本的停止命令 - 增强版
#[tauri::command]
pub async fn kernel_stop_enhanced(app_handle: AppHandle) -> Result<serde_json::Value, String> {
    info!("🛑 停止内核增强版");

    disable_kernel_guard().await;

    match stop_kernel().await {
        Ok(_) => {
            // 发送内核已停止事件
            let _ = app_handle.emit("kernel-stopped", json!({
                "process_running": false,
                "api_ready": false,
                "websocket_ready": false
            }));
            
            // 发送内核状态变化事件
            let _ = app_handle.emit("kernel-status-changed", json!({
                "process_running": false,
                "api_ready": false,
                "websocket_ready": false
            }));
            
            Ok(serde_json::json!({
                "success": true,
                "message": "内核停止成功".to_string()
            }))
        },
        Err(e) => {
            // 发送内核错误事件
            let _ = app_handle.emit("kernel-error", json!({
                "error": format!("停止失败: {}", e)
            }));
            
            Ok(serde_json::json!({
                "success": false,
                "message": format!("内核停止失败: {}", e)
            }))
        },
    }
}

/// 后台停止内核：快速返回，具体停止逻辑在后台执行
#[tauri::command]
pub async fn kernel_stop_background(app_handle: AppHandle) -> Result<serde_json::Value, String> {
    info!("🛑 后台请求停止内核（快速返回）");

    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        // 为停止设置超时，超时后尝试强制清理进程
        let stop_result = tokio::time::timeout(Duration::from_secs(6), stop_kernel()).await;
        match stop_result {
            Ok(Ok(_)) => {
                info!("✅ 后台停止内核完成");
            }
            Ok(Err(e)) => {
                error!("❌ 后台停止内核失败: {}", e);
                let _ = handle.emit("kernel-error", json!({
                    "error": format!("停止失败: {}", e)
                }));
            }
            Err(_) => {
                warn!("⏳ 停止内核超时，尝试强制清理");
                if let Err(e) = PROCESS_MANAGER.kill_existing_processes().await {
                    error!("强制清理内核进程失败: {}", e);
                }
            }
        }

        // 无论结果如何，发送停止/状态事件，便于前端同步
        let _ = handle.emit("kernel-stopped", json!({
            "process_running": false,
            "api_ready": false,
            "websocket_ready": false
        }));
        let _ = handle.emit("kernel-status-changed", json!({
            "process_running": false,
            "api_ready": false,
            "websocket_ready": false
        }));
    });

    Ok(json!({
        "success": true,
        "message": "已在后台请求停止内核"
    }))
}

/// 强制停止内核并退出应用：快速响应，后台执行停止逻辑
#[tauri::command]
pub async fn force_stop_and_exit(app_handle: AppHandle) -> Result<serde_json::Value, String> {
    info!("🛑 收到强制退出请求，后台停止内核并退出应用");

    let handle = app_handle.clone();
    tauri::async_runtime::spawn(async move {
        // 停止事件转发
        SHOULD_STOP_EVENTS.store(true, Ordering::Relaxed);
        cleanup_event_relay_tasks().await;

        // 尝试正常停止，超时则强杀
        let stop_result = tokio::time::timeout(Duration::from_secs(4), stop_kernel()).await;
        match stop_result {
            Ok(Ok(_)) => info!("✅ 内核正常停止"),
            Ok(Err(e)) => warn!("停止内核失败，尝试强制清理: {}", e),
            Err(_) => warn!("停止内核超时，尝试强制清理"),
        }

        // 强制兜底清理内核进程
        if let Err(e) = PROCESS_MANAGER.kill_existing_processes().await {
            error!("强制清理内核进程失败: {}", e);
        }

        // 通知前端状态（若仍在运行）
        let _ = handle.emit("kernel-stopped", json!({
            "process_running": false,
            "api_ready": false,
            "websocket_ready": false
        }));
        let _ = handle.emit("kernel-status-changed", json!({
            "process_running": false,
            "api_ready": false,
            "websocket_ready": false
        }));

        // 退出应用
        handle.exit(0);
    });

    Ok(json!({
        "success": true,
        "message": "正在后台停止内核并退出"
    }))
}

/// 重构版本的状态查询命令 - 增强版
#[tauri::command]
pub async fn kernel_get_status_enhanced(
    api_port: Option<u16>,
) -> Result<serde_json::Value, String> {
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

        // 检查WebSocket状态（简化版）
        if api_ready {
            let token = crate::app::core::proxy_service::get_api_token();
            let url_str = format!("ws://127.0.0.1:{}/traffic?token={}", port, token);

            // 使用超时连接WebSocket
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

        // 如果进程运行但API不可用，提供详细错误信息
        if !api_ready && error.is_none() {
            error = Some("内核进程运行中但API服务不可用".to_string());
        }
    }

    // 获取版本信息
    let version = if process_running {
        // 如果进程正在运行，尝试从API获取版本
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
        // 如果进程没有运行，尝试直接从内核文件获取版本
        match check_kernel_version().await {
            Ok(v) => Some(v.trim().to_string()),
            Err(_) => None,
        }
    };

    Ok(serde_json::json!({
        "process_running": process_running,
        "api_ready": api_ready,
        "websocket_ready": websocket_ready,
        "uptime_ms": 0,
        "version": version,
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

#[derive(Debug, Clone)]
struct AutoManageOptions {
    proxy_mode: Option<String>,
    api_port: Option<u16>,
    proxy_port: Option<u16>,
    prefer_ipv6: Option<bool>,
    system_proxy_bypass: Option<String>,
    tun_options: Option<TunProxyOptions>,
    system_proxy_enabled: Option<bool>,
    tun_enabled: Option<bool>,
    keep_alive: Option<bool>,
    force_restart: bool,
}

impl AutoManageOptions {
    fn from_app_config(config: AppConfig) -> Self {
        AutoManageOptions {
            proxy_mode: Some(config.proxy_mode.clone()),
            api_port: Some(config.api_port),
            proxy_port: Some(config.proxy_port),
            prefer_ipv6: Some(config.prefer_ipv6),
            system_proxy_bypass: Some(config.system_proxy_bypass.clone()),
            tun_options: Some(TunProxyOptions {
                ipv4_address: config.tun_ipv4.clone(),
                ipv6_address: config.tun_ipv6.clone(),
                mtu: config.tun_mtu,
                auto_route: config.tun_auto_route,
                strict_route: config.tun_strict_route,
                stack: config.tun_stack.clone(),
                enable_ipv6: config.tun_enable_ipv6,
                interface_name: None,
            }),
            system_proxy_enabled: Some(config.system_proxy_enabled),
            tun_enabled: Some(config.tun_enabled),
            keep_alive: Some(config.auto_start_kernel),
            force_restart: false,
        }
    }

    fn to_overrides(&self) -> ProxyOverrides {
        ProxyOverrides {
            proxy_mode: self.proxy_mode.clone(),
            api_port: self.api_port,
            proxy_port: self.proxy_port,
            prefer_ipv6: self.prefer_ipv6,
            system_proxy_bypass: self.system_proxy_bypass.clone(),
            tun_options: self.tun_options.clone(),
            system_proxy_enabled: self.system_proxy_enabled,
            tun_enabled: self.tun_enabled,
            keep_alive: self.keep_alive,
        }
    }
}

#[derive(Debug, Serialize)]
struct AutoManageResult {
    state: String,
    message: String,
    kernel_installed: bool,
    config_ready: bool,
    attempted_start: bool,
    last_start_message: Option<String>,
}

impl AutoManageResult {
    fn new(
        state: &str,
        message: impl Into<String>,
        kernel_installed: bool,
        config_ready: bool,
        attempted_start: bool,
        last_start_message: Option<String>,
    ) -> Self {
        AutoManageResult {
            state: state.to_string(),
            message: message.into(),
            kernel_installed,
            config_ready,
            attempted_start,
            last_start_message,
        }
    }

    fn missing_kernel() -> Self {
        AutoManageResult::new(
            "missing_kernel",
            "未检测到内核，请先下载内核",
            false,
            false,
            false,
            None,
        )
    }

    fn missing_config() -> Self {
        AutoManageResult::new(
            "missing_config",
            "未检测到配置，请先添加订阅或导入配置",
            true,
            false,
            false,
            None,
        )
    }

    fn invalid_config(message: String) -> Self {
        AutoManageResult::new(
            "invalid_config",
            format!("配置文件校验失败: {}", message),
            true,
            false,
            false,
            None,
        )
    }

    fn running(message: impl Into<String>, attempted: bool, last_message: Option<String>) -> Self {
        AutoManageResult::new(
            "running",
            message.into(),
            true,
            true,
            attempted,
            last_message,
        )
    }

    fn error(message: impl Into<String>, attempted: bool) -> Self {
        AutoManageResult::new(
            "error",
            message.into(),
            true,
            true,
            attempted,
            None,
        )
    }
}

fn kernel_binary_exists() -> bool {
    paths::get_kernel_path().exists()
}

fn kernel_config_exists() -> bool {
    paths::get_config_path().exists()
}

async fn auto_manage_kernel_internal(
    app_handle: AppHandle,
    options: AutoManageOptions,
) -> Result<AutoManageResult, String> {
    let resolved_state = resolve_proxy_runtime_state(&app_handle, options.to_overrides()).await?;
    let keep_alive_enabled = options
        .keep_alive
        .unwrap_or(resolved_state.auto_start_kernel);
    let api_port = resolved_state.api_port;

    let kernel_installed = kernel_binary_exists();
    if !kernel_installed {
        return Ok(AutoManageResult::missing_kernel());
    }

    let config_ready = kernel_config_exists();
    if !config_ready {
        return Ok(AutoManageResult::missing_config());
    }

    if let Err(err) = check_config_validity(String::new()).await {
        return Ok(AutoManageResult::invalid_config(err));
    }

    let mut _attempted_start = false;

    if let Err(e) = apply_proxy_runtime_state(&resolved_state.proxy) {
        warn!("自动管理应用代理配置失败: {}", e);
    }

    let mut running = is_kernel_running().await.unwrap_or(false);
    if options.force_restart && running {
        info!("自动管理请求触发内核重启");
        let _ = stop_kernel().await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        running = is_kernel_running().await.unwrap_or(false);
    }

    if !running {
        _attempted_start = true;
        let start_response =
            start_kernel_with_state(app_handle.clone(), &resolved_state, keep_alive_enabled)
                .await?;

        let success = start_response
            .get("success")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let message = start_response
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("内核启动状态未知")
            .to_string();

        if success {
            Ok(AutoManageResult::running(
                message.clone(),
                true,
                Some(message),
            ))
        } else {
            Ok(AutoManageResult::error(message, true))
        }
    } else {
        if keep_alive_enabled {
            enable_kernel_guard(app_handle.clone(), api_port).await;
        } else {
            disable_kernel_guard().await;
        }
        Ok(AutoManageResult::running(
            "内核已在运行中".to_string(),
            false,
            None,
        ))
    }
}

pub async fn auto_manage_with_saved_config(
    app_handle: &AppHandle,
    force_restart: bool,
    reason: &str,
) {
    match db_get_app_config(app_handle.clone()).await {
        Ok(config) => {
            if !config.auto_start_kernel && !force_restart {
                info!(
                    "自动管理({})跳过：auto_start_kernel 已禁用，确保守护已关闭",
                    reason
                );
                disable_kernel_guard().await;
                return;
            }

            let mut options = AutoManageOptions::from_app_config(config);
            options.force_restart = force_restart;

            match auto_manage_kernel_internal(app_handle.clone(), options).await {
                Ok(result) => {
                    info!(
                        "自动管理({})完成，状态: {}, 信息: {}",
                        reason, result.state, result.message
                    );
                }
                Err(err) => {
                    warn!("自动管理({})失败: {}", reason, err);
                }
            }
        }
        Err(err) => {
            warn!(
                "加载应用配置失败，跳过自动管理({}): {}",
                reason, err
            );
        }
    }
}

#[tauri::command]
pub async fn kernel_auto_manage(
    app_handle: AppHandle,
    proxy_mode: Option<String>,
    api_port: Option<u16>,
    proxy_port: Option<u16>,
    prefer_ipv6: Option<bool>,
    system_proxy_bypass: Option<String>,
    tun_options: Option<TunProxyOptions>,
    keep_alive: Option<bool>,
    system_proxy_enabled: Option<bool>,
    tun_enabled: Option<bool>,
    force_restart: Option<bool>,
) -> Result<serde_json::Value, String> {
    let options = AutoManageOptions {
        proxy_mode,
        api_port,
        proxy_port,
        prefer_ipv6,
        system_proxy_bypass,
        tun_options,
        keep_alive,
        system_proxy_enabled,
        tun_enabled,
        force_restart: force_restart.unwrap_or(false),
    };

    let result = auto_manage_kernel_internal(app_handle, options).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}
