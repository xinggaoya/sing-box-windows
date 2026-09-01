use crate::app::constants::paths;
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use std::error::Error;
use std::fs::File;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use tauri::AppHandle;
use tracing::{info, warn};

/// 切换代理模式的实现入口。
///
/// sing-box 1.14+ 已移除 `experimental.clash_api.default_mode`，代理模式改为通过
/// gRPC `SetClashMode` 运行时切换（该接口在阶段2 实现）。本函数当前只把模式写入
/// `route.rules` 最高优先级的 `clash_mode` 规则，重启内核后生效。
pub async fn toggle_proxy_mode_impl(app_handle: AppHandle, mode: String) -> Result<String, String> {
    if !["global", "rule"].contains(&mode.as_str()) {
        return Err(format!("无效的代理模式: {}", mode));
    }

    info!("正在切换代理模式为: {}", mode);

    let app_config = db_get_app_config(app_handle)
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;

    let path = resolve_proxy_mode_config_path(app_config.active_config_path.as_deref());

    if !path.exists() {
        return Err("配置文件不存在，请先添加订阅".to_string());
    }

    let mode_for_outbound = mode.clone();
    let path_for_outbound = path.clone();
    let write_result: Result<(), String> = tokio::task::spawn_blocking(move || {
        write_clash_mode_rule(&path_for_outbound, &mode_for_outbound)
            .map_err(|e| e.to_string())
    })
    .await
    .map_err(|e| format!("写入代理模式失败: {}", e))?;

    if let Err(e) = write_result {
        warn!("写入代理模式到配置文件失败: {}", e);
        return Err(format!("写入代理模式失败: {}", e));
    }

    // sing-box 1.14+ 的 gRPC API 提供 SetClashMode 运行时切换，
    // 由阶段2 接入；当前仅写配置，需重启内核生效。
    info!(
        "代理模式已写入配置文件: {}，当前运行中的内核需重启后生效",
        mode
    );
    Ok(format!("代理模式已保存为: {}，重启内核后生效", mode))
}

pub async fn get_current_proxy_mode_impl(app_handle: AppHandle) -> Result<String, String> {
    info!("正在获取当前代理模式");

    let app_config = db_get_app_config(app_handle)
        .await
        .map_err(|e| format!("获取应用配置失败: {}", e))?;

    let default_config_path = paths::get_config_dir().join("config.json");
    let candidates = collect_proxy_mode_config_paths(
        app_config.active_config_path.as_deref().map(Path::new),
        &default_config_path,
    );

    for candidate in candidates {
        if !candidate.exists() {
            continue;
        }
        match read_clash_mode_from_config(&candidate) {
            Ok(mode) => {
                info!("从配置文件 {:?} 读取代理模式: {}", candidate, mode);
                return Ok(mode);
            }
            Err(e) => warn!("从 {:?} 读取代理模式失败: {}", candidate, e),
        }
    }

    Ok("rule".to_string())
}

/// 把代理模式以最高优先级的 `clash_mode` 规则形式写入 sing-box 配置的 route.rules。
///
/// 已存在的 clash_mode 规则会被替换（幂等），始终保持在数组首位置以确保最高优先级。
fn write_clash_mode_rule(config_path: &Path, mode: &str) -> Result<(), Box<dyn Error>> {
    let mut file = File::open(config_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let mut config: serde_json::Value = serde_json::from_str(&content)?;
    ensure_route_rules(&mut config);
    upsert_clash_mode_rule(&mut config, mode)?;

    let updated = serde_json::to_string_pretty(&config)?;
    let mut file = File::create(config_path)?;
    file.write_all(updated.as_bytes())?;
    Ok(())
}

fn ensure_route_rules(config: &mut serde_json::Value) {
    let root = match config.as_object_mut() {
        Some(obj) => obj,
        None => return,
    };
    let route = root
        .entry("route".to_string())
        .or_insert_with(|| serde_json::json!({}));
    let route_obj = match route.as_object_mut() {
        Some(obj) => obj,
        None => return,
    };
    if !route_obj.contains_key("rules") {
        route_obj.insert("rules".to_string(), serde_json::json!([]));
    }
}

fn upsert_clash_mode_rule(
    config: &mut serde_json::Value,
    mode: &str,
) -> Result<(), Box<dyn Error>> {
    let rules = config
        .get_mut("route")
        .and_then(|r| r.get_mut("rules"))
        .and_then(|r| r.as_array_mut())
        .ok_or_else(|| "route.rules 不是数组".to_string())?;

    // 移除已有的 clash_mode 规则（幂等）
    rules.retain(|rule| rule.get("clash_mode").is_none());

    // 新规则：clash_mode → outbound（direct 走直连，其它走 default proxy）
    let outbound = if mode == "direct" { "direct" } else { "proxy" };
    let mut new_rule = serde_json::Map::new();
    new_rule.insert("clash_mode".to_string(), serde_json::json!(mode));
    new_rule.insert("outbound".to_string(), serde_json::json!(outbound));

    rules.insert(0, serde_json::Value::Object(new_rule));
    Ok(())
}

fn read_clash_mode_from_config(config_path: &Path) -> Result<String, Box<dyn Error>> {
    let mut file = File::open(config_path)?;
    let mut content = String::new();
    file.read_to_string(&mut content)?;

    let json: serde_json::Value = serde_json::from_str(&content)?;

    if let Some(rules) = json
        .get("route")
        .and_then(|r| r.get("rules"))
        .and_then(|r| r.as_array())
    {
        for rule in rules {
            if let Some(mode) = rule.get("clash_mode").and_then(|m| m.as_str()) {
                return Ok(mode.to_string());
            }
        }
    }

    Ok("rule".to_string())
}

fn resolve_proxy_mode_config_path(active_config_path: Option<&str>) -> PathBuf {
    active_config_path
        .map(PathBuf::from)
        .unwrap_or_else(|| paths::get_config_dir().join("config.json"))
}

fn collect_proxy_mode_config_paths(
    active_config_path: Option<&Path>,
    default_config_path: &Path,
) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(path) = active_config_path {
        paths.push(path.to_path_buf());
    }
    if !paths.iter().any(|path| path == default_config_path) {
        paths.push(default_config_path.to_path_buf());
    }
    paths
}

#[cfg(test)]
#[path = "mode.tests.rs"]
mod tests;