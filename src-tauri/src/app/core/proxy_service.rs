use crate::app::constants::{config, messages, network_config, paths};
use crate::app::core::tun_profile::{TunProfile, TunProxyOptions};
use crate::app::system::config_service;
use crate::entity::config_model;
use crate::utils::config_util::ConfigUtil;
use crate::utils::http_client;
use crate::utils::proxy_util::{disable_system_proxy, enable_system_proxy, DEFAULT_BYPASS_LIST};
use futures_util::stream::{self, StreamExt};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use std::fs;
use std::time::Duration;
use tracing::{error, info, warn};
use url::Url;

#[derive(Debug, Clone)]
pub struct ProxyRuntimeState {
    pub proxy_port: u16,
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

fn build_inbounds_for_state(state: &ProxyRuntimeState) -> Vec<config_model::Inbound> {
    if state.tun_enabled {
        let mut inbounds =
            TunProfile::from_options(&state.tun_options).to_inbounds(state.proxy_port);
        if let Some(mixed) = inbounds.get_mut(0) {
            mixed.listen = Some(network_config::DEFAULT_CLASH_API_ADDRESS.to_string());
            mixed.set_system_proxy = Some(state.system_proxy_enabled);
        }
        return inbounds;
    }

    vec![config_model::Inbound {
        r#type: config::DEFAULT_INBOUND_TYPE.to_string(),
        tag: config::DEFAULT_INBOUND_TAG.to_string(),
        listen: Some(network_config::DEFAULT_CLASH_API_ADDRESS.to_string()),
        interface_name: None,
        listen_port: Some(state.proxy_port),
        address: None,
        auto_route: None,
        strict_route: None,
        stack: None,
        sniff: Some(true),
        sniff_override_destination: Some(true),
        mtu: None,
        route_address: None,
        route_exclude_address: None,
        set_system_proxy: Some(state.system_proxy_enabled),
    }]
}

use crate::app::storage::enhanced_storage_service::db_get_app_config;
use tauri::AppHandle;

pub async fn apply_proxy_runtime_state(
    app_handle: &AppHandle,
    state: &ProxyRuntimeState,
) -> Result<(), String> {
    config_service::ensure_singbox_config(app_handle)
        .await
        .map_err(|e| format!("准备配置失败: {}", e))?;

    // 从数据库获取配置路径
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

    if state.system_proxy_enabled {
        let bypass = state.system_proxy_bypass.trim();
        let normalized_bypass = if bypass.is_empty() {
            DEFAULT_BYPASS_LIST.to_string()
        } else {
            bypass.to_string()
        };
        enable_system_proxy(
            network_config::DEFAULT_CLASH_API_ADDRESS,
            state.proxy_port,
            Some(normalized_bypass.as_str()),
        )
        .map_err(|e| format!("设置系统代理失败: {}", e))?;
        info!(
            "系统代理已启用，端口 {}，绕过列表: {}",
            state.proxy_port, normalized_bypass
        );
    } else if let Err(err) = disable_system_proxy() {
        warn!("关闭系统代理失败: {}", err);
    }

    Ok(())
}

// 修改代理模式为系统代理
#[tauri::command]
pub async fn set_system_proxy(
    app_handle: AppHandle,
    port: u16,
    system_proxy_bypass: Option<String>,
) -> Result<(), String> {
    let runtime_state = ProxyRuntimeState {
        proxy_port: port,
        system_proxy_enabled: true,
        tun_enabled: false,
        system_proxy_bypass: system_proxy_bypass.unwrap_or_else(|| DEFAULT_BYPASS_LIST.to_string()),
        tun_options: TunProxyOptions::default(),
    };
    apply_proxy_runtime_state(&app_handle, &runtime_state).await
}

// 设置手动代理模式（不自动设置系统代理）
#[tauri::command]
pub async fn set_manual_proxy(app_handle: AppHandle, port: u16) -> Result<(), String> {
    let runtime_state = ProxyRuntimeState {
        proxy_port: port,
        system_proxy_enabled: false,
        tun_enabled: false,
        system_proxy_bypass: DEFAULT_BYPASS_LIST.to_string(),
        tun_options: TunProxyOptions::default(),
    };
    apply_proxy_runtime_state(&app_handle, &runtime_state).await
}

// 修改TUN 模式为代理模式
#[tauri::command]
pub async fn set_tun_proxy(
    app_handle: AppHandle,
    port: u16,
    tun_options: Option<TunProxyOptions>,
) -> Result<(), String> {
    let runtime_state = ProxyRuntimeState {
        proxy_port: port,
        system_proxy_enabled: false,
        tun_enabled: true,
        system_proxy_bypass: DEFAULT_BYPASS_LIST.to_string(),
        tun_options: tun_options.unwrap_or_default(),
    };
    apply_proxy_runtime_state(&app_handle, &runtime_state).await
}

pub async fn update_dns_strategy(app_handle: &AppHandle, prefer_ipv6: bool) -> Result<(), String> {
    // 从数据库获取配置路径
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

    // 兼容不同用户的配置：如果没有 dns 区块则补充一个基础结构
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

    // 同步更新所有 DNS 服务器的 strategy，确保优先级实时生效
    if let Some(servers) = dns_object.get_mut("servers").and_then(|s| s.as_array_mut()) {
        for server in servers.iter_mut() {
            if let Some(server_obj) = server.as_object_mut() {
                // 只在存在 address 的条目上更新，避免污染特殊类型（如 rcode）
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

// 切换 IPV6版本模式
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


// 获取API令牌
#[tauri::command]
pub fn get_api_token() -> String {
    // 目前返回空字符串
    "".to_string()
}

// 获取代理节点列表
#[tauri::command]
pub async fn get_proxies(port: u16) -> Result<Value, String> {
    let url = format!("http://127.0.0.1:{}/proxies", port);

    match http_client::get_json::<Value>(&url).await {
        Ok(data) => Ok(data),
        Err(e) => {
            error!("获取代理列表失败: {}", e);
            Err(format!("获取代理列表失败: {}", e))
        }
    }
}

// 切换代理节点
#[tauri::command]
pub async fn change_proxy(group: String, proxy: String, port: u16) -> Result<(), String> {
    let url = format!("http://127.0.0.1:{}/proxies/{}", port, group);
    let data = json!({
        "name": proxy
    });

    let client = http_client::get_client();
    match client
        .put(&url)
        .json(&data)
        .timeout(Duration::from_secs(5))
        .send()
        .await
    {
        Ok(response) => {
            if response.status().is_success() {
                info!("代理节点已切换: {} -> {}", group, proxy);
                Ok(())
            } else {
                let error_msg = format!("切换代理节点失败，HTTP状态码: {}", response.status());
                error!("{}", error_msg);
                Err(error_msg)
            }
        }
        Err(e) => {
            let error_msg = format!("切换代理节点请求失败: {}", e);
            error!("{}", error_msg);
            Err(error_msg)
        }
    }
}

async fn resolve_group_nodes(port: u16, group: &str) -> Result<Vec<String>, String> {
    let data = get_proxies(port).await?;

    let group_value = data
        .get("proxies")
        .and_then(|v| v.get(group))
        .ok_or_else(|| format!("未找到代理组: {}", group))?;

    let nodes = group_value
        .get("all")
        .and_then(|v| v.as_array())
        .ok_or_else(|| format!("代理组 {} 不包含可测试节点列表", group))?
        .iter()
        .filter_map(|v| v.as_str().map(|s| s.to_string()))
        .collect::<Vec<_>>();

    if nodes.is_empty() {
        return Err(format!("代理组 {} 节点为空", group));
    }

    Ok(nodes)
}

/// 测试代理组延迟（兼容旧接口名）。
///
/// 说明：早期实现使用 Clash API 的 `/group/{group}/delay`，该接口在部分内核/配置下可能返回不完整，
/// 导致前端出现“有些节点没有数据也没有错误”的体验。
/// 这里改为：先从 `/proxies` 获取组内节点列表，再逐个调用 `/proxies/{name}/delay` 并汇总结果。
#[tauri::command]
pub async fn test_group_delay(
    app_handle: AppHandle,
    group: String,
    server: Option<String>,
    port: u16,
    options: Option<DelayTestOptions>,
) -> Result<Vec<ProxyDelayTestResult>, String> {
    let proxies = resolve_group_nodes(port, &group).await?;

    // 兼容旧参数名 server：作为 URL 覆盖来源
    let mut merged_options = options.unwrap_or(DelayTestOptions {
        timeout_ms: None,
        url: None,
        concurrency: None,
        samples: None,
    });
    if merged_options.url.is_none() {
        merged_options.url = server;
    }

    test_nodes_delay(app_handle, proxies, Some(merged_options), port).await
}

// 获取规则信息
#[tauri::command]
pub async fn get_rules(port: u16) -> Result<Value, String> {
    let url = format!("http://127.0.0.1:{}/rules", port);

    match http_client::get_json::<Value>(&url).await {
        Ok(data) => Ok(data),
        Err(e) => {
            error!("获取规则信息失败: {}", e);
            Err(format!("获取规则信息失败: {}", e))
        }
    }
}

// -----------------------------
// 延迟测试（测速）
// -----------------------------

/// sing-box 开启 `experimental.clash_api` 后，会提供 Clash RESTful API 兼容接口。
/// 其中 `/proxies/{name}/delay?timeout=xxxx&url=...` 用于对指定节点发起 URLTest 并返回延迟（单位 ms）。
///
/// 这里做了两层改进：
/// 1) 默认使用数据库中的 `singbox_urltest_url`（Setting 页面可配置），避免使用不带 204 的 URL 导致误差/超时。
/// 2) 统一实现“多节点测速”，供单测/组测/批测复用，并返回每个节点的失败原因，避免 UI 出现“无数据也无错误”。
const DEFAULT_DELAY_TEST_URL: &str = "https://connectivitycheck.gstatic.com/generate_204";
const DEFAULT_DELAY_TEST_TIMEOUT_MS: u64 = 8000;
const DEFAULT_DELAY_TEST_CONCURRENCY: usize = 6;
const DEFAULT_DELAY_TEST_SAMPLES: u8 = 2;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DelayTestOptions {
    pub timeout_ms: Option<u64>,
    pub url: Option<String>,
    pub concurrency: Option<usize>,
    /// 每个节点的采样次数（取中位数以降低抖动）
    pub samples: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ProxyDelayTestResult {
    pub proxy: String,
    /// 延迟（ms）。当 `ok=false` 时返回 0。
    pub delay: u64,
    pub ok: bool,
    /// 失败原因（用于前端展示）。
    pub error: Option<String>,
    /// 实际成功采样次数（可能小于 options.samples）
    pub success_samples: u8,
}

fn normalize_test_url(candidate: &str) -> String {
    // 允许用户输入 http(s) URL；其它情况直接回退默认值，避免构造出无效的 query 导致测速失真。
    if let Ok(parsed) = Url::parse(candidate) {
        if parsed.scheme() == "http" || parsed.scheme() == "https" {
            return candidate.to_string();
        }
    }
    DEFAULT_DELAY_TEST_URL.to_string()
}

async fn resolve_delay_test_url(app_handle: &AppHandle, override_url: Option<String>) -> String {
    if let Some(url) = override_url {
        return normalize_test_url(&url);
    }

    match db_get_app_config(app_handle.clone()).await {
        Ok(cfg) => normalize_test_url(&cfg.singbox_urltest_url),
        Err(_) => DEFAULT_DELAY_TEST_URL.to_string(),
    }
}

fn build_clash_delay_url(port: u16, proxy: &str, timeout_ms: u64, test_url: &str) -> Result<Url, String> {
    let mut url = Url::parse(&format!("http://127.0.0.1:{}/", port))
        .map_err(|e| format!("构造 Clash API 地址失败: {}", e))?;

    // 使用 path_segments 自动处理需要转义的字符，避免节点名包含空格/emoji 导致请求失败。
    url.path_segments_mut()
        .map_err(|_| "无法修改 URL path_segments".to_string())?
        .extend(["proxies", proxy, "delay"]);

    url.query_pairs_mut()
        .append_pair("timeout", &timeout_ms.to_string())
        .append_pair("url", test_url);

    Ok(url)
}

async fn fetch_single_delay(port: u16, proxy: &str, timeout_ms: u64, test_url: &str) -> Result<u64, String> {
    let url = build_clash_delay_url(port, proxy, timeout_ms, test_url)?;

    // 使用专用短超时客户端，外层请求超时要略大于 Clash API 的 timeout，避免“核心还在测，HTTP 已被我们断开”。
    let request_timeout = Duration::from_millis(timeout_ms).saturating_add(Duration::from_secs(3));
    let client = http_client::get_proxy_client();

    let response = client
        .get(url)
        .timeout(request_timeout)
        .send()
        .await
        .map_err(|e| format!("请求失败: {}", e))?;

    if !response.status().is_success() {
        return Err(format!("HTTP {}", response.status().as_u16()));
    }

    let data = response
        .json::<Value>()
        .await
        .map_err(|e| format!("解析响应失败: {}", e))?;

    let delay = data.get("delay").and_then(|d| d.as_u64()).unwrap_or(0);
    if delay == 0 {
        return Err("delay=0".to_string());
    }
    Ok(delay)
}

fn median_u64(mut values: Vec<u64>) -> Option<u64> {
    if values.is_empty() {
        return None;
    }
    values.sort_unstable();
    Some(values[values.len() / 2])
}

async fn measure_proxy_delay(
    port: u16,
    proxy: String,
    timeout_ms: u64,
    test_url: &str,
    samples: u8,
) -> ProxyDelayTestResult {
    let mut ok_values: Vec<u64> = Vec::new();
    let mut last_error: Option<String> = None;

    let samples = samples.max(1);
    for _ in 0..samples {
        match fetch_single_delay(port, &proxy, timeout_ms, test_url).await {
            Ok(delay) => ok_values.push(delay),
            Err(err) => last_error = Some(err),
        }
        // 轻微间隔，避免极端情况下对同一节点瞬时并发过高。
        tokio::time::sleep(Duration::from_millis(80)).await;
    }

    if let Some(delay) = median_u64(ok_values.clone()) {
        ProxyDelayTestResult {
            proxy,
            delay,
            ok: true,
            error: None,
            success_samples: ok_values.len() as u8,
        }
    } else {
        ProxyDelayTestResult {
            proxy,
            delay: 0,
            ok: false,
            error: last_error.or_else(|| Some("无可用结果".to_string())),
            success_samples: 0,
        }
    }
}

/// 测试多个节点延迟（批量/组测速统一入口）。
#[tauri::command]
pub async fn test_nodes_delay(
    app_handle: AppHandle,
    proxies: Vec<String>,
    options: Option<DelayTestOptions>,
    port: u16,
) -> Result<Vec<ProxyDelayTestResult>, String> {
    let options = options.unwrap_or(DelayTestOptions {
        timeout_ms: None,
        url: None,
        concurrency: None,
        samples: None,
    });

    let timeout_ms = options.timeout_ms.unwrap_or(DEFAULT_DELAY_TEST_TIMEOUT_MS);
    let samples = options.samples.unwrap_or(DEFAULT_DELAY_TEST_SAMPLES);
    let concurrency = options.concurrency.unwrap_or(DEFAULT_DELAY_TEST_CONCURRENCY).max(1);
    let test_url = resolve_delay_test_url(&app_handle, options.url).await;

    // 去重，避免重复节点浪费测试资源；保留原顺序。
    let mut seen = std::collections::HashSet::new();
    let proxies: Vec<String> = proxies
        .into_iter()
        .filter(|p| seen.insert(p.clone()))
        .collect();

    let results = stream::iter(proxies.into_iter().map(|proxy| {
        let test_url = test_url.clone();
        async move { measure_proxy_delay(port, proxy, timeout_ms, &test_url, samples).await }
    }))
    .buffer_unordered(concurrency)
    .collect::<Vec<_>>()
    .await;

    Ok(results)
}

// 测试单个节点延迟（兼容旧接口名）
#[tauri::command]
pub async fn test_node_delay(
    app_handle: AppHandle,
    proxy: String,
    server: Option<String>,
    port: u16,
) -> Result<ProxyDelayTestResult, String> {
    let test_url = resolve_delay_test_url(&app_handle, server).await;
    Ok(measure_proxy_delay(
        port,
        proxy,
        DEFAULT_DELAY_TEST_TIMEOUT_MS,
        &test_url,
        DEFAULT_DELAY_TEST_SAMPLES.max(3),
    )
    .await)
}
