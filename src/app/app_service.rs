// 下载订阅
#[tauri::command]
pub async fn download_subscription(url: String) -> Result<(), String> {
    download_and_process_subscription(url)
        .await
        .map_err(|e| format!("下载订阅失败: {}", e))?;
    let _ = set_system_proxy();  // 使用 let _ 忽略结果
    Ok(())
}

#[tauri::command]
pub fn set_proxy_mode(mode: String) -> Result<(), String> {
    if let Err(e) = set_system_proxy() {
        return Err(format!("设置系统代理失败: {}", e));
    }
    Ok(())
}

#[allow(dead_code)]
fn disable_proxy() -> Result<(), Box<dyn Error>> {
    // ... existing code ...
    Ok(())
} 