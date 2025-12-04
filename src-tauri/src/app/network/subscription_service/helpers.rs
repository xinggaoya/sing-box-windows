use crate::app::core::proxy_service::ProxyRuntimeState;
use crate::app::core::tun_profile::{TunProxyOptions, TUN_ROUTE_EXCLUDES};
use crate::app::storage::state_model::AppConfig;
use crate::utils::app_util::get_work_dir_sync;
use serde_json::{json, Map, Value};
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

pub fn apply_inbounds_settings(config_obj: &mut Map<String, Value>, app_config: &AppConfig) {
    let mut tun_addresses = vec![app_config.tun_ipv4.clone()];
    if app_config.tun_enable_ipv6 {
        tun_addresses.push(app_config.tun_ipv6.clone());
    }

    let mut inbounds = Vec::new();
    inbounds.push(json!({
        "type": "mixed",
        "tag": "mixed-in",
        "listen": "127.0.0.1",
        "listen_port": app_config.proxy_port,
        "sniff": true,
        "set_system_proxy": app_config.system_proxy_enabled
    }));

    if app_config.tun_enabled {
        inbounds.push(json!({
            "type": "tun",
            "tag": "tun-in",
            "address": tun_addresses,
            "auto_route": app_config.tun_auto_route,
            "strict_route": app_config.tun_strict_route,
            "stack": app_config.tun_stack,
            "mtu": app_config.tun_mtu,
            "sniff": true,
            "sniff_override_destination": true,
            "route_exclude_address": TUN_ROUTE_EXCLUDES
        }));
    }

    config_obj.insert("inbounds".to_string(), json!(inbounds));
}

pub fn apply_app_settings_to_config(config: &mut Value, app_config: &AppConfig) {
    if let Some(config_obj) = config.as_object_mut() {
        apply_inbounds_settings(config_obj, app_config);

        let experimental = config_obj
            .entry("experimental".to_string())
            .or_insert(json!({}));
        if let Some(exp_obj) = experimental.as_object_mut() {
            let clash_api = exp_obj.entry("clash_api".to_string()).or_insert(json!({}));
            if let Some(clash_api_obj) = clash_api.as_object_mut() {
                clash_api_obj.insert(
                    "external_controller".to_string(),
                    json!(format!("127.0.0.1:{}", app_config.api_port)),
                );
            }
        }

        let dns = config_obj.entry("dns".to_string()).or_insert(json!({}));
        if let Some(dns_obj) = dns.as_object_mut() {
            let strategy = if app_config.prefer_ipv6 {
                "prefer_ipv6"
            } else {
                "ipv4_only"
            };
            dns_obj.insert("strategy".to_string(), json!(strategy));
        }
    }
}
