use crate::utils::log_util;
use app::storage::EnhancedStorageService;
use std::sync::Arc;
use tauri::{AppHandle, Manager, RunEvent, WindowEvent};
use tauri_plugin_autostart::MacosLauncher;
use tokio::sync::OnceCell;

pub mod app;
pub mod entity;
pub mod platform;
pub mod process;
pub mod utils;

#[derive(Debug, Clone, Copy)]
struct StartupLaunchContext {
    should_start_hidden: bool,
    should_start_lightweight: bool,
    close_behavior: crate::app::tray::TrayCloseBehavior,
}

fn resolve_startup_launch_context<R: tauri::Runtime>(app: &tauri::App<R>) -> StartupLaunchContext {
    let args: Vec<String> = std::env::args().collect();
    let launched_via_autostart = args
        .iter()
        .any(|arg| arg == "--autostart" || arg == "--hide");
    let startup_preferences = crate::app::storage::read_startup_preferences_sync(app.handle());
    let close_behavior =
        crate::app::tray::TrayCloseBehavior::from_raw(&startup_preferences.tray_close_behavior);
    let should_start_hidden =
        launched_via_autostart && startup_preferences.auto_hide_to_tray_on_autostart;

    StartupLaunchContext {
        should_start_hidden,
        should_start_lightweight: should_start_hidden
            && close_behavior == crate::app::tray::TrayCloseBehavior::Lightweight,
        close_behavior,
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    // 初始化日志，支持文件输出 + 定期清理
    let log_dir = log_util::init_logging();

    tauri::Builder::default()
        .on_window_event(|window, event| {
            if window.label() != "main" {
                return;
            }

            if let WindowEvent::CloseRequested { api, .. } = event {
                api.prevent_close();
                if let Err(err) = crate::app::tray::close_main_window(window.app_handle()) {
                    tracing::warn!("拦截主窗口关闭事件失败: {}", err);
                }
            }
        })
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_os::init()) // 添加 OS 信息插件
        .plugin(tauri_plugin_opener::init()) // 统一打开外部版本页面
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--autostart"]),
        ))
        .plugin(tauri_plugin_single_instance::init(|app, argv, cwd| {
            // 已有实例正在运行时的处理
            tracing::info!("应用已在运行中: {:?}, {:?}", argv, cwd);
            // 让已运行的实例窗口显示到前台
            show_window(app);
        }))
        .plugin(tauri_plugin_window_state::Builder::default().build())
        .setup(move |app| {
            // if cfg!(debug_assertions) {
            //     app.handle().plugin(
            //         tauri_plugin_log::Builder::default()
            //             .level("debug")
            //             .build(),
            //     )?;
            // }
            let startup_context = resolve_startup_launch_context(app);
            crate::app::tray::apply_startup_preferences(
                startup_context.close_behavior,
                !startup_context.should_start_hidden,
            );

            if startup_context.should_start_hidden {
                if let Some(window) = app.get_webview_window("main") {
                    if let Err(err) = window.hide() {
                        tracing::warn!("启动时隐藏主窗口失败: {}", err);
                    }
                } else {
                    tracing::warn!("启动时未找到 main 窗口");
                }
            }

            if let Err(err) = crate::app::tray::init_tray(app.handle()) {
                tracing::error!("初始化托盘失败: {}", err);
            }

            // 启动日志目录定时清理
            if let Some(dir) = log_dir.clone() {
                // 后台清理日志，不阻塞应用启动
                log_util::spawn_log_cleanup_task(dir);
            }

            // 重新启用增强版存储服务（数据库）
            let enhanced_storage: Arc<OnceCell<Arc<EnhancedStorageService>>> =
                Arc::new(OnceCell::const_new());
            app.manage(enhanced_storage);

            // 异步初始化数据库服务（单例）
            let app_handle = app.handle().clone();
            let should_start_lightweight = startup_context.should_start_lightweight;
            let storage_cell_state = app.state::<Arc<OnceCell<Arc<EnhancedStorageService>>>>();
            let storage_cell = Arc::clone(&*storage_cell_state);
            tauri::async_runtime::spawn(async move {
                if let Err(e) = storage_cell
                    .get_or_try_init(|| async {
                        EnhancedStorageService::new(&app_handle).await.map(Arc::new)
                    })
                    .await
                {
                    tracing::error!("Failed to initialize enhanced storage service: {}", e);
                    return;
                }

                tracing::info!("Enhanced storage service initialized successfully");

                match crate::app::system::startup_restore_service::prepare_startup_restore(
                    &app_handle,
                )
                .await
                {
                    Ok(Some(active_config_path)) => {
                        tracing::info!(
                            "启动恢复已确认生效配置路径: {}",
                            active_config_path
                        );
                    }
                    Ok(None) => {
                        tracing::info!("启动恢复未解析到活动配置路径，将继续使用默认配置");
                    }
                    Err(err) => {
                        tracing::warn!("启动恢复准备失败，将继续沿用现有配置: {}", err);
                    }
                }

                // 启动时清理可能残留的内核进程，避免复用非本程序启动的内核实例。
                if let Err(e) = crate::process::manager::ProcessManager::new()
                    .kill_existing_processes(Some(&app_handle))
                    .await
                {
                    tracing::warn!("启动时清理内核进程失败: {}", e);
                }

                // 应用升级后：尝试刷新当前活动订阅一次，尽量在首次拉起内核前完成配置迁移。
                crate::app::system::startup_refresh_service::start_upgrade_subscription_refresh(
                    &app_handle,
                )
                .await;

                // 后端启动后立即尝试自动管理内核（尊重 auto_start_kernel 设置）
                crate::app::core::kernel_auto_manage::auto_manage_with_saved_config(
                    &app_handle,
                    false,
                    "app-start",
                )
                .await;

                if let Err(err) =
                    crate::app::tray::refresh_runtime_state_from_backend(&app_handle, true).await
                {
                    tracing::warn!("启动阶段刷新托盘运行态失败: {}", err);
                }

                // 启动后台任务（自动更新检查、健康巡检等）
                crate::app::system::background_tasks::start_background_tasks(&app_handle).await;
                // 启动订阅自动刷新
                crate::app::network::subscription_service::auto_update::start_subscription_auto_update(
                    &app_handle,
                )
                .await;

                if should_start_lightweight {
                    if let Err(err) =
                        crate::app::tray::enter_startup_background_mode(&app_handle, true)
                    {
                        tracing::warn!("开机自启轻量模式进入失败: {}", err);
                    } else if let Err(err) =
                        crate::app::tray::refresh_runtime_state_from_backend(&app_handle, true)
                            .await
                    {
                        tracing::warn!("轻量模式启动后刷新托盘失败: {}", err);
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
            // Core - Kernel service commands (legacy)
            crate::app::core::kernel_service::download::download_kernel,
            crate::app::core::kernel_service::import::pick_kernel_import_file,
            crate::app::core::kernel_service::import::import_kernel_executable,
            crate::app::core::kernel_service::versioning::get_latest_kernel_version_cmd,
            crate::app::core::kernel_service::versioning::get_kernel_releases_cmd,
            crate::app::core::kernel_service::versioning::check_kernel_version,
            crate::app::core::kernel_service::status::is_kernel_running,
            crate::app::core::kernel_service::status::get_system_uptime,
            // Core - Kernel service commands (enhanced)
            crate::app::core::kernel_service::runtime::kernel_start_enhanced,
            crate::app::core::kernel_service::runtime::kernel_stop_enhanced,
            crate::app::core::kernel_service::runtime::kernel_restart_fast,
            crate::app::core::kernel_service::status::kernel_get_status_enhanced,
            crate::app::core::kernel_service::status::kernel_get_snapshot,
            crate::app::core::kernel_service::status::kernel_check_health,
            crate::app::core::kernel_auto_manage::kernel_auto_manage,
            crate::app::core::kernel_service::runtime::apply_proxy_settings,
            // Network - Subscription service commands
            crate::app::network::subscription_service::download_subscription,
            crate::app::network::subscription_service::add_manual_subscription,
            crate::app::network::subscription_service::get_current_config,
            crate::app::network::subscription_service::set_active_config_path,
            crate::app::network::subscription_service::delete_subscription_config,
            crate::app::network::subscription_service::rollback_subscription_config,
            crate::app::network::subscription_service::toggle_proxy_mode,
            crate::app::network::subscription_service::get_current_proxy_mode,
            // System - System service commands
            crate::app::system::system_service::check_admin,
            crate::app::system::system_service::restart_as_admin,
            crate::app::system::system_service::check_network_connectivity,
            crate::app::system::system_service::wait_for_network_ready,
            crate::app::system::system_service::open_devtools,
            crate::app::system::backup_service::backup_export_snapshot,
            crate::app::system::backup_service::backup_import_snapshot,
            // System - Sudo service commands (Linux/macOS TUN 提权)
            crate::app::system::sudo_service::sudo_password_status,
            crate::app::system::sudo_service::sudo_set_password,
            crate::app::system::sudo_service::sudo_clear_password,
            // System - Update service commands
            crate::app::system::update_service::check_update,
            crate::app::system::update_service::download_update,
            crate::app::system::update_service::download_and_install_update,
            crate::app::system::update_service::get_platform_info,
            crate::app::system::update_service::get_detailed_platform_info,
            // System - Config service commands
            crate::app::system::config_service::update_singbox_ports,
            // Core - Proxy service commands
            crate::app::core::proxy_service::set_system_proxy,
            crate::app::core::proxy_service::set_manual_proxy,
            crate::app::core::proxy_service::set_tun_proxy,
            crate::app::core::proxy_service::toggle_ip_version,
            crate::app::core::proxy_service::get_api_token,
            crate::app::core::proxy_service::get_proxies,
            crate::app::core::proxy_service::get_proxy_providers,
            crate::app::core::proxy_service::change_proxy,
            crate::app::core::proxy_service::test_node_delay,
            crate::app::core::proxy_service::test_group_delay,
            crate::app::core::proxy_service::test_nodes_delay,
            crate::app::core::proxy_service::get_rules,
            crate::app::core::proxy_service::get_rule_providers,
            crate::app::core::proxy_service::update_proxy_provider,
            crate::app::core::proxy_service::update_rule_provider,
            crate::app::core::proxy_service::toggle_rule_disabled,
            crate::app::core::proxy_service::close_all_connections,
            crate::app::core::proxy_service::close_connection,
            // Tray commands
            crate::app::tray::commands::tray_sync_state,
            crate::app::tray::commands::tray_set_last_visible_route,
            crate::app::tray::commands::tray_show_main_window,
            crate::app::tray::commands::tray_hide_main_window,
            crate::app::tray::commands::tray_close_main_window,
            crate::app::tray::commands::tray_consume_pending_restore_route,
            crate::app::tray::commands::tray_consume_pending_proxy_toggle,
            crate::app::tray::commands::tray_request_app_exit,
        ])
        .build(tauri::generate_context!())
        .expect("error while building tauri application")
        .run(|_, event| {
            if let RunEvent::ExitRequested { api, .. } = event {
                if crate::app::tray::should_prevent_exit() {
                    tracing::info!("主窗口已销毁，保留托盘与后台任务，阻止应用退出");
                    api.prevent_exit();
                }
            }
        });
}

#[allow(dead_code)]
fn show_window(app: &AppHandle) {
    if let Err(err) = crate::app::tray::show_main_window(app, true) {
        tracing::warn!("通过托盘服务恢复窗口失败，回退为旧逻辑: {}", err);
        let windows = app.webview_windows();
        if let Some(window) = windows.values().next() {
            let _ = window.show();
            let _ = window.set_focus();
        }
    }
}
