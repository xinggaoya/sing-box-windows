use crate::app::constants::paths;
use crate::app::core::kernel_auto_manage::auto_manage_with_saved_config;
use crate::app::storage::enhanced_storage_service::get_enhanced_storage;
use crate::app::storage::state_model::{
    AppConfig, LocaleConfig, Subscription, ThemeConfig, UpdateConfig, WindowConfig,
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::path::Component;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

const SNAPSHOT_FORMAT_VERSION: u32 = 2;
const SNAPSHOT_CONFIGS_DIR: &str = "configs";

#[derive(Debug, Clone, Copy)]
enum SnapshotPathKind {
    ActiveConfig,
    SubscriptionConfig,
    SubscriptionBackup,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct BackupSnapshot {
    #[serde(default)]
    format_version: u32,
    #[serde(default)]
    created_at: u64,
    #[serde(default)]
    app_config: AppConfig,
    #[serde(default)]
    theme_config: ThemeConfig,
    #[serde(default)]
    locale_config: LocaleConfig,
    #[serde(default)]
    window_config: WindowConfig,
    #[serde(default)]
    update_config: UpdateConfig,
    #[serde(default)]
    subscriptions: Vec<Subscription>,
    #[serde(default)]
    active_subscription_index: Option<i64>,
    #[serde(default)]
    active_config_path: Option<String>,
    #[serde(default)]
    active_config_content: Option<String>,
}

impl Default for BackupSnapshot {
    fn default() -> Self {
        Self {
            format_version: SNAPSHOT_FORMAT_VERSION,
            created_at: 0,
            app_config: AppConfig::default(),
            theme_config: ThemeConfig::default(),
            locale_config: LocaleConfig::default(),
            window_config: WindowConfig::default(),
            update_config: UpdateConfig::default(),
            subscriptions: Vec::new(),
            active_subscription_index: None,
            active_config_path: None,
            active_config_content: None,
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupExportResult {
    pub file_path: String,
    pub created_at: u64,
    pub subscriptions_count: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct BackupImportResult {
    pub file_path: String,
    pub valid: bool,
    pub restored: bool,
    pub subscriptions_count: usize,
    pub warnings: Vec<String>,
}

fn now_millis() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn with_json_extension(mut path: PathBuf) -> PathBuf {
    if path.extension().is_none() {
        path.set_extension("json");
    }
    path
}

fn resolve_export_path(file_path: Option<String>) -> Result<PathBuf, String> {
    if let Some(path) = file_path {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Ok(with_json_extension(PathBuf::from(trimmed)));
        }
    }

    let default_name = format!("sing-box-windows-backup-{}.json", now_millis());
    FileDialog::new()
        .set_title("导出备份")
        .set_file_name(&default_name)
        .add_filter("JSON", &["json"])
        .save_file()
        .map(with_json_extension)
        .ok_or_else(|| "已取消导出备份".to_string())
}

fn resolve_import_path(file_path: Option<String>) -> Result<PathBuf, String> {
    if let Some(path) = file_path {
        let trimmed = path.trim();
        if !trimmed.is_empty() {
            return Ok(PathBuf::from(trimmed));
        }
    }

    FileDialog::new()
        .set_title("导入备份")
        .add_filter("JSON", &["json"])
        .pick_file()
        .ok_or_else(|| "已取消导入备份".to_string())
}

fn default_active_config_path() -> PathBuf {
    paths::get_config_dir().join("config.json")
}

fn default_configs_dir() -> PathBuf {
    paths::get_config_dir().join(SNAPSHOT_CONFIGS_DIR)
}

fn sanitize_file_name(raw: &str, default_name: &str) -> String {
    let mut sanitized: String = raw
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '.' || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect();

    if sanitized.is_empty() || sanitized == "." || sanitized == ".." {
        sanitized = default_name.to_string();
    }

    sanitized
}

fn path_to_snapshot_string(path: &Path) -> String {
    let mut segments = Vec::new();
    for component in path.components() {
        if let Component::Normal(seg) = component {
            segments.push(seg.to_string_lossy().to_string());
        }
    }
    segments.join("/")
}

fn normalize_relative_snapshot_path(path: &Path) -> Option<String> {
    let mut segments = Vec::new();
    for component in path.components() {
        match component {
            Component::Normal(seg) => segments.push(seg.to_string_lossy().to_string()),
            Component::CurDir => {}
            Component::ParentDir => return None,
            Component::RootDir | Component::Prefix(_) => return None,
        }
    }
    if segments.is_empty() {
        None
    } else {
        Some(segments.join("/"))
    }
}

fn snapshot_relative_to_path_buf(path: &str) -> PathBuf {
    let mut buf = PathBuf::new();
    for segment in path.split('/') {
        if !segment.is_empty() {
            buf.push(segment);
        }
    }
    buf
}

fn relative_path_from_kind(path: &Path, kind: SnapshotPathKind) -> String {
    let default_name = match kind {
        SnapshotPathKind::ActiveConfig => "config.json",
        SnapshotPathKind::SubscriptionConfig => "subscription.json",
        SnapshotPathKind::SubscriptionBackup => "subscription.bak",
    };
    let file_name = path
        .file_name()
        .map(|v| v.to_string_lossy().to_string())
        .unwrap_or_else(|| default_name.to_string());
    let safe_file_name = sanitize_file_name(&file_name, default_name);

    match kind {
        SnapshotPathKind::ActiveConfig => {
            if safe_file_name.eq_ignore_ascii_case("config.json") {
                "config.json".to_string()
            } else {
                format!("{}/{}", SNAPSHOT_CONFIGS_DIR, safe_file_name)
            }
        }
        SnapshotPathKind::SubscriptionConfig | SnapshotPathKind::SubscriptionBackup => {
            format!("{}/{}", SNAPSHOT_CONFIGS_DIR, safe_file_name)
        }
    }
}

fn enforce_snapshot_relative_policy(path: &str, kind: SnapshotPathKind) -> String {
    let trimmed = path.trim().replace('\\', "/");
    let candidate = PathBuf::from(trimmed);
    let normalized = normalize_relative_snapshot_path(&candidate);
    match normalized {
        Some(p) => match kind {
            SnapshotPathKind::ActiveConfig => {
                if p.eq_ignore_ascii_case("config.json") || p.starts_with("configs/") {
                    p
                } else {
                    relative_path_from_kind(Path::new(&p), kind)
                }
            }
            SnapshotPathKind::SubscriptionConfig | SnapshotPathKind::SubscriptionBackup => {
                if p.starts_with("configs/") {
                    p
                } else {
                    relative_path_from_kind(Path::new(&p), kind)
                }
            }
        },
        None => relative_path_from_kind(Path::new(path), kind),
    }
}

fn encode_path_for_snapshot(raw_path: &str, kind: SnapshotPathKind) -> String {
    let candidate = PathBuf::from(raw_path.trim());
    if candidate.is_absolute() {
        let root = paths::get_config_dir();
        if let Ok(relative) = candidate.strip_prefix(&root) {
            let rel = path_to_snapshot_string(relative);
            return enforce_snapshot_relative_policy(&rel, kind);
        }
        return relative_path_from_kind(&candidate, kind);
    }

    if let Some(normalized) = normalize_relative_snapshot_path(&candidate) {
        return enforce_snapshot_relative_policy(&normalized, kind);
    }

    relative_path_from_kind(&candidate, kind)
}

#[derive(Default)]
struct PathRewriteStats {
    absolute_rewrites: usize,
    policy_rewrites: usize,
    subscription_path_rewrites: usize,
    backup_path_rewrites: usize,
    active_path_rewritten: bool,
}

fn decode_snapshot_path_to_local(
    raw_path: &str,
    kind: SnapshotPathKind,
    stats: &mut PathRewriteStats,
) -> PathBuf {
    let trimmed = raw_path.trim();
    if trimmed.is_empty() {
        stats.policy_rewrites += 1;
        return match kind {
            SnapshotPathKind::ActiveConfig => default_active_config_path(),
            SnapshotPathKind::SubscriptionConfig | SnapshotPathKind::SubscriptionBackup => {
                default_configs_dir().join("subscription.json")
            }
        };
    }

    let candidate = PathBuf::from(trimmed);
    let mut should_count_policy_rewrite = false;
    let relative = if candidate.is_absolute() {
        stats.absolute_rewrites += 1;
        should_count_policy_rewrite = true;
        let root = paths::get_config_dir();
        match candidate.strip_prefix(&root) {
            Ok(relative) => {
                enforce_snapshot_relative_policy(&path_to_snapshot_string(relative), kind)
            }
            Err(_) => relative_path_from_kind(&candidate, kind),
        }
    } else if let Some(normalized) = normalize_relative_snapshot_path(&candidate) {
        let enforced = enforce_snapshot_relative_policy(&normalized, kind);
        if normalized != enforced {
            should_count_policy_rewrite = true;
        }
        enforced
    } else {
        should_count_policy_rewrite = true;
        relative_path_from_kind(&candidate, kind)
    };

    if should_count_policy_rewrite {
        stats.policy_rewrites += 1;
    }

    let local_path = paths::get_config_dir().join(snapshot_relative_to_path_buf(&relative));

    match kind {
        SnapshotPathKind::ActiveConfig => {
            if should_count_policy_rewrite || candidate.is_absolute() {
                stats.active_path_rewritten = true;
            }
        }
        SnapshotPathKind::SubscriptionConfig => {
            if should_count_policy_rewrite || candidate.is_absolute() {
                stats.subscription_path_rewrites += 1;
            }
        }
        SnapshotPathKind::SubscriptionBackup => {
            if should_count_policy_rewrite || candidate.is_absolute() {
                stats.backup_path_rewrites += 1;
            }
        }
    }

    local_path
}

fn rewrite_paths_for_snapshot(
    snapshot: &BackupSnapshot,
) -> (AppConfig, Vec<Subscription>, PathRewriteStats) {
    let mut stats = PathRewriteStats::default();
    let mut app_config = snapshot.app_config.clone();
    let active_source = app_config
        .active_config_path
        .clone()
        .or_else(|| snapshot.active_config_path.clone());
    let active_local = match active_source {
        Some(path) => {
            decode_snapshot_path_to_local(&path, SnapshotPathKind::ActiveConfig, &mut stats)
        }
        None => {
            stats.policy_rewrites += 1;
            stats.active_path_rewritten = true;
            default_active_config_path()
        }
    };
    app_config.active_config_path = Some(active_local.to_string_lossy().to_string());

    let mut subscriptions = snapshot.subscriptions.clone();
    for sub in subscriptions.iter_mut() {
        if let Some(path) = sub.config_path.clone() {
            let local = decode_snapshot_path_to_local(
                &path,
                SnapshotPathKind::SubscriptionConfig,
                &mut stats,
            );
            sub.config_path = Some(local.to_string_lossy().to_string());
        }

        if let Some(path) = sub.backup_path.clone() {
            let local = decode_snapshot_path_to_local(
                &path,
                SnapshotPathKind::SubscriptionBackup,
                &mut stats,
            );
            sub.backup_path = Some(local.to_string_lossy().to_string());
        }
    }

    (app_config, subscriptions, stats)
}

fn rewrite_stats_warnings(stats: &PathRewriteStats, dry_run: bool) -> Vec<String> {
    let mut warnings = Vec::new();

    if stats.absolute_rewrites > 0 {
        warnings.push(format!(
            "{}检测到 {} 个绝对路径，已重定位到当前机器目录",
            if dry_run { "预检：" } else { "" },
            stats.absolute_rewrites
        ));
    }
    if stats.subscription_path_rewrites > 0 {
        warnings.push(format!(
            "{}重写了 {} 条订阅配置路径",
            if dry_run { "预检：" } else { "" },
            stats.subscription_path_rewrites
        ));
    }
    if stats.backup_path_rewrites > 0 {
        warnings.push(format!(
            "{}重写了 {} 条订阅备份路径",
            if dry_run { "预检：" } else { "" },
            stats.backup_path_rewrites
        ));
    }
    if stats.active_path_rewritten {
        warnings.push(format!(
            "{}活动配置路径将使用当前机器本地路径",
            if dry_run { "预检：" } else { "" }
        ));
    }

    warnings
}

fn resolve_active_config_file_path(active_config_path: Option<&str>) -> PathBuf {
    match active_config_path {
        Some(path) => {
            let candidate = PathBuf::from(path);
            if candidate.is_absolute() {
                candidate
            } else {
                paths::get_config_dir().join(candidate)
            }
        }
        None => default_active_config_path(),
    }
}

fn write_config_content(path: &Path, content: &str) -> Result<(), String> {
    ensure_parent_dir(path)?;
    std::fs::write(path, content).map_err(|e| format!("恢复活动配置文件失败: {}", e))
}

async fn build_snapshot(app: &tauri::AppHandle) -> Result<BackupSnapshot, String> {
    let storage = get_enhanced_storage(app).await?;
    let app_config = storage.get_app_config().await.map_err(|e| e.to_string())?;
    let theme_config = storage
        .get_theme_config()
        .await
        .map_err(|e| e.to_string())?;
    let locale_config = storage
        .get_locale_config()
        .await
        .map_err(|e| e.to_string())?;
    let window_config = storage
        .get_window_config()
        .await
        .map_err(|e| e.to_string())?;
    let update_config = storage
        .get_update_config()
        .await
        .map_err(|e| e.to_string())?;
    let subscriptions = storage
        .get_subscriptions()
        .await
        .map_err(|e| e.to_string())?;
    let active_subscription_index = storage
        .get_active_subscription_index()
        .await
        .map_err(|e| e.to_string())?;

    let active_config_file =
        resolve_active_config_file_path(app_config.active_config_path.as_deref());
    let active_config_content = std::fs::read_to_string(&active_config_file).ok();

    let mut snapshot_app_config = app_config.clone();
    snapshot_app_config.active_config_path = app_config
        .active_config_path
        .as_deref()
        .map(|path| encode_path_for_snapshot(path, SnapshotPathKind::ActiveConfig));

    let mut snapshot_subscriptions = subscriptions.clone();
    for sub in snapshot_subscriptions.iter_mut() {
        sub.config_path = sub
            .config_path
            .as_deref()
            .map(|path| encode_path_for_snapshot(path, SnapshotPathKind::SubscriptionConfig));
        sub.backup_path = sub
            .backup_path
            .as_deref()
            .map(|path| encode_path_for_snapshot(path, SnapshotPathKind::SubscriptionBackup));
    }

    Ok(BackupSnapshot {
        format_version: SNAPSHOT_FORMAT_VERSION,
        created_at: now_millis(),
        app_config: snapshot_app_config.clone(),
        theme_config,
        locale_config,
        window_config,
        update_config,
        subscriptions: snapshot_subscriptions,
        active_subscription_index,
        active_config_path: snapshot_app_config.active_config_path,
        active_config_content,
    })
}

fn parse_snapshot(content: &str) -> Result<BackupSnapshot, String> {
    let snapshot: BackupSnapshot =
        serde_json::from_str(content).map_err(|e| format!("解析备份文件失败: {}", e))?;

    if snapshot.format_version == 0 {
        return Err("备份文件缺少 format_version，无法校验兼容性".to_string());
    }
    if snapshot.format_version > SNAPSHOT_FORMAT_VERSION {
        return Err(format!(
            "备份版本 {} 高于当前支持版本 {}",
            snapshot.format_version, SNAPSHOT_FORMAT_VERSION
        ));
    }

    Ok(snapshot)
}

fn ensure_parent_dir(path: &Path) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| format!("创建目录失败: {}", e))?;
    }
    Ok(())
}

async fn apply_snapshot(
    app: &tauri::AppHandle,
    snapshot: &BackupSnapshot,
) -> Result<Vec<String>, String> {
    let mut warnings = Vec::new();
    if snapshot.app_config.active_config_path.is_none() && snapshot.active_config_path.is_none() {
        warnings.push("备份中缺少 active_config_path，已回退为默认 config.json".to_string());
    }
    let storage = get_enhanced_storage(app).await?;
    let (mut app_config, rewritten_subscriptions, rewrite_stats) =
        rewrite_paths_for_snapshot(snapshot);
    warnings.extend(rewrite_stats_warnings(&rewrite_stats, false));

    let mut update_config = snapshot.update_config.clone();
    if update_config.update_channel.is_none() {
        update_config.update_channel = Some("stable".to_string());
        warnings.push("备份中缺少 update_channel，已回退为 stable".to_string());
    }

    let mut config_path = app_config
        .active_config_path
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(default_active_config_path);

    if let Some(content) = snapshot.active_config_content.clone() {
        if let Err(primary_err) = write_config_content(&config_path, &content) {
            let fallback_path = default_active_config_path();
            if fallback_path != config_path {
                warnings.push(format!(
                    "备份中的 active_config_path 不可写，已回退默认 config.json: {}",
                    primary_err
                ));
                warn!(
                    "备份恢复时写入活动配置失败，尝试回退到默认路径 {:?}: {}",
                    fallback_path, primary_err
                );
                write_config_content(&fallback_path, &content)?;
                config_path = fallback_path;
            } else {
                return Err(primary_err);
            }
        }
        // 恢复后确保 app_config 与最终可写入的活动配置路径一致。
        app_config.active_config_path = Some(config_path.to_string_lossy().to_string());
    } else {
        warnings.push("备份中不包含 active_config_content，已跳过配置文件回写".to_string());
    }

    storage
        .save_app_config(&app_config)
        .await
        .map_err(|e| format!("恢复应用配置失败: {}", e))?;
    storage
        .save_theme_config(&snapshot.theme_config)
        .await
        .map_err(|e| format!("恢复主题配置失败: {}", e))?;
    storage
        .save_locale_config(&snapshot.locale_config)
        .await
        .map_err(|e| format!("恢复语言配置失败: {}", e))?;
    storage
        .save_window_config(&snapshot.window_config)
        .await
        .map_err(|e| format!("恢复窗口配置失败: {}", e))?;
    storage
        .save_update_config(&update_config)
        .await
        .map_err(|e| format!("恢复更新配置失败: {}", e))?;
    storage
        .save_subscriptions(&rewritten_subscriptions)
        .await
        .map_err(|e| format!("恢复订阅列表失败: {}", e))?;
    storage
        .save_active_subscription_index(snapshot.active_subscription_index)
        .await
        .map_err(|e| format!("恢复激活订阅索引失败: {}", e))?;

    // 恢复后按最新配置尝试同步运行态（不会强制重启内核）。
    auto_manage_with_saved_config(app, false, "backup-restore").await;

    Ok(warnings)
}

#[tauri::command]
pub async fn backup_export_snapshot(
    app: tauri::AppHandle,
    file_path: Option<String>,
) -> Result<BackupExportResult, String> {
    let snapshot = build_snapshot(&app).await?;
    let target_path = resolve_export_path(file_path)?;

    ensure_parent_dir(&target_path)?;
    let serialized =
        serde_json::to_string_pretty(&snapshot).map_err(|e| format!("序列化备份失败: {}", e))?;
    std::fs::write(&target_path, serialized).map_err(|e| format!("写入备份文件失败: {}", e))?;

    info!("备份导出成功: {:?}", target_path);
    Ok(BackupExportResult {
        file_path: target_path.to_string_lossy().to_string(),
        created_at: snapshot.created_at,
        subscriptions_count: snapshot.subscriptions.len(),
    })
}

#[tauri::command]
pub async fn backup_import_snapshot(
    app: tauri::AppHandle,
    file_path: Option<String>,
    dry_run: Option<bool>,
) -> Result<BackupImportResult, String> {
    let source_path = resolve_import_path(file_path)?;
    let content =
        std::fs::read_to_string(&source_path).map_err(|e| format!("读取备份文件失败: {}", e))?;
    let snapshot = parse_snapshot(&content)?;
    let is_dry_run = dry_run.unwrap_or(false);

    let mut warnings = Vec::new();
    if snapshot.format_version < SNAPSHOT_FORMAT_VERSION {
        warnings.push(format!(
            "检测到旧版备份格式 v{}，导入时将自动迁移路径",
            snapshot.format_version
        ));
    }
    if snapshot.created_at == 0 {
        warnings.push("备份文件缺少 created_at，可能来自旧版本".to_string());
    }
    if snapshot.subscriptions.is_empty() {
        warnings.push("备份中没有订阅记录".to_string());
    }

    if is_dry_run {
        if snapshot.app_config.active_config_path.is_none() && snapshot.active_config_path.is_none()
        {
            warnings
                .push("预检：备份中缺少 active_config_path，将回退为默认 config.json".to_string());
        }
        let (_, _, rewrite_stats) = rewrite_paths_for_snapshot(&snapshot);
        warnings.extend(rewrite_stats_warnings(&rewrite_stats, true));
        return Ok(BackupImportResult {
            file_path: source_path.to_string_lossy().to_string(),
            valid: true,
            restored: false,
            subscriptions_count: snapshot.subscriptions.len(),
            warnings,
        });
    }

    let apply_warnings = apply_snapshot(&app, &snapshot).await?;
    warnings.extend(apply_warnings);

    warn!("已从备份恢复应用状态: {:?}", source_path);
    Ok(BackupImportResult {
        file_path: source_path.to_string_lossy().to_string(),
        valid: true,
        restored: true,
        subscriptions_count: snapshot.subscriptions.len(),
        warnings,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::storage::state_model::Subscription;

    fn build_subscription(path: &str) -> Subscription {
        Subscription {
            name: "test-sub".to_string(),
            url: "https://example.com/sub".to_string(),
            is_loading: false,
            last_update: None,
            is_manual: false,
            manual_content: None,
            use_original_config: false,
            config_path: Some(path.to_string()),
            backup_path: None,
            auto_update_interval_minutes: Some(720),
            subscription_upload: None,
            subscription_download: None,
            subscription_total: None,
            subscription_expire: None,
            auto_update_fail_count: None,
            last_auto_update_attempt: None,
            last_auto_update_error: None,
            last_auto_update_error_type: None,
            last_auto_update_backoff_until: None,
        }
    }

    fn legacy_absolute_path(file_name: &str) -> String {
        #[cfg(target_os = "windows")]
        {
            format!(
                "C:\\Users\\legacy-user\\AppData\\Local\\sing-box-windows\\sing-box\\configs\\{}",
                file_name
            )
        }

        #[cfg(not(target_os = "windows"))]
        {
            format!(
                "/tmp/legacy-user/sing-box-windows/sing-box/configs/{}",
                file_name
            )
        }
    }

    #[test]
    fn encode_absolute_path_for_snapshot_should_return_relative_path() {
        let local_abs = paths::get_config_dir().join("configs").join("sample.json");
        let encoded = encode_path_for_snapshot(
            &local_abs.to_string_lossy(),
            SnapshotPathKind::SubscriptionConfig,
        );
        assert_eq!(encoded, "configs/sample.json");
    }

    #[test]
    fn rewrite_paths_for_snapshot_should_migrate_legacy_absolute_paths() {
        let mut snapshot = BackupSnapshot::default();
        snapshot.format_version = 1;
        snapshot.app_config.active_config_path = Some(legacy_absolute_path("active.json"));
        snapshot.subscriptions = vec![build_subscription(&legacy_absolute_path("sub.json"))];

        let (app_config, subscriptions, stats) = rewrite_paths_for_snapshot(&snapshot);
        let active_path = PathBuf::from(app_config.active_config_path.unwrap_or_default());
        let sub_path = PathBuf::from(subscriptions[0].config_path.clone().unwrap_or_default());

        assert!(active_path.starts_with(paths::get_config_dir()));
        assert!(sub_path.starts_with(paths::get_config_dir()));
        assert!(stats.absolute_rewrites >= 2);
        assert!(stats.active_path_rewritten);
    }
}
