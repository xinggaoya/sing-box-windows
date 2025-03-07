use tracing_appender::{
    non_blocking::WorkerGuard,
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    fmt,
    layer::SubscriberExt,
    Layer,
    filter::LevelFilter,
    EnvFilter, Registry,
};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Once;
use std::sync::Mutex;
use once_cell::sync::Lazy;
use crate::app::constants::log;

use crate::config::LogConfig;

// 静态标志，用于确保只初始化一次
static LOGGER_INITIALIZED: AtomicBool = AtomicBool::new(false);
static INIT_ONCE: Once = Once::new();

// 静态存储所有WorkerGuard，确保它们在程序整个生命周期内不被丢弃
static GLOBAL_GUARDS: Lazy<Mutex<Vec<WorkerGuard>>> = Lazy::new(|| Mutex::new(Vec::new()));

pub struct Logger;

impl Logger {
    // 初始化日志系统
    pub fn init(config: &LogConfig) -> Self {
        // 使用原子操作和Once确保只初始化一次
        let already_initialized = LOGGER_INITIALIZED.load(Ordering::Relaxed);
        if already_initialized {
            return Self;
        }
        
        // 使用Once确保初始化代码只执行一次
        INIT_ONCE.call_once(|| {
            // 设置原子标志
            LOGGER_INITIALIZED.store(true, Ordering::Relaxed);
            
            // 确保日志目录存在
            std::fs::create_dir_all(&config.dir).expect("无法创建日志目录");

            // 设置默认日志级别
            if std::env::var("RUST_LOG").is_err() {
                std::env::set_var("RUST_LOG", &config.level);
            }

            // 获取日志级别设置
            let env_filter_string = std::env::var("RUST_LOG").unwrap_or_else(|_| config.level.clone());

            // 配置文件输出轮换
            let rotation = match config.rotation.as_str() {
                log::rotation::HOURLY => Rotation::HOURLY,
                log::rotation::DAILY => Rotation::DAILY,
                log::rotation::NEVER => Rotation::NEVER,
                _ => Rotation::DAILY, // 默认每天轮转
            };
            
            // 创建临时guard容器
            let mut guards = Vec::new();

            // 创建全局日志文件，用于记录所有级别的日志
            let all_appender = RollingFileAppender::builder()
                .rotation(rotation.clone())
                .filename_prefix(&format!("{}-all", &config.file_name_prefix))
                .max_log_files(config.max_files as usize)
                .build(&config.dir)
                .expect("创建全局日志appender失败");
            
            // 配置更短的刷新间隔，使日志更快写入文件
            let (all_non_blocking, all_guard) = tracing_appender::non_blocking(all_appender);
            guards.push(all_guard);

            // 创建错误日志文件
            let error_appender = RollingFileAppender::builder()
                .rotation(rotation.clone())
                .filename_prefix(&format!("{}-error", &config.file_name_prefix))
                .max_log_files(config.max_files as usize)
                .build(&config.dir)
                .expect("创建错误日志appender失败");
            
            let (error_non_blocking, error_guard) = tracing_appender::non_blocking(error_appender);
            guards.push(error_guard);

            // 创建警告日志文件
            let warn_appender = RollingFileAppender::builder()
                .rotation(rotation.clone())
                .filename_prefix(&format!("{}-warn", &config.file_name_prefix))
                .max_log_files(config.max_files as usize)
                .build(&config.dir)
                .expect("创建警告日志appender失败");
            
            let (warn_non_blocking, warn_guard) = tracing_appender::non_blocking(warn_appender);
            guards.push(warn_guard);

            // 创建信息日志文件
            let info_appender = RollingFileAppender::builder()
                .rotation(rotation)
                .filename_prefix(&format!("{}-info", &config.file_name_prefix))
                .max_log_files(config.max_files as usize)
                .build(&config.dir)
                .expect("创建信息日志appender失败");
            
            let (info_non_blocking, info_guard) = tracing_appender::non_blocking(info_appender);
            guards.push(info_guard);

            // 创建全局订阅器
            let registry = Registry::default()
                // 控制台输出
                .with(
                    fmt::layer()
                        .with_ansi(true)
                        .with_line_number(true)
                        .with_thread_ids(true)
                        .with_target(true)
                        .with_writer(std::io::stdout)
                        .with_filter(EnvFilter::new(&env_filter_string))
                )
                // 全局日志文件 - 包含所有级别
                .with(
                    fmt::layer()
                        .with_ansi(false)
                        .with_line_number(true)
                        .with_thread_ids(true)
                        .with_target(true)
                        .with_file(true)
                        .with_writer(all_non_blocking)
                        .with_filter(EnvFilter::new(&env_filter_string))
                )
                // 错误日志文件 - 只有ERROR级别
                .with(
                    fmt::layer()
                        .with_ansi(false)
                        .with_line_number(true)
                        .with_thread_ids(true)
                        .with_target(true)
                        .with_file(true)
                        .with_writer(error_non_blocking)
                        .with_filter(LevelFilter::ERROR)
                )
                // 警告日志文件 - 只有WARN级别
                .with(
                    fmt::layer()
                        .with_ansi(false)
                        .with_line_number(true)
                        .with_thread_ids(true)
                        .with_target(true)
                        .with_file(true)
                        .with_writer(warn_non_blocking)
                        .with_filter(LevelFilter::WARN)
                )
                // 信息日志文件 - 只有INFO级别
                .with(
                    fmt::layer()
                        .with_ansi(false)
                        .with_line_number(true)
                        .with_thread_ids(true)
                        .with_target(true)
                        .with_file(true)
                        .with_writer(info_non_blocking)
                        .with_filter(LevelFilter::INFO)
                );

            // 设置全局默认订阅器
            tracing::subscriber::set_global_default(registry)
                .expect("设置全局日志订阅器失败");


            
            // 将guards移动到全局静态存储中
            if let Ok(mut global_guards) = GLOBAL_GUARDS.lock() {
                global_guards.extend(guards);
            } else {
                eprintln!("警告: 无法锁定全局logger guards，日志可能无法正确写入文件");
            }
        });

        Self
    }
}

// 提供一个便捷的初始化方法
pub fn init_logger() -> Logger {
    Logger::init(&LogConfig::default())
}
