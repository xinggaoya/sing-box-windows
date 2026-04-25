use crate::app::constants::paths;
use crate::app::storage::enhanced_storage_service::{
    db_get_app_config, db_save_app_config_internal,
};
use crate::utils::http_client;
use semver::Version;
use std::io::Cursor;
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
                db_save_app_config_internal(config, app_handle).await?;
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
        let cleaned =
            token.trim_matches(|c: char| c == ':' || c == ',' || c == ';' || c == ')' || c == '(');
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

/// 确保 metacubexd 外部 UI 已就绪。
/// 首次启动时 sing-box 会从 GitHub 下载 metacubexd，此下载在 API 启动前执行，
/// 可能导致稳定性校验超时。此函数在内核启动前预下载，消除该阻塞。
const METACUBEXD_URL: &str =
    "https://github.com/MetaCubeX/metacubexd/archive/refs/heads/gh-pages.zip";
const METACUBEXD_DIR: &str = "metacubexd";
const METACUBEXD_DOWNLOAD_TIMEOUT_SECS: u64 = 120;

pub async fn ensure_external_ui() -> Result<(), String> {
    let work_dir = paths::get_kernel_work_dir();
    let ui_dir = work_dir.join(METACUBEXD_DIR);

    // 检测 UI 是否已存在（以 index.html 为标志）
    if ui_dir.join("index.html").exists() {
        return Ok(());
    }

    info!("metacubexd UI 不存在，开始预下载: {}", METACUBEXD_URL);

    let client = http_client::get_client();
    let response = client
        .get(METACUBEXD_URL)
        .timeout(std::time::Duration::from_secs(METACUBEXD_DOWNLOAD_TIMEOUT_SECS))
        .send()
        .await
        .map_err(|e| format!("下载 metacubexd 失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!(
            "下载 metacubexd 失败，HTTP 状态码: {}",
            response.status()
        ));
    }

    let bytes = response
        .bytes()
        .await
        .map_err(|e| format!("读取 metacubexd 响应体失败: {}", e))?;

    info!(
        "metacubexd 下载完成 ({} 字节)，开始解压",
        bytes.len()
    );

    // 解压到临时目录，成功后原子重命名到目标位置
    let temp_dir = work_dir.join(format!("{}.tmp", METACUBEXD_DIR));
    if temp_dir.exists() {
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    }

    extract_zip_to_dir(&bytes, &temp_dir)?;

    // GitHub zip 内顶层目录名形如 "metacubexd-gh-pages"，需提取其内容
    let extracted_content = find_single_subdirectory(&temp_dir);
    let source_dir = extracted_content.as_ref().unwrap_or(&temp_dir);

    // 原子替换：先清理旧目录（如果有），再重命名
    if ui_dir.exists() {
        let _ = tokio::fs::remove_dir_all(&ui_dir).await;
    }
    tokio::fs::rename(source_dir, &ui_dir)
        .await
        .map_err(|e| format!("移动 metacubexd 目录失败: {}", e))?;

    // 清理临时目录
    if temp_dir.exists() {
        let _ = tokio::fs::remove_dir_all(&temp_dir).await;
    }

    info!("metacubexd UI 预下载安装完成: {:?}", ui_dir);
    Ok(())
}

/// 将 zip 数据解压到指定目录
fn extract_zip_to_dir(bytes: &[u8], target_dir: &Path) -> Result<(), String> {
    use std::fs;
    fs::create_dir_all(target_dir)
        .map_err(|e| format!("创建临时目录失败: {}", e))?;

    let reader = Cursor::new(bytes);
    let mut archive = zip::ZipArchive::new(reader)
        .map_err(|e| format!("解析 zip 失败: {}", e))?;

    for i in 0..archive.len() {
        let mut file = archive
            .by_index(i)
            .map_err(|e| format!("读取 zip 条目失败: {}", e))?;

        let out_path = match file.enclosed_name() {
            Some(path) => target_dir.join(path),
            None => continue,
        };

        if file.is_dir() {
            fs::create_dir_all(&out_path)
                .map_err(|e| format!("创建目录失败: {}", e))?;
        } else {
            if let Some(parent) = out_path.parent() {
                fs::create_dir_all(parent)
                    .map_err(|e| format!("创建父目录失败: {}", e))?;
            }
            let mut outfile = fs::File::create(&out_path)
                .map_err(|e| format!("创建文件失败: {}", e))?;
            std::io::copy(&mut file, &mut outfile)
                .map_err(|e| format!("写入文件失败: {}", e))?;

            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                if let Some(mode) = file.unix_mode() {
                    let perms = fs::Permissions::from_mode(mode);
                    fs::set_permissions(&out_path, perms)
                        .map_err(|e| format!("set permissions failed: {}", e))?;
                }
            }
        }
    }

    Ok(())
}

/// 查找 zip 解压后是否只有一个子目录（GitHub zip 通常如此）
fn find_single_subdirectory(dir: &Path) -> Option<std::path::PathBuf> {
    let mut entries = std::fs::read_dir(dir).ok()?;
    let first = entries.next()?.ok()?;
    // 如果只有一个条目且是目录，使用它作为源目录
    if entries.next().is_none() && first.file_type().ok()?.is_dir() {
        Some(first.path())
    } else {
        None
    }
}
