use crate::app::log as log_constants;
use crate::utils::app_util::get_work_dir_sync;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::async_runtime::JoinHandle;
use tracing::{info, warn};
use tracing_appender::{
    non_blocking::{self, NonBlocking, WorkerGuard},
    rolling,
};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::{fmt, EnvFilter};

static FILE_GUARD: OnceLock<WorkerGuard> = OnceLock::new();
static LOG_DIR: OnceLock<PathBuf> = OnceLock::new();
const CLEANUP_INTERVAL_HOURS: u64 = 24;

/// 日志 sink：非阻塞文件 writer + 所属目录
///
/// devtools 模式与直连模式共用同一份 sink，区别只在于「谁来写」。
pub struct LogSink {
    writer: NonBlocking,
    dir: PathBuf,
}

impl LogSink {
    /// 日志目录
    pub fn dir(&self) -> &Path {
        &self.dir
    }

    /// 非阻塞文件 writer
    pub fn writer(&self) -> &NonBlocking {
        &self.writer
    }
}

/// 在指定目录创建日志 sink
///
/// guard 交调用方持有：只有 drop guard 才会通知后台线程退出并刷盘
/// （`NonBlocking::flush` 是空操作）。
pub fn create_log_sink_in(dir: PathBuf) -> io::Result<(LogSink, WorkerGuard)> {
    let (writer, guard) = create_file_writer(&dir)?;
    Ok((LogSink { writer, dir }, guard))
}

/// 在默认工作目录创建日志 sink
///
/// guard 存入模块级 OnceLock，保证进程存活期写入线程不被回收。
pub fn create_log_sink() -> Option<LogSink> {
    let dir = match prepare_log_dir() {
        Ok(dir) => dir,
        Err(e) => {
            eprintln!("文件日志初始化失败，使用控制台日志: {e}");
            return None;
        }
    };

    let (sink, guard) = match create_log_sink_in(dir.clone()) {
        Ok(pair) => pair,
        Err(e) => {
            eprintln!("文件日志初始化失败，使用控制台日志: {e}");
            return None;
        }
    };

    // 保存 guard，确保异步写入线程存活
    let _ = FILE_GUARD.set(guard);
    let _ = LOG_DIR.set(dir);
    Some(sink)
}

/// 当前日志目录（供定时清理任务使用）
pub fn current_log_dir() -> Option<PathBuf> {
    LOG_DIR.get().cloned()
}

/// 直连模式：安装全局 fmt subscriber（文件 + 控制台）
///
/// devtools 模式下**不能**调用本函数——全局 subscriber 已被插件占用，
/// 再次设置会失败（`tracing_subscriber` 的全局默认只能设置一次）。
pub fn install_direct_subscriber(sink: Option<&LogSink>) {
    let env_filter = build_env_filter();

    let Some(sink) = sink else {
        init_console_only(env_filter);
        return;
    };

    let combined_writer = sink.writer().clone().and(io::stdout);

    if let Err(e) = fmt()
        .with_env_filter(env_filter.clone())
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .with_writer(combined_writer)
        .try_init()
    {
        eprintln!("初始化日志订阅器失败，回退到控制台日志: {e}");
        init_console_only(env_filter);
        return;
    }

    info!("日志写入目录：{}", sink.dir().display());
}

/// 启动定时清理任务，仅保留最新的日志文件
pub fn spawn_log_cleanup_task(log_dir: PathBuf) -> JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = cleanup_once(log_dir.clone()).await {
            warn!("初始化日志清理任务失败: {}", e);
        }

        let mut interval =
            tokio::time::interval(Duration::from_secs(CLEANUP_INTERVAL_HOURS * 3600));
        loop {
            interval.tick().await;
            if let Err(e) = cleanup_once(log_dir.clone()).await {
                warn!("定时清理日志失败: {}", e);
            }
        }
    })
}

fn build_env_filter() -> EnvFilter {
    EnvFilter::try_from_default_env().unwrap_or_else(|_| {
        // 缺省时启用调试日志，兼顾 sing-box 与 tauri
        // 关闭或降级部分第三方库日志，避免噪音（如 sqlx）
        std::env::set_var(
            "RUST_LOG",
            "debug,sing_box_windows=debug,tauri=info,sqlx=warn",
        );
        EnvFilter::from_default_env()
    })
}

fn prepare_log_dir() -> io::Result<PathBuf> {
    let base = PathBuf::from(get_work_dir_sync());
    let log_dir = base.join(log_constants::DEFAULT_DIR);
    std::fs::create_dir_all(&log_dir)?;
    Ok(log_dir)
}

fn create_file_writer(log_dir: &Path) -> io::Result<(non_blocking::NonBlocking, WorkerGuard)> {
    let file_name = format!("{}.log", log_constants::DEFAULT_FILE_PREFIX);

    // 只支持 hourly/daily/never，其他值回退为 daily，避免新增枚举导致非穷尽匹配
    let appender = match log_constants::rotation::DEFAULT {
        log_constants::rotation::HOURLY => rolling::hourly(log_dir, &file_name),
        log_constants::rotation::NEVER => rolling::never(log_dir, &file_name),
        _ => rolling::daily(log_dir, &file_name),
    };

    Ok(tracing_appender::non_blocking(appender))
}

fn init_console_only(env_filter: EnvFilter) {
    let _ = fmt()
        .with_env_filter(env_filter)
        .with_target(false)
        .with_thread_ids(true)
        .with_file(true)
        .with_line_number(true)
        .try_init();
}

async fn cleanup_once(log_dir: PathBuf) -> Result<(), String> {
    tokio::task::spawn_blocking(move || perform_cleanup(&log_dir))
        .await
        .map_err(|e| format!("清理任务 Join 失败: {e}"))?
        .map_err(|e| format!("清理日志出错: {e}"))
}

fn perform_cleanup(log_dir: &Path) -> io::Result<()> {
    if !log_dir.exists() {
        return Ok(());
    }

    let max_files = log_constants::DEFAULT_MAX_FILES as usize;
    let base_file_name = format!("{}.log", log_constants::DEFAULT_FILE_PREFIX);

    let mut entries: Vec<(PathBuf, SystemTime)> = std::fs::read_dir(log_dir)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }

            let file_name = path.file_name()?.to_string_lossy();
            // 仅清理 app.log 及其每日滚动文件（app.log.YYYY-MM-DD）
            if !file_name.starts_with(&base_file_name) {
                return None;
            }

            let modified = entry
                .metadata()
                .ok()
                .and_then(|metadata| metadata.modified().ok())
                .unwrap_or(UNIX_EPOCH);
            Some((path, modified))
        })
        .collect();

    let mut removed = 0usize;

    entries.sort_by_key(|(_, modified)| *modified);
    entries.reverse(); // 最新在前，方便按数量截断

    for (idx, (path, _)) in entries.iter().enumerate() {
        if idx >= max_files {
            if let Err(e) = std::fs::remove_file(path) {
                warn!("删除过期日志失败 {:?}: {}", path, e);
            } else {
                removed += 1;
            }
        }
    }

    if removed > 0 {
        info!("日志清理完成，删除 {} 个文件", removed);
    }

    Ok(())
}

#[cfg(test)]
#[path = "log_util.tests.rs"]
mod tests;
