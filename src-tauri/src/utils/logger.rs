use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    fmt::{self, format::FmtSpan},
    layer::SubscriberExt,
    util::SubscriberInitExt,
    EnvFilter, Layer,
};

use crate::config::LogConfig;

pub struct Logger {
    _guard: WorkerGuard,
}

impl Logger {
    pub fn init(config: &LogConfig) -> Self {
        // 确保日志目录存在
        std::fs::create_dir_all(&config.dir).expect("Failed to create log directory");

        // 设置默认日志级别
        if std::env::var("RUST_LOG").is_err() {
            std::env::set_var("RUST_LOG", &config.level);
        }

        // 配置文件输出
        let rotation = match config.rotation.as_str() {
            "hourly" => Rotation::HOURLY,
            "daily" => Rotation::DAILY,
            "never" => Rotation::NEVER,
            _ => Rotation::DAILY, // 默认每天轮转
        };

        let file_appender = RollingFileAppender::builder()
            .rotation(rotation)
            .filename_prefix(&config.file_name_prefix)
            .max_log_files(config.max_files as usize)
            .build(&config.dir)
            .expect("Failed to create file appender");

        let (non_blocking, guard) = tracing_appender::non_blocking(file_appender);

        // 创建环境过滤器
        let stdout_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));
        let file_filter =
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new(&config.level));

        // 注册所有日志层
        tracing_subscriber::registry()
            .with(
                // 控制台输出层 - 使用彩色和人类可读的格式
                fmt::layer()
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_line_number(true)
                    .with_file(true)
                    .with_ansi(true) // 启用彩色输出
                    .with_writer(std::io::stdout)
                    .with_span_events(FmtSpan::ACTIVE) // 记录span的生命周期事件
                    .with_filter(stdout_filter),
            )
            .with(
                // 文件输出层 - 使用JSON格式
                fmt::layer()
                    .json() // 使用JSON格式
                    .with_target(true)
                    .with_thread_ids(true)
                    .with_line_number(true)
                    .with_file(true)
                    .with_ansi(false) // 禁用彩色输出
                    .with_current_span(true) // 包含当前span信息
                    .with_span_list(true) // 包含span列表
                    .with_writer(non_blocking)
                    .with_filter(file_filter),
            )
            .init();

        Self { _guard: guard }
    }
}

// 提供一个便捷的初始化方法
pub fn init_logger() -> Logger {
    Logger::init(&LogConfig::default())
}
