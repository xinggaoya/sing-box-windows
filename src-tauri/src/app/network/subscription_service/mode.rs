use crate::app::singbox_api::{ApiClientConfig, ApiClientHandle};
use crate::app::storage::enhanced_storage_service::{db_get_app_config, db_save_app_config_internal};
use tauri::AppHandle;
use tracing::{info, warn};

/// 支持的代理模式（与前端 HomeView 的节点模式开关一致）
pub const PROXY_MODES: [&str; 2] = ["global", "rule"];

/// 切换代理模式的实现入口。
///
/// sing-box 1.14+ 已移除 `experimental.clash_api.default_mode`，代理模式完全由
/// gRPC `SetClashMode` 运行时切换；路由语义由配置生成器写入的
/// `clash_mode: global → default_outbound` / `clash_mode: direct → direct` 规则对承接
/// （sing-box 仅在运行时模式与规则值相等时命中该规则）。
///
/// 因此本函数只做两件事：
/// 1. 把模式持久化到 AppConfig（`clash_mode` 列）；
/// 2. 内核运行中时通过 gRPC 立即生效；未运行则返回"重启后生效"提示
///    （前端按返回文本是否包含"重启后生效"区分提示样式），重启后由
///    `apply_persisted_clash_mode` 恢复。
///
/// 注意：不要再往 route.rules 写 `{clash_mode: <mode>, outbound: ...}` —— 运行时模式
/// 默认是 rule，写 `clash_mode: rule` 的 catch-all 规则会让全部流量命中单条规则，
/// 且 outbound tag 无法与订阅生成的出站 tag 对齐（历史实现硬编码 "proxy" 导致内核
/// 校验失败无法启动）。
pub async fn toggle_proxy_mode_impl(app_handle: AppHandle, mode: String) -> Result<String, String> {
    if !PROXY_MODES.contains(&mode.as_str()) {
        return Err(format!("无效的代理模式: {}", mode));
    }

    info!("正在切换代理模式为: {}", mode);

    // 1) 持久化（整读整写，只改 clash_mode 一个字段，其余字段原样保存）
    let mut app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;
    app_config.clash_mode = mode.clone();
    db_save_app_config_internal(app_config, &app_handle)
        .await
        .map_err(|e| format!("保存代理模式失败: {}", e))?;

    // 2) 运行中的内核立即生效；未运行则等待下次启动时恢复
    match set_clash_mode_via_grpc(&app_handle, &mode).await {
        Ok(_) => {
            info!("代理模式已切换: {}", mode);
            Ok(format!("代理模式已切换为: {}", mode))
        }
        Err(e) => {
            info!(
                "gRPC 切换代理模式失败（内核未运行或未就绪）: {}，模式已持久化，重启内核后生效",
                e
            );
            Ok(format!("代理模式已保存为: {}，重启内核后生效", mode))
        }
    }
}

pub async fn get_current_proxy_mode_impl(app_handle: AppHandle) -> Result<String, String> {
    // 内核运行中：以 gRPC 实时状态为准
    if let Ok(mode) = get_clash_mode_via_grpc(&app_handle).await {
        if PROXY_MODES.contains(&mode.as_str()) {
            return Ok(mode);
        }
    }

    // 未运行 / gRPC 失败：回退到持久化值
    let app_config = db_get_app_config(app_handle)
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;
    if PROXY_MODES.contains(&app_config.clash_mode.as_str()) {
        Ok(app_config.clash_mode)
    } else {
        Ok("rule".to_string())
    }
}

/// 内核启动成功后应用持久化的代理模式（gRPC SetClashMode）。
///
/// 启动稳定性校验只做 TCP 端口探测，gRPC 服务可能仍在初始化，因此带有限次重试。
/// 失败仅告警，不影响启动流程——前端首页的模式指示以 gRPC 实时状态为准。
pub async fn apply_persisted_clash_mode(app_handle: &AppHandle) {
    let mode = match db_get_app_config(app_handle.clone()).await {
        Ok(c) => c.clash_mode,
        Err(e) => {
            warn!("读取持久化代理模式失败，跳过应用: {}", e);
            return;
        }
    };
    if !PROXY_MODES.contains(&mode.as_str()) {
        return;
    }

    const MAX_ATTEMPTS: usize = 3;
    for attempt in 1..=MAX_ATTEMPTS {
        match set_clash_mode_via_grpc(app_handle, &mode).await {
            Ok(_) => {
                info!("已应用持久化代理模式: {}", mode);
                return;
            }
            Err(e) => {
                if attempt < MAX_ATTEMPTS {
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                } else {
                    warn!(
                        "应用持久化代理模式 {} 失败（重试 {} 次后放弃）: {}",
                        mode, MAX_ATTEMPTS, e
                    );
                }
            }
        }
    }
}

async fn make_grpc_handle(app_handle: &AppHandle) -> Result<ApiClientHandle, String> {
    let app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;
    Ok(ApiClientHandle::new(ApiClientConfig::localhost(
        app_config.api_port,
    )))
}

async fn set_clash_mode_via_grpc(app_handle: &AppHandle, mode: &str) -> Result<(), String> {
    let handle = make_grpc_handle(app_handle).await?;
    handle
        .set_clash_mode(mode)
        .await
        .map_err(|e| format!("gRPC SetClashMode 失败: {}", e))
}

async fn get_clash_mode_via_grpc(app_handle: &AppHandle) -> Result<String, String> {
    let handle = make_grpc_handle(app_handle).await?;
    handle
        .get_clash_mode_status()
        .await
        .map(|s| s.current_mode)
        .map_err(|e| format!("gRPC GetClashModeStatus 失败: {}", e))
}
