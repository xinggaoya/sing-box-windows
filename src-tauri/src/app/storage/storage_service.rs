use crate::app::storage::{AppState, StorageResult, AppConfig, ThemeConfig, LocaleConfig, WindowConfig, UpdateConfig, KernelInfo};
use std::path::PathBuf;
use std::fs;
use std::sync::Arc;
use tauri::{AppHandle, Manager, State};

/// 存储服务
#[derive(Debug, Clone)]
pub struct StorageService {
    state_file: PathBuf,
}

impl StorageService {
    pub fn new(app_handle: &AppHandle) -> Self {
        let app_data_dir = app_handle
            .path()
            .app_data_dir()
            .unwrap_or_else(|_| std::env::current_dir().unwrap());
        
        // 确保目录存在
        fs::create_dir_all(&app_data_dir).ok();
        
        let state_file = app_data_dir.join("app_state.json");
        
        Self {
            state_file,
        }
    }

    /// 加载应用状态
    pub fn load_state(&self) -> StorageResult<AppState> {
        if !self.state_file.exists() {
            // 如果文件不存在，返回默认状态
            return Ok(self.get_default_state());
        }

        let content = fs::read_to_string(&self.state_file)?;
        let state: AppState = serde_json::from_str(&content)?;
        
        Ok(state)
    }

    /// 保存应用状态
    pub fn save_state(&self, state: &AppState) -> StorageResult<()> {
        let content = serde_json::to_string_pretty(state)?;
        fs::write(&self.state_file, content)?;
        Ok(())
    }

    /// 获取默认应用状态
    pub fn get_default_state(&self) -> AppState {
        AppState {
            app_config: AppConfig {
                auto_start_kernel: false,
                prefer_ipv6: false,
                proxy_port: 12080,
                api_port: 12081,
                proxy_mode: Default::default(),
                tray_instance_id: None,
            },
            theme_config: ThemeConfig::default(),
            locale_config: LocaleConfig {
                locale: Default::default(),
            },
            window_config: WindowConfig::default(),
            update_config: UpdateConfig::default(),
            subscriptions: Vec::new(),
            kernel_info: KernelInfo {
                version: None,
                new_version: None,
            },
        }
    }

    /// 更新应用配置
    pub fn update_app_config<F>(&self, updater: F) -> StorageResult<()>
    where
        F: FnOnce(&mut AppConfig),
    {
        let mut state = self.load_state()?;
        updater(&mut state.app_config);
        self.save_state(&state)?;
        Ok(())
    }

    /// 更新主题配置
    pub fn update_theme_config<F>(&self, updater: F) -> StorageResult<()>
    where
        F: FnOnce(&mut ThemeConfig),
    {
        let mut state = self.load_state()?;
        updater(&mut state.theme_config);
        self.save_state(&state)?;
        Ok(())
    }

    /// 更新语言配置
    pub fn update_locale_config<F>(&self, updater: F) -> StorageResult<()>
    where
        F: FnOnce(&mut LocaleConfig),
    {
        let mut state = self.load_state()?;
        updater(&mut state.locale_config);
        self.save_state(&state)?;
        Ok(())
    }

    /// 更新窗口配置
    pub fn update_window_config<F>(&self, updater: F) -> StorageResult<()>
    where
        F: FnOnce(&mut WindowConfig),
    {
        let mut state = self.load_state()?;
        updater(&mut state.window_config);
        self.save_state(&state)?;
        Ok(())
    }

    /// 更新更新配置
    pub fn update_update_config<F>(&self, updater: F) -> StorageResult<()>
    where
        F: FnOnce(&mut UpdateConfig),
    {
        let mut state = self.load_state()?;
        updater(&mut state.update_config);
        self.save_state(&state)?;
        Ok(())
    }

    /// 更新订阅列表
    pub fn update_subscriptions<F>(&self, updater: F) -> StorageResult<()>
    where
        F: FnOnce(&mut Vec<crate::app::storage::Subscription>),
    {
        let mut state = self.load_state()?;
        updater(&mut state.subscriptions);
        self.save_state(&state)?;
        Ok(())
    }

    /// 更新内核信息
    pub fn update_kernel_info<F>(&self, updater: F) -> StorageResult<()>
    where
        F: FnOnce(&mut KernelInfo),
    {
        let mut state = self.load_state()?;
        updater(&mut state.kernel_info);
        self.save_state(&state)?;
        Ok(())
    }

    /// 获取应用配置
    pub fn get_app_config(&self) -> StorageResult<AppConfig> {
        let state = self.load_state()?;
        Ok(state.app_config)
    }

    /// 获取主题配置
    pub fn get_theme_config(&self) -> StorageResult<ThemeConfig> {
        let state = self.load_state()?;
        Ok(state.theme_config)
    }

    /// 获取语言配置
    pub fn get_locale_config(&self) -> StorageResult<LocaleConfig> {
        let state = self.load_state()?;
        Ok(state.locale_config)
    }

    /// 获取窗口配置
    pub fn get_window_config(&self) -> StorageResult<WindowConfig> {
        let state = self.load_state()?;
        Ok(state.window_config)
    }

    /// 获取更新配置
    pub fn get_update_config(&self) -> StorageResult<UpdateConfig> {
        let state = self.load_state()?;
        Ok(state.update_config)
    }

    /// 获取订阅列表
    pub fn get_subscriptions(&self) -> StorageResult<Vec<crate::app::storage::Subscription>> {
        let state = self.load_state()?;
        Ok(state.subscriptions)
    }

    /// 获取内核信息
    pub fn get_kernel_info(&self) -> StorageResult<KernelInfo> {
        let state = self.load_state()?;
        Ok(state.kernel_info)
    }

    /// 重置所有状态
    pub fn reset_state(&self) -> StorageResult<()> {
        let default_state = self.get_default_state();
        self.save_state(&default_state)?;
        Ok(())
    }

    /// 备份状态
    pub fn backup_state(&self, backup_path: &str) -> StorageResult<()> {
        let state = self.load_state()?;
        let content = serde_json::to_string_pretty(&state)?;
        fs::write(backup_path, content)?;
        Ok(())
    }

    /// 恢复状态
    pub fn restore_state(&self, backup_path: &str) -> StorageResult<()> {
        let content = fs::read_to_string(backup_path)?;
        let state: AppState = serde_json::from_str(&content)?;
        self.save_state(&state)?;
        Ok(())
    }
}

// Tauri 命令实现
#[tauri::command]
pub async fn load_state(
    storage: State<'_, Arc<StorageService>>,
) -> Result<AppState, String> {
    storage
        .load_state()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn save_state(
    state: AppState,
    storage: State<'_, Arc<StorageService>>,
) -> Result<(), String> {
    storage
        .save_state(&state)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_app_config(
    storage: State<'_, Arc<StorageService>>,
) -> Result<AppConfig, String> {
    storage
        .get_app_config()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_app_config(
    updates: serde_json::Value,
    storage: State<'_, Arc<StorageService>>,
) -> Result<(), String> {
    storage
        .update_app_config(|config| {
            // 简单的字段更新，这里可以根据需要实现更复杂的逻辑
            if let Some(auto_start_kernel) = updates.get("auto_start_kernel").and_then(|v| v.as_bool()) {
                config.auto_start_kernel = auto_start_kernel;
            }
            if let Some(prefer_ipv6) = updates.get("prefer_ipv6").and_then(|v| v.as_bool()) {
                config.prefer_ipv6 = prefer_ipv6;
            }
            if let Some(proxy_port) = updates.get("proxy_port").and_then(|v| v.as_u64()).map(|v| v as u16) {
                config.proxy_port = proxy_port;
            }
            if let Some(api_port) = updates.get("api_port").and_then(|v| v.as_u64()).map(|v| v as u16) {
                config.api_port = api_port;
            }
            if let Some(proxy_mode) = updates.get("proxy_mode").and_then(|v| v.as_str()) {
                config.proxy_mode = match proxy_mode {
                    "system" => crate::app::storage::ProxyMode::System,
                    "tun" => crate::app::storage::ProxyMode::Tun,
                    "manual" => crate::app::storage::ProxyMode::Manual,
                    _ => config.proxy_mode.clone(),
                };
            }
            if let Some(tray_instance_id) = updates.get("tray_instance_id").and_then(|v| v.as_str()) {
                config.tray_instance_id = Some(tray_instance_id.to_string());
            }
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_theme_config(
    storage: State<'_, Arc<StorageService>>,
) -> Result<ThemeConfig, String> {
    storage
        .get_theme_config()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_theme_config(
    is_dark: bool,
    storage: State<'_, Arc<StorageService>>,
) -> Result<(), String> {
    storage
        .update_theme_config(|config| {
            config.is_dark = is_dark;
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_locale_config(
    storage: State<'_, Arc<StorageService>>,
) -> Result<LocaleConfig, String> {
    storage
        .get_locale_config()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_locale_config(
    locale: String,
    storage: State<'_, Arc<StorageService>>,
) -> Result<(), String> {
    storage
        .update_locale_config(|config| {
            config.locale = match locale.as_str() {
                "auto" => crate::app::storage::Locale::Auto,
                "zh-CN" => crate::app::storage::Locale::ZhCN,
                "en-US" => crate::app::storage::Locale::EnUS,
                "ru-RU" => crate::app::storage::Locale::RuRU,
                "ja-JP" => crate::app::storage::Locale::JaJP,
                _ => config.locale.clone(),
            };
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_window_config(
    storage: State<'_, Arc<StorageService>>,
) -> Result<WindowConfig, String> {
    storage
        .get_window_config()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_window_config(
    updates: serde_json::Value,
    storage: State<'_, Arc<StorageService>>,
) -> Result<(), String> {
    storage
        .update_window_config(|config| {
            if let Some(is_visible) = updates.get("is_visible").and_then(|v| v.as_bool()) {
                config.is_visible = is_visible;
            }
            if let Some(is_fullscreen) = updates.get("is_fullscreen").and_then(|v| v.as_bool()) {
                config.is_fullscreen = is_fullscreen;
            }
            if let Some(is_maximized) = updates.get("is_maximized").and_then(|v| v.as_bool()) {
                config.is_maximized = is_maximized;
            }
            if let Some(last_visible_path) = updates.get("last_visible_path").and_then(|v| v.as_str()) {
                config.last_visible_path = last_visible_path.to_string();
            }
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_update_config(
    storage: State<'_, Arc<StorageService>>,
) -> Result<UpdateConfig, String> {
    storage
        .get_update_config()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_update_config(
    updates: serde_json::Value,
    storage: State<'_, Arc<StorageService>>,
) -> Result<(), String> {
    storage
        .update_update_config(|config| {
            if let Some(app_version) = updates.get("app_version").and_then(|v| v.as_str()) {
                config.app_version = app_version.to_string();
            }
            if let Some(auto_check_update) = updates.get("auto_check_update").and_then(|v| v.as_bool()) {
                config.auto_check_update = auto_check_update;
            }
            if let Some(skip_version) = updates.get("skip_version").and_then(|v| v.as_str()) {
                config.skip_version = Some(skip_version.to_string());
            }
            if let Some(accept_prerelease) = updates.get("accept_prerelease").and_then(|v| v.as_bool()) {
                config.accept_prerelease = accept_prerelease;
            }
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_subscriptions(
    storage: State<'_, Arc<StorageService>>,
) -> Result<Vec<crate::app::storage::Subscription>, String> {
    storage
        .get_subscriptions()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_subscriptions(
    subscriptions: Vec<crate::app::storage::Subscription>,
    storage: State<'_, Arc<StorageService>>,
) -> Result<(), String> {
    storage
        .update_subscriptions(|list| {
            *list = subscriptions;
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_kernel_info(
    storage: State<'_, Arc<StorageService>>,
) -> Result<KernelInfo, String> {
    storage
        .get_kernel_info()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn update_kernel_info(
    updates: serde_json::Value,
    storage: State<'_, Arc<StorageService>>,
) -> Result<(), String> {
    storage
        .update_kernel_info(|info| {
            if let Some(version) = updates.get("version") {
                if let Ok(version_info) = serde_json::from_value::<Option<crate::app::storage::VersionInfo>>(version.clone()) {
                    info.version = version_info;
                }
            }
            if let Some(new_version) = updates.get("new_version").and_then(|v| v.as_str()) {
                info.new_version = Some(new_version.to_string());
            }
        })
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn reset_state(
    storage: State<'_, Arc<StorageService>>,
) -> Result<(), String> {
    storage
        .reset_state()
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn backup_state(
    backup_path: String,
    storage: State<'_, Arc<StorageService>>,
) -> Result<(), String> {
    storage
        .backup_state(&backup_path)
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn restore_state(
    backup_path: String,
    storage: State<'_, Arc<StorageService>>,
) -> Result<(), String> {
    storage
        .restore_state(&backup_path)
        .map_err(|e| e.to_string())
}