//! devtools 模式下的日志桥接。
//!
//! `tauri-plugin-devtools` 会独占全局 tracing subscriber（`Builder::try_init` 内部
//! 调用 `tracing_subscriber::registry().with(...).try_init()`，失败即 panic）。
//! 因此 devtools 模式下由插件持有全局 subscriber，本模块负责把事件转成
//! 「文件 + 控制台」输出，通过官方 `Builder::attach_logger` 挂载回插件。

use chrono::Local;

/// 默认日志指令串，与 `log_util::build_env_filter` 的缺省值保持一致
pub const DEFAULT_LOG_DIRECTIVES: &str = "debug,sing_box_windows=debug,tauri=info,sqlx=warn";

/// 单条日志指令：`target=level` 或仅 `level`
#[derive(Debug, Clone, PartialEq, Eq)]
struct Directive {
    target_prefix: Option<String>,
    level: log::LevelFilter,
}

/// RUST_LOG 风格指令过滤器
///
/// 匹配规则（简化自 `tracing_subscriber::EnvFilter`）：
/// - 命中 target 前缀的指令中，**最长前缀**优先
/// - 没有命中时回落到「无 target 的全局指令」
/// - 没有任何可用指令时一律不放行
#[derive(Debug, Clone, Default)]
pub struct LogDirectiveFilter {
    directives: Vec<Directive>,
}

impl LogDirectiveFilter {
    /// 解析逗号分隔的指令串；非法级别指令被忽略
    pub fn parse(spec: &str) -> Self {
        let directives = spec
            .split(',')
            .filter_map(|item| {
                let item = item.trim();
                if item.is_empty() {
                    return None;
                }
                match item.split_once('=') {
                    Some((target, level)) => {
                        let level = parse_level_filter(level)?;
                        let target = target.trim();
                        if target.is_empty() {
                            None
                        } else {
                            Some(Directive {
                                target_prefix: Some(target.to_string()),
                                level,
                            })
                        }
                    }
                    None => {
                        let level = parse_level_filter(item)?;
                        Some(Directive {
                            target_prefix: None,
                            level,
                        })
                    }
                }
            })
            .collect();

        Self { directives }
    }

    /// 读取 `RUST_LOG`；未设置或为空白时使用默认指令串
    pub fn from_env() -> Self {
        let spec = std::env::var("RUST_LOG")
            .ok()
            .filter(|value| !value.trim().is_empty())
            .unwrap_or_else(|| DEFAULT_LOG_DIRECTIVES.to_string());
        Self::parse(&spec)
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
pub fn format_record_line(record: &log::Record) -> String {
    let timestamp = Local::now().format("%Y-%m-%dT%H:%M:%S%.3f").to_string();
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

#[cfg(test)]
#[path = "log_bridge.tests.rs"]
mod tests;
