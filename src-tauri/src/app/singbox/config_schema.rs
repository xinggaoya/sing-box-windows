//! sing-box 配置序列化结构体（仅覆盖本项目会生成/维护的字段）
//!
//! 说明：
//! - sing-box 官方配置字段很多且随版本演进；这里不追求“全量强类型”，只把我们需要稳定维护的部分结构化。
//! - 对于订阅节点等多形态对象，仍然使用 `serde_json::Value` 透传，避免强行绑定某一种出站协议结构。

use serde::Serialize;
use serde_json::Value;

#[derive(Debug, Clone, Serialize)]
pub(crate) struct SingBoxConfig {
    pub log: LogConfig,
    pub experimental: ExperimentalConfig,
    pub dns: DnsConfig,
    pub inbounds: Vec<Value>,
    pub outbounds: Vec<Value>,
    pub route: RouteConfig,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LogConfig {
    pub disabled: bool,
    pub level: String,
    pub timestamp: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ExperimentalConfig {
    pub cache_file: CacheFileConfig,
    pub clash_api: ClashApiConfig,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CacheFileConfig {
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ClashApiConfig {
    pub external_controller: String,
    pub external_ui: String,
    pub external_ui_download_url: String,
    pub external_ui_download_detour: String,
    pub default_mode: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DnsConfig {
    pub servers: Vec<DnsServerConfig>,
    pub rules: Vec<Value>,
    pub independent_cache: bool,
    #[serde(rename = "final")]
    pub final_server: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DnsServerConfig {
    pub tag: String,
    pub address: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_resolver: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strategy: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detour: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RouteConfig {
    #[serde(rename = "rule_set")]
    pub rule_set: Vec<Value>,
    pub rules: Vec<Value>,
    #[serde(rename = "final")]
    pub final_outbound: String,
    pub auto_detect_interface: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_domain_resolver: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RemoteRuleSetConfig {
    pub tag: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub format: String,
    pub url: String,
    pub download_detour: String,
    pub update_interval: String,
}

