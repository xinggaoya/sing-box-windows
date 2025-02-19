use crate::app::app_service::{
    check_admin, download_latest_kernel, download_subscription, get_memory_usage, get_traffic_data,
    restart_as_admin, set_system_proxy, set_tun_proxy, start_kernel, stop_kernel,
};
// use lazy_static::lazy_static;
use tauri::Manager;
use tauri_plugin_autostart::MacosLauncher;

pub mod app;
pub mod entity;
pub mod process;
pub mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hide"]),
        ))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
            }
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
            stop_kernel,
            set_system_proxy,
            set_tun_proxy,
            get_memory_usage,
            get_traffic_data,
            check_admin,
            restart_as_admin,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
