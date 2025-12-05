use crate::app::constants::{common::messages, paths};
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use serde::Deserialize;
use serde_json;
use std::process::Command;
use tauri::AppHandle;
use tracing::{info, warn};

pub(super) async fn get_latest_kernel_version(
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    #[derive(Deserialize)]
    struct GitHubRelease {
        tag_name: String,
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("sing-box-windows/1.8.2")
        .build()?;

    let api_urls = vec![
        "https://api.github.com/repos/SagerNet/sing-box/releases/latest",
        "https://v6.gh-proxy.com/https://api.github.com/repos/SagerNet/sing-box/releases/latest",
        "https://gh-proxy.com/https://api.github.com/repos/SagerNet/sing-box/releases/latest",
        "https://ghfast.top/https://api.github.com/repos/SagerNet/sing-box/releases/latest",
    ];

    for (index, api_url) in api_urls.iter().enumerate() {
        info!("尝试第 {} 个 API 源获取版本: {}", index + 1, api_url);

        match client.get(*api_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let release: GitHubRelease = response.json().await?;
                    let tag_name = release.tag_name;

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

pub(super) async fn get_kernel_releases() -> Result<Vec<String>, Box<dyn std::error::Error + Send + Sync>> {
    #[derive(Deserialize)]
    struct GitHubRelease {
        tag_name: String,
        prerelease: bool,
    }

    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(20))
        .user_agent("sing-box-windows/1.8.2")
        .build()?;

    let api_urls = vec![
        "https://api.github.com/repos/SagerNet/sing-box/releases",
        "https://v6.gh-proxy.com/https://api.github.com/repos/SagerNet/sing-box/releases",
        "https://gh-proxy.com/https://api.github.com/repos/SagerNet/sing-box/releases",
        "https://ghfast.top/https://api.github.com/repos/SagerNet/sing-box/releases",
    ];

    for (index, api_url) in api_urls.iter().enumerate() {
        info!("尝试第 {} 个 API 源获取版本列表: {}", index + 1, api_url);

        match client.get(*api_url).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    let releases: Vec<GitHubRelease> = response.json().await?;
                    let versions: Vec<String> = releases.into_iter()
                        .filter(|r| !r.prerelease) // Filter out GitHub pre-releases
                        .map(|r| {
                            let v = if r.tag_name.starts_with('v') {
                                r.tag_name[1..].to_string()
                            } else {
                                r.tag_name
                            };
                            v
                        })
                        .filter(|v| {
                            let lower = v.to_lowercase();
                            !lower.contains("rc") && !lower.contains("beta") && !lower.contains("alpha")
                        })
                        .collect();

                    info!("成功获取版本列表（已过滤正式版），共 {} 个版本 (来源: {})", versions.len(), api_url);
                    return Ok(versions);
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

    Err("所有 API 源都获取版本列表失败".into())
}

fn normalize_version_str(raw: &str) -> String {
    let mut cleaned = raw.trim();
    if cleaned.starts_with("sing-box") {
        cleaned = cleaned.trim_start_matches("sing-box").trim();
    }
    if cleaned.is_empty() {
        return String::new();
    }

    if let Some(token) = cleaned.split_whitespace().find(|part| {
        part.chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == 'v')
    }) {
        return token.trim_start_matches('v').to_string();
    }

    cleaned.trim_start_matches('v').to_string()
}

fn extract_clean_version(output: &str) -> String {
    let trimmed = output.trim();
    if trimmed.is_empty() {
        return String::new();
    }

    if let Ok(value) = serde_json::from_str::<serde_json::Value>(trimmed) {
        if let Some(ver) = value.get("version").and_then(|v| v.as_str()) {
            return normalize_version_str(ver);
        }
    }

    if let Some(pos) = trimmed.find("version") {
        let after_version = trimmed[pos + "version".len()..]
            .trim_start_matches(|c: char| c == ':' || c.is_whitespace());

        if let Some(token) = after_version.split_whitespace().next() {
            if !token.is_empty() {
                return normalize_version_str(token);
            }
        }
    }

    if let Some(token) = trimmed.split_whitespace().find(|part| {
        part.chars()
            .all(|c| c.is_ascii_digit() || c == '.' || c == 'v')
    }) {
        return normalize_version_str(token);
    }

    normalize_version_str(trimmed.split("Environment").next().unwrap_or(trimmed))
}

#[tauri::command]
pub async fn check_kernel_version(app_handle: AppHandle) -> Result<String, String> {
    // 1. 尝试从数据库读取缓存的版本号
    use crate::app::storage::enhanced_storage_service::db_get_app_config;
    if let Ok(config) = db_get_app_config(app_handle.clone()).await {
        if let Some(ver) = config.installed_kernel_version {
            if !ver.is_empty() {
                // optional: 验证一下文件是否存在，避免只是数据库有记录但文件没了
                 let kernel_path = paths::get_kernel_path();
                 if kernel_path.exists() {
                     info!("从数据库读取缓存的内核版本: {}", ver);
                     return Ok(ver);
                 }
            }
        }
    }

    // 2. 如果数据库没有或文件不存在，回退到执行命令检查
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
    let version = extract_clean_version(&version_info);
    
    // 3. 将查到的版本回写到数据库，下次就不用查了
    use crate::app::storage::enhanced_storage_service::db_save_app_config_internal;
    if let Ok(mut config) = db_get_app_config(app_handle.clone()).await {
        // 只有当如果不一致时才保存? 或者总是保存确保最新
        config.installed_kernel_version = Some(version.clone());
        let _ = db_save_app_config_internal(config, app_handle).await;
    }

    Ok(version)
}

#[tauri::command]
pub async fn check_config_validity(
    app_handle: AppHandle,
    config_path: String,
) -> Result<(), String> {
    let kernel_path = paths::get_kernel_path();

    if !kernel_path.exists() {
        return Err(messages::ERR_KERNEL_NOT_FOUND.to_string());
    }

    let path = if config_path.is_empty() {
        let app_config = db_get_app_config(app_handle)
            .await
            .map_err(|e| format!("获取应用配置失败: {}", e))?;

        if let Some(path_str) = app_config.active_config_path {
            path_str
        } else {
            paths::get_config_dir()
                .join("config.json")
                .to_string_lossy()
                .to_string()
        }
    } else {
        config_path
    };

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

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("配置检查失败: {}", error));
    }

    Ok(())
}

pub(super) fn get_system_arch() -> &'static str {
    if let Ok(force_arch) = std::env::var("SING_BOX_FORCE_ARCH") {
        info!("用户手动指定架构: {}", force_arch);
        return match force_arch.as_str() {
            "amd64" | "x86_64" => "amd64",
            "386" | "i386" => "386",
            "arm64" | "aarch64" => "arm64",
            "armv5" => "armv5",
            _ => "amd64",
        };
    }

    info!("Rust ARCH 常量: {}", std::env::consts::ARCH);

    if cfg!(target_os = "windows") {
        match std::env::consts::ARCH {
            "x86_64" => "amd64",
            "x86" => "386",
            "aarch64" => "arm64",
            _ => "amd64",
        }
    } else if cfg!(target_os = "linux") {
        let mut detected_arch = "amd64";

        if let Ok(output) = Command::new("uname").arg("-m").output() {
            if let Ok(arch_str) = String::from_utf8(output.stdout) {
                let arch = arch_str.trim();
                info!("uname -m 输出: '{}'", arch);

                detected_arch = match arch {
                    "x86_64" | "amd64" => "amd64",
                    "i386" | "i486" | "i586" | "i686" => "386",
                    "aarch64" | "arm64" => "arm64",
                    "armv7l" | "armv6l" => "armv5",
                    _ => match std::env::consts::ARCH {
                        "x86_64" => "amd64",
                        "x86" => "386",
                        "aarch64" => "arm64",
                        _ => "amd64",
                    },
                };
                info!("通过 uname 检测到的架构: {}", detected_arch);
            }
        } else {
            info!("uname 命令执行失败，使用 Rust ARCH 常量");
        }

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
        let mut detected_arch = "amd64";

        if let Ok(output) = Command::new("uname").arg("-m").output() {
            if let Ok(arch_str) = String::from_utf8(output.stdout) {
                let arch = arch_str.trim();
                info!("uname -m 输出: '{}'", arch);

                detected_arch = match arch {
                    "x86_64" | "amd64" => "amd64",
                    "i386" | "i486" | "i586" | "i686" => "386",
                    "aarch64" | "arm64" => "arm64",
                    "armv7l" | "armv6l" => "armv5",
                    _ => match std::env::consts::ARCH {
                        "x86_64" => "amd64",
                        "x86" => "386",
                        "aarch64" => "arm64",
                        _ => "amd64",
                    },
                };
                info!("通过 uname 检测到的架构: {}", detected_arch);
            }
        } else {
            info!("uname 命令执行失败，使用 Rust ARCH 常量");
        }

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
        "amd64"
    }
}

#[tauri::command]
pub async fn get_latest_kernel_version_cmd() -> Result<String, String> {
    get_latest_kernel_version().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_kernel_releases_cmd() -> Result<Vec<String>, String> {
    get_kernel_releases().await.map_err(|e| e.to_string())
}
