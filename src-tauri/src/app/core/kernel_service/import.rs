use crate::app::constants::paths;
use crate::app::core::kernel_auto_manage::auto_manage_with_saved_config;
use crate::app::core::kernel_service::runtime::stop_kernel;
use crate::app::core::kernel_service::status::is_kernel_running;
use crate::app::core::kernel_service::versioning::extract_clean_version;
use crate::app::core::kernel_service::PROCESS_MANAGER;
use crate::app::storage::enhanced_storage_service::{
    db_get_app_config, db_save_app_config_internal,
};
use serde::Serialize;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::AppHandle;
use tokio::process::Command;
use tracing::{info, warn};

#[derive(Debug, Clone, Serialize)]
pub struct KernelImportResult {
    pub imported_version: String,
    pub restarted: bool,
    pub backup_path: Option<String>,
    pub message: String,
}

#[tauri::command]
pub fn pick_kernel_import_file() -> Result<Option<String>, String> {
    // 这里不限制后缀，让 Linux/macOS 下无扩展名二进制也可选择。
    let file = rfd::FileDialog::new()
        .set_title("选择 sing-box 内核文件（可执行文件或压缩包）")
        .pick_file();
    Ok(file.map(|p| p.to_string_lossy().to_string()))
}

/// 导入内核文件（支持可执行文件或压缩包）。
///
/// 支持：
/// - Windows: `sing-box.exe` / `.zip`
/// - Linux/macOS: `sing-box` / `.tar.gz` / `.tgz` / `.zip` / `.tar`
#[tauri::command]
pub async fn import_kernel_executable(
    app_handle: AppHandle,
    file_path: String,
) -> Result<KernelImportResult, String> {
    let raw_path = file_path.trim();
    if raw_path.is_empty() {
        return Err("请选择要导入的内核文件".to_string());
    }

    let selected_path = PathBuf::from(raw_path);
    if !selected_path.exists() {
        return Err(format!("文件不存在: {}", selected_path.to_string_lossy()));
    }
    if !selected_path.is_file() {
        return Err("请选择文件而不是目录".to_string());
    }

    let kernel_dir = paths::get_config_dir();
    tokio::fs::create_dir_all(&kernel_dir)
        .await
        .map_err(|e| format!("创建内核目录失败: {}", e))?;

    let temp_dir = kernel_dir.join(format!("manual-import-{}", now_timestamp_secs()));
    tokio::fs::create_dir_all(&temp_dir)
        .await
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    let result = import_kernel_executable_inner(&app_handle, &selected_path, &temp_dir).await;

    if let Err(e) = tokio::fs::remove_dir_all(&temp_dir).await {
        warn!("清理临时导入目录失败 {:?}: {}", temp_dir, e);
    }

    result
}

async fn import_kernel_executable_inner(
    app_handle: &AppHandle,
    selected_path: &Path,
    temp_dir: &Path,
) -> Result<KernelImportResult, String> {
    let source_binary_path = resolve_kernel_binary_source(selected_path, temp_dir).await?;
    let staged_binary_path = stage_kernel_binary(&source_binary_path, temp_dir).await?;
    let imported_version = validate_kernel_binary(&staged_binary_path).await?;

    let kernel_path = paths::get_kernel_path();
    let was_running_before_import = is_kernel_running().await.unwrap_or(false);
    if was_running_before_import {
        stop_running_kernel_for_replace(app_handle).await?;
    }

    let backup_path = replace_installed_kernel(&staged_binary_path, &kernel_path).await?;

    let restarted = if was_running_before_import {
        auto_manage_with_saved_config(app_handle, true, "kernel-manual-import").await;
        let restarted = wait_kernel_running(Duration::from_secs(10)).await;

        if !restarted {
            if let Some(path) = backup_path.as_deref() {
                warn!("新内核重启失败，尝试回滚到旧内核: {}", path);
                restore_kernel_from_backup(&kernel_path, Path::new(path)).await?;
                auto_manage_with_saved_config(app_handle, true, "kernel-manual-import-rollback")
                    .await;
            }
            return Err("新内核导入成功但重启失败，已自动回滚到旧内核".to_string());
        }

        true
    } else {
        false
    };

    if let Ok(mut app_config) = db_get_app_config(app_handle.clone()).await {
        app_config.installed_kernel_version = Some(imported_version.clone());
        if let Err(e) = db_save_app_config_internal(app_config, app_handle.clone()).await {
            warn!("保存导入后的内核版本失败: {}", e);
        }
    }

    Ok(KernelImportResult {
        imported_version: imported_version.clone(),
        restarted,
        backup_path,
        message: format!(
            "内核导入成功，版本 {}{}",
            imported_version,
            if restarted {
                "，已自动重启内核"
            } else {
                ""
            }
        ),
    })
}

fn now_timestamp_secs() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_else(|_| Duration::from_secs(0))
        .as_secs()
}

fn kernel_executable_name() -> &'static str {
    if cfg!(target_os = "windows") {
        "sing-box.exe"
    } else {
        "sing-box"
    }
}

fn is_archive_file(path: &Path) -> bool {
    let lower = path.to_string_lossy().to_ascii_lowercase();
    lower.ends_with(".zip")
        || lower.ends_with(".tar.gz")
        || lower.ends_with(".tgz")
        || lower.ends_with(".tar")
}

async fn resolve_kernel_binary_source(
    selected_path: &Path,
    temp_dir: &Path,
) -> Result<PathBuf, String> {
    if !is_archive_file(selected_path) {
        return Ok(selected_path.to_path_buf());
    }

    let extract_dir = temp_dir.join("extract");
    std::fs::create_dir_all(&extract_dir).map_err(|e| format!("创建解压目录失败: {}", e))?;
    extract_archive(selected_path, &extract_dir)?;
    find_executable_file(&extract_dir, kernel_executable_name())
}

async fn stage_kernel_binary(
    source_binary_path: &Path,
    temp_dir: &Path,
) -> Result<PathBuf, String> {
    let staged_path = temp_dir.join(kernel_executable_name());
    tokio::fs::copy(source_binary_path, &staged_path)
        .await
        .map_err(|e| format!("复制导入文件失败: {}", e))?;
    set_executable_permission(&staged_path)?;
    Ok(staged_path)
}

async fn validate_kernel_binary(binary_path: &Path) -> Result<String, String> {
    let mut cmd = Command::new(binary_path);
    cmd.arg("version");

    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);

    let output = cmd
        .output()
        .await
        .map_err(|e| format!("执行内核版本校验失败: {}", e))?;

    if !output.status.success() {
        let stderr = String::from_utf8_lossy(&output.stderr);
        return Err(format!("内核版本校验失败: {}", stderr));
    }

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let merged = format!("{stdout}\n{stderr}").to_ascii_lowercase();

    let mut version = extract_clean_version(&stdout);
    if version.is_empty() {
        version = extract_clean_version(&stderr);
    }

    let file_name_hint = binary_path
        .file_name()
        .and_then(|n| n.to_str())
        .map(|n| n.to_ascii_lowercase().contains("sing-box"))
        .unwrap_or(false);

    if version.is_empty() || (!merged.contains("sing-box") && !file_name_hint) {
        return Err("选中的文件不是有效的 sing-box 内核".to_string());
    }

    Ok(version)
}

async fn stop_running_kernel_for_replace(app_handle: &AppHandle) -> Result<(), String> {
    info!("检测到内核正在运行，准备停止以执行手动导入替换");

    for attempt in 1..=5 {
        let _ = stop_kernel(Some(app_handle)).await;
        if !is_kernel_running().await.unwrap_or(true) {
            info!("内核已停止，可继续替换");
            return Ok(());
        }
        warn!("第 {} 次停止内核后仍在运行，准备重试", attempt);
        tokio::time::sleep(Duration::from_millis(500)).await;
    }

    warn!("常规停止失败，尝试强制终止残留进程");
    PROCESS_MANAGER
        .kill_existing_processes(Some(app_handle))
        .await
        .map_err(|e| format!("强制终止内核进程失败: {}", e))?;

    tokio::time::sleep(Duration::from_millis(500)).await;
    if is_kernel_running().await.unwrap_or(false) {
        return Err("无法停止当前内核进程，已中止导入".to_string());
    }

    Ok(())
}

async fn replace_installed_kernel(
    staged_binary_path: &Path,
    kernel_path: &Path,
) -> Result<Option<String>, String> {
    if let Some(parent) = kernel_path.parent() {
        tokio::fs::create_dir_all(parent)
            .await
            .map_err(|e| format!("创建内核目录失败: {}", e))?;
    }

    let backup_path = if kernel_path.exists() {
        let backup = build_backup_path(kernel_path)?;
        if backup.exists() {
            tokio::fs::remove_file(&backup)
                .await
                .map_err(|e| format!("清理旧备份失败: {}", e))?;
        }
        tokio::fs::rename(kernel_path, &backup)
            .await
            .map_err(|e| format!("备份旧内核失败: {}", e))?;
        Some(backup)
    } else {
        None
    };

    if let Err(err) = move_file_with_fallback(staged_binary_path, kernel_path).await {
        if let Some(backup) = backup_path.as_ref() {
            let _ = restore_kernel_from_backup(kernel_path, backup).await;
        }
        return Err(format!("替换内核文件失败: {}", err));
    }

    Ok(backup_path.map(|p| p.to_string_lossy().to_string()))
}

fn build_backup_path(kernel_path: &Path) -> Result<PathBuf, String> {
    let file_name = kernel_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| "目标内核文件名无效".to_string())?;
    let parent = kernel_path
        .parent()
        .ok_or_else(|| "目标内核目录无效".to_string())?;
    Ok(parent.join(format!("{}.bak-import-{}", file_name, now_timestamp_secs())))
}

async fn move_file_with_fallback(from: &Path, to: &Path) -> Result<(), String> {
    match tokio::fs::rename(from, to).await {
        Ok(_) => Ok(()),
        Err(_) => {
            tokio::fs::copy(from, to)
                .await
                .map_err(|e| format!("复制文件失败: {}", e))?;
            tokio::fs::remove_file(from)
                .await
                .map_err(|e| format!("清理临时文件失败: {}", e))?;
            Ok(())
        }
    }
}

async fn restore_kernel_from_backup(kernel_path: &Path, backup_path: &Path) -> Result<(), String> {
    if !backup_path.exists() {
        return Err("回滚失败：未找到备份内核".to_string());
    }

    if kernel_path.exists() {
        tokio::fs::remove_file(kernel_path)
            .await
            .map_err(|e| format!("回滚失败：删除新内核失败: {}", e))?;
    }

    move_file_with_fallback(backup_path, kernel_path).await
}

async fn wait_kernel_running(timeout: Duration) -> bool {
    let start = tokio::time::Instant::now();
    while start.elapsed() < timeout {
        if is_kernel_running().await.unwrap_or(false) {
            return true;
        }
        tokio::time::sleep(Duration::from_millis(400)).await;
    }
    false
}

fn extract_archive(archive_path: &Path, extract_to: &Path) -> Result<(), String> {
    let lower = archive_path.to_string_lossy().to_ascii_lowercase();
    if lower.ends_with(".zip") {
        extract_zip_archive(archive_path, extract_to)?;
    } else if lower.ends_with(".tar.gz") || lower.ends_with(".tgz") {
        extract_tar_gz_archive(archive_path, extract_to)?;
    } else if lower.ends_with(".tar") {
        extract_tar_archive(archive_path, extract_to)?;
    } else {
        return Err("不支持的压缩包格式，仅支持 zip / tar.gz / tgz / tar".to_string());
    }

    Ok(())
}

fn extract_zip_archive(archive_path: &Path, extract_to: &Path) -> Result<(), String> {
    use zip::ZipArchive;

    let file = std::fs::File::open(archive_path).map_err(|e| format!("打开 ZIP 失败: {}", e))?;
    let mut zip = ZipArchive::new(file).map_err(|e| format!("解析 ZIP 失败: {}", e))?;

    for i in 0..zip.len() {
        let mut entry = zip
            .by_index(i)
            .map_err(|e| format!("读取 ZIP 条目失败: {}", e))?;
        let target = extract_to.join(entry.name());

        if entry.name().ends_with('/') {
            std::fs::create_dir_all(&target).map_err(|e| format!("创建目录失败: {}", e))?;
            continue;
        }

        if let Some(parent) = target.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
        }

        let mut out = std::fs::File::create(&target).map_err(|e| format!("写入文件失败: {}", e))?;
        std::io::copy(&mut entry, &mut out).map_err(|e| format!("解压 ZIP 失败: {}", e))?;
    }

    Ok(())
}

fn extract_tar_gz_archive(archive_path: &Path, extract_to: &Path) -> Result<(), String> {
    use flate2::read::GzDecoder;
    use tar::Archive;

    let file = std::fs::File::open(archive_path).map_err(|e| format!("打开 tar.gz 失败: {}", e))?;
    let decoder = GzDecoder::new(file);
    let mut archive = Archive::new(decoder);
    archive
        .unpack(extract_to)
        .map_err(|e| format!("解压 tar.gz 失败: {}", e))
}

fn extract_tar_archive(archive_path: &Path, extract_to: &Path) -> Result<(), String> {
    use tar::Archive;

    let file = std::fs::File::open(archive_path).map_err(|e| format!("打开 tar 失败: {}", e))?;
    let mut archive = Archive::new(file);
    archive
        .unpack(extract_to)
        .map_err(|e| format!("解压 tar 失败: {}", e))
}

fn find_executable_file(search_dir: &Path, executable_name: &str) -> Result<PathBuf, String> {
    let direct_path = search_dir.join(executable_name);
    if direct_path.exists() && direct_path.is_file() {
        return Ok(direct_path);
    }

    for entry in walkdir::WalkDir::new(search_dir).into_iter().flatten() {
        let path = entry.path();
        if path
            .file_name()
            .and_then(|name| name.to_str())
            .map(|name| name == executable_name)
            .unwrap_or(false)
            && path.is_file()
        {
            return Ok(path.to_path_buf());
        }
    }

    Err(format!(
        "在压缩包内未找到内核可执行文件: {}",
        executable_name
    ))
}

#[cfg(unix)]
fn set_executable_permission(file_path: &Path) -> Result<(), String> {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = std::fs::metadata(file_path)
        .map_err(|e| format!("读取文件权限失败: {}", e))?
        .permissions();
    perms.set_mode(perms.mode() | 0o755);
    std::fs::set_permissions(file_path, perms).map_err(|e| format!("设置执行权限失败: {}", e))
}

#[cfg(not(unix))]
fn set_executable_permission(_file_path: &Path) -> Result<(), String> {
    Ok(())
}
