//! devtools 模式下的日志桥接。
//!
//! `tauri-plugin-devtools` 会独占全局 tracing subscriber（`Builder::try_init` 内部
//! 调用 `tracing_subscriber::registry().with(...).try_init()`，失败即 panic）。
//! 因此 devtools 模式下由插件持有全局 subscriber，本模块负责把事件转成
//! 「文件 + 控制台」输出，通过官方 `Builder::attach_logger` 挂载回插件。
//!
//! 过滤语义对齐 `tracing_subscriber::EnvFilter`，避免两种日志模式行为相反：
//! - 指令串逗号分隔，支持 `target=level`、全局 `level`、裸 `target`（等价 `target=trace`）
//! - 任一指令非法 → 整个指令串解析失败（`EnvFilter::parse` 同款语义）
//! - target 前缀最长匹配优先

use chrono::Utc;
use std::io::Write;
use tracing_appender::non_blocking::NonBlocking;

/// 默认日志指令串，与 `log_util::build_env_filter` 的缺省值保持一致
pub const DEFAULT_LOG_DIRECTIVES: &str = "debug,sing_box_windows=debug,tauri=info,sqlx=warn";

/// 指令串解析失败
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogDirectiveParseError {
    /// 出错的指令片段
    pub directive: String,
}

impl std::fmt::Display for LogDirectiveParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "非法的日志指令: {}", self.directive)
    }
}

impl std::error::Error for LogDirectiveParseError {}

/// 单条日志指令：`target=level`、全局 `level` 或裸 `target`（等价 trace）
#[derive(Debug, Clone, PartialEq, Eq)]
struct Directive {
    target_prefix: Option<String>,
    level: log::LevelFilter,
}

/// RUST_LOG 风格指令过滤器
#[derive(Debug, Clone, Default)]
pub struct LogDirectiveFilter {
    directives: Vec<Directive>,
}

impl LogDirectiveFilter {
    /// 解析指令串
    ///
    /// 空串得到「不放行任何日志」的过滤器（与 `EnvFilter::parse("")` 一致）；
    /// 任一指令非法则整体失败（与 `EnvFilter::parse` 一致）。
    pub fn parse(spec: &str) -> Result<Self, LogDirectiveParseError> {
        let mut directives: Vec<Directive> = Vec::new();

        for item in spec.split(',') {
            let item = item.trim();
            if item.is_empty() {
                continue;
            }

            let directive = match item.split_once('=') {
                Some((target, level)) => {
                    let target = target.trim();
                    if target.is_empty() {
                        return Err(LogDirectiveParseError {
                            directive: item.to_string(),
                        });
                    }
                    let level =
                        parse_level_filter(level).ok_or_else(|| LogDirectiveParseError {
                            directive: item.to_string(),
                        })?;
                    Directive {
                        target_prefix: Some(target.to_string()),
                        level,
                    }
                }
                None => match parse_level_filter(item) {
                    Some(level) => Directive {
                        target_prefix: None,
                        level,
                    },
                    // 裸 token 不是合法级别 → 视为 target（`EnvFilter` 语义：等价 trace）
                    None => Directive {
                        target_prefix: Some(item.to_string()),
                        level: log::LevelFilter::Trace,
                    },
                },
            };

            // 同一 target 重复出现时后者覆盖前者（与 `EnvFilter` 一致）
            match &directive.target_prefix {
                Some(prefix) => match directives
                    .iter_mut()
                    .find(|existing| existing.target_prefix.as_deref() == Some(prefix.as_str()))
                {
                    Some(existing) => *existing = directive,
                    None => directives.push(directive),
                },
                None => match directives
                    .iter_mut()
                    .find(|existing| existing.target_prefix.is_none())
                {
                    Some(existing) => *existing = directive,
                    None => directives.push(directive),
                },
            }
        }

        Ok(Self { directives })
    }

    /// 解析指令串；失败时回退到默认指令串
    ///
    /// 避免出现「零指令 → 全静默」的状态（与 `build_env_filter` 的兜底一致）。
    pub fn parse_or_default(spec: &str) -> Self {
        match Self::parse(spec) {
            Ok(filter) => filter,
            Err(e) => {
                eprintln!("{e}，回退到默认日志指令: {DEFAULT_LOG_DIRECTIVES}");
                Self::parse(DEFAULT_LOG_DIRECTIVES).unwrap_or_default()
            }
        }
    }

    /// 从 `RUST_LOG` 读取过滤规则
    ///
    /// - 未设置 → 默认指令串（与 `build_env_filter` 一致）
    /// - 设置为空 → 不放行任何日志（与 `EnvFilter::parse("")` 一致）
    /// - 含非法指令 → 整串作废，回退默认指令串（与 `build_env_filter` 一致）
    pub fn from_env() -> Self {
        match std::env::var("RUST_LOG") {
            Ok(spec) => Self::parse_or_default(&spec),
            Err(_) => Self::parse_or_default(DEFAULT_LOG_DIRECTIVES),
        }
    }

    /// 判断某条日志是否应当输出
    pub fn enabled(&self, target: &str, level: log::Level) -> bool {
        match self.level_for(target) {
            Some(filter) => filter >= level,
            None => false,
        }
    }

    /// 解析目标生效的级别：最长前缀优先，其次全局指令
    fn level_for(&self, target: &str) -> Option<log::LevelFilter> {
        let mut matched: Option<&Directive> = None;
        for directive in &self.directives {
            let Some(prefix) = directive.target_prefix.as_deref() else {
                continue;
            };
            if !target.starts_with(prefix) {
                continue;
            }
            let better = match matched {
                None => true,
                Some(current) => {
                    prefix.len() > current.target_prefix.as_deref().map(str::len).unwrap_or(0)
                }
            };
            if better {
                matched = Some(directive);
            }
        }

        if let Some(directive) = matched {
            return Some(directive.level);
        }

        self.directives
            .iter()
            .find(|directive| directive.target_prefix.is_none())
            .map(|directive| directive.level)
    }
}

fn parse_level_filter(raw: &str) -> Option<log::LevelFilter> {
    match raw.trim().to_ascii_lowercase().as_str() {
        "off" => Some(log::LevelFilter::Off),
        "error" => Some(log::LevelFilter::Error),
        "warn" => Some(log::LevelFilter::Warn),
        "info" => Some(log::LevelFilter::Info),
        "debug" => Some(log::LevelFilter::Debug),
        "trace" => Some(log::LevelFilter::Trace),
        _ => None,
    }
}

/// 按 fmt 层风格拼装一行日志：`时间 级别 线程 文件:行号 消息`
pub fn format_log_line(
    timestamp: &str,
    level: log::Level,
    thread: &str,
    file: Option<&str>,
    line: Option<u32>,
    message: &str,
) -> String {
    let location = match (file, line) {
        (Some(file), Some(line)) => format!(" {file}:{line}"),
        (Some(file), None) => format!(" {file}"),
        _ => String::new(),
    };

    format!("{timestamp} {level:>5} {thread}{location} {message}")
}

/// 把 `log::Record` 格式化成一行日志
///
/// 时间戳与 fmt 层（`fmt::time::SystemTime`）保持一致：UTC RFC3339、微秒精度、带 `Z`。
pub fn format_record_line(record: &log::Record) -> String {
    let timestamp = Utc::now().format("%Y-%m-%dT%H:%M:%S%.6fZ").to_string();
    let thread = format!("{:?}", std::thread::current().id());

    format_log_line(
        &timestamp,
        record.level(),
        &thread,
        record.file(),
        record.line(),
        &record.args().to_string(),
    )
}

/// 日志写入载荷：行内容 + 结尾换行
///
/// 换行必须与行内容**一次性**写入：`tracing_appender` 的 `NonBlocking` 是 lossy 通道，
/// 每次 `write_all` 独立投递一条消息，拆开写时换行消息可能被单独丢弃，
/// 导致日志文件出现行粘连。
pub fn render_log_payload(line: &str) -> String {
    let mut payload = String::with_capacity(line.len() + 1);
    payload.push_str(line);
    payload.push('\n');
    payload
}

/// devtools 模式下的日志写入器
///
/// 插件的 `BridgeLayer` 会把 tracing 事件转成 `log::Record` 转发到这里，
/// 因此文件与控制台输出与 DevTools 面板的数据同源。
pub struct FileConsoleLogger {
    /// 文件写入器；为 `None` 时只输出到控制台（文件 sink 创建失败时的降级）
    writer: Option<NonBlocking>,
    filter: LogDirectiveFilter,
}

impl FileConsoleLogger {
    /// 用非阻塞文件 writer + 过滤器构造
    pub fn new(writer: NonBlocking, filter: LogDirectiveFilter) -> Self {
        Self {
            writer: Some(writer),
            filter,
        }
    }

    /// 仅控制台输出（文件 sink 创建失败时的降级）
    pub fn console_only(filter: LogDirectiveFilter) -> Self {
        Self {
            writer: None,
            filter,
        }
    }
}

impl log::Log for FileConsoleLogger {
    fn enabled(&self, metadata: &log::Metadata) -> bool {
        self.filter.enabled(metadata.target(), metadata.level())
    }

    fn log(&self, record: &log::Record) {
        if !self.enabled(record.metadata()) {
            return;
        }

        let line = format_record_line(record);

        if let Some(writer) = &self.writer {
            // `log::Log::log` 只拿到 `&self`，而 `NonBlocking` 的写接口需要 `&mut`，
            // 克隆一份写入（克隆共享同一条 channel）；行内容与换行一次性投递
            let mut writer = writer.clone();
            let _ = writer.write_all(render_log_payload(&line).as_bytes());
        }

        // 写终端失败不应 panic（日志回调里的 panic 会冒泡进业务代码）
        let mut stdout = std::io::stdout();
        let _ = writeln!(stdout, "{line}");
    }

    fn flush(&self) {
        if let Some(writer) = &self.writer {
            let mut writer = writer.clone();
            let _ = writer.flush();
        }
    }
}

#[cfg(test)]
#[path = "log_bridge.tests.rs"]
mod tests;
