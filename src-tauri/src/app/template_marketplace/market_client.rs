use serde::{de::DeserializeOwned, Serialize};

use crate::app::template_marketplace::models::{
    MarketHealth, MarketPublishRequest, MarketPublishResult, MarketTemplate, MarketTemplateList,
};
use crate::utils::http_client;

/// 规范化服务地址：去尾斜杠 + 协议校验。
pub fn normalize_base_url(service_url: &str) -> Result<String, String> {
    let trimmed = service_url.trim().trim_end_matches('/');
    if trimmed.is_empty() {
        return Err("未配置模板市场服务地址，请先在模板市场页设置".to_string());
    }
    if !trimmed.starts_with("http://") && !trimmed.starts_with("https://") {
        return Err(format!(
            "服务地址必须以 http:// 或 https:// 开头: {trimmed}"
        ));
    }
    Ok(trimmed.to_string())
}

async fn send<T: DeserializeOwned, B: Serialize>(
    method: reqwest::Method,
    url: String,
    edit_token: Option<&str>,
    json_body: Option<B>,
) -> Result<T, String> {
    let mut request = http_client::get_client().request(method, url);
    if let Some(token) = edit_token {
        request = request.header("x-edit-token", token);
    }
    if let Some(body) = json_body {
        request = request.json(&body);
    }

    let response = request
        .send()
        .await
        .map_err(|e| format!("请求模板市场失败: {e}"))?;
    let status = response.status();
    let text = response
        .text()
        .await
        .map_err(|e| format!("读取模板市场响应失败: {e}"))?;

    if !status.is_success() {
        let message = serde_json::from_str::<serde_json::Value>(&text)
            .ok()
            .and_then(|v| v.get("error").and_then(|e| e.as_str()).map(str::to_string))
            .unwrap_or_else(|| format!("模板市场返回 {status}"));
        return Err(message);
    }

    serde_json::from_str(&text).map_err(|e| format!("解析模板市场响应失败: {e}"))
}

pub async fn health(base: &str) -> Result<MarketHealth, String> {
    send::<MarketHealth, ()>(
        reqwest::Method::GET,
        format!("{base}/api/v1/health"),
        None,
        None,
    )
    .await
}

pub async fn list(
    base: &str,
    search: Option<&str>,
    sort: Option<&str>,
    page: i64,
    page_size: i64,
) -> Result<MarketTemplateList, String> {
    let mut url = format!("{base}/api/v1/templates?page={page}&page_size={page_size}");
    if let Some(keyword) = search.map(str::trim).filter(|s| !s.is_empty()) {
        url.push_str(&format!("&search={}", urlencoding::encode(keyword)));
    }
    if let Some(sort) = sort.map(str::trim).filter(|s| !s.is_empty()) {
        url.push_str(&format!("&sort={}", urlencoding::encode(sort)));
    }
    send::<MarketTemplateList, ()>(reqwest::Method::GET, url, None, None).await
}

pub async fn detail(
    base: &str,
    remote_id: &str,
    edit_token: Option<&str>,
) -> Result<MarketTemplate, String> {
    send::<MarketTemplate, ()>(
        reqwest::Method::GET,
        format!("{base}/api/v1/templates/{remote_id}"),
        edit_token,
        None,
    )
    .await
}

pub async fn publish(
    base: &str,
    req: &MarketPublishRequest,
) -> Result<MarketPublishResult, String> {
    send(
        reqwest::Method::POST,
        format!("{base}/api/v1/templates"),
        None,
        Some(req),
    )
    .await
}

/// 更新市场记录：内容变更后服务端会重置为待审核。
pub async fn update(
    base: &str,
    remote_id: &str,
    edit_token: &str,
    req: &MarketPublishRequest,
) -> Result<MarketTemplate, String> {
    send(
        reqwest::Method::PUT,
        format!("{base}/api/v1/templates/{remote_id}"),
        Some(edit_token),
        Some(req),
    )
    .await
}

pub async fn delete(base: &str, remote_id: &str, edit_token: &str) -> Result<(), String> {
    let _: serde_json::Value = send(
        reqwest::Method::DELETE,
        format!("{base}/api/v1/templates/{remote_id}"),
        Some(edit_token),
        None::<()>,
    )
    .await?;
    Ok(())
}

/// 获取已上架模板内容（服务端同时累计下载次数）。
pub async fn download(base: &str, remote_id: &str) -> Result<MarketTemplate, String> {
    send::<MarketTemplate, ()>(
        reqwest::Method::POST,
        format!("{base}/api/v1/templates/{remote_id}/download"),
        None,
        None,
    )
    .await
}
