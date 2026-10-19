//! gRPC API 桥接（sing-box 1.14+ 官方 `type: api`）
//!
//! 把 20 余条前端命令映射到 sing-box 官方 gRPC 方法。所有命令都需要先拿到
//! [`ApiClientHandle`]（按 db 中的 `api_port` 组装），因此统一经 [`make_handle_async`]。

use crate::app::singbox_api::{ApiClientConfig, ApiClientHandle, Groups};
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use tauri::AppHandle;

/// 异步获取 gRPC 客户端句柄
pub(super) async fn make_handle_async(
    app_handle: AppHandle,
) -> Result<(ApiClientHandle, ApiClientConfig), String> {
    let app_config = db_get_app_config(app_handle)
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;
    let config = ApiClientConfig::localhost(app_config.api_port);
    let handle = ApiClientHandle::new(config.clone()).map_err(|e| e.to_string())?;
    Ok((handle, config))
}

/// 获取 sing-box 内核版本号
#[tauri::command]
pub async fn get_kernel_version(app_handle: AppHandle) -> Result<String, String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .get_version()
        .await
        .map(|(v, _)| v)
        .map_err(|e| format!("获取内核版本失败: {}", e))
}

/// 获取所有节点组（一次快照）
#[tauri::command]
pub async fn get_groups(app_handle: AppHandle) -> Result<Groups, String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .get_groups_snapshot()
        .await
        .map_err(|e| format!("获取节点组失败: {}", e))
}

/// 切换代理节点
#[tauri::command]
pub async fn select_outbound(
    app_handle: AppHandle,
    group: String,
    proxy: String,
) -> Result<(), String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .select_outbound(&group, &proxy)
        .await
        .map_err(|e| format!("切换节点失败: {}", e))
}

/// 触发整组测速（URLTest）
/// sing-box 1.14 的 URLTest 是 server-streaming,unary 触发后不等结果。
/// 测速过程通过 URLTestProgress stream 推送,完成后 SubscribeGroups 也会推一帧新 Groups
/// (由 groups relay 监听并 emit 给前端)。
#[tauri::command]
pub async fn url_test(app_handle: AppHandle, group: String) -> Result<(), String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .url_test(&group)
        .await
        .map_err(|e| format!("触发测速失败: {}", e))
}

/// 关闭所有连接
#[tauri::command]
pub async fn close_all_connections(app_handle: AppHandle) -> Result<(), String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .close_all_connections()
        .await
        .map_err(|e| format!("关闭所有连接失败: {}", e))
}

/// 关闭单个连接
#[tauri::command]
pub async fn close_connection(app_handle: AppHandle, id: String) -> Result<(), String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .close_connection(&id)
        .await
        .map_err(|e| format!("关闭连接失败: {}", e))
}

/// 获取 Clash 模式状态（mode_list + current_mode）
#[tauri::command]
pub async fn get_clash_mode_status(
    app_handle: AppHandle,
) -> Result<crate::app::singbox_api::ClashModeStatus, String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .get_clash_mode_status()
        .await
        .map_err(|e| format!("获取 Clash 模式失败: {}", e))
}

/// 运行时切换 Clash 模式
#[tauri::command]
pub async fn set_clash_mode(app_handle: AppHandle, mode: String) -> Result<(), String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .set_clash_mode(&mode)
        .await
        .map_err(|e| format!("切换 Clash 模式失败: {}", e))
}

/// 设置组展开状态（持久化到内核）
#[tauri::command]
pub async fn set_group_expand(
    app_handle: AppHandle,
    group: String,
    is_expand: bool,
) -> Result<(), String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .set_group_expand(&group, is_expand)
        .await
        .map_err(|e| format!("设置组展开状态失败: {}", e))
}

/// 获取内核启动时间戳（秒）
#[tauri::command]
pub async fn get_started_at(app_handle: AppHandle) -> Result<i64, String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .get_started_at()
        .await
        .map_err(|e| format!("获取启动时间失败: {}", e))
}

// ============ sing-box 1.14+ 新增 gRPC API 桥接 ============

/// 获取所有路由规则（解锁 P2.7 RulesView 恢复）
#[tauri::command]
pub async fn get_rules(app_handle: AppHandle) -> Result<crate::app::singbox_api::RuleList, String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .get_rules()
        .await
        .map_err(|e| format!("获取路由规则失败: {}", e))
}

/// 获取所有运行中的服务（解锁 P2.3 Tailscale / USB/IP 状态显示）
#[tauri::command]
pub async fn get_services(
    app_handle: AppHandle,
) -> Result<crate::app::singbox_api::ServiceList, String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .get_services()
        .await
        .map_err(|e| format!("获取服务列表失败: {}", e))
}

/// 获取所有出站快照（SubscribeOutbounds 首帧）
/// 数据源用于恢复代理提供者 / 全量出站列表 UI 与按延迟排序
#[tauri::command]
pub async fn get_outbounds(
    app_handle: AppHandle,
) -> Result<crate::app::singbox_api::OutboundList, String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .get_outbounds()
        .await
        .map_err(|e| format!("获取出站列表失败: {}", e))
}

/// 获取已弃用字段告警（GetDeprecatedWarnings）
/// 内核启动后调用，提示配置中将在 1.16 被移除的字段（主要是用户手改的自定义配置）
#[tauri::command]
pub async fn get_deprecated_warnings(
    app_handle: AppHandle,
) -> Result<crate::app::singbox_api::DeprecatedWarnings, String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .get_deprecated_warnings()
        .await
        .map_err(|e| format!("获取弃用告警失败: {}", e))
}

/// 探测 sing-box 服务运行状态（SubscribeServiceStatus 首帧）
/// 这是 1.14 替代 `experimental.clash_api` HTTP `/version` 的官方方式，
/// 返回 { status, errorMessage }，status 取值 idle/starting/started/stopping/fatal。
#[tauri::command]
pub async fn probe_service_status(
    app_handle: AppHandle,
) -> Result<crate::app::singbox_api::ServiceStatusSnapshot, String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .probe_service_status()
        .await
        .map_err(|e| format!("探测服务状态失败: {}", e))
}

/// 兼容占位：gRPC API 时代"API token"概念被 `secret` 取代，恒返回空串
#[tauri::command]
pub fn get_api_token() -> String {
    String::new()
}
