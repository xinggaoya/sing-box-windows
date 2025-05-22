use crate::utils::app_util::get_work_dir;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::error::Error;
use std::fs;
use std::path::Path;
use std::sync::Mutex;
use tracing::{error, info};
use serde_json::Value;

// 端口配置结构
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct PortConfig {
    pub proxy_port: u16,
    pub api_port: u16,
}

// 默认端口配置
impl Default for PortConfig {
    fn default() -> Self {
        Self {
            proxy_port: 12080, // 默认代理端口
            api_port: 12081, // 默认API端口
        }
    }
}

// 配置文件管理
lazy_static::lazy_static! {
    static ref PORT_CONFIG: Mutex<PortConfig> = Mutex::new(PortConfig::default());
}

// 获取配置文件路径
fn get_config_file_path() -> String {
    let work_dir = get_work_dir();
    Path::new(&work_dir)
        .join("config")
        .join("ports.json")
        .to_string_lossy()
        .into_owned()
}

// 初始化配置服务
pub fn init_config_service() -> Result<(), Box<dyn Error>> {
    let config_path = get_config_file_path();
    let config_dir = Path::new(&config_path).parent().unwrap();

    // 确保配置目录存在
    if !config_dir.exists() {
        info!("创建配置目录: {:?}", config_dir);
        fs::create_dir_all(config_dir)?;
    }

    // 尝试读取配置文件
    let file_exists = Path::new(&config_path).exists();
    if file_exists {
        info!("读取端口配置文件: {}", config_path);
        match fs::read_to_string(&config_path) {
            Ok(content) => {
                match serde_json::from_str::<PortConfig>(&content) {
                    Ok(config) => {
                        let mut port_config = PORT_CONFIG.lock().unwrap();
                        *port_config = config;
                        info!("已加载端口配置: 代理端口={}, API端口={}", 
                            port_config.proxy_port, port_config.api_port);
                    }
                    Err(e) => {
                        error!("解析端口配置失败，使用默认值: {}", e);
                        // 文件格式不正确，使用默认值并保存
                        save_port_config()?;
                    }
                }
            }
            Err(e) => {
                error!("读取端口配置文件失败，使用默认值: {}", e);
                // 文件读取失败，使用默认值并保存
                save_port_config()?;
            }
        }
    } else {
        info!("端口配置文件不存在，创建默认配置");
        // 创建默认配置文件
        save_port_config()?;
    }

    Ok(())
}

// 保存端口配置
fn save_port_config() -> Result<(), Box<dyn Error>> {
    let config_path = get_config_file_path();
    let config = PORT_CONFIG.lock().unwrap();
    
    let content = serde_json::to_string_pretty(&*config)?;
    fs::write(&config_path, content)?;
    
    info!("已保存端口配置: 代理端口={}, API端口={}", 
        config.proxy_port, config.api_port);
    
    Ok(())
}

// 获取端口配置
#[tauri::command]
pub fn get_port_config() -> Result<PortConfig, String> {
    match PORT_CONFIG.lock() {
        Ok(config) => Ok(config.clone()),
        Err(e) => Err(format!("获取端口配置失败: {}", e)),
    }
}

// 更新sing-box配置文件中的端口设置
fn update_singbox_config_ports(proxy_port: u16, api_port: u16) -> Result<(), Box<dyn Error>> {
    let work_dir = get_work_dir();
    let config_path = Path::new(&work_dir).join("sing-box/config.json");
    
    // 检查配置文件是否存在
    if !config_path.exists() {
        info!("sing-box配置文件不存在，跳过更新");
        return Ok(());
    }
    
    info!("正在更新sing-box配置文件中的端口设置: 代理端口={}, API端口={}", proxy_port, api_port);
    
    // 读取现有的配置文件
    let config_content = match fs::read_to_string(&config_path) {
        Ok(content) => content,
        Err(e) => {
            error!("读取sing-box配置失败: {}", e);
            return Err(Box::new(e));
        }
    };
    
    // 解析为JSON
    let mut config: Value = match serde_json::from_str(&config_content) {
        Ok(json) => json,
        Err(e) => {
            error!("解析sing-box配置JSON失败: {}", e);
            return Err(Box::new(e));
        }
    };
    
    // 修改API端口和代理端口
    if let Some(config_obj) = config.as_object_mut() {
        // 修改experimental.clash_api配置（如果存在）
        if let Some(experimental) = config_obj.get_mut("experimental") {
            if let Some(exp_obj) = experimental.as_object_mut() {
                // 添加或修改clash_api配置
                let clash_api = exp_obj.entry("clash_api").or_insert(json!({}));
                
                if let Some(clash_api_obj) = clash_api.as_object_mut() {
                    // 设置external_controller为本地端口
                    clash_api_obj.insert(
                        "external_controller".to_string(),
                        json!(format!("127.0.0.1:{}", api_port)),
                    );
                }
            }
        } else {
            // 如果不存在experimental字段，添加它
            config_obj.insert(
                "experimental".to_string(), 
                json!({
                    "clash_api": {
                        "external_controller": format!("127.0.0.1:{}", api_port),
                        "external_ui": "metacubexd",
                        "default_mode": "rule"
                    }
                })
            );
        }
        
        // 修改入站端口（如果有inbounds）
        if let Some(inbounds) = config_obj.get_mut("inbounds") {
            if let Some(inbounds_array) = inbounds.as_array_mut() {
                for inbound in inbounds_array {
                    if let Some(inbound_obj) = inbound.as_object_mut() {
                        if inbound_obj.get("tag").and_then(|t| t.as_str()) == Some("mixed-in") {
                            inbound_obj.insert("listen_port".to_string(), json!(proxy_port));
                        }
                    }
                }
            }
        }
    }
    
    // 保存修改后的配置
    match fs::write(&config_path, serde_json::to_string_pretty(&config)?) {
        Ok(_) => {
            info!("已更新sing-box配置文件中的端口设置");
            Ok(())
        },
        Err(e) => {
            error!("保存sing-box配置失败: {}", e);
            Err(Box::new(e))
        }
    }
}

// 更新端口配置
#[tauri::command]
pub fn update_port_config(proxy_port: u16, api_port: u16) -> Result<bool, String> {
    // 验证端口范围
    if proxy_port < 1024 || api_port < 1024 {
        return Err("端口号必须在1024-65535之间".to_string());
    }

    // 验证端口不冲突
    if proxy_port == api_port {
        return Err("代理端口和API端口不能相同".to_string());
    }

    // 更新配置
    {
        let mut config = match PORT_CONFIG.lock() {
            Ok(config) => config,
            Err(e) => return Err(format!("锁定配置失败: {}", e)),
        };

        config.proxy_port = proxy_port;
        config.api_port = api_port;
        
        info!("更新端口配置: 代理端口={}, API端口={}", proxy_port, api_port);
    }

    // 保存配置到ports.json
    match save_port_config() {
        Ok(_) => {
            // 同步更新sing-box配置文件中的端口设置
            if let Err(e) = update_singbox_config_ports(proxy_port, api_port) {
                error!("更新sing-box配置端口失败，但端口配置已保存: {}", e);
                // 即使sing-box配置更新失败，仍然返回成功，因为端口配置已保存
            }
            Ok(true)
        },
        Err(e) => Err(format!("保存端口配置失败: {}", e)),
    }
}

// 获取当前代理端口
pub fn get_proxy_port() -> u16 {
    match PORT_CONFIG.lock() {
        Ok(config) => config.proxy_port,
        Err(_) => 12080, // 默认代理端口
    }
}

// 获取当前API端口
pub fn get_api_port() -> u16 {
    match PORT_CONFIG.lock() {
        Ok(config) => config.api_port,
        Err(_) => 12081, // 默认API端口
    }
} 