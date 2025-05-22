use tauri::{AppHandle, Manager};use tracing_subscriber::{fmt, EnvFilter};use std::sync::Mutex;

pub mod app;
pub mod entity;
pub mod process;
pub mod utils;

pub struct AppState {
    #[allow(dead_code)]
    token: Mutex<String>,
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    // 初始化配置服务
    if let Err(e) = crate::app::system::config_service::init_config_service() {
        tracing::error!("初始化配置服务失败: {}", e);
        // 继续执行，使用默认端口配置
    }

    let state = AppState {
        token: Mutex::new(String::new()),
    };

    tauri::Builder::default()
        .manage(state)
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_autostart::Builder::new().build())
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            // 已有实例正在运行时的处理
            tracing::info!("应用已在运行中: {:?}, {:?}", argv, cwd);
            // 让已运行的实例窗口显示到前台
            show_window(app);
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .invoke_handler(tauri::generate_handler![
                        // Core - Kernel service commands            
            crate::app::core::kernel_service::start_kernel,            
            crate::app::core::kernel_service::stop_kernel,            
            crate::app::core::kernel_service::restart_kernel,            
            crate::app::core::kernel_service::download_latest_kernel,            
            crate::app::core::kernel_service::check_kernel_version,            
            crate::app::core::kernel_service::start_websocket_relay,
            crate::app::core::kernel_service::is_kernel_running,

            // Network - Subscription service commands
            crate::app::network::subscription_service::download_subscription,
            crate::app::network::subscription_service::add_manual_subscription,
            crate::app::network::subscription_service::get_current_config,
            crate::app::network::subscription_service::toggle_proxy_mode,
            crate::app::network::subscription_service::get_current_proxy_mode,

            // System - System service commands
            crate::app::system::system_service::check_admin,
            crate::app::system::system_service::restart_as_admin,
            crate::app::system::system_service::exit_application,
            crate::app::system::system_service::install_service,
            crate::app::system::system_service::uninstall_service,
            crate::app::system::system_service::check_service_status,

            // System - Update service commands
            crate::app::system::update_service::check_update,
            crate::app::system::update_service::download_and_install_update,

            // System - Config service commands
            crate::app::system::config_service::get_port_config,
            crate::app::system::config_service::update_port_config,

            // Core - Proxy service commands
            crate::app::core::proxy_service::set_system_proxy,
            crate::app::core::proxy_service::set_manual_proxy,
            crate::app::core::proxy_service::set_tun_proxy,
            crate::app::core::proxy_service::toggle_ip_version,
            crate::app::core::proxy_service::get_api_token,
            crate::app::core::proxy_service::get_proxies,
            crate::app::core::proxy_service::change_proxy,
            crate::app::core::proxy_service::test_node_delay,
            crate::app::core::proxy_service::test_group_delay,
            crate::app::core::proxy_service::get_version_info,
            crate::app::core::proxy_service::get_rules,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[allow(dead_code)]
fn show_window(app: &AppHandle) {
    let windows = app.webview_windows();

    windows
        .values()
        .next()
        .expect("Sorry, no window found")
        .set_focus()
        .expect("Can't Bring Window to Focus");
}
