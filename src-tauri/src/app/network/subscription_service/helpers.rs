use crate::app::core::proxy_service::ProxyRuntimeState;
use crate::app::core::tun_profile::TunProxyOptions;
use crate::app::storage::state_model::AppConfig;
use crate::utils::app_util::get_work_dir_sync;
use std::path::{Path, PathBuf};
use tracing::warn;

fn sanitize_file_name(raw: &str) -> String {
    raw.chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .collect()
}

pub fn runtime_state_from_config(app_config: &AppConfig) -> ProxyRuntimeState {
    ProxyRuntimeState {
        proxy_port: app_config.proxy_port,
        system_proxy_enabled: app_config.system_proxy_enabled,
        tun_enabled: app_config.tun_enabled,
        system_proxy_bypass: app_config.system_proxy_bypass.clone(),
        tun_options: TunProxyOptions {
            ipv4_address: app_config.tun_ipv4.clone(),
            ipv6_address: app_config.tun_ipv6.clone(),
            mtu: app_config.tun_mtu,
            auto_route: app_config.tun_auto_route,
            strict_route: app_config.tun_strict_route,
            stack: app_config.tun_stack.clone(),
            enable_ipv6: app_config.tun_enable_ipv6,
            interface_name: None,
        },
    }
}

pub fn resolve_target_config_path(
    file_name: Option<String>,
    config_path: Option<String>,
) -> Result<PathBuf, String> {
    if let Some(path) = config_path {
        let candidate = PathBuf::from(path);
        if let Some(parent) = candidate.parent() {
            std::fs::create_dir_all(parent).map_err(|e| format!("创建配置目录失败: {}", e))?;
        }
        return Ok(candidate);
    }

    let work_dir = get_work_dir_sync();
    let config_dir = Path::new(&work_dir).join("sing-box/configs");
    if let Err(e) = std::fs::create_dir_all(&config_dir) {
        return Err(format!("创建配置目录失败: {}", e));
    }

    let file = file_name
        .and_then(|name| {
            Path::new(&name)
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
        })
        .unwrap_or_else(|| {
            let ts = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();
            format!("config-{}.json", ts)
        });
    let safe_file = sanitize_file_name(&file);

    Ok(config_dir.join(safe_file))
}

pub fn backup_existing_config(target: &Path) -> Option<PathBuf> {
    if target.exists() {
        let backup = target.with_extension("bak");
        if let Err(e) = std::fs::copy(target, &backup) {
            warn!("备份配置失败: {}", e);
            None
        } else {
            Some(backup)
        }
    } else {
        None
    }
}
