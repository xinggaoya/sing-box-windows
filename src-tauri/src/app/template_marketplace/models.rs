use serde::{Deserialize, Serialize};
use ts_rs::TS;

use crate::app::singbox::config_generator::NODES_PLACEHOLDER;

/// 官方内置模板的固定 ID（对应 config_generator::generate_base_config 硬编码骨架）。
pub const OFFICIAL_TEMPLATE_ID: &str = "official";

/// 模板来源。
pub const SOURCE_BUILTIN: &str = "builtin";
pub const SOURCE_LOCAL: &str = "local";
pub const SOURCE_MARKET: &str = "market";

/// 市场审核状态（与服务端对齐；`deleted` 为客户端本地标记：市场记录已不存在）。
pub const MARKET_STATUS_PENDING: &str = "pending";
pub const MARKET_STATUS_APPROVED: &str = "approved";
pub const MARKET_STATUS_REJECTED: &str = "rejected";
pub const MARKET_STATUS_DELETED: &str = "deleted";

/// 配置模板：一份 sing-box 配置骨架，通过 `{{NODES}}` 占位符或节点注入生成最终配置。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/ConfigTemplate.ts")]
pub struct ConfigTemplate {
    /// 本地唯一 ID；官方模板固定为 `official`
    pub id: String,
    pub name: String,
    /// 内核类型，目前仅 `singbox`
    pub kernel_type: String,
    /// 模板内容（完整 sing-box JSON，可含 `{{NODES}}` 占位符）
    pub content: String,
    /// 来源：builtin / local / market
    pub source: String,
    pub description: Option<String>,
    pub author: Option<String>,
    /// 市场模板 ID（发布或从市场下载后有值）
    #[ts(optional)]
    pub remote_id: Option<String>,
    /// 我发布的市场模板的编辑令牌（仅保存在本地，用于更新/删除市场记录）
    #[ts(optional)]
    pub edit_token: Option<String>,
    /// 市场审核状态：pending / approved / rejected
    #[ts(optional)]
    pub market_status: Option<String>,
    /// 审核拒绝理由
    #[ts(optional)]
    pub market_reject_reason: Option<String>,
    /// 关联的内核 schema 版本（用于兼容性提示）
    #[ts(optional)]
    pub schema_version: Option<String>,
    /// 编辑版本号（每次保存自增）
    pub revision: i64,
    /// 创建时间（毫秒）
    pub created_at: i64,
    /// 更新时间（毫秒）
    pub updated_at: i64,
}

/// 内置默认市场服务地址：开发构建回落本地调试服务，发布构建指向线上地址。
pub fn default_market_service_url() -> String {
    if cfg!(debug_assertions) {
        "http://127.0.0.1:8787".to_string()
    } else {
        "http://sub.moncn.cn:8787".to_string()
    }
}

/// 模板市场设置（generic_config KV 持久化）。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/TemplateMarketSettings.ts")]
pub struct TemplateMarketSettings {
    /// 市场服务地址；空表示使用内置默认地址
    pub service_url: String,
    /// 当前生效模板：`official` 或本地模板 ID；订阅刷新/下载时统一使用
    pub active_template_id: String,
}

impl Default for TemplateMarketSettings {
    fn default() -> Self {
        Self {
            service_url: String::new(),
            active_template_id: OFFICIAL_TEMPLATE_ID.to_string(),
        }
    }
}

/// 市场模板（列表/详情共用；列表项不含 content）。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/MarketTemplate.ts")]
pub struct MarketTemplate {
    pub id: String,
    pub name: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub author_name: String,
    #[serde(default = "default_kernel_type")]
    pub kernel_type: String,
    #[serde(default)]
    pub schema_version: Option<String>,
    /// 详情才有：pending / approved / rejected
    #[serde(default)]
    #[ts(optional)]
    pub status: Option<String>,
    #[serde(default)]
    #[ts(optional)]
    pub reject_reason: Option<String>,
    #[serde(default)]
    pub downloads: i64,
    /// 列表项无 content，详情/下载接口才有
    #[serde(default)]
    #[ts(optional)]
    pub content: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

fn default_kernel_type() -> String {
    "singbox".to_string()
}

/// 市场模板列表响应。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/MarketTemplateList.ts")]
pub struct MarketTemplateList {
    pub items: Vec<MarketTemplate>,
    pub total: i64,
    pub page: i64,
    pub page_size: i64,
}

/// 发布结果：edit_token 明文仅此一次返回，客户端负责持久保存。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/MarketPublishResult.ts")]
pub struct MarketPublishResult {
    pub id: String,
    pub edit_token: String,
    pub status: String,
}

/// 市场服务健康检查响应。
#[derive(Debug, Clone, Serialize, Deserialize, TS)]
#[ts(export, export_to = "../src/types/generated/MarketHealth.ts")]
pub struct MarketHealth {
    pub status: String,
    pub service: String,
    pub version: String,
}

/// 发布/更新市场的请求体（与服务端 PublishTemplateRequest 对齐）。
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketPublishRequest {
    pub name: String,
    pub description: String,
    pub author_name: String,
    pub kernel_type: String,
    pub schema_version: Option<String>,
    pub content: String,
}

/// 官方模板的展示名（前端可按 id=official 做 i18n 覆盖）。
pub const OFFICIAL_TEMPLATE_NAME: &str = "官方默认模板";

/// 官方内置模板条目（不落库，读取时按当前 AppConfig 生成骨架内容）。
pub fn official_template(
    app_config: &crate::app::storage::state_model::AppConfig,
) -> ConfigTemplate {
    let content = crate::app::singbox::config_generator::generate_base_config(app_config);
    ConfigTemplate {
        id: OFFICIAL_TEMPLATE_ID.to_string(),
        name: OFFICIAL_TEMPLATE_NAME.to_string(),
        kernel_type: "singbox".to_string(),
        content: serde_json::to_string_pretty(&content).unwrap_or_else(|_| "{}".to_string()),
        source: SOURCE_BUILTIN.to_string(),
        description: Some(format!(
            "内置通用配置骨架（国内直连/代理分流 + DNS 分层），支持 {} 占位符导出编辑",
            NODES_PLACEHOLDER
        )),
        author: None,
        remote_id: None,
        edit_token: None,
        market_status: None,
        market_reject_reason: None,
        schema_version: None,
        revision: 1,
        created_at: 0,
        updated_at: 0,
    }
}
