//! 内核安装态探测（文件层事实）
//!
//! # 为什么需要这个模块
//!
//! 「内核是否已安装」与「内核是否正在运行」是两件独立的事，但前端此前只有一个
//! `version` 字段同时承担两者——它取自**运行中**内核的 gRPC `GetVersion`，
//! 于是产生两个用户可见的错觉：
//!
//! 1. 内核没在跑时 `version` 是 `null`，界面一律显示「未安装」——哪怕二进制文件
//!    好好地躺在 `%LOCALAPPDATA%\sing-box-windows\sing-box\sing-box.exe`；
//! 2. 内核启动失败（配置非法 / 被杀毒软件隔离 / 端口占用）时，界面同样只显示
//!    「未安装」，把真实原因藏了起来，用户只能靠猜。
//!
//! 这里给出**文件层**的权威事实：文件在不在、版本读不读得到、曾经装过现在是不是
//! 不见了（疑似被杀毒软件隔离或被清理）。运行态仍由 `kernel_service/status` 负责，
//! 两者在前端分开展示，不再互相顶替。
//!
//! 探测分两档，避免把「3 秒轮询」变成「3 秒拉起一个子进程」：
//!
//! - **轻量**（`installed` / `installed_version`）：只查文件是否存在 + 读数据库里
//!   缓存的版本号，不 spawn 任何进程，随状态轮询一起返回；
//! - **完整**（`get_kernel_installation`）：必要时才执行一次 `sing-box version`，
//!   仅在启动、下载完成后、以及用户手动点「检查安装」时调用。

use crate::app::constants::{common::messages, paths};
use crate::app::core::kernel_service::versioning::extract_clean_version;
use crate::app::storage::enhanced_storage_service::{
    db_get_app_config, db_save_app_config_internal,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use tauri::AppHandle;
use tracing::{info, warn};

/// 内核安装态（文件层）
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InstallState {
    /// 内核文件不存在，且没有任何历史安装记录 —— 从未安装
    Missing,
    /// 内核文件曾经存在过（数据库有版本记录），现在不见了 —— 疑似被杀毒软件隔离或清理
    FileLost,
    /// 内核文件存在，版本可读 —— 已安装
    Installed,
    /// 内核文件存在，但读不出版本 —— 无法执行（被安全软件拦截）或文件损坏
    Unreadable,
}

impl InstallState {
    /// 是否可以认为「内核已安装」（文件确实在磁盘上）
    pub fn is_installed(&self) -> bool {
        matches!(self, InstallState::Installed | InstallState::Unreadable)
    }
}

/// 内核安装态探测结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelInstallation {
    /// 安装态
    pub state: InstallState,
    /// 已安装版本号（读得到时才有）
    pub version: Option<String>,
    /// 技术细节（失败时的 stderr 片段等），用于排障；面向用户的文案由前端按 `state` 映射
    pub detail: Option<String>,
    /// 内核可执行文件路径
    pub kernel_path: String,
}

/// 纯函数判定安装态
///
/// 拆成纯函数是为了能直接单测：文件在不在、历史上装过没有、版本读不读得到，
/// 三者组合出四种状态。
pub(crate) fn classify(
    file_exists: bool,
    had_recorded_version: bool,
    version_readable: bool,
) -> InstallState {
    if file_exists {
        if version_readable {
            InstallState::Installed
        } else {
            InstallState::Unreadable
        }
    } else if had_recorded_version {
        InstallState::FileLost
    } else {
        InstallState::Missing
    }
}

/// 执行内核二进制读取版本号
///
/// 失败时返回可读的 stderr 片段——「文件在但跑不起来」最常见的原因是安全软件
/// 拦截了执行，把原始错误带出去才能让用户知道该去哪儿看。
pub async fn read_version_from_binary(kernel_path: &Path) -> Result<String, String> {
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
        let error = error.trim();
        return Err(if error.is_empty() {
            messages::ERR_GET_VERSION_FAILED.to_string()
        } else {
            format!("{}: {}", messages::ERR_GET_VERSION_FAILED, error)
        });
    }

    let version_info = String::from_utf8_lossy(&output.stdout);
    Ok(extract_clean_version(&version_info))
}

/// 读取数据库里缓存的已安装版本号
async fn recorded_version(app_handle: &AppHandle) -> Option<String> {
    match db_get_app_config(app_handle.clone()).await {
        Ok(config) => config.installed_kernel_version.filter(|v| !v.is_empty()),
        Err(e) => {
            warn!("读取已安装内核版本记录失败: {}", e);
            None
        }
    }
}

/// 把探测到的版本号回写数据库（下次就不用再执行二进制了）
async fn persist_version(app_handle: &AppHandle, version: &str) {
    if let Ok(mut config) = db_get_app_config(app_handle.clone()).await {
        if config.installed_kernel_version.as_deref() == Some(version) {
            return;
        }
        config.installed_kernel_version = Some(version.to_string());
        if let Err(e) = db_save_app_config_internal(config, app_handle).await {
            warn!("保存已安装内核版本失败: {}", e);
        }
    }
}

/// 轻量探测：只查文件是否存在 + 已知版本记录，不执行任何子进程
///
/// 供 3 秒轮询的状态接口使用。`recorded` 由调用方传入（调用方往往刚读过同一份
/// 应用配置），避免一次轮询里重复读库。
pub fn probe_installation_light_with(recorded: Option<String>) -> KernelInstallation {
    let kernel_path = paths::get_kernel_path();
    let file_exists = kernel_path.exists();
    let recorded = recorded.filter(|v| !v.is_empty());

    let state = classify(
        file_exists,
        recorded.is_some(),
        file_exists && recorded.is_some(),
    );
    // 文件不在时，缓存版本号一并作废——那描述的是一个已经不存在的文件
    let version = if file_exists { recorded } else { None };

    KernelInstallation {
        state,
        version,
        detail: None,
        kernel_path: kernel_path.to_string_lossy().to_string(),
    }
}

/// 轻量探测（自行读库版本记录）
pub async fn probe_installation_light(app_handle: &AppHandle) -> KernelInstallation {
    let recorded = recorded_version(app_handle).await;
    probe_installation_light_with(recorded)
}

/// 完整探测：必要时执行一次 `sing-box version` 读出版本
///
/// 仅在启动、下载完成后、以及用户手动点「检查安装」时调用——它会拉起一个子进程，
/// 不适合放进高频轮询。
pub async fn probe_installation(app_handle: &AppHandle) -> KernelInstallation {
    let kernel_path = paths::get_kernel_path();
    let file_exists = kernel_path.exists();
    let recorded = recorded_version(app_handle).await;

    if !file_exists {
        let state = classify(false, recorded.is_some(), false);
        let detail = if state == InstallState::FileLost {
            Some(messages::ERR_KERNEL_FILE_LOST.to_string())
        } else {
            None
        };
        return KernelInstallation {
            state,
            version: None,
            detail,
            kernel_path: kernel_path.to_string_lossy().to_string(),
        };
    }

    // 文件在：优先用数据库缓存版本，避免每次都拉起子进程
    if let Some(version) = recorded.clone() {
        return KernelInstallation {
            state: classify(true, true, true),
            version: Some(version),
            detail: None,
            kernel_path: kernel_path.to_string_lossy().to_string(),
        };
    }

    // 数据库没有缓存（首次运行 / 手动替换过二进制）：执行一次读版本
    match read_version_from_binary(&kernel_path).await {
        Ok(version) => {
            info!("探测到已安装内核版本: {}", version);
            persist_version(app_handle, &version).await;
            KernelInstallation {
                state: classify(true, false, true),
                version: Some(version),
                detail: None,
                kernel_path: kernel_path.to_string_lossy().to_string(),
            }
        }
        Err(e) => {
            warn!("内核文件存在但无法读取版本: {}", e);
            KernelInstallation {
                state: classify(true, false, false),
                version: None,
                detail: Some(e),
                kernel_path: kernel_path.to_string_lossy().to_string(),
            }
        }
    }
}

/// 查询内核安装态（完整探测）
#[tauri::command]
pub async fn get_kernel_installation(app_handle: AppHandle) -> Result<KernelInstallation, String> {
    Ok(probe_installation(&app_handle).await)
}

#[cfg(test)]
#[path = "installation.tests.rs"]
mod tests;
