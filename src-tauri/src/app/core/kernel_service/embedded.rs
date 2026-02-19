use crate::app::constants::paths;
use crate::app::storage::enhanced_storage_service::{
    db_get_app_config, db_save_app_config_internal,
};
use semver::Version;
use std::path::Path;
use tauri::{AppHandle, Manager};
use tracing::{info, warn};

pub async fn ensure_embedded_kernel(app_handle: &AppHandle) -> Result<Option<String>, String> {
    let kernel_path = paths::get_kernel_path();

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

    let (embedded_dir, embedded_kernel_path) = match (embedded_dir, embedded_kernel_path) {
        (Some(dir), Some(path)) => (dir, path),
        _ => {
            info!("未找到内嵌内核资源文件，跳过安装");
            return Ok(None);
        }
    };
    let embedded_version = read_embedded_version(&embedded_dir).await;

    if kernel_path.exists() {
        let Some(target_version) = embedded_version.as_deref() else {
            info!("当前已存在本地内核，且内嵌资源缺少版本信息，跳过覆盖更新");
            return Ok(None);
        };

        let installed_version = resolve_installed_version(app_handle, &kernel_path).await;
        let Some(current_version) = installed_version else {
            warn!("当前已存在本地内核，但无法识别版本，跳过覆盖更新");
            return Ok(None);
        };

        match is_embedded_newer(&current_version, target_version) {
            Some(true) => {
                info!(
                    "检测到内嵌内核版本更新，将覆盖安装: {} -> {}",
                    current_version, target_version
                );
            }
            Some(false) => {
                info!(
                    "本地内核版本不低于内嵌版本，跳过覆盖: 本地={}, 内嵌={}",
                    current_version, target_version
                );
                let _ = save_installed_version(app_handle, current_version).await;
                return Ok(None);
            }
            None => {
                warn!(
                    "无法比较版本，保守跳过覆盖更新: 本地={}, 内嵌={}",
                    current_version, target_version
                );
                return Ok(None);
            }
        }
    } else {
        info!("未检测到本地内核，准备安装内嵌内核");
    }

    if let Some(parent) = kernel_path.parent() {
        if let Err(e) = tokio::fs::create_dir_all(parent).await {
            return Err(format!("创建内核目录失败: {}", e));
        }
    }

    // 安装或覆盖更新：从应用资源目录复制内核到工作目录
    tokio::fs::copy(&embedded_kernel_path, &kernel_path)
        .await
        .map_err(|e| format!("复制内嵌内核失败: {}", e))?;

    if cfg!(target_os = "linux") || cfg!(target_os = "macos") {
        if let Err(e) = set_executable_permission(&kernel_path) {
            warn!("设置内核执行权限失败: {}", e);
        }
    }

    if let Some(version) = embedded_version.clone() {
        let _ = save_installed_version(app_handle, version).await;
    }

    info!("内嵌内核已安装: {:?}", kernel_path);
    Ok(embedded_version)
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

async fn read_embedded_version(embedded_dir: &Path) -> Option<String> {
    let version_path = embedded_dir.join("version.txt");
    match tokio::fs::read_to_string(&version_path).await {
        Ok(content) => {
            let trimmed = content.trim();
            if trimmed.is_empty() {
                None
            } else {
                Some(trimmed.to_string())
            }
        }
        Err(_) => None,
    }
}

async fn resolve_installed_version(app_handle: &AppHandle, kernel_path: &Path) -> Option<String> {
    if let Some(version) = read_kernel_version_from_binary(kernel_path).await {
        return Some(version);
    }

    if let Ok(config) = db_get_app_config(app_handle.clone()).await {
        if let Some(version) = config.installed_kernel_version {
            let normalized = normalize_version_string(&version);
            if !normalized.is_empty() {
                return Some(normalized);
            }
        }
    }

    None
}

async fn save_installed_version(app_handle: &AppHandle, version: String) -> Result<(), String> {
    let normalized = normalize_version_string(&version);
    if normalized.is_empty() {
        return Ok(());
    }

    match db_get_app_config(app_handle.clone()).await {
        Ok(mut config) => {
            if config.installed_kernel_version.as_deref() != Some(normalized.as_str()) {
                config.installed_kernel_version = Some(normalized);
                db_save_app_config_internal(config, app_handle.clone()).await?;
            }
            Ok(())
        }
        Err(e) => {
            warn!("读取应用配置失败，无法保存内核版本信息: {}", e);
            Ok(())
        }
    }
}

async fn read_kernel_version_from_binary(kernel_path: &Path) -> Option<String> {
    let mut cmd = tokio::process::Command::new(kernel_path);
    cmd.arg("version");

    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);

    let output = cmd.output().await.ok()?;
    if !output.status.success() {
        return None;
    }

    extract_version_from_output(&String::from_utf8_lossy(&output.stdout))
}

fn extract_version_from_output(output: &str) -> Option<String> {
    for token in output.split_whitespace() {
        let cleaned = token
            .trim_matches(|c: char| c == ':' || c == ',' || c == ';' || c == ')' || c == '(');
        let normalized = normalize_version_string(cleaned);
        if normalized.is_empty() {
            continue;
        }
        if normalized.chars().any(|c| c.is_ascii_digit()) {
            return Some(normalized);
        }
    }
    None
}

fn normalize_version_string(raw: &str) -> String {
    raw.trim().trim_start_matches('v').to_string()
}

fn is_embedded_newer(current: &str, embedded: &str) -> Option<bool> {
    let current = normalize_version_string(current);
    let embedded = normalize_version_string(embedded);

    if current.is_empty() || embedded.is_empty() {
        return None;
    }

    match (Version::parse(&current), Version::parse(&embedded)) {
        (Ok(current_ver), Ok(embedded_ver)) => Some(embedded_ver > current_ver),
        _ if current == embedded => Some(false),
        _ => None,
    }
}
