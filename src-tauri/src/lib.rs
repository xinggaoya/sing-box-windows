// Core services imports
use crate::app::core::kernel_service::{
    check_kernel_version, download_latest_kernel, restart_kernel, start_kernel,
    start_websocket_relay, stop_kernel,
};
use crate::app::core::proxy_service::{
    change_proxy, get_api_token, get_proxies, get_rules, get_version_info, set_manual_proxy,
    set_system_proxy, set_tun_proxy, test_group_delay, test_node_delay, toggle_ip_version,
};

// Network services imports
use crate::app::network::subscription_service::{
    add_manual_subscription, download_subscription, get_current_config, get_current_proxy_mode,
    toggle_proxy_mode,
};

// System services imports
use crate::app::system::system_service::{
    check_admin, exit_application, restart_as_admin,
};
use crate::app::system::update_service::{check_update, download_and_install_update};
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::MacosLauncher;
use tracing_subscriber::{fmt, EnvFilter};

pub mod app;
pub mod entity;
pub mod process;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    fmt().with_env_filter(EnvFilter::from_default_env()).init();

    tauri::Builder::default()
        .plugin(tauri_plugin_store::Builder::new().build())
        .plugin(tauri_plugin_websocket::init())
        .plugin(tauri_plugin_single_instance::init(|app, _args, _cwd| {
            let _ = show_window(app);
        }))
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hide"]),
        ))
        .setup(|app| {
            // if cfg!(debug_assertions) {
            //     app.handle().plugin(
            //         tauri_plugin_log::Builder::default()
            //             .level("info")
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
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            // Core - Kernel service commands
            start_kernel,
            stop_kernel,
            restart_kernel,
            download_latest_kernel,
            check_kernel_version,
            start_websocket_relay,
            // Core - Proxy service commands
            set_system_proxy,
            set_tun_proxy,
            set_manual_proxy,
            toggle_ip_version,
            get_proxies,
            change_proxy,
            test_group_delay,
            test_node_delay,
            get_version_info,
            get_rules,
            get_api_token,
            // Network - Subscription service commands
            download_subscription,
            add_manual_subscription,
            get_current_config,
            toggle_proxy_mode,
            get_current_proxy_mode,
            // System service commands
            check_admin,
            restart_as_admin,
            exit_application,
            // Update service commands
            check_update,
            download_and_install_update,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

fn show_window(app: &AppHandle) {
    let windows = app.webview_windows();

    windows
        .values()
        .next()
        .expect("Sorry, no window found")
        .set_focus()
        .expect("Can't Bring Window to Focus");
}
