use crate::app::constants::{api, messages};
use crate::utils::app_util::get_work_dir;
use serde_json::json;
use std::os::windows::process::CommandExt;
use std::path::Path;
use tauri::Emitter;
use crate::app::network_config;

// 添加新的结构体用于版本信息
#[derive(serde::Serialize)]
pub struct UpdateInfo {
    pub latest_version: String,
    pub download_url: String,
    pub has_update: bool,
}

// 检查更新
#[tauri::command]
pub async fn check_update(current_version: String) -> Result<UpdateInfo, String> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(
            network_config::HTTP_TIMEOUT_SECONDS,
        ))
        .no_proxy() // 禁用代理
        .build()
        .map_err(|e| format!("{}: {}", messages::ERR_HTTP_CLIENT_FAILED, e))?;

    // 获取最新版本信息
    let response = client
        .get(api::GITHUB_API_URL)
        .header("User-Agent", api::USER_AGENT)
        .send()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_GET_VERSION_FAILED, e))?;

    let release: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("{}: {}", messages::ERR_GET_VERSION_FAILED, e))?;

    // 获取最新版本号
    let tag_name = release["tag_name"]
        .as_str()
        .ok_or_else(|| format!("{}: 无法解析版本号", messages::ERR_GET_VERSION_FAILED))
        .map(|v| v.trim_start_matches('v').to_string())?;

    // 获取下载链接
    let assets = release["assets"]
        .as_array()
        .ok_or_else(|| format!("{}: 无法获取下载资源", messages::ERR_GET_VERSION_FAILED))?;

    // 查找Windows安装程序
    let mut download_url = String::new();
    for asset in assets {
        let name = asset["name"].as_str().unwrap_or("");
        if name.ends_with(".msi") || name.ends_with(".exe") {
            download_url = asset["browser_download_url"]
                .as_str()
                .unwrap_or("")
                .to_string();
            break;
        }
    }

    if download_url.is_empty() {
        return Err(format!(
            "{}: 无法获取下载链接",
            messages::ERR_GET_VERSION_FAILED
        ));
    }

    // 简单比较版本号
    let has_update = tag_name != current_version;

    Ok(UpdateInfo {
        latest_version: tag_name.to_string(),
        download_url,
        has_update,
    })
}

// 下载并安装更新
#[tauri::command]
pub async fn download_and_install_update(
    window: tauri::Window,
    download_url: String,
) -> Result<(), String> {
    let work_dir = get_work_dir();
    let download_path = Path::new(&work_dir).join("update.exe");

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
        return Err(format!("下载更新失败: {}", e));
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

    // 启动安装程序
    std::process::Command::new(download_path)
        .creation_flags(0x08000000)
        .spawn()
        .map_err(|e| format!("启动安装程序失败: {}", e))?;

    Ok(())
}
