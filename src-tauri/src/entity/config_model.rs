use serde::Serialize;

#[derive(Debug, serde::Deserialize, Serialize)]
#[allow(dead_code)]
pub struct Inbound {
    pub r#type: String,
    pub tag: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listen_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inet4_address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_route: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub strict_route: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stack: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sniff: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub set_system_proxy: Option<bool>,
}


#[derive(Debug, serde::Deserialize, Serialize)]
#[allow(dead_code)]
pub struct ClashApiConfig {
    pub external_controller: String,
    pub external_ui: String,
    pub external_ui_download_url: String,
    pub external_ui_download_detour: String,
    pub default_mode: String,
}

#[derive(Debug, serde::Deserialize, Serialize)]
#[allow(dead_code)]
pub struct CacheFileConfig {
    pub enabled: bool,
}

#[derive(Debug, serde::Deserialize, Serialize)]
#[allow(dead_code)]
pub struct Config {
    pub clash_api: ClashApiConfig,
    pub cache_file: CacheFileConfig,
}
