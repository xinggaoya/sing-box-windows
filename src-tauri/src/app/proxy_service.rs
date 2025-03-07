use crate::entity::config_model;
use crate::utils::app_util::get_work_dir;
use crate::utils::config_util::ConfigUtil;
use std::error::Error;
use std::path::Path;
use tracing::info;
use crate::app::constants::{paths, network, config as config_constants, messages};

// 修改代理模式为系统代理
#[tauri::command]
pub fn set_system_proxy() -> Result<(), String> {
    let config_path = paths::get_config_path();
    let json_util =
        ConfigUtil::new(config_path.to_str().unwrap()).map_err(|e| format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e))?;

    let mut json_util = json_util;
    let target_keys = vec!["inbounds"];
    let new_structs = vec![config_model::Inbound {
        r#type: config_constants::DEFAULT_INBOUND_TYPE.to_string(),
        tag: config_constants::DEFAULT_INBOUND_TAG.to_string(),
        listen: Some(network::DEFAULT_LISTEN_ADDRESS.to_string()),
        listen_port: Some(network::DEFAULT_PROXY_PORT),
        address: None,
        auto_route: None,
        strict_route: None,
        stack: None,
        sniff: None,
        set_system_proxy: Some(true),
    }];

    json_util.update_key(target_keys.clone(), serde_json::to_value(new_structs).unwrap());
    match json_util.save_to_file() {
        Ok(_) => {
            info!("{}", messages::INFO_PROXY_MODE_ENABLED);
            Ok(())
        }
        Err(e) => Err(format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e)),
    }
}

// 修改TUN 模式为代理模式
#[tauri::command]
pub fn set_tun_proxy() -> Result<(), String> {
    set_tun_proxy_impl().map_err(|e| format!("设置TUN代理失败: {}", e))
}

fn set_tun_proxy_impl() -> Result<(), Box<dyn Error>> {
    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    let mut json_util = ConfigUtil::new(path.to_str().unwrap())?;

    let target_keys = vec!["inbounds"]; // 修改为你的属性路径
    let new_structs = vec![
        config_model::Inbound {
            r#type: "mixed".to_string(),
            tag: "mixed-in".to_string(),
            listen: Some("0.0.0.0".to_string()),
            listen_port: Some(2080),
            address: None,
            auto_route: None,
            strict_route: None,
            stack: None,
            sniff: None,
            set_system_proxy: None,
        },
        config_model::Inbound {
            r#type: "tun".to_string(),
            tag: "tun-in".to_string(),
            listen: None,
            listen_port: None,
            address: Some(vec![
                "172.18.0.1/30".to_string(),
                "fdfe:dcba:9876::1/126".to_string(),
            ]),
            auto_route: Some(true),
            strict_route: Some(true),
            stack: Some("mixed".to_string()),
            sniff: None,
            set_system_proxy: None,
        },
    ];

    json_util.modify_property(
        &target_keys,
        serde_json::to_value(new_structs).map_err(|e| format!("序列化配置失败: {}", e))?,
    );
    json_util
        .save()
        .map_err(|e| format!("保存配置文件失败: {}", e))?;

    info!("TUN代理模式已设置");
    Ok(())
}

// 切换 IPV6版本模式
#[tauri::command]
pub fn toggle_ip_version(prefer_ipv6: bool) -> Result<(), String> {
    info!(
        "开始切换IP版本模式: {}",
        if prefer_ipv6 { "IPv6优先" } else { "仅IPv4" }
    );

    let work_dir = get_work_dir();
    let path = Path::new(&work_dir).join("sing-box/config.json");
    info!("配置文件路径: {}", path.display());

    // 读取文件内容
    let content = std::fs::read_to_string(&path).map_err(|e| format!("读取配置文件失败: {}", e))?;

    // 直接替换字符串
    let modified_content = if prefer_ipv6 {
        content.replace("\"ipv4_only\"", "\"prefer_ipv6\"")
    } else {
        content.replace("\"prefer_ipv6\"", "\"ipv4_only\"")
    };

    // 验证修改后的内容是否是有效的 JSON
    serde_json::from_str::<serde_json::Value>(&modified_content)
        .map_err(|e| format!("修改后的配置不是有效的 JSON: {}", e))?;

    // 保存修改后的内容
    std::fs::write(&path, modified_content).map_err(|e| format!("保存配置文件失败: {}", e))?;

    info!(
        "IP版本模式已成功切换为: {}",
        if prefer_ipv6 { "IPv6优先" } else { "仅IPv4" }
    );
    Ok(())
} 