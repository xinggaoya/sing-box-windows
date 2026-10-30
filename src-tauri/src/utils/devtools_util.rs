//! CrabNebula DevTools 插件装配。
//!
//! 仅在 cargo feature `devtools` 开启时编译（见 `src-tauri/Cargo.toml`）。
//!
//! 插件必须在安装任何全局 tracing subscriber **之前**初始化：
//! `tauri_plugin_devtools::Builder::try_init` 会调用
//! `tracing_subscriber::registry().with(...).try_init()` 抢占全局 subscriber，
//! 已被占用时 `init()` 会 panic。

use crate::utils::log_bridge::{FileConsoleLogger, LogDirectiveFilter};
use crate::utils::log_util::LogSink;
use tauri::plugin::TauriPlugin;

/// 初始化 devtools 插件与 devtools-app（Embedded 形态）插件
///
/// 返回 `(probe 插件, app 插件)`，调用方负责 `.plugin(...)` 注册。
/// 文件与控制台日志通过 `attach_logger` 桥接回本进程，
/// 避免与插件持有的全局 tracing subscriber 冲突。
pub fn init(sink: Option<LogSink>) -> (TauriPlugin<tauri::Wry>, TauriPlugin<tauri::Wry>) {
    let logger = sink.map(|sink| {
        let filter = LogDirectiveFilter::from_env();
        Box::new(FileConsoleLogger::new(sink.writer().clone(), filter)) as Box<dyn log::Log>
    });

    let mut builder = tauri_plugin_devtools::Builder::default();
    if let Some(logger) = logger {
        builder.attach_logger(logger);
    }

    let probe = builder.init();
    let app = tauri_plugin_devtools_app::init();

    (probe, app)
}
