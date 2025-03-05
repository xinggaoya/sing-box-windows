use crate::utils::app_util::get_work_dir;
use crate::utils::file_util::download_file;
use serde_json::json;
use std::os::windows::process::CommandExt;
use std::path::Path;
use tauri::Emitter;

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
    let client = reqwest::Client::new();

    // 获取最新版本信息
    let releases_url = "https://api.github.com/repos/xinggaoya/sing-box-windows/releases/latest";
    let response = client
        .get(releases_url)
        .header("User-Agent", "sing-box-windows")
        .send()
        .await
        .map_err(|e| format!("获取版本信息失败: {}", e))?;

    let release: serde_json::Value = response
        .json()
        .await
        .map_err(|e| format!("解析版本信息失败: {}", e))?;

    let latest_version = release["tag_name"]
        .as_str()
        .ok_or("无法获取最新版本号")?
        .trim_start_matches('v')
        .to_string();

    // 查找 .exe 资源
    let assets = release["assets"].as_array().ok_or("无法获取发布资源")?;

    let exe_asset = assets
        .iter()
        .find(|asset| {
            asset["name"]
                .as_str()
                .map(|name| name.ends_with(".exe"))
                .unwrap_or(false)
        })
        .ok_or("未找到可执行文件")?;

        let download_url = format!("https://gh-proxy.com/{}", exe_asset["browser_download_url"]
        .as_str()
        .ok_or("无法获取下载链接")?);


    // 比较版本号
    let has_update = latest_version != current_version;

    Ok(UpdateInfo {
        latest_version,
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
    download_file(
        download_url.to_string(),
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
    .map_err(|e| format!("下载更新失败: {}", e))?;

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