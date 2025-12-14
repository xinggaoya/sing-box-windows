use crate::app::constants::{api, messages};
use crate::app::network_config;
use crate::utils::app_util::get_work_dir_sync;
use semver::Version;
use serde_json::json;
use std::path::Path;
use tauri::{Emitter, Manager};
use std::env;

// 获取当前平台标识符 - 使用 Rust 标准库，更准确
fn get_platform_identifier() -> &'static str {
    env::consts::OS
}

// 获取当前架构
fn get_current_arch() -> &'static str {
    env::consts::ARCH
}

// 检查文件是否匹配当前平台
fn is_platform_compatible(filename: &str) -> bool {
    let platform = get_platform_identifier();
    let arch = get_current_arch();

    // 只支持桌面平台
    let extension_match = match platform {
        "windows" => filename.ends_with(".msi") || filename.ends_with(".exe"),
        "linux" => filename.ends_with(".AppImage") || filename.ends_with(".deb"),
        "macos" => filename.ends_with(".dmg") || filename.ends_with(".app.tar.gz"),
        _ => false,
    };

    if !extension_match {
        return false;
    }

    // 检查架构兼容性
    check_arch_compatibility(filename, arch)
}

// 检查架构兼容性（仅桌面平台）
fn check_arch_compatibility(filename: &str, current_arch: &str) -> bool {
    let filename_lower = filename.to_lowercase();

    match current_arch {
        "x86_64" => {
            // x64 架构优先选择 x64 包，也接受通用包
            filename_lower.contains("x64") ||
            filename_lower.contains("x86_64") ||
            filename_lower.contains("amd64") ||
            !filename_lower.contains("arm") // 没有架构标识时默认兼容
        }
        "aarch64" => {
            // ARM64 Mac 优先选择 ARM64 或 Universal 包
            filename_lower.contains("arm64") ||
            filename_lower.contains("aarch64") ||
            filename_lower.contains("universal")
        }
        "arm" | "armv7" => {
            // ARM32
            filename_lower.contains("arm32") ||
            filename_lower.contains("armv7") ||
            (filename_lower.contains("arm") && !filename_lower.contains("64"))
        }
        "x86" => {
            // 32位 x86
            filename_lower.contains("i386") ||
            filename_lower.contains("386") ||
            (filename_lower.contains("x86") && !filename_lower.contains("64"))
        }
        _ => true, // 其他架构保守处理
    }
}


// 获取平台优先级分数（用于选择最合适的安装包）
fn get_platform_priority(filename: &str) -> i32 {
    let platform = get_platform_identifier();
    let arch = get_current_arch();
    let filename_lower = filename.to_lowercase();

    // 基础优先级（桌面平台）
    let base_priority = match platform {
        "windows" => {
            if filename.ends_with(".exe") { 20 }
            else if filename.ends_with(".msi") { 10 }
            else { 0 }
        }
        "linux" => {
            if filename.ends_with(".deb") { 20 }
            else if filename.ends_with(".AppImage") { 10 }
            else { 0 }
        }
        "macos" => {
            if filename.ends_with(".dmg") { 20 }
            else if filename.ends_with(".app.tar.gz") { 10 }
            else { 0 }
        }
        _ => 0,
    };

    if base_priority == 0 {
        return 0;
    }

    // 架构匹配加分
    let arch_bonus = match arch {
        "x86_64" => {
            if filename_lower.contains("x64") || filename_lower.contains("x86_64") || filename_lower.contains("amd64") {
                5
            } else {
                0
            }
        }
        "aarch64" => {
            if filename_lower.contains("arm64") || filename_lower.contains("aarch64") {
                5
            } else if filename_lower.contains("universal") {
                4  // macOS Universal 包
            } else {
                0
            }
        }
        _ => 0,
    };

    // 特殊标识加分
    let special_bonus = if filename_lower.contains("portable") {
        2
    } else if filename_lower.contains("installer") || filename_lower.contains("latest") {
        1
    } else {
        0
    };

    base_priority + arch_bonus + special_bonus
}

// 更新信息结构体
#[derive(serde::Serialize, Debug)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub download_url: String,
    pub has_update: bool,
    pub release_notes: Option<String>,
    pub release_date: Option<String>,
    pub file_size: Option<u64>,
    pub is_prerelease: bool,
}

// 版本比较函数
fn compare_versions(current: &str, latest: &str) -> bool {
    // 清理版本号，移除 'v' 前缀和其他非版本信息
    let clean_current = current
        .trim_start_matches('v')
        .split_whitespace()
        .next()
        .unwrap_or(current);
    let clean_latest = latest
        .trim_start_matches('v')
        .split_whitespace()
        .next()
        .unwrap_or(latest);

    // 尝试使用 semver 进行比较
    match (Version::parse(clean_current), Version::parse(clean_latest)) {
        (Ok(curr), Ok(lat)) => lat > curr,
        _ => {
            // 如果无法解析为语义版本，则进行字符串比较
            clean_latest != clean_current
        }
    }
}

// 检查更新
#[tauri::command]
pub async fn check_update(
    current_version: String,
    include_prerelease: Option<bool>,
) -> Result<UpdateInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(
            network_config::HTTP_TIMEOUT_SECONDS,
        ))
        .no_proxy() // 禁用代理
        .build()
        .map_err(|e| format!("{}: {}", messages::ERR_HTTP_CLIENT_FAILED, e))?;

    let include_prerelease = include_prerelease.unwrap_or(false);

    // 根据是否包含预发布版本选择不同的API端点
    let api_url = if include_prerelease {
        // 获取所有版本（包括预发布版本），然后筛选最新的
        "https://api.github.com/repos/xinggaoya/sing-box-windows/releases"
    } else {
        // 只获取最新的正式版本
        api::GITHUB_API_URL
    };

    // 获取版本信息
    let response = client
        .get(api_url)
        .header("User-Agent", api::USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_GET_VERSION_FAILED, e))?;

    let release: serde_json::Value = if include_prerelease {
        // 获取所有版本，取第一个（最新的）
        let releases: Vec<serde_json::Value> = response
            .json()
            .await
            .map_err(|e| format!("{}: {}", messages::ERR_GET_VERSION_FAILED, e))?;

        if releases.is_empty() {
            return Err(format!(
                "{}: 无法获取版本列表",
                messages::ERR_GET_VERSION_FAILED
            ));
        }

        releases[0].clone()
    } else {
        response
            .json()
            .await
            .map_err(|e| format!("{}: {}", messages::ERR_GET_VERSION_FAILED, e))?
    };

    // 获取最新版本号
    let tag_name = release["tag_name"]
        .as_str()
        .ok_or_else(|| format!("{}: 无法解析版本号", messages::ERR_GET_VERSION_FAILED))
        .map(|v| v.trim_start_matches('v').to_string())?;

    // 获取发布说明
    let release_notes = release["body"].as_str().map(|s| s.to_string());

    // 获取发布日期
    let release_date = release["published_at"].as_str().map(|s| s.to_string());

    // 检查是否为预发布版本
    let is_prerelease = release["prerelease"].as_bool().unwrap_or(false);

    // 获取下载链接和文件大小
    let assets = release["assets"]
        .as_array()
        .ok_or_else(|| format!("{}: 无法获取下载资源", messages::ERR_GET_VERSION_FAILED))?;

    // 根据当前平台查找对应的安装程序
    let mut download_url = String::new();
    let mut file_size: Option<u64> = None;
    let mut best_priority = 0;

    // 遍历所有资源，找到最适合当前平台的安装包
    for asset in assets {
        let name = asset["name"].as_str().unwrap_or("");

        // 检查文件是否与当前平台兼容
        if is_platform_compatible(name) {
            let priority = get_platform_priority(name);

            // 选择优先级最高的安装包
            if priority > best_priority {
                download_url = asset["browser_download_url"]
                    .as_str()
                    .unwrap_or("")
                    .to_string();
                file_size = asset["size"].as_u64();
                best_priority = priority;

                // 如果找到了最高优先级的包，可以提前退出
                if priority == 2 {
                    break;
                }
            }
        }
    }

    if download_url.is_empty() {
        return Err(format!(
            "{}: 无法获取下载链接",
            messages::ERR_GET_VERSION_FAILED
        ));
    }

    // 使用改进的版本比较
    let has_update = compare_versions(&current_version, &tag_name);

    Ok(UpdateInfo {
        latest_version: tag_name.to_string(),
        download_url,
        has_update,
        release_notes,
        release_date,
        file_size,
        is_prerelease,
    })
}

// 下载更新
#[tauri::command]
pub async fn download_update(app_handle: tauri::AppHandle) -> Result<(), String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or("无法获取主窗口")?;

    // 这里可以实现实际的下载逻辑
    // 目前先发送一个模拟的完成事件
    let _ = window.emit(
        "update-progress",
        json!({
            "status": "completed",
            "progress": 100,
            "message": "下载功能待实现"
        }),
    );

    Ok(())
}


// 获取当前平台信息（简化版，兼容旧接口）
#[tauri::command]
pub async fn get_platform_info() -> Result<String, String> {
    Ok(get_platform_identifier().to_string())
}

// 获取详细的平台信息（包括操作系统和架构）
#[tauri::command]
pub async fn get_detailed_platform_info() -> Result<PlatformDetailedInfo, String> {
    Ok(PlatformDetailedInfo::current())
}

// 详细平台信息结构体
#[derive(Debug, Clone, serde::Serialize)]
pub struct PlatformDetailedInfo {
    pub os: String,           // 操作系统：windows, linux, macos
    pub arch: String,         // 架构：x86_64, aarch64, etc.
    pub display_name: String, // 显示名称：Windows x64, macOS ARM64 等
}

impl PlatformDetailedInfo {
    pub fn current() -> Self {
        let os = env::consts::OS.to_string();
        let arch = env::consts::ARCH.to_string();

        // 生成友好的显示名称
        let display_name = match (os.as_str(), arch.as_str()) {
            ("windows", "x86_64") => "Windows x64".to_string(),
            ("windows", "x86") => "Windows x86".to_string(),
            ("windows", "aarch64") => "Windows ARM64".to_string(),
            ("linux", "x86_64") => "Linux x64".to_string(),
            ("linux", "x86") => "Linux x86".to_string(),
            ("linux", "aarch64") => "Linux ARM64".to_string(),
            ("linux", "arm") => "Linux ARM".to_string(),
            ("macos", "x86_64") => "macOS Intel".to_string(),
            ("macos", "aarch64") => "macOS Apple Silicon".to_string(),
            ("macos", "arm") => "macOS ARM".to_string(),
            _ => format!("{} ({})", os, arch),
        };

        Self {
            os,
            arch,
            display_name,
        }
    }
}

// 下载并安装更新
#[tauri::command]
pub async fn download_and_install_update(
    app_handle: tauri::AppHandle,
    download_url: String,
) -> Result<(), String> {
    let window = app_handle
        .get_webview_window("main")
        .ok_or("无法获取主窗口")?;
    let work_dir = get_work_dir_sync();

    // 根据下载链接和平台确定下载文件名
    let update_filename = if download_url.ends_with(".msi") {
        "update.msi"
    } else if download_url.ends_with(".exe") {
        "update.exe"
    } else if download_url.ends_with(".AppImage") {
        "update.AppImage"
    } else if download_url.ends_with(".deb") {
        "update.deb"
    } else if download_url.ends_with(".dmg") {
        "update.dmg"
    } else if download_url.ends_with(".app.tar.gz") {
        "update.app.tar.gz"
    } else {
        // 根据平台使用默认扩展名
        match get_platform_identifier() {
            "windows" => "update.exe",
            "linux" => "update.AppImage",
            "macos" => "update.dmg",
            _ => "update.bin",
        }
    };

    let download_path = Path::new(&work_dir).join(update_filename);

    // 发送开始下载事件
    let _ = window.emit(
        "update-progress",
        json!({
            "status": "downloading",
            "progress": 0,
            "message": "开始下载更新..."
        }),
    );

    // 下载更新文件
    let window_clone = window.clone();
    // 使用fallback下载函数
    if let Err(e) = crate::utils::file_util::download_with_fallback(
        &download_url,
        download_path.to_str().unwrap(),
        move |progress| {
            let _ = window_clone.emit(
                "update-progress",
                json!({
                    "status": "downloading",
                    "progress": progress,
                    "message": format!("正在下载: {}%", progress)
                }),
            );
        },
    )
    .await
    {
        let _ = window.emit(
            "update-progress",
            json!({
                "status": "error",
                "progress": 0,
                "message": format!("下载失败: {}", e)
            }),
        );
        return Err(format!("下载更新失败: {}", e));
    }

    // 验证下载的文件
    if !download_path.exists() {
        let error_msg = "下载的文件不存在";
        let _ = window.emit(
            "update-progress",
            json!({
                "status": "error",
                "progress": 0,
                "message": error_msg
            }),
        );
        return Err(error_msg.to_string());
    }

    // 发送下载完成事件
    let _ = window.emit(
        "update-progress",
        json!({
            "status": "completed",
            "progress": 100,
            "message": "下载完成，准备安装..."
        }),
    );

    // 启动安装程序（在后台运行）
    let install_result = match get_platform_identifier() {
        "windows" => {
            // Windows: 根据文件类型选择不同的处理方式
            if download_url.ends_with(".msi") {
                // MSI文件: 使用 msiexec 安装
                let mut cmd = tokio::process::Command::new("msiexec");
                cmd.arg("/i").arg(&download_path).arg("/passive");
                #[cfg(target_os = "windows")]
                cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);
                cmd.spawn()
            } else if download_url.ends_with(".exe") {
                // EXE文件: 直接运行
                let mut cmd = tokio::process::Command::new(&download_path);
                #[cfg(target_os = "windows")]
                cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);
                cmd.spawn()
            } else {
                // 其他文件：尝试用默认方式运行
                let mut cmd = tokio::process::Command::new(&download_path);
                #[cfg(target_os = "windows")]
                cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);
                cmd.spawn()
            }
        }
        "linux" => {
            // Linux: 根据文件类型执行不同的安装逻辑
            if download_url.ends_with(".AppImage") {
                // AppImage: 添加执行权限并运行
                let mut chmod_cmd = tokio::process::Command::new("chmod");
                chmod_cmd.arg("+x").arg(&download_path);
                chmod_cmd.spawn().and_then(|_| {
                    let mut run_cmd = tokio::process::Command::new(&download_path);
                    run_cmd.spawn()
                })
            } else if download_url.ends_with(".deb") {
                // DEB包: 使用pkexec安装（需要管理员权限）
                let mut cmd = tokio::process::Command::new("pkexec");
                cmd.arg("dpkg")
                    .arg("-i")
                    .arg(&download_path)
                    .arg("--force-architecture");
                cmd.spawn()
            } else {
                // 其他二进制文件
                let mut cmd = tokio::process::Command::new(&download_path);
                cmd.spawn()
            }
        }
        "macos" => {
            // macOS: 根据文件类型执行不同的安装逻辑
            if download_url.ends_with(".dmg") {
                // DMG: 使用open命令挂载
                let mut cmd = tokio::process::Command::new("open");
                cmd.arg(&download_path);
                cmd.spawn()
            } else if download_url.ends_with(".app.tar.gz") {
                // app.tar.gz: 解压并运行
                let mut cmd = tokio::process::Command::new("tar");
                cmd.arg("-xzf").arg(&download_path);
                cmd.spawn()
            } else {
                let mut cmd = tokio::process::Command::new(&download_path);
                cmd.spawn()
            }
        }
        _ => {
            // 其他平台：尝试直接运行
            let mut cmd = tokio::process::Command::new(&download_path);
            cmd.spawn()
        }
    };

    match install_result {
        Ok(_) => {
            // 安装程序启动成功，发送安装开始事件
            let install_message = match get_platform_identifier() {
                "windows" => "安装程序已启动，请按照提示完成安装",
                "linux" => {
                    if download_url.ends_with(".AppImage") {
                        "正在启动新版本应用程序..."
                    } else if download_url.ends_with(".deb") {
                        "正在安装软件包，请根据提示输入密码..."
                    } else {
                        "正在启动更新程序..."
                    }
                }
                "macos" => {
                    if download_url.ends_with(".dmg") {
                        "正在挂载安装镜像..."
                    } else if download_url.ends_with(".app.tar.gz") {
                        "正在解压应用程序..."
                    } else {
                        "正在启动安装程序..."
                    }
                }
                _ => "正在启动安装程序...",
            };

            let _ = window.emit(
                "update-progress",
                json!({
                    "status": "installing",
                    "progress": 100,
                    "message": install_message
                }),
            );
            Ok(())
        }
        Err(e) => {
            let error_msg = format!("启动安装程序失败: {}", e);
            let _ = window.emit(
                "update-progress",
                json!({
                    "status": "error",
                    "progress": 0,
                    "message": error_msg.clone()
                }),
            );
            Err(error_msg)
        }
    }
}
