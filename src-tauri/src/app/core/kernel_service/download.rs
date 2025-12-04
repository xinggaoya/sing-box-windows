use crate::app::core::kernel_auto_manage::auto_manage_with_saved_config;
use crate::app::core::kernel_service::runtime::stop_kernel;
use crate::app::core::kernel_service::status::is_kernel_running;
use crate::app::core::kernel_service::versioning::{get_latest_kernel_version, get_system_arch};
use serde_json::json;
use std::path::Path;
use std::time::Duration;
use tauri::Manager;
use tauri::{AppHandle, Emitter, WebviewWindow};
use tracing::{info, warn};

#[tauri::command]
pub async fn download_latest_kernel(app_handle: AppHandle) -> Result<(), String> {
    info!("开始下载最新内核...");

    let window = app_handle
        .get_webview_window("main")
        .ok_or("无法获取主窗口")?;

    let _ = window.emit(
        "kernel-download-progress",
        json!({
            "status": "downloading",
            "progress": 0,
            "message": "开始下载内核..."
        }),
    );

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

    info!("检测到平台: {}, 架构: {}", platform, arch);

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

    let filename = if cfg!(target_os = "windows") {
        format!("sing-box-{}-windows-{}.zip", version, arch)
    } else if cfg!(target_os = "macos") {
        format!("sing-box-{}-darwin-{}.tar.gz", version, arch)
    } else {
        format!("sing-box-{}-linux-{}.tar.gz", version, arch)
    };

    let download_urls = vec![
        format!(
            "https://v6.gh-proxy.com/https://github.com/SagerNet/sing-box/releases/download/v{}/{}",
            version, filename
        ),
        format!(
            "https://gh-proxy.com/https://github.com/SagerNet/sing-box/releases/download/v{}/{}",
            version, filename
        ),
        format!(
            "https://ghfast.top/https://github.com/SagerNet/sing-box/releases/download/v{}/{}",
            version, filename
        ),
        format!(
            "https://hub.fastgit.xyz/SagerNet/sing-box/releases/download/v{}/{}",
            version, filename
        ),
        format!(
            "https://hub.fgit.cf/SagerNet/sing-box/releases/download/v{}/{}",
            version, filename
        ),
        format!(
            "https://cdn.jsdelivr.net/gh/SagerNet/sing-box@releases/download/v{}/{}",
            version, filename
        ),
        format!(
            "https://github.com/SagerNet/sing-box/releases/download/v{}/{}",
            version, filename
        ),
    ];

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

    let work_dir = crate::utils::app_util::get_work_dir_sync();
    let kernel_dir = Path::new(&work_dir).join("sing-box");

    if let Err(e) = std::fs::create_dir_all(&kernel_dir) {
        return Err(format!("创建内核目录失败: {}", e));
    }

    let download_path = kernel_dir.join(&filename);

    let _ = window.emit(
        "kernel-download-progress",
        json!({
            "status": "downloading",
            "progress": 10,
            "message": "正在下载内核文件..."
        }),
    );

    for (index, download_url) in download_urls.iter().enumerate() {
        info!("尝试第 {} 个下载源: {}", index + 1, download_url);

        let _ = window.emit(
            "kernel-download-progress",
            json!({
                "status": "downloading",
                "progress": 15 + (index * 5),
                "message": format!("尝试第 {} 个下载源...", index + 1)
            }),
        );

        match download_file(download_url, &download_path, &window).await {
            Ok(_) => {
                info!("下载成功，使用下载源: {}", download_url);
                break;
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
                warn!("下载源 {} 失败: {}", source_name, e);

                let _ = window.emit(
                    "kernel-download-progress",
                    json!({
                        "status": "downloading",
                        "progress": 15 + (index * 5),
                        "message": format!("?? {} - 尝试下一个下载源...", error_details)
                    }),
                );

                let _ = std::fs::remove_file(&download_path);

                if index < download_urls.len() - 1 {
                    continue;
                }

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

    if !download_path.exists() {
        return Err("下载的文件不存在".to_string());
    }

    let was_running_before_update = is_kernel_running().await.unwrap_or(false);
    if was_running_before_update {
        info!("内核更新前检测到正在运行，先尝试停止以便替换");
        let _ = stop_kernel().await;
        tokio::time::sleep(Duration::from_millis(300)).await;
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

    let _ = std::fs::remove_file(&download_path);

    let executable_name = if cfg!(target_os = "windows") {
        "sing-box.exe"
    } else {
        "sing-box"
    };

    info!("开始查找可执行文件: {}", executable_name);

    let found_executable_path = find_executable_file(&kernel_dir, executable_name).await?;
    let target_executable_path = kernel_dir.join(executable_name);

    if found_executable_path != target_executable_path {
        info!(
            "迁移内核文件从 {:?} 到 {:?}",
            found_executable_path, target_executable_path
        );

        if target_executable_path.exists() {
            if let Err(e) = std::fs::remove_file(&target_executable_path) {
                warn!("删除已存在的目标文件失败: {}, 将继续...", e);
            }
        }

        if let Err(_e) = std::fs::rename(&found_executable_path, &target_executable_path) {
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

        if let Some(parent_dir) = found_executable_path.parent() {
            info!("清理版本目录: {:?}", parent_dir);

            if let Err(e) = std::fs::remove_dir_all(parent_dir) {
                warn!("删除版本目录失败: {}, 将继续...", e);
            } else {
                info!("成功删除版本目录: {:?}", parent_dir);
            }
        }

        if let Err(e) = cleanup_kernel_directory(&kernel_dir, executable_name) {
            warn!("清理内核目录失败: {}, 将继续...", e);
        }
    }

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

    if was_running_before_update {
        info!("内核更新完成，自动重新启动内核");
        auto_manage_with_saved_config(&app_handle, true, "kernel-update").await;
    }

    Ok(())
}

async fn download_file(
    url: &str,
    path: &Path,
    window: &WebviewWindow,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use futures_util::StreamExt;
    use tokio::fs::File;
    use tokio::io::AsyncWriteExt;

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(300))
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
                    "progress": progress.min(70),
                    "message": format!("下载中... {}/{} bytes", downloaded, total_size)
                }),
            );
        }
    }

    file.flush().await?;
    Ok(())
}

async fn extract_archive(
    archive_path: &Path,
    extract_to: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("开始解压文件: {:?}", archive_path);

    if !archive_path.exists() {
        return Err(format!("压缩文件不存在: {:?}", archive_path).into());
    }

    let metadata = std::fs::metadata(archive_path)?;
    let file_size = metadata.len();
    info!("压缩文件大小: {} bytes", file_size);

    if file_size == 0 {
        return Err("压缩文件为空".into());
    }

    let file_extension = archive_path
        .extension()
        .and_then(|ext| ext.to_str())
        .unwrap_or("")
        .to_ascii_lowercase();

    if file_extension == "zip" {
        extract_zip_archive(archive_path, extract_to).await?;
    } else if file_extension == "gz" || archive_path.to_string_lossy().ends_with(".tar.gz") {
        extract_tar_gz_archive(archive_path, extract_to).await?;
    } else {
        return Err(format!("不支持的压缩格式: {}", file_extension).into());
    }

    if let Ok(entries) = std::fs::read_dir(extract_to) {
        info!("解压后的文件:");
        for entry in entries.flatten() {
            info!("  - {:?}", entry.path());
        }
    }

    Ok(())
}

async fn extract_zip_archive(
    archive_path: &Path,
    extract_to: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use zip::ZipArchive;

    info!("解压 ZIP 文件: {:?}", archive_path);

    let file = std::fs::File::open(archive_path)?;
    let mut zip = ZipArchive::new(file)?;

    if !extract_to.exists() {
        std::fs::create_dir_all(extract_to)?;
    }

    for i in 0..zip.len() {
        let mut file = zip.by_index(i)?;
        let file_path = extract_to.join(file.name());

        if file.name().ends_with('/') {
            if let Some(parent) = file_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            continue;
        }

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

async fn extract_tar_gz_archive(
    archive_path: &Path,
    extract_to: &Path,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use flate2::read::GzDecoder;
    use std::fs::File;
    use tar::Archive;

    info!("解压 TAR.GZ 文件: {:?}", archive_path);

    let file = File::open(archive_path)?;
    let gz = GzDecoder::new(file);
    let mut archive = Archive::new(gz);

    if !extract_to.exists() {
        std::fs::create_dir_all(extract_to)?;
    }

    match archive.unpack(extract_to) {
        Ok(_) => info!("TAR.GZ 文件解压完成"),
        Err(e) => return Err(format!("TAR.GZ 解压失败: {}", e).into()),
    }

    Ok(())
}

async fn find_executable_file(
    search_dir: &Path,
    executable_name: &str,
) -> Result<std::path::PathBuf, String> {
    info!(
        "在目录 {:?} 中查找可执行文件: {}",
        search_dir, executable_name
    );

    let direct_path = search_dir.join(executable_name);
    if direct_path.exists() && direct_path.is_file() {
        info!("直接找到可执行文件: {:?}", direct_path);
        return Ok(direct_path);
    }

    let mut found_files = Vec::new();
    for entry in walkdir::WalkDir::new(search_dir).into_iter().flatten() {
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == executable_name)
            .unwrap_or(false)
            && path.is_file()
        {
            found_files.push(path.to_path_buf());
        }
    }

    if found_files.is_empty() {
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

    Ok(found_files[0].clone())
}

#[cfg(unix)]
fn set_executable_permission(file_path: &Path) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = std::fs::metadata(file_path)?.permissions();
    perms.set_mode(perms.mode() | 0o755);
    std::fs::set_permissions(file_path, perms)?;

    info!("已设置执行权限: {:?}", file_path);
    Ok(())
}

#[cfg(not(unix))]
fn set_executable_permission(_file_path: &Path) -> Result<(), std::io::Error> {
    Ok(())
}

fn cleanup_kernel_directory(
    kernel_dir: &Path,
    executable_name: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    info!("清理内核目录，只保留可执行文件: {}", executable_name);

    if let Ok(entries) = std::fs::read_dir(kernel_dir) {
        for entry in entries.flatten() {
            let path = entry.path();

            if path
                .file_name()
                .and_then(|name| name.to_str())
                .map(|name| name == executable_name)
                .unwrap_or(false)
            {
                continue;
            }

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

#[tauri::command]
pub async fn install_kernel() -> Result<(), String> {
    info!("内核安装完成");
    Ok(())
}
