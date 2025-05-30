use crate::app::constants::{messages, network_config, paths, process};
use crate::process::manager::ProcessManager;
use crate::utils::app_util::get_work_dir;
use crate::utils::file_util::unzip_file;
use futures_util::StreamExt;
use serde_json::json;
use serde_json::Value;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::sync::Arc;
use tauri::Emitter;
use tauri::{Runtime, Window};
use tokio::sync::mpsc;
use tokio::task;
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};
use tracing::{error, info, warn};
use url::Url;

// 全局进程管理器
lazy_static::lazy_static! {
    pub(crate) static ref PROCESS_MANAGER: Arc<ProcessManager> = Arc::new(ProcessManager::new());
}

// 检查内核版本
#[tauri::command]
pub async fn check_kernel_version() -> Result<String, String> {
    let kernel_path = paths::get_kernel_path();

    if !kernel_path.exists() {
        return Err(messages::ERR_KERNEL_NOT_FOUND.to_string());
    }

    let output = std::process::Command::new(kernel_path)
        .arg("version")
        .creation_flags(process::CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("{}: {}", messages::ERR_VERSION_CHECK_FAILED, e))?;

    if !output.status.success() {
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("{}: {}", messages::ERR_GET_VERSION_FAILED, error));
    }

    let version_info = String::from_utf8_lossy(&output.stdout);
    Ok(version_info.to_string())
}

// 检查配置是否正常
#[tauri::command]
pub async fn check_config_validity(config_path: String) -> Result<(), String> {
    let kernel_path = paths::get_kernel_path();

    if !kernel_path.exists() {
        return Err(messages::ERR_KERNEL_NOT_FOUND.to_string());
    }

    // 确保配置文件路径存在
    let path = if config_path.is_empty() {
        paths::get_config_path().to_string_lossy().to_string()
    } else {
        config_path
    };

    // 检查配置文件是否存在
    if !std::path::Path::new(&path).exists() {
        return Err(format!("配置文件不存在: {}", path));
    }

    let output = std::process::Command::new(kernel_path)
        .arg("check")
        .arg("--config")
        .arg(path)
        .creation_flags(process::CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("执行配置检查命令失败: {}", e))?;

    // 检查命令是否成功执行
    if !output.status.success() {
        // 如果有错误输出，返回错误信息
        let error = String::from_utf8_lossy(&output.stderr);
        return Err(format!("配置检查失败: {}", error));
    }

    // 检查是否有标准输出（即使命令成功，可能也有警告信息）
    let stdout = String::from_utf8_lossy(&output.stdout);
    if !stdout.is_empty() {
        info!("配置检查有输出信息: {}", stdout);
        return Err(format!("配置检查警告: {}", stdout));
    }

    // 如果没有任何输出且命令成功执行，则配置正常
    Ok(())
}

// 运行内核
#[tauri::command]
pub async fn start_kernel(_proxy_mode: Option<String>) -> Result<(), String> {
    // 启动内核进程
    match PROCESS_MANAGER.start().await {
        Ok(_) => {
            info!("内核进程启动成功，现在配置代理模式");

            // 创建HTTP客户端用于检查内核状态
            let client = reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(2))
                .no_proxy()
                .build()
                .map_err(|e| format!("创建HTTP客户端失败: {}", e))?;
            
            let api_port = crate::app::system::config_service::get_api_port();
            let url = format!("http://127.0.0.1:{}/version?token=", api_port);
            
            // 内核启动检查的最大时间（秒）
            let max_check_time_secs = 20;
            // 每次检查的间隔（毫秒）
            let check_interval_ms = 1000;
            // 最大检查次数
            let max_checks = max_check_time_secs * 1000 / check_interval_ms;
            
            // 进行定时检查
            let mut api_ready = false;
            for i in 0..max_checks {
                info!("检查内核API是否就绪 (第{}次检查)...", i + 1);
                
                match client.get(&url).send().await {
                    Ok(_) => {
                        info!("内核API服务连接成功，内核已就绪");
                        api_ready = true;
                        break;
                    },
                    Err(e) => {
                        if i < max_checks - 1 {
                            info!("内核API服务暂未就绪，将在{}ms后重试: {}", check_interval_ms, e);
                            tokio::time::sleep(tokio::time::Duration::from_millis(check_interval_ms)).await;
                        } else {
                            info!("内核API服务在最大等待时间内无响应，将由前端确认启动状态");
                        }
                    }
                }
            }
            
            if !api_ready {
                info!("内核API服务在设定时间内未就绪，但进程已启动，将通过WebSocket确认状态");
                // 即使API未就绪也返回成功，交由前端通过WebSocket确认最终状态
            }
            
            Ok(())
        },
        Err(e) => {
            // 从错误中提取关键信息
            let error_string = e.to_string();
            let shortened_error = if error_string.len() > 300 {
                // 提取前300个字符
                format!("{}...(错误信息过长)", &error_string[..300])
            } else {
                error_string
            };
            
            error!("内核启动失败: {}", shortened_error);
            Err(shortened_error)
        }
    }
}

// 停止内核
#[tauri::command]
pub async fn stop_kernel() -> Result<(), String> {
    // 先尝试关闭系统代理，无论如何都继续执行后续操作
    if let Err(e) = crate::utils::proxy_util::disable_system_proxy() {
        warn!("关闭系统代理失败: {}", e);
    } else {
        info!("{}", messages::INFO_SYSTEM_PROXY_DISABLED);
    }

    // 停止内核
    PROCESS_MANAGER.stop().await.map_err(|e| e.to_string())
}

// 重启内核
#[tauri::command]
pub async fn restart_kernel() -> Result<(), String> {
    PROCESS_MANAGER.restart().await.map_err(|e| e.to_string())
}

// 下载内核
#[tauri::command]
pub async fn download_latest_kernel(window: tauri::Window) -> Result<(), String> {
    let work_dir = get_work_dir();
    info!("当前工作目录: {}", work_dir);

    let path = Path::new(&work_dir).join("sing-box/");
    info!("目标下载目录: {}", path.display());

    // 如果目录已存在，先检查是否为有效目录
    if path.exists() {
        if !path.is_dir() {
            error!("sing-box 路径存在但不是目录");
            return Err("sing-box 路径存在但不是目录".to_string());
        }
    }

    // 确保目录存在
    if let Err(e) = std::fs::create_dir_all(&path) {
        error!("创建目录失败: {}", e);
        return Err(format!("创建目录失败: {}", e));
    }
    info!("已确保下载目录存在");

    info!("正在准备下载最新版本...");
    // 发送进度事件
    let _ = window.emit(
        "download-progress",
        json!({
            "status": "checking",
            "progress": 0,
            "message": "正在获取最新版本信息..."
        }),
    );

    // 获取最新版本信息
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(
            network_config::HTTP_TIMEOUT_SECONDS,
        ))
        .no_proxy() // 禁用代理
        .build()
        .map_err(|e| format!("{}: {}", messages::ERR_HTTP_CLIENT_FAILED, e))?;
    let releases_url = "https://api.github.com/repos/SagerNet/sing-box/releases/latest";
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

    // 获取版本号
    let version = release["tag_name"]
        .as_str()
        .ok_or("无法获取版本号")?
        .trim_start_matches('v')
        .to_string();

    // 获取当前系统平台和架构
    let platform = std::env::consts::OS;
    let mut arch = std::env::consts::ARCH;
    if arch == "x86_64" {
        arch = "amd64";
    }

    // 构建目标文件名
    let target_asset_name = format!("sing-box-{}-{}-{}.zip", version, platform, arch);
    info!("目标文件名: {}", target_asset_name);

    // 查找Windows版本资源
    let assets = release["assets"].as_array().ok_or("无法获取发布资源")?;
    let asset = assets
        .iter()
        .find(|asset| {
            if let Some(name) = asset["name"].as_str() {
                name.contains("windows-amd64") && name.ends_with(".zip")
            } else {
                false
            }
        })
        .ok_or("未找到适用于Windows的资源")?;

    // 获取下载链接
    let original_url = asset["browser_download_url"]
        .as_str()
        .ok_or("无法获取下载链接")?;

    info!("找到下载链接: {}", original_url);

    let download_path = Path::new(&path).join(&target_asset_name);
    info!("目标下载路径: {}", download_path.display());

    // 发送进度事件
    let _ = window.emit(
        "download-progress",
        json!({
            "status": "downloading",
            "progress": 20,
            "message": format!("开始下载文件: {}", target_asset_name)
        }),
    );

    // 下载文件
    let window_clone = window.clone();
    if let Err(e) = crate::utils::file_util::download_with_fallback(
        original_url,
        download_path.to_str().unwrap(),
        move |progress| {
            let real_progress = 20 + (progress as f64 * 0.6) as u32; // 20-80%的进度用于下载
            let _ = window_clone.emit(
                "download-progress",
                json!({
                    "status": "downloading",
                    "progress": real_progress,
                    "message": format!("正在下载: {}%", progress)
                }),
            );
        },
    )
    .await
    {
        error!("下载失败: {}", e);
        return Err(format!(
            "下载失败: {}。\n您可以尝试手动下载：\n1. 访问 https://github.com/SagerNet/sing-box/releases/latest\n2. 下载 {}\n3. 解压并将文件放置在 {}/sing-box/ 目录下",
            e, target_asset_name, get_work_dir()
        ));
    }

    // 解压文件
    info!("开始解压文件...");
    // 发送进度事件
    let _ = window.emit(
        "download-progress",
        json!({
            "status": "extracting",
            "progress": 80,
            "message": "正在解压文件..."
        }),
    );

    let out_path = Path::new(&work_dir).join("sing-box");
    match unzip_file(download_path.to_str().unwrap(), out_path.to_str().unwrap()).await {
        Ok(_) => {
            info!("内核已下载并解压到: {}", out_path.display());

            // 删除原始的zip压缩包
            if let Err(e) = std::fs::remove_file(&download_path) {
                error!("删除压缩包失败: {}", e);
                info!("压缩包路径: {}", download_path.display());
                // 仅发出警告，不中断流程
            } else {
                info!("成功删除原始压缩包: {}", download_path.display());
            }

            // 发送完成事件
            let _ = window.emit(
                "download-progress",
                json!({
                    "status": "completed",
                    "progress": 100,
                    "message": "下载完成！"
                }),
            );
        }
        Err(e) => {
            error!("解压文件失败: {}", e);
            return Err(format!("解压文件失败: {}", e));
        }
    }

    Ok(())
}

/// 启动WebSocket数据中继
#[tauri::command]
pub async fn start_websocket_relay<R: Runtime>(window: Window<R>) -> Result<(), String> {
    // 启动四个不同类型的WebSocket中继
    start_traffic_relay(window.clone()).await?;
    start_memory_relay(window.clone()).await?;
    start_logs_relay(window.clone()).await?;
    start_connections_relay(window.clone()).await?;

    Ok(())
}

/// 启动流量数据中继
async fn start_traffic_relay<R: Runtime>(window: Window<R>) -> Result<(), String> {
    let window_clone = window.clone();
    let window_for_error = window.clone(); // 用于错误处理的窗口克隆
    let (tx, mut rx) = mpsc::channel(32);
    let api_port = crate::app::system::config_service::get_api_port();
    let token = crate::app::core::proxy_service::get_api_token();

    // 启动WebSocket连接和数据处理任务
    let _handle = task::spawn(async move {
        let url = Url::parse(&format!(
            "ws://127.0.0.1:{}/traffic?token={}",
            api_port,
            token
        ))
        .unwrap();

        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                // 连接成功，发送通知
                let _ = window_for_error.emit(
                    "traffic-connection",
                    json!({
                        "status": "connected"
                    }),
                );

                let (mut _write, mut read) = ws_stream.split();

                // 持续读取WebSocket消息
                while let Some(message) = read.next().await {
                    match message {
                        Ok(Message::Text(text)) => {
                            if let Ok(data) = serde_json::from_str::<Value>(&text) {
                                let _ = tx.send(data).await;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            error!("WebSocket流量连接关闭");
                            // 发送连接关闭通知
                            let _ = window_for_error.emit(
                                "traffic-connection",
                                json!({
                                    "status": "closed",
                                    "error": "WebSocket连接已关闭"
                                }),
                            );
                            break;
                        }
                        Err(e) => {
                            error!("WebSocket流量数据读取错误: {}", e);
                            // 发送错误通知
                            let _ = window_for_error.emit(
                                "traffic-connection",
                                json!({
                                    "status": "error",
                                    "error": format!("数据读取错误: {}", e)
                                }),
                            );
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                error!("WebSocket流量连接失败: {}", e);
                // 发送连接失败通知
                let _ = window_for_error.emit(
                    "traffic-connection",
                    json!({
                        "status": "failed",
                        "error": format!("连接失败: {}", e)
                    }),
                );
            }
        }
    });

    // 启动事件发送任务
    task::spawn(async move {
        while let Some(data) = rx.recv().await {
            let _ = window_clone.emit("traffic-data", data);
        }
    });

    Ok(())
}

/// 启动内存数据中继
async fn start_memory_relay<R: Runtime>(window: Window<R>) -> Result<(), String> {
    let window_clone = window.clone();
    let window_for_error = window.clone(); // 用于错误处理的窗口克隆
    let (tx, mut rx) = mpsc::channel(32);
    let api_port = crate::app::system::config_service::get_api_port();
    let token = crate::app::core::proxy_service::get_api_token();

    // 启动WebSocket连接和数据处理任务
    let _handle = task::spawn(async move {
        let url = Url::parse(&format!(
            "ws://127.0.0.1:{}/memory?token={}",
            api_port,
            token
        ))
        .unwrap();

        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                // 连接成功，发送通知
                let _ = window_for_error.emit(
                    "memory-connection",
                    json!({
                        "status": "connected"
                    }),
                );

                let (mut _write, mut read) = ws_stream.split();

                // 持续读取WebSocket消息
                while let Some(message) = read.next().await {
                    match message {
                        Ok(Message::Text(text)) => {
                            if let Ok(data) = serde_json::from_str::<Value>(&text) {
                                let _ = tx.send(data).await;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            error!("WebSocket内存连接关闭");
                            // 发送连接关闭通知
                            let _ = window_for_error.emit(
                                "memory-connection",
                                json!({
                                    "status": "closed",
                                    "error": "WebSocket连接已关闭"
                                }),
                            );
                            break;
                        }
                        Err(e) => {
                            error!("WebSocket内存数据读取错误: {}", e);
                            // 发送错误通知
                            let _ = window_for_error.emit(
                                "memory-connection",
                                json!({
                                    "status": "error",
                                    "error": format!("数据读取错误: {}", e)
                                }),
                            );
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                error!("WebSocket内存连接失败: {}", e);
                // 发送连接失败通知
                let _ = window_for_error.emit(
                    "memory-connection",
                    json!({
                        "status": "failed",
                        "error": format!("连接失败: {}", e)
                    }),
                );
            }
        }
    });

    // 启动事件发送任务
    task::spawn(async move {
        while let Some(data) = rx.recv().await {
            let _ = window_clone.emit("memory-data", data);
        }
    });

    Ok(())
}

/// 启动日志数据中继
async fn start_logs_relay<R: Runtime>(window: Window<R>) -> Result<(), String> {
    let window_clone = window.clone();
    let window_for_error = window.clone(); // 用于错误处理的窗口克隆
    let (tx, mut rx) = mpsc::channel(32);
    let api_port = crate::app::system::config_service::get_api_port();
    let token = crate::app::core::proxy_service::get_api_token();

    // 启动WebSocket连接和数据处理任务
    let _handle = task::spawn(async move {
        let url = Url::parse(&format!(
            "ws://127.0.0.1:{}/logs?token={}",
            api_port,
            token
        ))
        .unwrap();

        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                // 连接成功，发送通知
                let _ = window_for_error.emit(
                    "logs-connection",
                    json!({
                        "status": "connected"
                    }),
                );

                let (mut _write, mut read) = ws_stream.split();

                // 持续读取WebSocket消息
                while let Some(message) = read.next().await {
                    match message {
                        Ok(Message::Text(text)) => {
                            if let Ok(data) = serde_json::from_str::<Value>(&text) {
                                let _ = tx.send(data).await;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            error!("WebSocket日志连接关闭");
                            // 发送连接关闭通知
                            let _ = window_for_error.emit(
                                "logs-connection",
                                json!({
                                    "status": "closed",
                                    "error": "WebSocket连接已关闭"
                                }),
                            );
                            break;
                        }
                        Err(e) => {
                            error!("WebSocket日志数据读取错误: {}", e);
                            // 发送错误通知
                            let _ = window_for_error.emit(
                                "logs-connection",
                                json!({
                                    "status": "error",
                                    "error": format!("数据读取错误: {}", e)
                                }),
                            );
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                error!("WebSocket日志连接失败: {}", e);
                // 发送连接失败通知
                let _ = window_for_error.emit(
                    "logs-connection",
                    json!({
                        "status": "failed",
                        "error": format!("连接失败: {}", e)
                    }),
                );
            }
        }
    });

    // 启动事件发送任务
    task::spawn(async move {
        while let Some(data) = rx.recv().await {
            let _ = window_clone.emit("log-data", data);
        }
    });

    Ok(())
}

/// 启动连接数据中继
async fn start_connections_relay<R: Runtime>(window: Window<R>) -> Result<(), String> {
    let window_clone = window.clone();
    let window_for_error = window.clone(); // 用于错误处理的窗口克隆
    let (tx, mut rx) = mpsc::channel(32);
    let api_port = crate::app::system::config_service::get_api_port();
    let token = crate::app::core::proxy_service::get_api_token();

    // 启动WebSocket连接和数据处理任务
    let _handle = task::spawn(async move {
        let url = Url::parse(&format!(
            "ws://127.0.0.1:{}/connections?token={}",
            api_port,
            token
        ))
        .unwrap();

        match connect_async(url).await {
            Ok((ws_stream, _)) => {
                // 连接成功，发送通知
                let _ = window_for_error.emit(
                    "connections-connection",
                    json!({
                        "status": "connected"
                    }),
                );

                let (mut _write, mut read) = ws_stream.split();

                // 持续读取WebSocket消息
                while let Some(message) = read.next().await {
                    match message {
                        Ok(Message::Text(text)) => {
                            if let Ok(data) = serde_json::from_str::<Value>(&text) {
                                let _ = tx.send(data).await;
                            }
                        }
                        Ok(Message::Close(_)) => {
                            error!("WebSocket连接数据连接关闭");
                            // 发送连接关闭通知
                            let _ = window_for_error.emit(
                                "connections-connection",
                                json!({
                                    "status": "closed",
                                    "error": "WebSocket连接已关闭"
                                }),
                            );
                            break;
                        }
                        Err(e) => {
                            error!("WebSocket连接数据读取错误: {}", e);
                            // 发送错误通知
                            let _ = window_for_error.emit(
                                "connections-connection",
                                json!({
                                    "status": "error",
                                    "error": format!("数据读取错误: {}", e)
                                }),
                            );
                            break;
                        }
                        _ => {}
                    }
                }
            }
            Err(e) => {
                error!("WebSocket连接数据连接失败: {}", e);
                // 发送连接失败通知
                let _ = window_for_error.emit(
                    "connections-connection",
                    json!({
                        "status": "failed",
                        "error": format!("连接失败: {}", e)
                    }),
                );
            }
        }
    });

    // 启动事件发送任务
    task::spawn(async move {
        while let Some(data) = rx.recv().await {
            let _ = window_clone.emit("connections-data", data);
        }
    });

    Ok(())
}

// 检查内核是否正在运行
#[tauri::command]
pub async fn is_kernel_running() -> Result<bool, String> {
    // 通过tasklist命令检查sing-box.exe是否在运行
    let output = std::process::Command::new("tasklist")
        .args(&["/FI", "IMAGENAME eq sing-box.exe", "/FO", "CSV", "/NH"])
        .creation_flags(crate::app::constants::process::CREATE_NO_WINDOW)
        .output()
        .map_err(|e| format!("检查内核进程失败: {}", e))?;
    
    // 检查输出中是否包含sing-box.exe
    let stdout = String::from_utf8_lossy(&output.stdout);
    let is_running = stdout.contains("sing-box.exe");
    
    info!("内核运行状态检查: {}", is_running);
    Ok(is_running)
}

