use crate::app::kernel_service::{
    check_kernel_version, download_latest_kernel, get_memory_usage, get_process_status,
    restart_kernel, start_kernel, stop_kernel, start_websocket_relay
};
use crate::app::proxy_service::{
    set_system_proxy, set_tun_proxy, toggle_ip_version, get_proxies, change_proxy, 
    test_node_delay, batch_test_nodes, get_version_info, get_rules
};
use crate::app::subscription_service::{download_subscription, add_manual_subscription, get_current_config, toggle_proxy_mode, get_current_proxy_mode};
use crate::app::system_service::{check_admin, get_traffic_data, restart_as_admin};
use crate::app::update_service::{check_update, download_and_install_update};
use tauri::{AppHandle, Manager};
use tauri_plugin_autostart::MacosLauncher;

pub mod app;
pub mod config;
pub mod entity;
pub mod process;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_positioner::init())
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
            start_kernel,
            download_latest_kernel,
            download_subscription,
            add_manual_subscription,
            get_current_config,
            stop_kernel,
            set_system_proxy,
            set_tun_proxy,
            get_memory_usage,
            get_traffic_data,
            check_admin,
            restart_as_admin,
            restart_kernel,
            toggle_ip_version,
            check_update,
            download_and_install_update,
            get_process_status,
            check_kernel_version,
            toggle_proxy_mode,
            get_current_proxy_mode,
            get_proxies,
            change_proxy,
            test_node_delay,
            batch_test_nodes,
            get_version_info,
            get_rules,
            start_websocket_relay,
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
