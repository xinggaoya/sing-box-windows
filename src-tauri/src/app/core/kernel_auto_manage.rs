use crate::app::constants::paths;
use crate::app::core::kernel_service::{
    check_config_validity, is_kernel_running, resolve_proxy_runtime_state, start_kernel_with_state,
    stop_kernel, ProxyOverrides,
};
use crate::app::core::tun_profile::TunProxyOptions;
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use crate::app::storage::state_model::AppConfig;
use serde::Serialize;
use tauri::AppHandle;
use tokio::time::Duration;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct AutoManageOptions {
    pub proxy_mode: Option<String>,
    pub api_port: Option<u16>,
    pub proxy_port: Option<u16>,
    pub prefer_ipv6: Option<bool>,
    pub system_proxy_bypass: Option<String>,
    pub tun_options: Option<TunProxyOptions>,
    pub system_proxy_enabled: Option<bool>,
    pub tun_enabled: Option<bool>,
    pub keep_alive: Option<bool>,
    pub force_restart: bool,
}

impl AutoManageOptions {
    pub fn from_app_config(config: AppConfig) -> Self {
        AutoManageOptions {
            proxy_mode: Some(config.proxy_mode.clone()),
            api_port: Some(config.api_port),
            proxy_port: Some(config.proxy_port),
            prefer_ipv6: Some(config.prefer_ipv6),
            system_proxy_bypass: Some(config.system_proxy_bypass.clone()),
            tun_options: Some(TunProxyOptions {
                ipv4_address: config.tun_ipv4.clone(),
                ipv6_address: config.tun_ipv6.clone(),
                mtu: config.tun_mtu,
                auto_route: config.tun_auto_route,
                strict_route: config.tun_strict_route,
                stack: config.tun_stack.clone(),
                enable_ipv6: config.tun_enable_ipv6,
                interface_name: None,
            }),
            system_proxy_enabled: Some(config.system_proxy_enabled),
            tun_enabled: Some(config.tun_enabled),
            keep_alive: Some(config.auto_start_kernel),
            force_restart: false,
        }
    }

    fn to_overrides(&self) -> ProxyOverrides {
        ProxyOverrides {
            proxy_mode: self.proxy_mode.clone(),
            api_port: self.api_port,
            proxy_port: self.proxy_port,
            prefer_ipv6: self.prefer_ipv6,
            system_proxy_bypass: self.system_proxy_bypass.clone(),
            tun_options: self.tun_options.clone(),
            system_proxy_enabled: self.system_proxy_enabled,
            tun_enabled: self.tun_enabled,
            keep_alive: self.keep_alive,
        }
    }
}

#[derive(Debug, Serialize)]
struct AutoManageResult {
    state: String,
    message: String,
    kernel_installed: bool,
    config_ready: bool,
    attempted_start: bool,
    last_start_message: Option<String>,
}

impl AutoManageResult {
    fn new(
        state: &str,
        message: impl Into<String>,
        kernel_installed: bool,
        config_ready: bool,
        attempted_start: bool,
        last_start_message: Option<String>,
    ) -> Self {
        AutoManageResult {
            state: state.to_string(),
            message: message.into(),
            kernel_installed,
            config_ready,
            attempted_start,
            last_start_message,
        }
    }

    fn missing_kernel() -> Self {
        AutoManageResult::new(
            "missing_kernel",
            "未检测到内核，请先下载内核",
            false,
            false,
            false,
            None,
        )
    }

    fn missing_config() -> Self {
        AutoManageResult::new(
            "missing_config",
            "未检测到配置，请先添加订阅或导入配置",
            true,
            false,
            false,
            None,
        )
    }

    fn invalid_config(message: String) -> Self {
        AutoManageResult::new(
            "invalid_config",
            format!("配置文件校验失败: {}", message),
            true,
            false,
            false,
            None,
        )
    }

    fn running(message: impl Into<String>, attempted: bool, last_message: Option<String>) -> Self {
        AutoManageResult::new(
            "running",
            message.into(),
            true,
            true,
            attempted,
            last_message,
        )
    }

    fn error(message: impl Into<String>, attempted: bool) -> Self {
        AutoManageResult::new(
            "error",
            message.into(),
            true,
            true,
            attempted,
            None,
        )
    }
}

fn kernel_binary_exists() -> bool {
    paths::get_kernel_path().exists()
}

async fn auto_manage_kernel_internal(
    app_handle: AppHandle,
    options: AutoManageOptions,
) -> Result<AutoManageResult, String> {
    let resolved_state = resolve_proxy_runtime_state(&app_handle, options.to_overrides()).await?;

    let kernel_installed = kernel_binary_exists();
    if !kernel_installed {
        return Ok(AutoManageResult::missing_kernel());
    }

    if let Err(err) = check_config_validity(app_handle.clone(), String::new()).await {
        return Ok(AutoManageResult::invalid_config(err));
    }

    let mut was_running = is_kernel_running().await.unwrap_or(false);
    if options.force_restart && was_running {
        info!("自动管理请求触发内核重启");
        let _ = stop_kernel().await;
        tokio::time::sleep(Duration::from_millis(500)).await;
        was_running = false;
    }

    let start_response = start_kernel_with_state(app_handle.clone(), &resolved_state).await?;

    let success = start_response
        .get("success")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let message = start_response
        .get("message")
        .and_then(|value| value.as_str())
        .unwrap_or("内核启动状态未知")
        .to_string();

    let attempted_start = !was_running || options.force_restart;

    if success {
        Ok(AutoManageResult::running(
            message.clone(),
            attempted_start,
            Some(message),
        ))
    } else {
        Ok(AutoManageResult::error(message, attempted_start))
    }
}

pub async fn auto_manage_with_saved_config(
    app_handle: &AppHandle,
    force_restart: bool,
    reason: &str,
) {
    match db_get_app_config(app_handle.clone()).await {
        Ok(config) => {
            let mut options = AutoManageOptions::from_app_config(config);
            options.force_restart = force_restart;

            match auto_manage_kernel_internal(app_handle.clone(), options).await {
                Ok(result) => {
                    info!(
                        "自动管理({})完成，状态: {}, 信息: {}",
                        reason, result.state, result.message
                    );
                }
                Err(err) => {
                    warn!("自动管理({})失败: {}", reason, err);
                }
            }
        }
        Err(err) => {
            warn!(
                "加载应用配置失败，跳过自动管理({}): {}",
                reason, err
            );
        }
    }
}

#[tauri::command]
pub async fn kernel_auto_manage(
    app_handle: AppHandle,
    proxy_mode: Option<String>,
    api_port: Option<u16>,
    proxy_port: Option<u16>,
    prefer_ipv6: Option<bool>,
    system_proxy_bypass: Option<String>,
    tun_options: Option<TunProxyOptions>,
    keep_alive: Option<bool>,
    system_proxy_enabled: Option<bool>,
    tun_enabled: Option<bool>,
    force_restart: Option<bool>,
) -> Result<serde_json::Value, String> {
    let options = AutoManageOptions {
        proxy_mode,
        api_port,
        proxy_port,
        prefer_ipv6,
        system_proxy_bypass,
        tun_options,
        keep_alive,
        system_proxy_enabled,
        tun_enabled,
        force_restart: force_restart.unwrap_or(false),
    };

    let result = auto_manage_kernel_internal(app_handle, options).await?;
    serde_json::to_value(result).map_err(|e| e.to_string())
}
