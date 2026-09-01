use crate::app::constants::paths;
use crate::app::core::kernel_service::embedded::ensure_embedded_kernel;
use crate::app::core::kernel_service::utils::emit_kernel_error_with_context;
use crate::app::core::kernel_service::{
    check_config_validity, is_kernel_running, orchestrated_restart_kernel,
    orchestrated_start_kernel, KernelRuntimeConfig, ProxyOverrides,
};
use crate::app::core::tun_profile::TunProxyOptions;
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use crate::app::storage::state_model::AppConfig;
use serde::Serialize;
use tauri::AppHandle;
use tracing::{info, warn};

#[derive(Debug, Clone)]
pub struct AutoManageOptions {
    pub config: KernelRuntimeConfig,
}

impl AutoManageOptions {
    pub fn from_app_config(config: AppConfig) -> Self {
        AutoManageOptions {
            config: KernelRuntimeConfig::from_app_config(&config),
        }
    }

    fn to_overrides(&self) -> ProxyOverrides {
        ProxyOverrides {
            proxy_mode: self.config.proxy_mode.clone(),
            api_port: self.config.api_port,
            proxy_port: self.config.proxy_port,
            prefer_ipv6: self.config.prefer_ipv6,
            system_proxy_bypass: self.config.system_proxy_bypass.clone(),
            tun_options: self.config.tun_options.clone(),
            system_proxy_enabled: self.config.system_proxy_enabled,
            tun_enabled: self.config.tun_enabled,
            keep_alive: self.config.keep_alive,
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
        AutoManageResult::new("error", message.into(), true, true, attempted, None)
    }
}

fn kernel_binary_exists() -> bool {
    paths::get_kernel_path().exists()
}

async fn auto_manage_kernel_internal(
    app_handle: AppHandle,
    options: AutoManageOptions,
) -> Result<AutoManageResult, String> {
    let _attempt_id = crate::app::core::kernel_service::state::KERNEL_STATE
        .begin_attempt("kernel-auto-manage");

    if let Err(err) = ensure_embedded_kernel(&app_handle).await {
        warn!("安装内嵌内核失败，继续按现有逻辑处理: {}", err);
    }

    let overrides = options.to_overrides();

    let kernel_installed = kernel_binary_exists();
    if !kernel_installed {
        return Ok(AutoManageResult::missing_kernel());
    }

    if let Err(err) = check_config_validity(app_handle.clone(), String::new()).await {
        return Ok(AutoManageResult::invalid_config(err));
    }

    let was_running = is_kernel_running().await.unwrap_or(false);
    let attempted_start = !was_running || options.config.force_restart;

    if options.config.force_restart && was_running {
        info!("自动管理请求触发内核重启");
        let restart_response =
            orchestrated_restart_kernel(app_handle.clone(), overrides.clone()).await?;
        let success = restart_response
            .get("success")
            .and_then(|value| value.as_bool())
            .unwrap_or(false);
        let message = restart_response
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("内核重启状态未知")
            .to_string();

        if success {
            return Ok(AutoManageResult::running(
                message.clone(),
                attempted_start,
                Some(message),
            ));
        }

        return Ok(AutoManageResult::error(message, attempted_start));
    }

    let start_response = orchestrated_start_kernel(app_handle.clone(), overrides).await?;

    let success = start_response
        .get("success")
        .and_then(|value| value.as_bool())
        .unwrap_or(false);
    let message = start_response
        .get("message")
        .and_then(|value| value.as_str())
        .unwrap_or("内核启动状态未知")
        .to_string();

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
            options.config.force_restart = force_restart;

            match auto_manage_kernel_internal(app_handle.clone(), options).await {
                Ok(result) => {
                    info!(
                        "自动管理({})完成，状态: {}, 信息: {}",
                        reason, result.state, result.message
                    );

                    match result.state.as_str() {
                        "invalid_config" => {
                            emit_kernel_error_with_context(
                                app_handle,
                                "KERNEL_CONFIG_INVALID",
                                "内核启动失败：配置校验未通过",
                                Some(&result.message),
                                Some("kernel.auto_manage"),
                                true,
                            );
                        }
                        "error" => {
                            emit_kernel_error_with_context(
                                app_handle,
                                "KERNEL_AUTO_MANAGE_FAILED",
                                "内核自动管理失败",
                                Some(&result.message),
                                Some("kernel.auto_manage"),
                                true,
                            );
                        }
                        "missing_kernel" => {
                            emit_kernel_error_with_context(
                                app_handle,
                                "KERNEL_BINARY_MISSING",
                                "未检测到内核文件，请先安装内核",
                                Some(&result.message),
                                Some("kernel.auto_manage"),
                                false,
                            );
                        }
                        _ => {}
                    }
                }
                Err(err) => {
                    warn!("自动管理({})失败: {}", reason, err);
                    emit_kernel_error_with_context(
                        app_handle,
                        "KERNEL_AUTO_MANAGE_FAILED",
                        "内核自动管理异常中断",
                        Some(&err),
                        Some("kernel.auto_manage"),
                        true,
                    );
                }
            }
        }
        Err(err) => {
            warn!("加载应用配置失败，跳过自动管理({}): {}", reason, err);
        }
    }
}

#[tauri::command]
#[allow(clippy::too_many_arguments)] // Tauri 命令需要保持与前端一致的参数形态
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
        config: KernelRuntimeConfig {
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
        },
    };

    let result = auto_manage_kernel_internal(app_handle.clone(), options).await?;
    match result.state.as_str() {
        "invalid_config" => {
            emit_kernel_error_with_context(
                &app_handle,
                "KERNEL_CONFIG_INVALID",
                "内核启动失败：配置校验未通过",
                Some(&result.message),
                Some("kernel.auto_manage"),
                true,
            );
        }
        "error" => {
            emit_kernel_error_with_context(
                &app_handle,
                "KERNEL_AUTO_MANAGE_FAILED",
                "内核自动管理失败",
                Some(&result.message),
                Some("kernel.auto_manage"),
                true,
            );
        }
        "missing_kernel" => {
            emit_kernel_error_with_context(
                &app_handle,
                "KERNEL_BINARY_MISSING",
                "未检测到内核文件，请先安装内核",
                Some(&result.message),
                Some("kernel.auto_manage"),
                false,
            );
        }
        _ => {}
    }
    serde_json::to_value(result).map_err(|e| e.to_string())
}
