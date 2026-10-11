//! 入站写入与 OS 代理控制
//!
//! 负责把代理模式（系统代理 / TUN / 手动）翻译成 sing-box inbound 配置并落盘，
//! 以及直接操作操作系统的代理设置。与 gRPC 无关 —— 这条边界很重要：
//! 「写配置」是启动前的行为，「问运行中的内核」是启动后的行为，
//! 混在一起时很容易在重启路径上写出半新半旧的配置。

use crate::app::constants::{config, messages, network_config, paths};
use crate::app::core::tun_profile::{TunProfile, TunProxyOptions};
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use crate::app::system::config_service;
use crate::entity::config_model;
use crate::utils::config_util::ConfigUtil;
use crate::utils::proxy_util::{disable_system_proxy, enable_system_proxy, DEFAULT_BYPASS_LIST};
use serde_json::Value;
use std::fs;
use tauri::AppHandle;
use tracing::{info, warn};

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
        dns_mode: None,
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

    // sing-box 1.12+ 已废弃顶层 `dns.strategy` 与 per-server `strategy`
    // （settings_patch 会把旧字段删掉）。IP 版本偏好现在只存在两处：
    //   1. dns.servers[].domain_resolver 对象内的 strategy
    //   2. route.default_domain_resolver 对象内的 strategy
    // 这里同步更新已存在的这两类对象；不凭空创建 default_domain_resolver
    // （缺 server 字段可能过不了内核校验——本应用生成的配置都带完整对象）。
    let mut updated = 0;
    if let Some(servers) = config
        .get_mut("dns")
        .and_then(|dns| dns.get_mut("servers"))
        .and_then(|s| s.as_array_mut())
    {
        for server in servers.iter_mut() {
            let Some(server_obj) = server.as_object_mut() else {
                continue;
            };
            // 清理历史版本写入的废弃 per-server strategy
            server_obj.remove("strategy");
            if let Some(dr) = server_obj.get_mut("domain_resolver") {
                if let Some(dr_obj) = dr.as_object_mut() {
                    dr_obj.insert(
                        "strategy".to_string(),
                        Value::String(strategy_value.to_string()),
                    );
                    updated += 1;
                }
            }
        }
    }
    if let Some(dns_obj) = config.get_mut("dns").and_then(|dns| dns.as_object_mut()) {
        dns_obj.remove("strategy");
    }
    if let Some(ddr) = config
        .get_mut("route")
        .and_then(|route| route.get_mut("default_domain_resolver"))
        .and_then(|ddr| ddr.as_object_mut())
    {
        ddr.insert(
            "strategy".to_string(),
            Value::String(strategy_value.to_string()),
        );
        updated += 1;
    }
    if updated == 0 {
        warn!(
            "配置中不存在 domain_resolver 对象，IP 版本偏好仅在下次配置再生时生效（persisted prefer_ipv6={})",
            prefer_ipv6
        );
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
