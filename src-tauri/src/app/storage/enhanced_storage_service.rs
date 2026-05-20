use super::DatabaseService;
use crate::app::core::kernel_auto_manage::auto_manage_with_saved_config;
use crate::app::core::tun_profile::normalize_tun_route_exclude_address;
use crate::app::storage::error::StorageResult;
use crate::app::storage::state_model::{
    AppConfig, LocaleConfig, StartupPreferences, Subscription, ThemeConfig, UpdateConfig,
    WindowConfig,
};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::OnceCell;

const STARTUP_PREFERENCES_FILE: &str = "startup_preferences.json";

/// 将全局设置同步到指定的配置文件
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConfigPatchMode {
    Full,
    PortsOnly,
}

fn resolve_patch_mode_for_subscription(subscription: Option<&Subscription>) -> ConfigPatchMode {
    match subscription {
        Some(sub) if sub.use_original_config => ConfigPatchMode::PortsOnly,
        _ => ConfigPatchMode::Full,
    }
}

fn resolve_patch_mode_with_hint(
    subscription: Option<&Subscription>,
    use_original_config_hint: Option<bool>,
) -> ConfigPatchMode {
    match use_original_config_hint {
        // 显式提示来自当前前端交互，优先级高于数据库里的历史记录，可避免异步落库时的误判。
        Some(true) => ConfigPatchMode::PortsOnly,
        Some(false) => ConfigPatchMode::Full,
        None => resolve_patch_mode_for_subscription(subscription),
    }
}

fn sync_settings_to_config_file(
    config_path: &std::path::Path,
    app_config: &AppConfig,
    patch_mode: ConfigPatchMode,
) -> Result<(), String> {
    use crate::app::singbox::settings_patch::{
        apply_app_settings_to_config, apply_port_settings_only,
    };

    // 读取现有配置
    let content =
        std::fs::read_to_string(config_path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 解析 JSON
    let mut config: serde_json::Value =
        serde_json::from_str(&content).map_err(|e| format!("解析配置文件失败: {}", e))?;

    // 应用全局设置
    match patch_mode {
        ConfigPatchMode::Full => apply_app_settings_to_config(&mut config, app_config),
        ConfigPatchMode::PortsOnly => apply_port_settings_only(&mut config, app_config),
    }

    // 写回文件
    let updated =
        serde_json::to_string_pretty(&config).map_err(|e| format!("序列化配置失败: {}", e))?;
    std::fs::write(config_path, updated).map_err(|e| format!("写入配置文件失败: {}", e))?;

    Ok(())
}

async fn resolve_patch_mode_for_active_config(
    app: &AppHandle,
    active_config_path: &str,
    use_original_config_hint: Option<bool>,
) -> ConfigPatchMode {
    if use_original_config_hint.is_some() {
        return resolve_patch_mode_with_hint(None, use_original_config_hint);
    }

    let storage = match get_enhanced_storage(app).await {
        Ok(storage) => storage,
        Err(error) => {
            tracing::warn!("读取数据库服务失败，回退使用 Full patch: {}", error);
            return ConfigPatchMode::Full;
        }
    };

    match storage.get_subscriptions().await {
        Ok(subscriptions) => resolve_patch_mode_for_subscription(
            subscriptions
                .iter()
                .find(|sub| sub.config_path.as_deref() == Some(active_config_path)),
        ),
        Err(error) => {
            tracing::warn!("读取订阅列表失败，回退使用 Full patch: {}", error);
            ConfigPatchMode::Full
        }
    }
}

pub(crate) async fn apply_runtime_config_update(
    app: &AppHandle,
    effective_config: &AppConfig,
    use_original_config_hint: Option<bool>,
    force_restart: bool,
    reason: &'static str,
) {
    if let Some(path) = effective_config.active_config_path.as_deref() {
        let config_path = std::path::PathBuf::from(path);
        if config_path.exists() {
            let patch_mode =
                resolve_patch_mode_for_active_config(app, path, use_original_config_hint).await;
            if let Err(error) =
                sync_settings_to_config_file(&config_path, effective_config, patch_mode)
            {
                tracing::warn!("同步活动配置文件失败: {}", error);
            }
        }
    }

    auto_manage_with_saved_config(app, force_restart, reason).await;
}

/// 获取数据库服务的辅助函数（单例初始化）
pub async fn get_enhanced_storage<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<Arc<EnhancedStorageService>, String> {
    let cell_state = app.state::<Arc<OnceCell<Arc<EnhancedStorageService>>>>();
    let cell = Arc::clone(&*cell_state);

    cell.get_or_try_init(|| async {
        tracing::info!("?? 初始化新的数据库服务...");
        EnhancedStorageService::new(app).await.map(Arc::new)
    })
    .await
    .map(|svc| {
        tracing::info!("? 使用已初始化的数据库服务");
        svc.clone()
    })
    .map_err(|e| {
        tracing::error!("? 数据库服务初始化失败: {}", e);
        format!("Failed to initialize enhanced storage: {}", e)
    })
}

/// 增强版存储服务 - 使用 SQLite 数据库
#[derive(Debug, Clone)]
pub struct EnhancedStorageService {
    database: Arc<DatabaseService>,
}

impl EnhancedStorageService {
    pub async fn new<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> StorageResult<Self> {
        let app_data_dir = resolve_app_data_dir(app_handle);

        // 确保目录存在
        std::fs::create_dir_all(&app_data_dir)?;

        let database_path = app_data_dir.join("app_data.db");
        let database = Arc::new(DatabaseService::new(database_path.to_str().unwrap()).await?);

        Ok(Self { database })
    }

    // 应用配置
    pub async fn get_app_config(&self) -> StorageResult<AppConfig> {
        match self.database.load_app_config().await? {
            Some(config) => Ok(config),
            None => Ok(AppConfig::default()),
        }
    }

    pub async fn save_app_config(&self, config: &AppConfig) -> StorageResult<()> {
        self.database.save_app_config(config).await
    }

    // 主题配置
    pub async fn get_theme_config(&self) -> StorageResult<ThemeConfig> {
        match self.database.load_theme_config().await? {
            Some(config) => Ok(config),
            None => Ok(ThemeConfig::default()),
        }
    }

    pub async fn save_theme_config(&self, config: &ThemeConfig) -> StorageResult<()> {
        self.database.save_theme_config(config).await
    }

    // 语言配置
    pub async fn get_locale_config(&self) -> StorageResult<LocaleConfig> {
        match self.database.load_locale_config().await? {
            Some(config) => Ok(config),
            None => Ok(LocaleConfig::default()),
        }
    }

    pub async fn save_locale_config(&self, config: &LocaleConfig) -> StorageResult<()> {
        self.database.save_locale_config(config).await
    }

    // 窗口配置
    pub async fn get_window_config(&self) -> StorageResult<WindowConfig> {
        match self.database.load_window_config().await? {
            Some(config) => Ok(config),
            None => Ok(WindowConfig::default()),
        }
    }

    pub async fn save_window_config(&self, config: &WindowConfig) -> StorageResult<()> {
        self.database.save_window_config(config).await
    }

    // 更新配置
    pub async fn get_update_config(&self) -> StorageResult<UpdateConfig> {
        match self.database.load_update_config().await? {
            Some(config) => Ok(config),
            None => Ok(UpdateConfig::default()),
        }
    }

    pub async fn save_update_config(&self, config: &UpdateConfig) -> StorageResult<()> {
        self.database.save_update_config(config).await
    }

    // 订阅管理
    pub async fn get_subscriptions(&self) -> StorageResult<Vec<Subscription>> {
        match self
            .database
            .load_config::<Vec<Subscription>>("subscriptions")
            .await?
        {
            Some(subscriptions) => Ok(subscriptions),
            None => Ok(Vec::new()),
        }
    }

    pub async fn save_subscriptions(&self, subscriptions: &[Subscription]) -> StorageResult<()> {
        self.database
            .save_config("subscriptions", &subscriptions)
            .await
    }

    // 激活订阅索引
    pub async fn get_active_subscription_index(&self) -> StorageResult<Option<i64>> {
        match self
            .database
            .load_config::<i64>("active_subscription_index")
            .await?
        {
            Some(index) => Ok(Some(index)),
            None => Ok(None),
        }
    }

    pub async fn save_active_subscription_index(&self, index: Option<i64>) -> StorageResult<()> {
        if let Some(idx) = index {
            self.database
                .save_config("active_subscription_index", &idx)
                .await
        } else {
            self.database
                .remove_config("active_subscription_index")
                .await
        }
    }

    // 通用配置
    pub async fn get_config<T>(&self, key: &str) -> StorageResult<Option<T>>
    where
        T: for<'de> serde::Deserialize<'de>,
    {
        self.database.load_config(key).await
    }

    pub async fn save_config<T>(&self, key: &str, value: &T) -> StorageResult<()>
    where
        T: serde::Serialize,
    {
        self.database.save_config(key, value).await
    }

    pub async fn remove_config(&self, key: &str) -> StorageResult<()> {
        self.database.remove_config(key).await
    }
}

fn resolve_app_data_dir<R: tauri::Runtime>(app_handle: &AppHandle<R>) -> std::path::PathBuf {
    app_handle
        .path()
        .app_data_dir()
        .unwrap_or_else(|_| std::env::current_dir().unwrap())
}

fn resolve_startup_preferences_path<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
) -> std::path::PathBuf {
    resolve_app_data_dir(app_handle).join(STARTUP_PREFERENCES_FILE)
}

fn build_startup_preferences(config: &AppConfig) -> StartupPreferences {
    StartupPreferences {
        auto_start_app: config.auto_start_app,
        auto_hide_to_tray_on_autostart: config.auto_hide_to_tray_on_autostart,
        tray_close_behavior: config.tray_close_behavior.clone(),
    }
}

fn normalize_app_config_for_persistence(mut config: AppConfig) -> Result<AppConfig, String> {
    config.tun_route_exclude_address =
        normalize_tun_route_exclude_address(config.tun_route_exclude_address)?;
    Ok(config)
}

pub fn read_startup_preferences_sync<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
) -> StartupPreferences {
    let path = resolve_startup_preferences_path(app_handle);
    let content = match std::fs::read_to_string(&path) {
        Ok(content) => content,
        Err(error) => {
            tracing::debug!("读取启动偏好失败，使用默认值: {}", error);
            return StartupPreferences::default();
        }
    };

    serde_json::from_str(&content).unwrap_or_else(|error| {
        tracing::warn!("解析启动偏好失败，使用默认值: {}", error);
        StartupPreferences::default()
    })
}

pub fn save_startup_preferences_sync<R: tauri::Runtime>(
    app_handle: &AppHandle<R>,
    config: &AppConfig,
) -> Result<(), String> {
    let app_data_dir = resolve_app_data_dir(app_handle);
    std::fs::create_dir_all(&app_data_dir).map_err(|e| format!("创建应用数据目录失败: {}", e))?;

    let path = resolve_startup_preferences_path(app_handle);
    let payload = build_startup_preferences(config);
    let content =
        serde_json::to_string_pretty(&payload).map_err(|e| format!("序列化启动偏好失败: {}", e))?;
    std::fs::write(&path, content).map_err(|e| format!("写入启动偏好失败: {}", e))
}

// Tauri 命令实现
#[tauri::command]
pub async fn db_get_app_config(app: AppHandle) -> Result<AppConfig, String> {
    db_get_app_config_internal(&app).await
}

pub async fn db_get_app_config_internal<R: tauri::Runtime>(
    app: &AppHandle<R>,
) -> Result<AppConfig, String> {
    let storage = get_enhanced_storage(app).await?;
    #[allow(unused_mut)]
    let mut config = storage.get_app_config().await.map_err(|e| e.to_string())?;

    // Windows：非管理员启动时自动关闭 TUN，避免因缺少权限导致内核无法拉起
    // Linux/macOS：内核可通过 sudo 提权启动（应用本身无需 root），因此不在这里强制关闭。
    #[cfg(target_os = "windows")]
    if config.tun_enabled && !crate::app::system::system_service::check_admin() {
        let previous_mode = config.proxy_mode.clone();
        config.tun_enabled = false;
        config.proxy_mode = if config.system_proxy_enabled {
            "system".to_string()
        } else {
            "manual".to_string()
        };

        if let Err(err) = storage.save_app_config(&config).await {
            tracing::warn!("在非管理员模式下写入关闭 TUN 设置失败: {}", err);
        } else {
            tracing::info!(
                "检测到当前未获得管理员权限，已自动关闭 TUN 模式（原模式: {}）",
                previous_mode
            );
        }
    }

    Ok(config)
}

pub async fn db_save_app_config_internal<R: tauri::Runtime>(
    config: AppConfig,
    app: &AppHandle<R>,
) -> Result<(), String> {
    let config = normalize_app_config_for_persistence(config)?;
    let storage = get_enhanced_storage(app).await?;
    storage
        .save_app_config(&config)
        .await
        .map_err(|e| e.to_string())?;
    save_startup_preferences_sync(app, &config)?;
    Ok(())
}

#[tauri::command]
pub async fn db_save_app_config(
    config: AppConfig,
    app: AppHandle,
    apply_runtime: Option<bool>,
) -> Result<(), String> {
    db_save_app_config_internal(config, &app).await?;
    let apply_runtime = apply_runtime.unwrap_or(false);

    // 默认仅持久化，不自动触发运行态变更。
    // 这样可避免前端自动保存（输入框逐字变化）频繁触发内核重启/配置同步。
    if !apply_runtime {
        return Ok(());
    }

    let effective_config = db_get_app_config_internal(&app).await?;
    // 保存设置后，尽量把变更同步到“当前生效配置文件”，避免用户需要重新下载订阅/重启应用才能生效。
    // 同步逻辑采用“局部 patch”策略：如果配置文件不是本程序生成的结构，会尽量只修改端口/TUN/DNS 策略等通用字段。
    apply_runtime_config_update(&app, &effective_config, None, false, "app-config-updated").await;

    Ok(())
}

#[tauri::command]
pub async fn db_get_theme_config(app: AppHandle) -> Result<ThemeConfig, String> {
    let storage = get_enhanced_storage(&app).await?;
    storage.get_theme_config().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_save_theme_config(config: ThemeConfig, app: AppHandle) -> Result<(), String> {
    let storage = get_enhanced_storage(&app).await?;
    storage
        .save_theme_config(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_get_locale_config(app: AppHandle) -> Result<LocaleConfig, String> {
    let storage = get_enhanced_storage(&app).await?;
    storage.get_locale_config().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_save_locale_config(config: LocaleConfig, app: AppHandle) -> Result<(), String> {
    let storage = get_enhanced_storage(&app).await?;
    storage
        .save_locale_config(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_get_window_config(app: AppHandle) -> Result<WindowConfig, String> {
    let storage = get_enhanced_storage(&app).await?;
    storage.get_window_config().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_save_window_config(config: WindowConfig, app: AppHandle) -> Result<(), String> {
    let storage = get_enhanced_storage(&app).await?;
    storage
        .save_window_config(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_get_update_config(app: AppHandle) -> Result<UpdateConfig, String> {
    let storage = get_enhanced_storage(&app).await?;
    storage.get_update_config().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_save_update_config(config: UpdateConfig, app: AppHandle) -> Result<(), String> {
    let storage = get_enhanced_storage(&app).await?;
    storage
        .save_update_config(&config)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_get_subscriptions(app: AppHandle) -> Result<Vec<Subscription>, String> {
    let storage = get_enhanced_storage(&app).await?;
    storage.get_subscriptions().await.map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_save_subscriptions(
    subscriptions: Vec<Subscription>,
    app: AppHandle,
) -> Result<(), String> {
    let storage = get_enhanced_storage(&app).await?;
    storage
        .save_subscriptions(&subscriptions)
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_get_active_subscription_index(app: AppHandle) -> Result<Option<i64>, String> {
    let storage = get_enhanced_storage(&app).await?;
    storage
        .get_active_subscription_index()
        .await
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn db_save_active_subscription_index(
    index: Option<i64>,
    app: AppHandle,
) -> Result<(), String> {
    let storage = get_enhanced_storage(&app).await?;

    // Save the subscription index
    storage
        .save_active_subscription_index(index)
        .await
        .map_err(|e| e.to_string())?;

    // 重要说明：
    // - active_config_path 是内核启动时读取的“真实生效配置路径”（来源：AppConfig）。
    // - active_subscription_index 仅用于前端高亮/记忆选择位置，使用“索引”在列表变动时很容易漂移。
    // 因此这里不再尝试用索引反向覆盖 active_config_path，避免出现“前端索引写入导致内核配置跳到别的订阅”的问题。
    //
    // 但为了保持原有能力：当用户切换订阅时，把全局设置（端口/TUN/系统代理等）同步到该订阅配置文件。
    let app_config = storage.get_app_config().await.map_err(|e| e.to_string())?;

    let (target_config_path, patch_mode) = if let Some(idx) = index {
        let subscriptions = storage
            .get_subscriptions()
            .await
            .map_err(|e| e.to_string())?;
        let subscription = subscriptions.get(idx as usize);
        (
            subscription
                .and_then(|sub| sub.config_path.clone())
                .map(std::path::PathBuf::from),
            resolve_patch_mode_with_hint(subscription, None),
        )
    } else {
        (None, ConfigPatchMode::Full)
    };

    if let Some(config_path) = target_config_path {
        if config_path.exists() {
            match sync_settings_to_config_file(&config_path, &app_config, patch_mode) {
                Ok(_) => {
                    tracing::info!("已将全局设置同步到配置文件: {:?}", config_path);
                }
                Err(e) => {
                    tracing::warn!("同步设置到配置文件失败: {}", e);
                }
            }
        } else {
            tracing::warn!(
                "订阅索引写入时发现配置文件不存在，跳过同步: {:?}",
                config_path
            );
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{
        normalize_app_config_for_persistence, resolve_patch_mode_for_subscription,
        resolve_patch_mode_with_hint, ConfigPatchMode,
    };
    use crate::app::storage::state_model::Subscription;

    fn build_subscription(use_original_config: bool) -> Subscription {
        Subscription {
            name: "test".to_string(),
            url: "https://example.com/sub".to_string(),
            is_loading: false,
            last_update: None,
            is_manual: false,
            manual_content: None,
            use_original_config,
            config_path: Some("/tmp/sub.json".to_string()),
            backup_path: None,
            auto_update_interval_minutes: Some(60),
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

    #[test]
    fn should_use_ports_only_for_original_subscription_without_hint() {
        let subscription = build_subscription(true);
        assert_eq!(
            resolve_patch_mode_for_subscription(Some(&subscription)),
            ConfigPatchMode::PortsOnly
        );
    }

    #[test]
    fn should_trust_explicit_original_hint_when_subscription_missing() {
        assert_eq!(
            resolve_patch_mode_with_hint(None, Some(true)),
            ConfigPatchMode::PortsOnly
        );
    }

    #[test]
    fn should_trust_explicit_non_original_hint_over_subscription_state() {
        let subscription = build_subscription(true);
        assert_eq!(
            resolve_patch_mode_with_hint(Some(&subscription), Some(false)),
            ConfigPatchMode::Full
        );
    }

    #[test]
    fn should_normalize_blank_tun_route_exclude_address_to_none() {
        let normalized =
            normalize_app_config_for_persistence(crate::app::storage::state_model::AppConfig {
                tun_route_exclude_address: Some(vec!["  ".to_string(), "".to_string()]),
                ..crate::app::storage::state_model::AppConfig::default()
            })
            .expect("blank route exclude address should normalize");

        assert_eq!(normalized.tun_route_exclude_address, None);
    }

    #[test]
    fn should_reject_invalid_tun_route_exclude_address_on_save() {
        let error =
            normalize_app_config_for_persistence(crate::app::storage::state_model::AppConfig {
                tun_route_exclude_address: Some(vec!["invalid".to_string()]),
                ..crate::app::storage::state_model::AppConfig::default()
            })
            .expect_err("invalid route exclude address should be rejected");

        assert!(
            error.contains("invalid"),
            "error should mention invalid CIDR, got: {}",
            error
        );
    }
}
