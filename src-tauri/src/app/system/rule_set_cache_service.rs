//! rule-set 本地缓存（sing-box 1.14 `rule_set.initial_path` 冷启动兜底）
//!
//! 远程 rule-set 由内核启动时下载；离线 / 网络受限且本地缓存为空时，首次启动会
//! 长时间卡在 rule-set 下载上。本服务在应用启动后于后台预热缓存目录
//! `<app_data_dir>/rule-sets/<tag>.srs`，`config_generator` 生成配置时把这些路径
//! 写入 `initial_path`。内核仅在 cache_file 缓存缺失时读取一次 initial_path，
//! 之后照常按 update_interval 在后台更新。

use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::Duration;

use tauri::{AppHandle, Manager};
use tracing::{info, warn};

use crate::app::singbox::config_generator::remote_rule_set_sources;
use crate::app::storage::enhanced_storage_service::db_get_app_config;

static CACHE_DIR: OnceLock<PathBuf> = OnceLock::new();

/// 初始化缓存目录（应用启动时调用；重复调用返回已初始化目录）。
/// 失败（如磁盘不可写）返回 None，本次运行配置不写 initial_path。
pub fn init_cache_dir(app_handle: &AppHandle) -> Option<PathBuf> {
    if let Some(dir) = CACHE_DIR.get() {
        return Some(dir.clone());
    }
    let dir = app_handle.path().app_data_dir().ok()?.join("rule-sets");
    if let Err(e) = std::fs::create_dir_all(&dir) {
        warn!("创建 rule-set 缓存目录失败: {}（本次启动不写 initial_path）", e);
        return None;
    }
    let _ = CACHE_DIR.set(dir.clone());
    Some(dir)
}

/// 已初始化的缓存目录；未初始化（如单测环境）返回 None
pub fn cache_dir() -> Option<&'static PathBuf> {
    CACHE_DIR.get()
}

/// 启动后台预热任务：下载缺失 / 过期的 rule-set 副本，不阻塞启动流程
pub fn spawn_warm_task(app_handle: AppHandle) {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = warm_rule_set_cache(&app_handle).await {
            warn!("rule-set 缓存预热中断: {}", e);
        }
    });
}

async fn warm_rule_set_cache(app_handle: &AppHandle) -> Result<(), String> {
    let dir = match init_cache_dir(app_handle) {
        Some(dir) => dir,
        None => return Ok(()),
    };
    let app_config = db_get_app_config(app_handle.clone()).await?;
    let sources = remote_rule_set_sources(&app_config);
    if sources.is_empty() {
        return Ok(());
    }

    // gh-proxy 直连加速；禁用系统代理，避免 TUN 残留的 127.0.0.1 代理配置导致下载失败
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(60))
        .no_proxy()
        .build()
        .map_err(|e| format!("构建 HTTP 客户端失败: {}", e))?;

    let mut updated = 0usize;
    for source in &sources {
        let path = dir.join(format!("{}.srs", source.tag));
        if is_fresh(&path, source.update_interval) {
            continue;
        }
        let result = client.get(source.url).send().await;
        match result {
            Ok(resp) if resp.status().is_success() => match resp.bytes().await {
                Ok(bytes) => {
                    // 先写临时文件再原子重命名，避免中断留下截断的 .srs
                    let tmp = path.with_extension("srs.tmp");
                    if let Err(e) = std::fs::write(&tmp, &bytes) {
                        warn!("写入 rule-set 缓存失败 ({}): {}", source.tag, e);
                        continue;
                    }
                    if let Err(e) = std::fs::rename(&tmp, &path) {
                        warn!("落盘 rule-set 缓存失败 ({}): {}", source.tag, e);
                        let _ = std::fs::remove_file(&tmp);
                        continue;
                    }
                    updated += 1;
                }
                Err(e) => warn!("下载 rule-set 失败 ({}): {}", source.tag, e),
            },
            Ok(resp) => warn!("下载 rule-set 失败 ({}): HTTP {}", source.tag, resp.status()),
            Err(e) => warn!("下载 rule-set 失败 ({}): {}", source.tag, e),
        }
    }
    info!(
        "rule-set 缓存预热完成：更新 {} / 共 {} 个（目录 {}）",
        updated,
        sources.len(),
        dir.display()
    );
    Ok(())
}

/// 副本在 update_interval 内视为新鲜；文件缺失或解析失败视为过期
fn is_fresh(path: &Path, update_interval: &str) -> bool {
    let Ok(meta) = std::fs::metadata(path) else {
        return false;
    };
    let Some(max_age) = parse_interval(update_interval) else {
        return false;
    };
    meta.modified()
        .ok()
        .and_then(|m| m.elapsed().ok())
        .map(|age| age < max_age)
        .unwrap_or(false)
}

/// 解析 sing-box update_interval 形如 "1d" / "7d" / "12h" 的时长
fn parse_interval(interval: &str) -> Option<Duration> {
    let trimmed = interval.trim();
    let (value, unit) = trimmed.split_at(trimmed.len().saturating_sub(1));
    let value: u64 = value.trim().parse().ok()?;
    match unit {
        "s" => Some(Duration::from_secs(value)),
        "m" => Some(Duration::from_secs(value * 60)),
        "h" => Some(Duration::from_secs(value * 3600)),
        "d" => Some(Duration::from_secs(value * 86400)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_interval_supports_s_m_h_d() {
        assert_eq!(parse_interval("1d"), Some(Duration::from_secs(86400)));
        assert_eq!(parse_interval("7d"), Some(Duration::from_secs(7 * 86400)));
        assert_eq!(parse_interval("12h"), Some(Duration::from_secs(12 * 3600)));
        assert_eq!(parse_interval("30m"), Some(Duration::from_secs(1800)));
        assert_eq!(parse_interval("90s"), Some(Duration::from_secs(90)));
    }

    #[test]
    fn parse_interval_rejects_invalid() {
        assert_eq!(parse_interval(""), None);
        assert_eq!(parse_interval("abc"), None);
        assert_eq!(parse_interval("1w"), None);
    }

    #[test]
    fn is_fresh_missing_file_is_stale() {
        assert!(!is_fresh(Path::new("Z:/definitely/not/here.srs"), "1d"));
    }
}
