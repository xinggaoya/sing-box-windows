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
    /// 1.14 顶层 HTTP 客户端定义；为 None 时不写入（兼容老内核）
    /// 1.16 后 `rule_set.download_detour` 将被移除，统一通过 `http_clients` + `route.default_http_client` 表达
    #[serde(skip_serializing_if = "Option::is_none")]
    pub http_clients: Option<Vec<HttpClientConfig>>,
    /// 1.14 顶层 services（替代 `experimental.clash_api`）；由 config_generator.rs 注入
    #[serde(skip_serializing_if = "Option::is_none")]
    pub services: Option<Vec<Value>>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct HttpClientConfig {
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tls: Option<Value>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct LogConfig {
    pub disabled: bool,
    pub level: String,
    pub timestamp: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub output: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct ExperimentalConfig {
    pub cache_file: CacheFileConfig,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct CacheFileConfig {
    pub enabled: bool,
    /// 1.14 替代 1.13 deprecated `store_rdrc`；为 None 时不写入，避免污染配置
    #[serde(skip_serializing_if = "Option::is_none")]
    pub store_dns: Option<bool>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DnsConfig {
    pub servers: Vec<DnsServerConfig>,
    pub rules: Vec<Value>,
    /// 1.14 默认按 transport 隔离 DNS 缓存，`independent_cache` 已被移除
    #[serde(skip_serializing_if = "Option::is_none")]
    pub optimistic: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reverse_mapping: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    #[serde(rename = "final")]
    pub final_server: String,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct DnsServerConfig {
    pub tag: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub server_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub server_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interface: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inet4_range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inet6_range: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub domain_resolver: Option<Value>,
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
    pub default_domain_resolver: Option<Value>,
    /// 1.14 新增：默认 HTTP 客户端 tag，对应顶层 `http_clients[]`
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_http_client: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub(crate) struct RemoteRuleSetConfig {
    pub tag: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub format: String,
    pub url: String,
    /// 1.16 移除；1.14 仍然接受，但顶层 `http_clients` + `route.default_http_client` 才是新写法。
    /// 改为 Option，仅在用户明确指定下载出站时写入。
    #[serde(skip_serializing_if = "Option::is_none")]
    pub download_detour: Option<String>,
    pub update_interval: String,
}
