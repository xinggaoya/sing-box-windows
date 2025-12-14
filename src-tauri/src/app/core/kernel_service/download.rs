use crate::app::core::kernel_auto_manage::auto_manage_with_saved_config;
use crate::app::core::kernel_service::runtime::stop_kernel;
use crate::app::core::kernel_service::status::is_kernel_running;
use crate::app::core::kernel_service::versioning::{get_latest_kernel_version, get_system_arch};
use crate::app::core::kernel_service::PROCESS_MANAGER;
use serde_json::json;
use std::path::Path;
use std::time::Duration;
use tauri::Manager;
use tauri::{AppHandle, Emitter, WebviewWindow};
use tracing::{info, warn};

#[tauri::command]
pub async fn download_kernel(app_handle: AppHandle, version: Option<String>) -> Result<(), String> {
    info!("开始下载内核 (指定版本: {:?})...", version);

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

    let version = match version {
        Some(v) => v,
        None => match get_latest_kernel_version().await {
            Ok(v) => {
                info!("获取到最新版本号: {}", v);
                v
            }
            Err(e) => {
                warn!("获取最新版本失败: {}, 使用默认版本 1.12.10", e);
                "1.12.10".to_string()
            }
        },
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
    // 使用专门的临时目录进行下载和解压，避免污染主目录以及混淆旧文件搜索
    let temp_update_dir = kernel_dir.join("update_temp");

    if let Err(e) = std::fs::create_dir_all(&temp_update_dir) {
        return Err(format!("创建临时更新目录失败: {}", e));
    }
    
    // 清理旧的临时目录内容（如果有）
    if let Ok(entries) = std::fs::read_dir(&temp_update_dir) {
        for entry in entries.flatten() {
           if let Err(e) = if entry.path().is_dir() { 
               std::fs::remove_dir_all(entry.path()) 
           } else { 
               std::fs::remove_file(entry.path()) 
           } {
               warn!("清理临时目录失败: {}", e);
           }
        }
    }

    let download_path = temp_update_dir.join(&filename);

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
                
                // 失败时清理
                let _ = std::fs::remove_dir_all(&temp_update_dir);
                return Err(final_error);
            }
        }
    }

    if !download_path.exists() {
        let _ = std::fs::remove_dir_all(&temp_update_dir);
        return Err("下载的文件不存在".to_string());
    }

    let was_running_before_update = is_kernel_running().await.unwrap_or(false);
    if was_running_before_update {
        info!("内核更新前检测到正在运行，先尝试停止以便替换");
        
        // 尝试多次停止内核
        for i in 0..5 {
            let _ = stop_kernel().await; // stop_kernel 内部已有 guard disable 和 2s 等待
            
            if !is_kernel_running().await.unwrap_or(true) {
                info!("内核已成功停止");
                break;
            }
            warn!("停止内核尝试 {} 失败，等待重试...", i + 1);
            tokio::time::sleep(Duration::from_millis(500)).await;
        }
        
        // 最后再次确认
        if is_kernel_running().await.unwrap_or(false) {
             warn!("几次尝试后内核仍在运行，尝试强制终止进程...");
             if let Err(e) = PROCESS_MANAGER.kill_existing_processes().await {
                 warn!("强制终止内核进程失败: {}", e);
             }
             tokio::time::sleep(Duration::from_millis(500)).await;
        }
    }

    let _ = window.emit(
        "kernel-download-progress",
        json!({
            "status": "extracting",
            "progress": 80,
            "message": "正在解压内核文件..."
        }),
    );

    // 解压到临时目录
    if let Err(e) = extract_archive(&download_path, &temp_update_dir).await {
        let error_msg = format!("解压文件失败: {}", e);
        let _ = window.emit(
            "kernel-download-progress",
            json!({
                "status": "error",
                "progress": 0,
                "message": error_msg
            }),
        );
        let _ = std::fs::remove_dir_all(&temp_update_dir);
        return Err(error_msg);
    }

    // 删除下载包，只留解压内容
    let _ = std::fs::remove_file(&download_path);

    let executable_name = if cfg!(target_os = "windows") {
        "sing-box.exe"
    } else {
        "sing-box"
    };

    info!("开始在临时目录中查找新内核: {}", executable_name);

    // 在临时目录中查找
    let found_executable_path = match find_executable_file(&temp_update_dir, executable_name).await {
        Ok(p) => p,
        Err(e) => {
            let _ = std::fs::remove_dir_all(&temp_update_dir);
            return Err(e);
        }
    };
    
    let target_executable_path = kernel_dir.join(executable_name);

    info!(
        "准备迁移新内核文件从 {:?} 到 {:?}",
        found_executable_path, target_executable_path
    );

    // 目标如果存在，处理备份/重命名
    if target_executable_path.exists() {
        // Windows 下如果文件正在使用，无法直接删除，但通常可以重命名
        // 尝试将旧文件重命名为 .old
        let old_executable_path = if cfg!(target_os = "windows") {
            target_executable_path.with_extension("exe.old")
        } else {
            target_executable_path.with_extension("old")
        };

        // 如果已经存在 .old 文件，先尝试删除它
        if old_executable_path.exists() {
            let _ = std::fs::remove_file(&old_executable_path);
        }

        if let Err(e) = std::fs::rename(&target_executable_path, &old_executable_path) {
            warn!("重命名旧文件失败: {}, 尝试直接删除...", e);
            if let Err(e) = std::fs::remove_file(&target_executable_path) {
                // 如果实在删不掉，报错返回
                let _ = std::fs::remove_dir_all(&temp_update_dir);
                return Err(format!(
                    "无法删除或重命名旧内核文件 (可能正在使用?): {}. 请尝试手动停止内核或重启应用。", 
                    e
                ));
            }
        } else {
            info!("旧内核文件已重命名为: {:?}", old_executable_path);
        }
    }

    // 移动新文件到目标位置
    if let Err(_e) = std::fs::rename(&found_executable_path, &target_executable_path) {
        // 如果跨磁盘或者 rename 失败，尝试 copy + delete
        if let Err(copy_err) = std::fs::copy(&found_executable_path, &target_executable_path) {
            let _ = std::fs::remove_dir_all(&temp_update_dir);
            return Err(format!("复制新内核文件失败: {}", copy_err));
        }
    }
    
    info!("成功部署新内核文件");

    // 清理临时目录
    if let Err(e) = std::fs::remove_dir_all(&temp_update_dir) {
        warn!("清理临时更新目录失败: {}, 请手动清理 {:?}", e, temp_update_dir);
    }

    // 尝试清理残留的旧版本目录 (可选)
    // 之前下载解压可能留下的垃圾目录 sing-box-1.xx-windows-amd64 等
    if let Ok(entries) = std::fs::read_dir(&kernel_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            // 如果是目录，且名字看起来像 version 目录，且不是我们刚部署的文件
            if path.is_dir() && path.file_name().unwrap_or_default() != "logs" && path.file_name().unwrap_or_default() != "update_temp" {
                 let name = path.file_name().unwrap().to_string_lossy();
                 if name.starts_with("sing-box-") {
                     let _ = std::fs::remove_dir_all(&path);
                 }
            }
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

    // 更新安装版本信息到数据库
    use crate::app::storage::enhanced_storage_service::{db_save_app_config_internal, db_get_app_config};
    if let Ok(mut config) = db_get_app_config(app_handle.clone()).await {
        config.installed_kernel_version = Some(version);
        if let Err(e) = db_save_app_config_internal(config, app_handle).await {
            warn!("保存内核版本信息失败: {}", e);
        } else {
            info!("已更新数据库中的已安装内核版本信息");
        }
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


