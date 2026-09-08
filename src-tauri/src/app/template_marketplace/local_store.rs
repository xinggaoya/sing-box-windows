use tauri::AppHandle;

use crate::app::storage::enhanced_storage_service::{get_enhanced_storage, EnhancedStorageService};
use crate::app::template_marketplace::models::{
    default_market_service_url, official_template, ConfigTemplate, TemplateMarketSettings,
    OFFICIAL_TEMPLATE_ID,
};

/// generic_config KV 通道的键（复用 subscriptions/custom_rules 先例，避免新表/迁移）
pub const KEY_TEMPLATES: &str = "config_templates";
pub const KEY_MARKET_SETTINGS: &str = "template_market_settings";

async fn storage(app: &AppHandle) -> Result<std::sync::Arc<EnhancedStorageService>, String> {
    get_enhanced_storage(app).await
}

/// 读取全部本地模板（不含官方内置条目，官方条目由调用方按需拼装）。
pub async fn load_templates(app: &AppHandle) -> Result<Vec<ConfigTemplate>, String> {
    let storage = storage(app).await?;
    let templates = storage
        .load_generic_config::<Vec<ConfigTemplate>>(KEY_TEMPLATES)
        .await
        .map_err(|e| format!("读取本地模板失败: {e}"))?
        .unwrap_or_default();
    Ok(templates)
}

/// 持久化本地模板列表（供命令层批量更新审核状态后保存）。
pub async fn save_templates(app: &AppHandle, templates: &[ConfigTemplate]) -> Result<(), String> {
    let storage = storage(app).await?;
    storage
        .save_generic_config(KEY_TEMPLATES, &templates.to_vec())
        .await
        .map_err(|e| format!("保存本地模板失败: {e}"))
}

/// 读取市场设置；未配置时返回默认值（官方模板生效）。
pub async fn load_settings(app: &AppHandle) -> Result<TemplateMarketSettings, String> {
    let storage = storage(app).await?;
    let mut settings = storage
        .load_generic_config::<TemplateMarketSettings>(KEY_MARKET_SETTINGS)
        .await
        .map_err(|e| format!("读取模板市场设置失败: {e}"))?
        .unwrap_or_default();
    // 服务地址留空时回落内置默认（开发构建为本地调试地址，发布构建为线上地址）
    if settings.service_url.trim().is_empty() {
        settings.service_url = default_market_service_url();
    }
    Ok(settings)
}

pub async fn save_settings(
    app: &AppHandle,
    settings: &TemplateMarketSettings,
) -> Result<(), String> {
    let storage = storage(app).await?;
    storage
        .save_generic_config(KEY_MARKET_SETTINGS, settings)
        .await
        .map_err(|e| format!("保存模板市场设置失败: {e}"))
}

/// 按 ID 查找本地模板。
pub async fn get_template(
    app: &AppHandle,
    template_id: &str,
) -> Result<Option<ConfigTemplate>, String> {
    Ok(load_templates(app)
        .await?
        .into_iter()
        .find(|t| t.id == template_id))
}

/// 新增或更新本地模板（官方 ID 不可占用）。
pub async fn upsert_template(app: &AppHandle, template: ConfigTemplate) -> Result<(), String> {
    if template.id == OFFICIAL_TEMPLATE_ID {
        return Err("不能覆盖官方内置模板".to_string());
    }
    let mut templates = load_templates(app).await?;
    if let Some(existing) = templates.iter_mut().find(|t| t.id == template.id) {
        *existing = template;
    } else {
        templates.push(template);
    }
    save_templates(app, &templates).await
}

/// 删除本地模板；若它是当前激活模板，自动回退到官方模板。
pub async fn delete_template(
    app: &AppHandle,
    template_id: &str,
) -> Result<TemplateMarketSettings, String> {
    if template_id == OFFICIAL_TEMPLATE_ID {
        return Err("官方内置模板不可删除".to_string());
    }
    let mut templates = load_templates(app).await?;
    let before = templates.len();
    templates.retain(|t| t.id != template_id);
    if templates.len() == before {
        return Err(format!("模板不存在: {template_id}"));
    }
    save_templates(app, &templates).await?;

    let mut settings = load_settings(app).await?;
    if settings.active_template_id == template_id {
        settings.active_template_id = OFFICIAL_TEMPLATE_ID.to_string();
        save_settings(app, &settings).await?;
    }
    Ok(settings)
}

/// 当前激活模板：None 表示官方内置模板。
pub async fn load_active_template(app: &AppHandle) -> Result<Option<ConfigTemplate>, String> {
    let settings = load_settings(app).await?;
    if settings.active_template_id == OFFICIAL_TEMPLATE_ID {
        return Ok(None);
    }
    match get_template(app, &settings.active_template_id).await? {
        Some(template) => {
            tracing::info!("使用在线模板生成配置: {} ({})", template.name, template.id);
            Ok(Some(template))
        }
        // 激活的模板已被删除：视为配置损坏，交由上层报错而非静默回退官方模板
        None => Err(format!(
            "当前激活模板不存在（id={}），请在模板市场重新选择",
            settings.active_template_id
        )),
    }
}

/// 加载激活模板内容；返回 None 表示官方模板。
/// 模板已配置但内容缺失/被删时报错（订阅生成路径据此失败提示，不静默回退）。
pub async fn load_active_template_content(app: &AppHandle) -> Result<Option<String>, String> {
    match load_active_template(app).await? {
        None => Ok(None),
        Some(template) => {
            if template.content.trim().is_empty() {
                Err(format!(
                    "当前模板「{}」内容为空，请检查模板市场设置",
                    template.name
                ))
            } else {
                Ok(Some(template.content))
            }
        }
    }
}

/// 官方模板条目（读取时按 AppConfig 实时生成骨架）。
pub async fn builtin_template(app: &AppHandle) -> Result<ConfigTemplate, String> {
    let app_config = crate::app::storage::enhanced_storage_service::db_get_app_config(app.clone())
        .await
        .map_err(|e| format!("读取设置失败: {e}"))?;
    Ok(official_template(&app_config))
}
