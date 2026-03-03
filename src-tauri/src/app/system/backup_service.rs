use crate::app::constants::paths;
use crate::app::core::kernel_auto_manage::auto_manage_with_saved_config;
use crate::app::storage::enhanced_storage_service::get_enhanced_storage;
use crate::app::storage::state_model::{
    AppConfig, LocaleConfig, Subscription, ThemeConfig, UpdateConfig, WindowConfig,
};
use rfd::FileDialog;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tracing::{info, warn};

const SNAPSHOT_FORMAT_VERSION: u32 = 1;

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

fn write_config_content(path: &Path, content: &str) -> Result<(), String> {
    ensure_parent_dir(path)?;
    std::fs::write(path, content).map_err(|e| format!("恢复活动配置文件失败: {}", e))
}

async fn build_snapshot(app: &tauri::AppHandle) -> Result<BackupSnapshot, String> {
    let storage = get_enhanced_storage(app).await?;
    let app_config = storage.get_app_config().await.map_err(|e| e.to_string())?;
    let theme_config = storage.get_theme_config().await.map_err(|e| e.to_string())?;
    let locale_config = storage.get_locale_config().await.map_err(|e| e.to_string())?;
    let window_config = storage.get_window_config().await.map_err(|e| e.to_string())?;
    let update_config = storage.get_update_config().await.map_err(|e| e.to_string())?;
    let subscriptions = storage.get_subscriptions().await.map_err(|e| e.to_string())?;
    let active_subscription_index = storage
        .get_active_subscription_index()
        .await
        .map_err(|e| e.to_string())?;

    let active_config_path = app_config.active_config_path.clone();
    let active_config_file = active_config_path
        .clone()
        .map(PathBuf::from)
        .unwrap_or_else(|| paths::get_config_dir().join("config.json"));
    let active_config_content = std::fs::read_to_string(&active_config_file).ok();

    Ok(BackupSnapshot {
        format_version: SNAPSHOT_FORMAT_VERSION,
        created_at: now_millis(),
        app_config,
        theme_config,
        locale_config,
        window_config,
        update_config,
        subscriptions,
        active_subscription_index,
        active_config_path,
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
    let storage = get_enhanced_storage(app).await?;

    let mut app_config = snapshot.app_config.clone();
    if app_config.active_config_path.is_none() {
        app_config.active_config_path = snapshot.active_config_path.clone();
    }
    if app_config.active_config_path.is_none() {
        app_config.active_config_path = Some(
            default_active_config_path()
                .to_string_lossy()
                .to_string(),
        );
        warnings.push("备份中缺少 active_config_path，已回退为默认 config.json".to_string());
    }

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
        warnings.push("备份中不包含 active_config_content，跳过配置文件回写".to_string());
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
        .save_subscriptions(&snapshot.subscriptions)
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
    let content = std::fs::read_to_string(&source_path)
        .map_err(|e| format!("读取备份文件失败: {}", e))?;
    let snapshot = parse_snapshot(&content)?;
    let is_dry_run = dry_run.unwrap_or(false);

    let mut warnings = Vec::new();
    if snapshot.created_at == 0 {
        warnings.push("备份文件缺少 created_at，可能来自旧版本".to_string());
    }
    if snapshot.subscriptions.is_empty() {
        warnings.push("备份中没有订阅记录".to_string());
    }

    if is_dry_run {
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
