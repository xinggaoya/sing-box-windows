use tauri_plugin_autostart::MacosLauncher;
use crate::app::app_service::{
    download_latest_kernel, download_subscription, start_kernel, stop_kernel,
};
use crate::app::app_service::{set_system_proxy, set_tun_proxy};

mod app;
mod entity;
mod utils;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_autostart::init(MacosLauncher::LaunchAgent, Some(vec!["--flag1", "--flag2"]) /* arbitrary number of args to pass to your app */))
        .setup(|app| {
            if cfg!(debug_assertions) {
                app.handle().plugin(
                    tauri_plugin_log::Builder::default()
                        .level(log::LevelFilter::Info)
                        .build(),
                )?;
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
