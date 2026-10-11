//! 自定义规则 CRUD（issue #62）
//!
//! 走「写配置文件 + 重启内核」路径，不依赖 gRPC：
//! 自定义规则的生命周期与内核默认规则完全不同 —— 自定义规则持久化到本地并写入
//! 本程序生成的配置；内核默认规则的启停走 gRPC（运行时、不持久化）。
//! 两者在 UI 与代码里必须明确区分。

use crate::app::singbox::common::normalize_default_outbound;
use crate::app::singbox::config_generator::inject_custom_rules;
use crate::app::storage::custom_rule::{
    CustomRule, CustomRuleAction, CustomRuleMatchType, STORAGE_KEY,
};
use crate::app::storage::enhanced_storage_service::{db_get_app_config, get_enhanced_storage};
use chrono::Utc;
use serde_json::Value;
use std::fs;
use std::sync::Arc;
use tauri::AppHandle;
use tracing::{info, warn};

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
    let live_content =
        fs::read_to_string(config_path).map_err(|e| format!("读取配置失败: {}", e))?;
    let injected_path = config_path.with_extension("injected");
    let base_path = config_path.with_extension("base");

    // 计算干净基线（不含已注入的自定义规则）：
    // 1) 有 .injected 注入台账 → 基线 = 当前 live 内容剔除台账记录的注入规则。
    //    旧方案用一次性 .base 快照,订阅更新 / 端口修改 / 设置 patch 对 live 配置的
    //    修改会在下一次规则编辑时被旧快照静默回退;按台账"先剔除再注入"则永远以
    //    live 配置的最新内容为基线,其他写入方的修改得以保留。
    // 2) 无台账但有历史 .base → 沿用 .base（迁移路径,写完台账后删除）。
    // 3) 都没有 → live 本身就是干净基线（首次注入）。
    let baseline_content = if injected_path.exists() {
        let injected_values: Vec<Value> = fs::read_to_string(&injected_path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        strip_injected_rules(&live_content, &injected_values)?
    } else if base_path.exists() {
        fs::read_to_string(&base_path).map_err(|e| format!("读取 .base 备份失败: {}", e))?
    } else {
        live_content.clone()
    };

    let mut config: Value =
        serde_json::from_str(&baseline_content).map_err(|e| format!("解析配置基线失败: {}", e))?;
    inject_custom_rules(&mut config, rules, default_outbound);
    let serialized =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;

    // 先写台账再写配置：若中途崩溃,台账"超前"（记录了未实际写入的规则）是良性的——
    // 下次剔除时这些规则不在 live 里,剔除是空操作;反过来则可能漏剔导致重复注入。
    let injected_now: Vec<Value> = rules
        .iter()
        .filter_map(|r| r.to_route_rule(default_outbound))
        .collect();
    fs::write(
        &injected_path,
        serde_json::to_string(&injected_now).map_err(|e| format!("序列化注入台账失败: {}", e))?,
    )
    .map_err(|e| format!("写入注入台账失败: {}", e))?;
    fs::write(config_path, serialized).map_err(|e| format!("写入配置文件失败: {}", e))?;

    // 台账已接管幂等性,历史 .base 快照不再需要
    if base_path.exists() {
        let _ = fs::remove_file(&base_path);
    }
    Ok(())
}

/// 从配置内容中剔除与 `injected` 完全相同的路由规则（serde_json::Value 的对象比较
/// 与键序无关）,得到干净基线。
fn strip_injected_rules(content: &str, injected: &[Value]) -> Result<String, String> {
    if injected.is_empty() {
        return Ok(content.to_string());
    }
    let mut config: Value =
        serde_json::from_str(content).map_err(|e| format!("解析配置失败: {}", e))?;
    if let Some(rules_arr) = config
        .get_mut("route")
        .and_then(|r| r.get_mut("rules"))
        .and_then(|r| r.as_array_mut())
    {
        rules_arr.retain(|rule| !injected.iter().any(|inj| inj == rule));
    }
    serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))
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
