use crate::app::constants::{api, messages};
use crate::app::network_config;
use crate::utils::app_util::get_work_dir_sync;
use semver::Version;
use serde_json::json;
use std::path::Path;
use tauri::{Emitter, Manager};

// 获取当前平台标识符
fn get_platform_identifier() -> &'static str {
    if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unknown"
    }
}

// 检查文件是否匹配当前平台
fn is_platform_compatible(filename: &str) -> bool {
    let platform = get_platform_identifier();

    match platform {
        "windows" => filename.ends_with(".msi") || filename.ends_with(".exe"),
        "linux" => filename.ends_with(".AppImage") || filename.ends_with(".deb"),
        "macos" => filename.ends_with(".dmg") || filename.ends_with(".app.tar.gz"),
        _ => false,
    }
}

// 获取平台优先级分数（用于选择最合适的安装包）
fn get_platform_priority(filename: &str) -> i32 {
    let platform = get_platform_identifier();

    match platform {
        "windows" => {
            if filename.ends_with(".msi") { 2 } else if filename.ends_with(".exe") { 1 } else { 0 }
        }
        "linux" => {
            if filename.ends_with(".AppImage") { 2 } else if filename.ends_with(".deb") { 1 } else { 0 }
        }
        "macos" => {
            if filename.ends_with(".dmg") { 2 } else if filename.ends_with(".app.tar.gz") { 1 } else { 0 }
        }
        _ => 0,
    }
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
    let window = app_handle.get_webview_window("main")
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

// 安装更新
#[tauri::command]
pub async fn install_update(_download_path: String) -> Result<(), String> {
    // 简单的实现，返回成功
    Ok(())
}

// 获取当前平台信息
#[tauri::command]
pub async fn get_platform_info() -> Result<String, String> {
    Ok(get_platform_identifier().to_string())
}

// 下载并安装更新
#[tauri::command]
pub async fn download_and_install_update(
    app_handle: tauri::AppHandle,
    download_url: String,
) -> Result<(), String> {
    let window = app_handle.get_webview_window("main")
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
            // Windows: 直接运行安装程序
            let mut cmd = tokio::process::Command::new(&download_path);
            #[cfg(target_os = "windows")]
            cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);
            cmd.spawn()
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
                cmd.arg("dpkg").arg("-i").arg(&download_path);
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
