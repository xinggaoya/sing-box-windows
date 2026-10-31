// ============================================================================
// 模板：Tauri 2 项目的 CrabNebula DevTools 插件装配
//
// 复制到 `<crate>/src/utils/devtools_util.rs` 后：
//   1. 该模块必须用 `#[cfg(feature = "devtools")]` 门控（否则 release 也编译插件）
//   2. 依赖：`crate::utils::log_bridge::{FileConsoleLogger, LogDirectiveFilter}`
//            `crate::utils::log_util::LogSink`（按你的项目改名/替换）
//   3. `LogSink` 只需提供「日志目录 + 一个 tracing_appender::non_blocking::NonBlocking writer」
//
// 关键点：用 `try_init()` 而非 `init()`——init 内部 unwrap，抢占全局 subscriber
//         失败会 panic；try_init 失败可回退到直连日志模式，保证应用照常启动。
// ============================================================================

//! CrabNebula DevTools 插件装配。
//!
//! 仅在 cargo feature `devtools` 开启时编译（见 `src-tauri/Cargo.toml`）。
//!
//! 插件必须在安装任何全局 tracing subscriber **之前**初始化：
//! `tauri_plugin_devtools::Builder::try_init` 会调用
//! `tracing_subscriber::registry().with(...).try_init()` 抢占全局 subscriber，
//! 已被占用时 `init()`（内部 unwrap）会 panic。

use crate::utils::log_bridge::{FileConsoleLogger, LogDirectiveFilter};
use crate::utils::log_util::LogSink;
use tauri::plugin::TauriPlugin;

/// devtools 初始化结果
pub enum DevtoolsInit {
    /// 插件就绪，调用方负责注册这两个插件
    Ready {
        /// 采集端插件（gRPC 服务 + 全局 tracing subscriber）
        probe: Box<TauriPlugin<tauri::Wry>>,
        /// 内嵌面板插件（Embedded 形态）
        app: Box<TauriPlugin<tauri::Wry>>,
    },
    /// 插件初始化失败，已回退到直连日志模式；sink 交回调用方
    Fallback {
        /// 原日志 sink
        sink: Option<LogSink>,
    },
}

/// 初始化 devtools 插件与 devtools-app（Embedded 形态）插件
///
/// 文件与控制台日志通过 `attach_logger` 桥接回本进程，
/// 避免与插件持有的全局 tracing subscriber 冲突。
pub fn init(sink: Option<LogSink>) -> DevtoolsInit {
    let filter = LogDirectiveFilter::from_env();

    // 无论文件 sink 是否可用都要挂 logger：否则 BridgeLayer 里没有任何接收方，
    // 开发态会一条日志都不输出
    let logger: Box<dyn log::Log> = match sink.as_ref() {
        Some(sink) => Box::new(FileConsoleLogger::new(sink.writer().clone(), filter)),
        None => Box::new(FileConsoleLogger::console_only(filter)),
    };

    let mut builder = tauri_plugin_devtools::Builder::default();
    builder.attach_logger(logger);

    // 用 try_init 而非 init：init 内部是 unwrap，抢占全局 subscriber 失败会 panic，
    // 降级为「面板不可用但应用照常启动」
    match builder.try_init() {
        Ok(probe) => {
            let app = tauri_plugin_devtools_app::init();
            DevtoolsInit::Ready {
                probe: Box::new(probe),
                app: Box::new(app),
            }
        }
        Err(e) => {
            eprintln!("初始化 CrabNebula DevTools 失败，回退到直连日志模式: {e}");
            DevtoolsInit::Fallback { sink }
        }
    }
}
