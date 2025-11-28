use super::DatabaseService;
use crate::app::core::kernel_service::auto_manage_with_saved_config;
use crate::app::storage::error::StorageResult;
use crate::app::storage::state_model::{
    AppConfig, LocaleConfig, Subscription, ThemeConfig, UpdateConfig, WindowConfig,
};
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tokio::sync::OnceCell;

/// 获取数据库服务的辅助函数（单例初始化）
async fn get_enhanced_storage(app: &AppHandle) -> Result<Arc<EnhancedStorageService>, String> {
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
    pub async fn new(app_handle: &AppHandle) -> StorageResult<Self> {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap());

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
}

// Tauri 命令实现
#[tauri::command]
pub async fn db_get_app_config(app: AppHandle) -> Result<AppConfig, String> {
    let storage = get_enhanced_storage(&app).await?;
    let mut config = storage.get_app_config().await.map_err(|e| e.to_string())?;

    // 非管理员启动时自动关闭 TUN，避免因缺少权限导致内核无法拉起
    if config.tun_enabled && !crate::app::system::system_service::check_admin() {
        let previous_mode = config.proxy_mode.clone();
        config.tun_enabled = false;
        config.proxy_mode = if config.system_proxy_enabled {
            "system".to_string()
        } else {
            "manual".to_string()
        };

        if let Err(err) = storage.save_app_config(&config).await {
            tracing::warn!(
                "在非管理员模式下写入关闭 TUN 设置失败: {}",
                err
            );
        } else {
            tracing::info!(
                "检测到当前未获得管理员权限，已自动关闭 TUN 模式（原模式: {}）",
                previous_mode
            );
        }
    }

    Ok(config)
}

#[tauri::command]
pub async fn db_save_app_config(config: AppConfig, app: AppHandle) -> Result<(), String> {
    let storage = get_enhanced_storage(&app).await?;
    storage
        .save_app_config(&config)
        .await
        .map_err(|e| e.to_string())?;

    // 应用配置更新后，根据最新设置自动管理内核运行状态
    auto_manage_with_saved_config(&app, false, "app-config-updated").await;

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
    storage
        .save_active_subscription_index(index)
        .await
        .map_err(|e| e.to_string())
}
