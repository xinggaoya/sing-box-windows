use crate::app::constants::paths;
use crate::app::storage::enhanced_storage_service::{
    db_get_app_config, db_save_app_config_internal,
};
use tauri::{AppHandle, Manager};
use tracing::{info, warn};

pub async fn ensure_embedded_kernel(app_handle: &AppHandle) -> Result<Option<String>, String> {
    let kernel_path = paths::get_kernel_path();
    if kernel_path.exists() {
        return Ok(None);
    }

    let resource_dir = match app_handle.path().resource_dir() {
        Ok(dir) => dir,
        Err(e) => {
            warn!("无法获取资源目录，跳过内嵌内核检查: {}", e);
            return Ok(None);
        }
    };

    let platform = if cfg!(target_os = "windows") {
        "windows"
    } else if cfg!(target_os = "linux") {
        "linux"
    } else if cfg!(target_os = "macos") {
        "macos"
    } else {
        "unknown"
    };

    if platform == "unknown" {
        warn!("当前平台不支持内嵌内核安装");
        return Ok(None);
    }

    let arch = super::versioning::get_system_arch();
    let executable_name = if cfg!(target_os = "windows") {
        "sing-box.exe"
    } else {
        "sing-box"
    };

    let mut embedded_dir = None;
    let mut embedded_kernel_path = None;
    let candidate_bases = [
        resource_dir.join("kernel"),
        resource_dir.join("resources").join("kernel"),
    ];

    for base in candidate_bases {
        let dir = base.join(platform).join(arch);
        let path = dir.join(executable_name);
        if path.exists() {
            embedded_dir = Some(dir);
            embedded_kernel_path = Some(path);
            break;
        }
    }

    let embedded_dir = match embedded_dir {
        Some(dir) => dir,
        None => {
            info!("未找到内嵌内核资源文件，跳过安装");
            return Ok(None);
        }
    };
    let embedded_kernel_path = embedded_kernel_path.unwrap();

    if let Some(parent) = kernel_path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return Err(format!("创建内核目录失败: {}", e));
        }
    }

    // 首次安装：从应用资源目录复制内核到工作目录
    tokio::fs::copy(&embedded_kernel_path, &kernel_path)
        .await
        .map_err(|e| format!("复制内嵌内核失败: {}", e))?;

    if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        if let Err(e) = set_executable_permission(&kernel_path) {
            warn!("设置内核执行权限失败: {}", e);
        }
    }

    let version_path = embedded_dir.join("version.txt");
    let version = match tokio::fs::read_to_string(&version_path).await {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Err(_) => None,
    };

    if let Some(version) = version.clone() {
        if let Ok(mut config) = db_get_app_config(app_handle.clone()).await {
            config.installed_kernel_version = Some(version);
            if let Err(e) = db_save_app_config_internal(config, app_handle.clone()).await {
                warn!("保存内嵌内核版本信息失败: {}", e);
            }
        }
    }

    info!("内嵌内核已安装: {:?}", kernel_path);
    Ok(version)
}

#[cfg(unix)]
fn set_executable_permission(file_path: &std::path::Path) -> Result<(), std::io::Error> {
    use std::os::unix::fs::PermissionsExt;

    let mut perms = std::fs::metadata(file_path)?.permissions();
    perms.set_mode(perms.mode() | 0o755);
    std::fs::set_permissions(file_path, perms)?;
    Ok(())
}

#[cfg(not(unix))]
fn set_executable_permission(_file_path: &std::path::Path) -> Result<(), std::io::Error> {
    Ok(())
}
