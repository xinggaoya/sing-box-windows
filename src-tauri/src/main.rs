// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use app_lib::utils::logger::init_logger;

fn main() {
    init_logger();
    app_lib::run();
}
