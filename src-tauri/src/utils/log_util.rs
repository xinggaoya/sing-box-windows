use crate::app::log as log_constants;
use crate::utils::app_util::get_work_dir_sync;
use std::io;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tauri::async_runtime::JoinHandle;
use tracing::{info, warn};
use tracing_appender::{
    non_blocking::{self, WorkerGuard},
    rolling,
};
use tracing_subscriber::fmt::writer::MakeWriterExt;
use tracing_subscriber::{fmt, EnvFilter};

static FILE_GUARD: OnceLock<WorkerGuard> = OnceLock::new();
const CLEANUP_INTERVAL_HOURS: u64 = 24;
const RETENTION_DAYS: u64 = 1;

/// 初始化日志：控制台输出 + 按天轮转的文件输出
/// 返回日志目录，供后续定时清理使用
pub fn init_logging() -> Option<PathBuf> {
    let env_filter = build_env_filter();

    match prepare_log_dir()
        .and_then(|dir| create_file_writer(&dir).map(|writer| (dir, writer)))
    {
        Ok((log_dir, (writer, guard))) => {
            let combined_writer = writer.and(io::stdout);

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
                return None;
            }

            // 保存 guard，确保异步写入线程存活
            let _ = FILE_GUARD.set(guard);
            info!("日志写入目录：{}", log_dir.display());
            Some(log_dir)
        }
        Err(e) => {
            eprintln!("文件日志初始化失败，使用控制台日志: {e}");
            init_console_only(env_filter);
            None
        }
    }
}

/// 启动定时清理任务，按数量和单文件大小清理旧日志
pub fn spawn_log_cleanup_task(log_dir: PathBuf) -> JoinHandle<()> {
    tauri::async_runtime::spawn(async move {
        if let Err(e) = cleanup_once(log_dir.clone()).await {
            warn!("初始化日志清理任务失败: {}", e);
        }

        let mut interval = tokio::time::interval(Duration::from_secs(CLEANUP_INTERVAL_HOURS * 3600));
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
        std::env::set_var("RUST_LOG", "debug,sing_box_windows=debug,tauri=info,sqlx=warn");
        EnvFilter::from_default_env()
    })
}

fn prepare_log_dir() -> io::Result<PathBuf> {
    let base = PathBuf::from(get_work_dir_sync());
    let log_dir = base.join(log_constants::DEFAULT_DIR);
    std::fs::create_dir_all(&log_dir)?;
    Ok(log_dir)
}

fn create_file_writer(
    log_dir: &Path,
) -> io::Result<(non_blocking::NonBlocking, WorkerGuard)> {
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
    let max_file_size_bytes = log_constants::DEFAULT_MAX_FILE_SIZE * 1024 * 1024;

    let mut entries: Vec<(PathBuf, std::fs::Metadata)> = std::fs::read_dir(log_dir)?
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            if !path.is_file() {
                return None;
            }

            // 仅清理日志文件，避免误删其他文件
            if path.extension().and_then(|ext| ext.to_str()) != Some("log") {
                return None;
            }

            entry.metadata().ok().map(|metadata| (path, metadata))
        })
        .collect();

    let mut removed = 0usize;
    let retention = Duration::from_secs(RETENTION_DAYS * 24 * 3600);
    let now = SystemTime::now();
    entries.retain(|(path, metadata)| {
        if metadata.len() > max_file_size_bytes {
            if let Err(e) = std::fs::remove_file(path) {
                warn!("删除超大日志文件失败 {:?}: {}", path, e);
            } else {
                removed += 1;
            }
            false
        } else if let Ok(modified) = metadata.modified() {
            // 按天数保留，过期直接删除
            if now
                .duration_since(modified)
                .map(|age| age > retention)
                .unwrap_or(false)
            {
                if let Err(e) = std::fs::remove_file(path) {
                    warn!("删除过期日志失败 {:?}: {}", path, e);
                } else {
                    removed += 1;
                }
                false
            } else {
                true
            }
        } else {
            true
        }
    });

    entries.sort_by_key(|(_, metadata)| metadata.modified().unwrap_or(UNIX_EPOCH));
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
