/// 客户端侧模板校验：与服务端 sing-box-template-server 的 validation.rs 保持同一套规则，
/// 在保存/发布前预校验给出即时反馈，服务端仍做兜底校验。
pub const MAX_TEMPLATE_CONTENT_BYTES: usize = 512 * 1024;
pub const MAX_NAME_LEN: usize = 80;
pub const MAX_DESCRIPTION_LEN: usize = 2000;
pub const MAX_AUTHOR_NAME_LEN: usize = 60;
pub const SUPPORTED_KERNEL_TYPES: [&str; 1] = ["singbox"];

/// 模板内容校验：合法 JSON 对象 + `outbounds` 数组非空 + 大小限制。
pub fn validate_template_content(content: &str) -> Result<(), String> {
    if content.len() > MAX_TEMPLATE_CONTENT_BYTES {
        return Err(format!(
            "模板内容过大（{} 字节，上限 {MAX_TEMPLATE_CONTENT_BYTES} 字节）",
            content.len()
        ));
    }

    let value: serde_json::Value =
        serde_json::from_str(content.trim()).map_err(|e| format!("模板内容不是合法 JSON: {e}"))?;
    if !value.is_object() {
        return Err("模板内容必须是 JSON 对象".to_string());
    }

    let outbounds = value
        .get("outbounds")
        .ok_or("模板缺少 outbounds 字段（模板需要声明出站/分组骨架）")?;
    let outbounds = outbounds.as_array().ok_or("outbounds 必须是数组")?;
    if outbounds.is_empty() {
        return Err("outbounds 不能为空数组".to_string());
    }

    Ok(())
}

/// 模板元信息校验（名称/描述/署名/内核类型）。
pub fn validate_template_meta(
    name: &str,
    description: &str,
    author_name: &str,
    kernel_type: &str,
) -> Result<(), String> {
    let name = name.trim();
    if name.is_empty() {
        return Err("模板名称不能为空".to_string());
    }
    if name.chars().count() > MAX_NAME_LEN {
        return Err(format!("模板名称过长（最多 {MAX_NAME_LEN} 个字符）"));
    }

    if description.trim().chars().count() > MAX_DESCRIPTION_LEN {
        return Err(format!("模板描述过长（最多 {MAX_DESCRIPTION_LEN} 个字符）"));
    }

    if author_name.trim().chars().count() > MAX_AUTHOR_NAME_LEN {
        return Err(format!("作者署名过长（最多 {MAX_AUTHOR_NAME_LEN} 个字符）"));
    }

    let kernel = kernel_type.trim().to_lowercase();
    if !SUPPORTED_KERNEL_TYPES.contains(&kernel.as_str()) {
        return Err(format!(
            "暂不支持的内核类型: {kernel}（当前仅支持 {SUPPORTED_KERNEL_TYPES:?}）"
        ));
    }

    Ok(())
}

#[cfg(test)]
#[path = "validation.tests.rs"]
mod tests;
