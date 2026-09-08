use tauri::AppHandle;

use crate::app::storage::state_model::AppConfig;
use crate::app::template_marketplace::local_store;
use crate::app::template_marketplace::market_client;
use crate::app::template_marketplace::models::{
    official_template, ConfigTemplate, MarketHealth, MarketPublishRequest, MarketTemplate,
    MarketTemplateList, TemplateMarketSettings, MARKET_STATUS_APPROVED, MARKET_STATUS_DELETED,
    OFFICIAL_TEMPLATE_ID, SOURCE_LOCAL, SOURCE_MARKET,
};
use crate::app::template_marketplace::template_form::{
    generate_config_from_form, parse_form_from_template, TemplateFormOptions,
};
use crate::app::template_marketplace::validation::{
    validate_template_content, validate_template_meta,
};

fn now_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

fn new_template_id() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// 全部模板：官方内置条目 + 本地模板（按更新时间倒序）。
#[tauri::command]
pub async fn get_config_templates(app: AppHandle) -> Result<Vec<ConfigTemplate>, String> {
    let app_config = crate::app::storage::enhanced_storage_service::db_get_app_config(app.clone())
        .await
        .map_err(|e| format!("读取设置失败: {e}"))?;
    let mut templates = vec![official_template(&app_config)];
    let mut local = local_store::load_templates(&app).await?;
    local.sort_by_key(|t| std::cmp::Reverse(t.updated_at));
    templates.extend(local);
    Ok(templates)
}

/// 新增或保存本地模板（官方 ID 不可占用；保存前做元信息 + 内容校验）。
#[tauri::command]
pub async fn save_config_template(
    app: AppHandle,
    template: ConfigTemplate,
) -> Result<ConfigTemplate, String> {
    validate_template_meta(
        &template.name,
        template.description.as_deref().unwrap_or(""),
        template.author.as_deref().unwrap_or(""),
        &template.kernel_type,
    )?;
    validate_template_content(&template.content)?;

    let mut template = template;
    let existing = if template.id.is_empty() {
        None
    } else {
        local_store::get_template(&app, &template.id).await?
    };

    match existing {
        Some(mut old) => {
            old.name = template.name;
            old.kernel_type = template.kernel_type;
            old.content = template.content;
            old.description = template.description;
            old.author = template.author;
            old.schema_version = template.schema_version;
            old.revision += 1;
            old.updated_at = now_ms();
            local_store::upsert_template(&app, old.clone()).await?;
            Ok(old)
        }
        None => {
            template.id = new_template_id();
            if template.source.is_empty() {
                template.source = SOURCE_LOCAL.to_string();
            }
            template.revision = 1;
            template.created_at = now_ms();
            template.updated_at = template.created_at;
            local_store::upsert_template(&app, template.clone()).await?;
            Ok(template)
        }
    }
}

/// 删除本地模板；若删除的是当前激活模板则自动回退官方模板。
#[tauri::command]
pub async fn delete_config_template(app: AppHandle, template_id: String) -> Result<(), String> {
    local_store::delete_template(&app, &template_id).await?;
    Ok(())
}

#[tauri::command]
pub async fn get_template_market_settings(
    app: AppHandle,
) -> Result<TemplateMarketSettings, String> {
    local_store::load_settings(&app).await
}

/// 保存市场设置（服务地址 + 当前激活模板）。
#[tauri::command]
pub async fn set_template_market_settings(
    app: AppHandle,
    settings: TemplateMarketSettings,
) -> Result<TemplateMarketSettings, String> {
    let mut settings = settings;
    let cleared_to_default = settings.service_url.trim().is_empty();
    if !cleared_to_default {
        settings.service_url = market_client::normalize_base_url(&settings.service_url)?;
    }
    if settings.active_template_id.is_empty() {
        settings.active_template_id = OFFICIAL_TEMPLATE_ID.to_string();
    }
    if settings.active_template_id != OFFICIAL_TEMPLATE_ID {
        let exists = local_store::get_template(&app, &settings.active_template_id)
            .await?
            .is_some();
        if !exists {
            return Err(format!("激活模板不存在: {}", settings.active_template_id));
        }
    }
    local_store::save_settings(&app, &settings).await?;
    // 清空存储 = 使用内置默认地址；返回有效值供前端回显
    if cleared_to_default {
        settings.service_url =
            crate::app::template_marketplace::models::default_market_service_url();
    }
    Ok(settings)
}

/// 切换当前生效模板（official 或本地模板 ID）。
#[tauri::command]
pub async fn set_active_template(
    app: AppHandle,
    template_id: String,
) -> Result<TemplateMarketSettings, String> {
    let mut settings = local_store::load_settings(&app).await?;
    settings.active_template_id = template_id;
    set_template_market_settings(app, settings).await
}

/// 导出官方骨架内容（作为新建模板的起点；节点由订阅刷新时注入，导出内容不含节点）。
#[tauri::command]
pub async fn export_official_template(app: AppHandle) -> Result<String, String> {
    let app_config = current_app_config(&app).await?;
    let official = official_template(&app_config);
    Ok(official.content)
}

async fn current_app_config(app: &AppHandle) -> Result<AppConfig, String> {
    crate::app::storage::enhanced_storage_service::db_get_app_config(app.clone())
        .await
        .map_err(|e| format!("读取设置失败: {e}"))
}

async fn require_base_url(app: &AppHandle) -> Result<String, String> {
    let settings = local_store::load_settings(app).await?;
    market_client::normalize_base_url(&settings.service_url)
}

/// 由可视化表单生成模板骨架内容（不含订阅节点，留空字段跟随当前应用设置）。
#[tauri::command]
pub async fn generate_template_from_form(
    app: AppHandle,
    form: TemplateFormOptions,
) -> Result<String, String> {
    let app_config = current_app_config(&app).await?;
    let config = generate_config_from_form(&app_config, &form)?;
    serde_json::to_string_pretty(&config).map_err(|e| format!("序列化模板失败: {e}"))
}

/// 尝试把模板骨架解析回可视化表单；模板不是"表单可表达"形状时返回错误（前端回退 JSON 模式）。
#[tauri::command]
pub async fn parse_template_form(content: String) -> Result<TemplateFormOptions, String> {
    parse_form_from_template(&content)
}

/// 校验市场服务可用性（地址未传时使用已保存设置）。
#[tauri::command]
pub async fn market_check_service(
    app: AppHandle,
    service_url: Option<String>,
) -> Result<MarketHealth, String> {
    let base = match service_url.as_deref() {
        Some(url) if !url.trim().is_empty() => market_client::normalize_base_url(url)?,
        _ => require_base_url(&app).await?,
    };
    market_client::health(&base).await
}

/// 浏览已上架模板（搜索/排序/分页）。
#[tauri::command]
pub async fn market_list_templates(
    app: AppHandle,
    search: Option<String>,
    sort: Option<String>,
    page: Option<i64>,
    page_size: Option<i64>,
) -> Result<MarketTemplateList, String> {
    let base = require_base_url(&app).await?;
    market_client::list(
        &base,
        search.as_deref(),
        sort.as_deref(),
        page.unwrap_or(1),
        page_size.unwrap_or(20),
    )
    .await
}

/// 查看市场模板详情；若本地持有该模板的编辑令牌则一并发送（可查看自己待审核的记录）。
#[tauri::command]
pub async fn market_get_template_detail(
    app: AppHandle,
    remote_id: String,
) -> Result<MarketTemplate, String> {
    let base = require_base_url(&app).await?;
    let edit_token = local_store::load_templates(&app)
        .await?
        .into_iter()
        .find(|t| t.remote_id.as_deref() == Some(&remote_id))
        .and_then(|t| t.edit_token);
    market_client::detail(&base, &remote_id, edit_token.as_deref()).await
}

fn publish_request_from_template(
    template: &ConfigTemplate,
    author_name: Option<String>,
) -> Result<MarketPublishRequest, String> {
    Ok(MarketPublishRequest {
        name: template.name.clone(),
        description: template.description.clone().unwrap_or_default(),
        author_name: author_name
            .or_else(|| template.author.clone())
            .unwrap_or_default(),
        kernel_type: template.kernel_type.clone(),
        schema_version: template.schema_version.clone(),
        content: template.content.clone(),
    })
}

/// 发布本地模板到市场（匿名；返回的编辑令牌自动保存到本地模板）。
#[tauri::command]
pub async fn market_publish_template(
    app: AppHandle,
    template_id: String,
    author_name: Option<String>,
) -> Result<ConfigTemplate, String> {
    let base = require_base_url(&app).await?;
    let template = local_store::get_template(&app, &template_id)
        .await?
        .ok_or_else(|| format!("本地模板不存在: {template_id}"))?;
    if template.remote_id.is_some() {
        return Err("该模板已发布过，请使用“更新发布”".to_string());
    }
    validate_template_content(&template.content)?;

    let request = publish_request_from_template(&template, author_name)?;
    let result = market_client::publish(&base, &request).await?;

    let mut template = template;
    template.remote_id = Some(result.id);
    template.edit_token = Some(result.edit_token);
    template.market_status = Some(result.status);
    template.market_reject_reason = None;
    template.updated_at = now_ms();
    local_store::upsert_template(&app, template.clone()).await?;
    Ok(template)
}

/// 将本地模板的修改推送到市场（重置为待审核）。
#[tauri::command]
pub async fn market_update_template(
    app: AppHandle,
    template_id: String,
) -> Result<ConfigTemplate, String> {
    let base = require_base_url(&app).await?;
    let template = local_store::get_template(&app, &template_id)
        .await?
        .ok_or_else(|| format!("本地模板不存在: {template_id}"))?;
    let remote_id = template
        .remote_id
        .clone()
        .ok_or_else(|| "该模板尚未发布到市场".to_string())?;
    let edit_token = template
        .edit_token
        .clone()
        .ok_or_else(|| "缺少编辑令牌（该模板可能不是在此设备发布的）".to_string())?;
    validate_template_content(&template.content)?;

    let request = publish_request_from_template(&template, None)?;
    let updated = market_client::update(&base, &remote_id, &edit_token, &request).await?;

    let mut template = template;
    template.market_status = updated.status;
    template.market_reject_reason = None;
    template.updated_at = now_ms();
    local_store::upsert_template(&app, template.clone()).await?;
    Ok(template)
}

/// 删除市场记录（本地模板保留，清除市场关联信息）。
#[tauri::command]
pub async fn market_delete_template(
    app: AppHandle,
    template_id: String,
) -> Result<ConfigTemplate, String> {
    let base = require_base_url(&app).await?;
    let template = local_store::get_template(&app, &template_id)
        .await?
        .ok_or_else(|| format!("本地模板不存在: {template_id}"))?;
    let remote_id = template
        .remote_id
        .clone()
        .ok_or_else(|| "该模板尚未发布到市场".to_string())?;
    let edit_token = template
        .edit_token
        .clone()
        .ok_or_else(|| "缺少编辑令牌（该模板可能不是在此设备发布的）".to_string())?;

    market_client::delete(&base, &remote_id, &edit_token).await?;

    let mut template = template;
    template.remote_id = None;
    template.edit_token = None;
    template.market_status = None;
    template.market_reject_reason = None;
    template.updated_at = now_ms();
    local_store::upsert_template(&app, template.clone()).await?;
    Ok(template)
}

/// 从市场下载模板内容并保存为本地模板（source=market，随后可在“当前模板”中启用）。
#[tauri::command]
pub async fn market_download_template(
    app: AppHandle,
    remote_id: String,
) -> Result<ConfigTemplate, String> {
    let base = require_base_url(&app).await?;
    let item = market_client::download(&base, &remote_id).await?;
    let content = item
        .content
        .clone()
        .ok_or_else(|| "市场模板内容为空".to_string())?;
    validate_template_content(&content)?;

    let template = ConfigTemplate {
        id: new_template_id(),
        name: item.name,
        kernel_type: item.kernel_type,
        content,
        source: SOURCE_MARKET.to_string(),
        description: Some(item.description),
        author: Some(item.author_name),
        remote_id: Some(remote_id),
        edit_token: None,
        market_status: Some(MARKET_STATUS_APPROVED.to_string()),
        market_reject_reason: None,
        schema_version: item.schema_version,
        revision: 1,
        created_at: now_ms(),
        updated_at: now_ms(),
    };
    local_store::upsert_template(&app, template.clone()).await?;
    Ok(template)
}

/// 刷新“我发布的模板”的审核状态（逐个查询市场详情，单项失败不影响其余）。
#[tauri::command]
pub async fn market_refresh_my_templates(app: AppHandle) -> Result<Vec<ConfigTemplate>, String> {
    let base = require_base_url(&app).await?;
    let mut templates = local_store::load_templates(&app).await?;
    let mut changed = false;
    for template in templates.iter_mut() {
        let (Some(remote_id), Some(edit_token)) =
            (template.remote_id.clone(), template.edit_token.clone())
        else {
            continue;
        };
        match market_client::detail(&base, &remote_id, Some(&edit_token)).await {
            Ok(detail) => {
                if template.market_status != detail.status {
                    template.market_status = detail.status.clone();
                    changed = true;
                }
                template.market_reject_reason = detail.reject_reason.clone();
            }
            // 模板可能已被管理员删除：标记为已下架，便于前端提示
            Err(_) => {
                if template.market_status.as_deref() != Some(MARKET_STATUS_DELETED) {
                    template.market_status = Some(MARKET_STATUS_DELETED.to_string());
                    changed = true;
                }
            }
        }
    }
    if changed {
        local_store::save_templates(&app, &templates).await?;
    }
    Ok(templates)
}
