//! 代理服务模块
//!
//! 负责：
//! 1. 写入 sing-box 配置（inbound、outbound、route），控制代理模式
//! 2. 通过 sing-box 1.14+ 官方 gRPC API（`type: api`）查询 / 控制运行中的内核
//! 3. 自定义规则 CRUD（写配置文件 + 重启内核）

use crate::app::constants::{config, messages, network_config, paths};
use crate::app::core::tun_profile::{TunProfile, TunProxyOptions};
use crate::app::singbox::config_generator::inject_custom_rules;
use crate::app::singbox::common::normalize_default_outbound;
use crate::app::singbox_api::{ApiClientConfig, ApiClientHandle, Groups, Status};
use crate::app::storage::custom_rule::{CustomRule, CustomRuleAction, CustomRuleMatchType, STORAGE_KEY};
use crate::app::storage::enhanced_storage_service::{db_get_app_config, get_enhanced_storage};
use crate::app::system::config_service;
use crate::entity::config_model;
use crate::utils::config_util::ConfigUtil;
use crate::utils::proxy_util::{disable_system_proxy, enable_system_proxy, DEFAULT_BYPASS_LIST};
use chrono::Utc;
use serde_json::{json, Value};
use std::fs;
use std::sync::Arc;
use tauri::AppHandle;
use tracing::{error, info, warn};

// =====================================================================
// Inbound 写入 + OS 代理控制（保留，与 gRPC 无关）
// =====================================================================

#[derive(Debug, Clone)]
pub struct ProxyRuntimeState {
    pub proxy_port: u16,
    pub allow_lan_access: bool,
    pub system_proxy_enabled: bool,
    pub tun_enabled: bool,
    pub system_proxy_bypass: String,
    pub tun_options: TunProxyOptions,
}

impl ProxyRuntimeState {
    pub fn derived_mode(&self) -> String {
        if self.tun_enabled {
            "tun".to_string()
        } else if self.system_proxy_enabled {
            "system".to_string()
        } else {
            "manual".to_string()
        }
    }
}

fn resolve_proxy_listen_address(state: &ProxyRuntimeState) -> &'static str {
    if state.allow_lan_access {
        network_config::DEFAULT_LISTEN_ADDRESS
    } else {
        network_config::DEFAULT_CLASH_API_ADDRESS
    }
}

fn build_inbounds_for_state(state: &ProxyRuntimeState) -> Vec<config_model::Inbound> {
    if state.tun_enabled {
        let mut inbounds =
            TunProfile::from_options(&state.tun_options, None).to_inbounds(state.proxy_port);
        if let Some(mixed) = inbounds.get_mut(0) {
            mixed.listen = Some(resolve_proxy_listen_address(state).to_string());
            mixed.set_system_proxy = None;
        }
        return inbounds;
    }

    vec![config_model::Inbound {
        r#type: config::DEFAULT_INBOUND_TYPE.to_string(),
        tag: config::DEFAULT_INBOUND_TAG.to_string(),
        listen: Some(resolve_proxy_listen_address(state).to_string()),
        interface_name: None,
        listen_port: Some(state.proxy_port),
        address: None,
        auto_route: None,
        strict_route: None,
        stack: None,
        mtu: None,
        route_address: None,
        route_exclude_address: None,
        set_system_proxy: None,
    }]
}

async fn load_allow_lan_access(app_handle: &AppHandle) -> bool {
    db_get_app_config(app_handle.clone())
        .await
        .map(|config| config.allow_lan_access)
        .unwrap_or(false)
}

pub async fn apply_proxy_runtime_state(
    app_handle: &AppHandle,
    state: &ProxyRuntimeState,
) -> Result<(), String> {
    write_inbounds_to_config(app_handle, state).await?;
    apply_os_proxy(state);
    Ok(())
}

pub async fn write_inbounds_to_config(
    app_handle: &AppHandle,
    state: &ProxyRuntimeState,
) -> Result<(), String> {
    config_service::ensure_singbox_config(app_handle)
        .await
        .map_err(|e| format!("准备配置失败: {}", e))?;

    let app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;

    let config_path = if let Some(path_str) = app_config.active_config_path {
        std::path::PathBuf::from(path_str)
    } else {
        paths::get_config_dir().join("config.json")
    };

    let config_path_str = config_path
        .to_str()
        .ok_or_else(|| "配置文件路径包含无效字符".to_string())?;

    let mut json_util = ConfigUtil::new(config_path_str)
        .map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))?;

    let inbounds = build_inbounds_for_state(state);
    json_util.update_key(
        vec!["inbounds"],
        serde_json::to_value(inbounds).map_err(|e| format!("序列化配置失败: {}", e))?,
    );
    json_util
        .save_to_file()
        .map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))?;

    Ok(())
}

pub fn apply_os_proxy(state: &ProxyRuntimeState) {
    if state.system_proxy_enabled {
        let bypass = state.system_proxy_bypass.trim();
        let normalized_bypass = if bypass.is_empty() {
            DEFAULT_BYPASS_LIST.to_string()
        } else {
            bypass.to_string()
        };
        match enable_system_proxy(
            network_config::DEFAULT_CLASH_API_ADDRESS,
            state.proxy_port,
            Some(normalized_bypass.as_str()),
        ) {
            Ok(()) => info!(
                "系统代理已启用，端口 {}，绕过列表: {}",
                state.proxy_port, normalized_bypass
            ),
            Err(e) => warn!("设置系统代理失败: {}", e),
        }
    } else if let Err(err) = disable_system_proxy() {
        warn!("关闭系统代理失败: {}", err);
    }
}

#[tauri::command]
pub async fn set_system_proxy(
    app_handle: AppHandle,
    port: u16,
    system_proxy_bypass: Option<String>,
) -> Result<(), String> {
    let allow_lan_access = load_allow_lan_access(&app_handle).await;
    let runtime_state = ProxyRuntimeState {
        proxy_port: port,
        allow_lan_access,
        system_proxy_enabled: true,
        tun_enabled: false,
        system_proxy_bypass: system_proxy_bypass.unwrap_or_else(|| DEFAULT_BYPASS_LIST.to_string()),
        tun_options: TunProxyOptions::default(),
    };
    apply_proxy_runtime_state(&app_handle, &runtime_state).await
}

#[tauri::command]
pub async fn set_manual_proxy(app_handle: AppHandle, port: u16) -> Result<(), String> {
    let allow_lan_access = load_allow_lan_access(&app_handle).await;
    let runtime_state = ProxyRuntimeState {
        proxy_port: port,
        allow_lan_access,
        system_proxy_enabled: false,
        tun_enabled: false,
        system_proxy_bypass: DEFAULT_BYPASS_LIST.to_string(),
        tun_options: TunProxyOptions::default(),
    };
    apply_proxy_runtime_state(&app_handle, &runtime_state).await
}

#[tauri::command]
pub async fn set_tun_proxy(
    app_handle: AppHandle,
    port: u16,
    tun_options: Option<TunProxyOptions>,
) -> Result<(), String> {
    let allow_lan_access = load_allow_lan_access(&app_handle).await;
    let runtime_state = ProxyRuntimeState {
        proxy_port: port,
        allow_lan_access,
        system_proxy_enabled: false,
        tun_enabled: true,
        system_proxy_bypass: DEFAULT_BYPASS_LIST.to_string(),
        tun_options: tun_options.unwrap_or_default(),
    };
    apply_proxy_runtime_state(&app_handle, &runtime_state).await
}

pub async fn update_dns_strategy(app_handle: &AppHandle, prefer_ipv6: bool) -> Result<(), String> {
    let app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;
    let config_path = if let Some(path_str) = app_config.active_config_path {
        std::path::PathBuf::from(path_str)
    } else {
        paths::get_config_dir().join("config.json")
    };

    let content =
        fs::read_to_string(&config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;
    let mut config: Value =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    let strategy_value = if prefer_ipv6 {
        "prefer_ipv6"
    } else {
        "ipv4_only"
    };

    let dns_object = if let Some(obj) = config
        .as_object_mut()
        .and_then(|obj| obj.get_mut("dns"))
        .and_then(|dns| dns.as_object_mut())
    {
        obj
    } else {
        let dns_value = json!({
            "servers": [],
            "strategy": strategy_value
        });
        config
            .as_object_mut()
            .ok_or_else(|| "配置文件结构异常，无法写入DNS配置".to_string())?
            .insert("dns".to_string(), dns_value);
        config
            .as_object_mut()
            .and_then(|obj| obj.get_mut("dns"))
            .and_then(|dns| dns.as_object_mut())
            .ok_or_else(|| "创建DNS配置失败".to_string())?
    };

    dns_object.insert(
        "strategy".to_string(),
        Value::String(strategy_value.to_string()),
    );

    if let Some(servers) = dns_object.get_mut("servers").and_then(|s| s.as_array_mut()) {
        for server in servers.iter_mut() {
            if let Some(server_obj) = server.as_object_mut() {
                if server_obj.get("address").is_some() {
                    server_obj.insert(
                        "strategy".to_string(),
                        Value::String(strategy_value.to_string()),
                    );
                }
            }
        }
    }

    let serialized =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(&config_path, serialized).map_err(|e| format!("保存配置文件失败: {}", e))?;
    Ok(())
}

#[tauri::command]
pub async fn toggle_ip_version(app_handle: AppHandle, prefer_ipv6: bool) -> Result<(), String> {
    info!(
        "开始切换IP版本模式: {}",
        if prefer_ipv6 { "IPv6优先" } else { "仅IPv4" }
    );
    update_dns_strategy(&app_handle, prefer_ipv6).await?;
    info!(
        "✅ IP版本模式已成功切换为: {}",
        if prefer_ipv6 { "IPv6优先" } else { "仅IPv4" }
    );
    Ok(())
}

/// 兼容占位：gRPC API 时代"API token"概念被 `secret` 取代，恒返回空串
#[tauri::command]
pub fn get_api_token() -> String {
    String::new()
}

// =====================================================================
// gRPC API 交互（sing-box 1.14+ 官方 type: api）
// =====================================================================

/// 创建默认 gRPC API 客户端句柄（指向本机 + AppConfig.api_port）
fn make_handle(app_handle: &AppHandle) -> Result<ApiClientHandle, String> {
    let config = get_grpc_config(app_handle)?;
    Ok(ApiClientHandle::new(config))
}

fn get_grpc_config(app_handle: &AppHandle) -> Result<ApiClientConfig, String> {
    let app_config = tauri::async_runtime::block_on(async {
        db_get_app_config(app_handle.clone()).await
    })
    .map_err(|e| format!("获取应用配置失败: {}", e))?;
    Ok(ApiClientConfig::localhost(app_config.api_port))
}

/// 异步获取 gRPC 客户端句柄
async fn make_handle_async(app_handle: AppHandle) -> Result<(ApiClientHandle, ApiClientConfig), String> {
    let app_config = db_get_app_config(app_handle)
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;
    let config = ApiClientConfig::localhost(app_config.api_port);
    Ok((ApiClientHandle::new(config.clone()), config))
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
pub async fn get_rules(
    app_handle: AppHandle,
) -> Result<crate::app::singbox_api::RuleList, String> {
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

/// 网络质量测试（unary：TCP RTT + 上下行带宽 + 延迟）
#[tauri::command]
pub async fn network_quality_test(
    app_handle: AppHandle,
) -> Result<crate::app::singbox_api::NetworkQualityResult, String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    handle
        .network_quality_test()
        .await
        .map_err(|e| format!("网络质量测试失败: {}", e))
}

/// 获取内核完整状态(sing-box 1.14+ 走 gRPC unary 探测,不用 streaming,
/// 也不用 1.13 时代 `experimental.clash_api` 的 HTTP /version 端点)。
///
/// 核心逻辑:**纯 TCP 端口探测 + gRPC GetVersion**,完全绕开 ProcessManager 内部状态
/// (用户实测:sing-box 实际在跑 + streaming 数据正常,但 is_kernel_running 因 ProcessManager
/// 状态问题返回 false,导致前端一直显示"已停止")。
///
/// - gRPC 端口(默认 12081)能连 = sing-box 在跑 = process_running=true
/// - gRPC GetVersion 成功 = kernel_state=running
/// - gRPC 端口都不通 = 真的没启动 = process_running=false
#[tauri::command]
pub async fn kernel_get_status_enhanced_v2(
    app_handle: AppHandle,
) -> Result<serde_json::Value, String> {
    use crate::app::core::kernel_service::state::KERNEL_STATE;
    use tokio::net::TcpStream;
    use std::time::Duration;

    // 0) binary 版本戳 —— 用户可凭此确认 Rust 后端是否真的更新
    tracing::info!("[STATUS_V2_BUILD] 2026-09-01-v4-binary-version-stamp");

    // 1) 从 db 读 api_port / proxy_port
    //    端口配置工作流:UI 改端口 → invoke `update_singbox_ports` 写 db + `kernel_restart_*` 触发重启,
    //    内核用 db 新端口启动。所以 db 永远是权威源,不需要前端注入端口参数。
    let (api_port, proxy_port) = match db_get_app_config(app_handle.clone()).await {
        Ok(c) => (c.api_port, c.proxy_port),
        Err(e) => {
            tracing::warn!("status_v2: db_get_app_config failed: {}", e);
            return Ok(serde_json::json!({
                "process_running": false, "api_ready": false, "websocket_ready": false,
                "kernel_state": "stopped",
                "error": format!("获取 app config 失败: {}", e),
            }));
        }
    };

    // 2) **纯 TCP 端口探测**(不依赖 ProcessManager)—— 端口能连 = 进程在跑
    let api_port_open = tokio::time::timeout(
        Duration::from_millis(500),
        TcpStream::connect(("127.0.0.1", api_port)),
    )
    .await
    .map(|r| r.is_ok())
    .unwrap_or(false);

    let proxy_port_open = tokio::time::timeout(
        Duration::from_millis(500),
        TcpStream::connect(("127.0.0.1", proxy_port)),
    )
    .await
    .map(|r| r.is_ok())
    .unwrap_or(false);

    // TCP 端口能连 → process_running=true
    // 这绕开 ProcessManager is_running() 可能的 false negative
    let process_running = api_port_open || proxy_port_open;
    tracing::info!(
        "[STATUS_V2_BUILD] api_port={} open={} | proxy_port={} open={} | process_running={}",
        api_port, api_port_open, proxy_port, proxy_port_open, process_running
    );

    // 3) gRPC GetVersion 探测(决定 api_ready 和 kernel_state)
    let (api_ready, kernel_state_str, error_msg, version) = if process_running {
        let handle = match make_handle_async(app_handle.clone()).await {
            Err(e) => {
                tracing::warn!("status_v2: make_handle_async failed: {}", e);
                return Ok(serde_json::json!({
                    "process_running": true, // TCP 端口通,进程在跑
                    "api_ready": false,
                    "websocket_ready": false,
                    "kernel_state": "starting",
                    "error": format!("创建 gRPC 句柄失败: {}", e),
                }));
            }
            Ok(v) => v.0,
        };
        let probe = tokio::time::timeout(Duration::from_secs(2), handle.get_version()).await;
        match probe {
            Ok(Ok((v, _api))) => {
                tracing::info!("[STATUS_V2_BUILD] gRPC GetVersion OK -> {}", v);
                (true, "running".to_string(), None, Some(v))
            }
            Ok(Err(e)) => {
                tracing::warn!("[STATUS_V2_BUILD] gRPC GetVersion Err: {} (端口通但 gRPC 失败 — sing-box 启动早期?)", e);
                (false, "starting".to_string(), Some(format!("gRPC 探测失败: {}", e)), None)
            }
            Err(_) => {
                tracing::warn!("[STATUS_V2_BUILD] gRPC GetVersion timeout (2s) — 端口通但 gRPC 不响应");
                (false, "starting".to_string(), Some("gRPC 探测超时(2s)".to_string()), None)
            }
        }
    } else {
        tracing::warn!("[STATUS_V2_BUILD] TCP 端口都不通 → sing-box 真的没启动");
        (false, "stopped".to_string(), None, None)
    };

    // 4) 同步 readiness
    let mut readiness = KERNEL_STATE.get_readiness();
    readiness.process_alive = process_running;
    readiness.api_ready = api_ready;

    Ok(serde_json::json!({
        "process_running": process_running,
        "api_ready": api_ready,
        "websocket_ready": api_ready,
        "version": version,
        "error": error_msg,
        "kernel_state": kernel_state_str,
        "readiness": {
            "process_alive": readiness.process_alive,
            "api_ready": readiness.api_ready,
            "relay_ready": readiness.relay_ready,
        }
    }))
}

/// 单次 Status 快照（不订阅）
#[tauri::command]
pub async fn kernel_get_snapshot_v2(app_handle: AppHandle) -> Result<Status, String> {
    let (handle, _) = make_handle_async(app_handle).await?;
    let mut sub = handle
        .subscribe_status(1_000_000_000)
        .await
        .map_err(|e| format!("订阅 Status 失败: {}", e))?;
    let snapshot = sub
        .next()
        .await
        .map_err(|e| format!("读取 Status 失败: {}", e))?;
    sub.close().await;
    snapshot.ok_or_else(|| "Status 快照为空".to_string())
}

// =====================================================================
// 自定义规则 CRUD（issue #62）—— 写文件路径，不依赖 gRPC
// =====================================================================

#[tauri::command]
pub async fn list_custom_rules(app_handle: AppHandle) -> Result<Vec<CustomRule>, String> {
    let storage = get_enhanced_storage(&app_handle)
        .await
        .map_err(|e| format!("初始化存储失败: {}", e))?;
    let rules: Option<Vec<CustomRule>> = storage
        .load_generic_config(STORAGE_KEY)
        .await
        .map_err(|e| format!("读取自定义规则失败: {}", e))?;
    let mut rules = rules.unwrap_or_default();
    rules.sort_by_key(|r| r.created_at);
    Ok(rules)
}

#[tauri::command]
pub async fn add_custom_rule(
    app_handle: AppHandle,
    match_type: CustomRuleMatchType,
    payload: String,
    action: CustomRuleAction,
    note: Option<String>,
) -> Result<CustomRule, String> {
    if payload.trim().is_empty() {
        return Err("匹配内容不能为空".to_string());
    }
    let storage = get_enhanced_storage(&app_handle)
        .await
        .map_err(|e| format!("初始化存储失败: {}", e))?;
    let mut rules: Vec<CustomRule> = storage
        .load_generic_config(STORAGE_KEY)
        .await
        .map_err(|e| format!("读取自定义规则失败: {}", e))?
        .unwrap_or_default();

    let now = Utc::now();
    let rule = CustomRule {
        id: uuid_v4(),
        enabled: true,
        match_type,
        payload,
        action,
        outbound: None,
        note,
        created_at: now,
        updated_at: now,
    };
    rules.push(rule.clone());
    storage
        .save_generic_config(STORAGE_KEY, &rules)
        .await
        .map_err(|e| format!("保存自定义规则失败: {}", e))?;
    inject_into_active_config(&app_handle).await;
    Ok(rule)
}

#[tauri::command]
pub async fn update_custom_rule(
    app_handle: AppHandle,
    id: String,
    match_type: CustomRuleMatchType,
    payload: String,
    action: CustomRuleAction,
    note: Option<String>,
) -> Result<(), String> {
    if payload.trim().is_empty() {
        return Err("匹配内容不能为空".to_string());
    }
    let storage = get_enhanced_storage(&app_handle)
        .await
        .map_err(|e| format!("初始化存储失败: {}", e))?;
    let mut rules: Vec<CustomRule> = storage
        .load_generic_config(STORAGE_KEY)
        .await
        .map_err(|e| format!("读取自定义规则失败: {}", e))?
        .unwrap_or_default();

    let target = rules
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or_else(|| "未找到对应的规则".to_string())?;
    target.match_type = match_type;
    target.payload = payload;
    target.action = action;
    target.note = note;
    target.updated_at = Utc::now();

    storage
        .save_generic_config(STORAGE_KEY, &rules)
        .await
        .map_err(|e| format!("保存自定义规则失败: {}", e))?;
    inject_into_active_config(&app_handle).await;
    Ok(())
}

#[tauri::command]
pub async fn delete_custom_rule(app_handle: AppHandle, id: String) -> Result<(), String> {
    let storage = get_enhanced_storage(&app_handle)
        .await
        .map_err(|e| format!("初始化存储失败: {}", e))?;
    let mut rules: Vec<CustomRule> = storage
        .load_generic_config(STORAGE_KEY)
        .await
        .map_err(|e| format!("读取自定义规则失败: {}", e))?
        .unwrap_or_default();
    let before = rules.len();
    rules.retain(|r| r.id != id);
    if rules.len() == before {
        return Err("未找到对应的规则".to_string());
    }
    storage
        .save_generic_config(STORAGE_KEY, &rules)
        .await
        .map_err(|e| format!("保存自定义规则失败: {}", e))?;
    inject_into_active_config(&app_handle).await;
    Ok(())
}

#[tauri::command]
pub async fn toggle_custom_rule(app_handle: AppHandle, id: String) -> Result<(), String> {
    let storage = get_enhanced_storage(&app_handle)
        .await
        .map_err(|e| format!("初始化存储失败: {}", e))?;
    let mut rules: Vec<CustomRule> = storage
        .load_generic_config(STORAGE_KEY)
        .await
        .map_err(|e| format!("读取自定义规则失败: {}", e))?
        .unwrap_or_default();
    let target = rules
        .iter_mut()
        .find(|r| r.id == id)
        .ok_or_else(|| "未找到对应的规则".to_string())?;
    target.enabled = !target.enabled;
    target.updated_at = Utc::now();
    storage
        .save_generic_config(STORAGE_KEY, &rules)
        .await
        .map_err(|e| format!("保存自定义规则失败: {}", e))?;
    inject_into_active_config(&app_handle).await;
    Ok(())
}

async fn inject_into_active_config(app_handle: &AppHandle) {
    if let Err(e) = inject_into_active_config_inner(app_handle).await {
        warn!("自定义规则注入活动配置失败（不影响持久化）: {}", e);
    }
}

async fn inject_into_active_config_inner(app_handle: &AppHandle) -> Result<(), String> {
    let storage = get_enhanced_storage(app_handle)
        .await
        .map_err(|e| format!("初始化存储失败: {}", e))?;
    let app_config = db_get_app_config(app_handle.clone())
        .await
        .map_err(|e| format!("读取应用配置失败: {}", e))?;

    if is_active_config_use_original(&storage, &app_config).await {
        info!("当前活动订阅为原始配置，跳过自定义规则注入");
        return Ok(());
    }

    let config_path = match &app_config.active_config_path {
        Some(p) => std::path::PathBuf::from(p),
        None => return Ok(()),
    };
    if !config_path.exists() {
        return Ok(());
    }

    let rules: Vec<CustomRule> = storage
        .load_generic_config(STORAGE_KEY)
        .await
        .map_err(|e| format!("读取自定义规则失败: {}", e))?
        .unwrap_or_default();

    let default_outbound = normalize_default_outbound(&app_config);
    inject_custom_rules_into_file(&config_path, &rules, default_outbound)?;
    let enabled_count = rules.iter().filter(|r| r.enabled).count();
    info!(
        "已把 {} 条自定义规则注入活动配置: {:?}",
        enabled_count, config_path
    );
    Ok(())
}

fn inject_custom_rules_into_file(
    config_path: &std::path::Path,
    rules: &[CustomRule],
    default_outbound: &str,
) -> Result<(), String> {
    let content = fs::read_to_string(config_path)
        .map_err(|e| format!("读取配置失败: {}", e))?;
    let base_path = config_path.with_extension("base");
    if !base_path.exists() {
        fs::write(&base_path, &content)
            .map_err(|e| format!("写入 .base 备份失败: {}", e))?;
    }
    let base_content = fs::read_to_string(&base_path)
        .map_err(|e| format!("读取 .base 备份失败: {}", e))?;
    let mut config: Value = serde_json::from_str(&base_content)
        .map_err(|e| format!("解析 .base 备份失败: {}", e))?;
    inject_custom_rules(&mut config, rules, default_outbound);
    let serialized = serde_json::to_string_pretty(&config)
        .map_err(|e| format!("序列化配置失败: {}", e))?;
    fs::write(config_path, serialized)
        .map_err(|e| format!("写入配置文件失败: {}", e))?;
    Ok(())
}

async fn is_active_config_use_original(
    storage: &Arc<crate::app::storage::enhanced_storage_service::EnhancedStorageService>,
    app_config: &crate::app::storage::state_model::AppConfig,
) -> bool {
    let active_path = match &app_config.active_config_path {
        Some(p) => p,
        None => return false,
    };
    let use_original: Option<bool> = storage
        .load_generic_config("use_original_config")
        .await
        .ok()
        .flatten();
    use_original.unwrap_or(false)
        && storage
            .load_generic_config::<String>(&format!("original_config_path:{}", active_path))
            .await
            .ok()
            .flatten()
            .is_some()
}

fn uuid_v4() -> String {
    uuid_v4_impl()
}

fn uuid_v4_impl() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_nanos())
        .unwrap_or(0);
    format!("{:x}", nanos)
}

// 注：proxy_service.tests.rs（旧版单元测试引用了已删除的 Clash API 类型）已废弃。
// 重写为 gRPC API 后，待阶段 5 补一份基于 singbox_api mock 的单测。