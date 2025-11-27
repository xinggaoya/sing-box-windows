use app::storage::EnhancedStorageService;
use std::sync::Arc;
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tracing_subscriber::{fmt, EnvFilter}; // 重新启用数据库存储

pub mod app;
pub mod entity;
pub mod process;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 设置默认的 debug 日志级别
    let env_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // 使用 RUST_LOG 环境变量，或者默认启用 debug 级别
        std::env::set_var("RUST_LOG", "debug,sing_box_windows=debug,tauri=info");
        EnvFilter::from_default_env()
    });

    fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build()) // 重新启用 store 插件
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hide"]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            // 已有实例正在运行时的处理
            tracing::info!("应用已在运行中: {:?}, {:?}", argv, cwd);
            // 让已运行的实例窗口显示到前台
            show_window(app);
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(|app| {
            // if cfg!(debug_assertions) {
            //     app.handle().plugin(
            //         tauri_plugin_log::Builder::default()
            //             .level("debug")
            //             .build(),
            //     )?;
            // }
            // 判断参数
            let args: Vec<String> = std::env::args().collect();
            if args.len() > 1 {
                if args[1] == "--hide" {
                    let window = app.get_window("main").unwrap();
                    window.hide().unwrap();
                }
            }

            // 重新启用增强版存储服务（数据库）
            let enhanced_storage =
                std::sync::Mutex::new(None as Option<Arc<EnhancedStorageService>>);
            app.manage(enhanced_storage);

            // 异步初始化数据库服务
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                match EnhancedStorageService::new(&app_handle).await {
                    Ok(service) => {
                        if let Ok(mut enhanced_storage) = app_handle
                            .state::<std::sync::Mutex<Option<Arc<EnhancedStorageService>>>>()
                            .lock()
                        {
                            *enhanced_storage = Some(Arc::new(service));
                        }
                        tracing::info!("Enhanced storage service initialized successfully");

                        // 后端启动后立即尝试自动管理内核（尊重 auto_start_kernel 设置）
                        crate::app::core::kernel_service::auto_manage_with_saved_config(
                            &app_handle,
                            false,
                            "app-start",
                        )
                        .await;
                    }
                    Err(e) => {
                        tracing::error!("Failed to initialize enhanced storage service: {}", e);
                    }
                }
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Enhanced Storage service commands (数据库)
            crate::app::storage::enhanced_storage_service::db_get_app_config,
            crate::app::storage::enhanced_storage_service::db_save_app_config,
            crate::app::storage::enhanced_storage_service::db_get_theme_config,
            crate::app::storage::enhanced_storage_service::db_save_theme_config,
            crate::app::storage::enhanced_storage_service::db_get_locale_config,
            crate::app::storage::enhanced_storage_service::db_save_locale_config,
            crate::app::storage::enhanced_storage_service::db_get_window_config,
            crate::app::storage::enhanced_storage_service::db_save_window_config,
            crate::app::storage::enhanced_storage_service::db_get_update_config,
            crate::app::storage::enhanced_storage_service::db_save_update_config,
            crate::app::storage::enhanced_storage_service::db_get_subscriptions,
            crate::app::storage::enhanced_storage_service::db_save_subscriptions,
            crate::app::storage::enhanced_storage_service::db_get_active_subscription_index,
            crate::app::storage::enhanced_storage_service::db_save_active_subscription_index,
            // Core - Kernel service commands (legacy)
            crate::app::core::kernel_service::start_kernel,
            crate::app::core::kernel_service::stop_kernel,
            crate::app::core::kernel_service::restart_kernel,
            crate::app::core::kernel_service::download_latest_kernel,
            crate::app::core::kernel_service::install_kernel,
            crate::app::core::kernel_service::get_latest_kernel_version_cmd,
            crate::app::core::kernel_service::check_kernel_version,
            crate::app::core::kernel_service::start_websocket_relay,
            crate::app::core::kernel_service::is_kernel_running,
            crate::app::core::kernel_service::check_kernel_status,
            crate::app::core::kernel_service::get_system_uptime,
            // Core - Kernel service commands (enhanced)
            crate::app::core::kernel_service::kernel_start_enhanced,
            crate::app::core::kernel_service::kernel_stop_enhanced,
            crate::app::core::kernel_service::kernel_stop_background,
            crate::app::core::kernel_service::force_stop_and_exit,
            crate::app::core::kernel_service::kernel_get_status_enhanced,
            crate::app::core::kernel_service::kernel_check_health,
            crate::app::core::kernel_service::kernel_auto_manage,
            crate::app::core::kernel_service::apply_proxy_settings,
            // Network - Subscription service commands
            crate::app::network::subscription_service::download_subscription,
            crate::app::network::subscription_service::add_manual_subscription,
            crate::app::network::subscription_service::get_current_config,
            crate::app::network::subscription_service::toggle_proxy_mode,
            crate::app::network::subscription_service::get_current_proxy_mode,
            // System - System service commands
            crate::app::system::system_service::check_admin,
            crate::app::system::system_service::restart_as_admin,
            crate::app::system::system_service::check_network_connectivity,
            crate::app::system::system_service::wait_for_network_ready,
            crate::app::system::system_service::toggle_devtools,
            crate::app::system::system_service::open_devtools,
            crate::app::system::system_service::close_devtools,
            crate::app::system::system_service::is_devtools_open,
            // System - Update service commands
            crate::app::system::update_service::check_update,
            crate::app::system::update_service::download_update,
            crate::app::system::update_service::install_update,
            crate::app::system::update_service::download_and_install_update,
            crate::app::system::update_service::get_platform_info,
            // System - Config service commands
            crate::app::system::config_service::update_singbox_ports,
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
