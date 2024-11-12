use tauri::{Manager, Window};
use crate::app::app_service::{
    download_latest_kernel, download_subscription, start_kernel, stop_kernel,
};
use crate::app::app_service::{set_system_proxy, set_tun_proxy};
use tauri_plugin_autostart::MacosLauncher;

mod app;
mod entity;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_window_state::Builder::new().build())
        .plugin(tauri_plugin_autostart::init(
            MacosLauncher::LaunchAgent,
            Some(vec!["--hide"]), /* arbitrary number of args to pass to your app */
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
            set_tun_proxy
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
