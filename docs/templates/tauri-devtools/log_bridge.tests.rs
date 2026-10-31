// ============================================================================
// 模板：日志桥接模块的单元测试（与 log_bridge.rs 配套）
//
// 复制到 `<crate>/src/utils/log_bridge.tests.rs`，
// 并确认 log_bridge.rs 末尾有：
//     #[cfg(test)]
//     #[path = "log_bridge.tests.rs"]
//     mod tests;
//
// 覆盖的语义（与 EnvFilter 对齐）：
//   - 未匹配 target 回落到全局级别 / target 指令覆盖全局级别 / 最长前缀优先
//   - 空指令串不放行 / 非法指令整串失败 / off 级别全关
//   - 裸 target 视为 trace / 同 target 重复时后者覆盖前者 / 非法串回退默认
//   - 行格式、UTC 时间戳、载荷单换行
//   - 桥接写文件：一条记录一行、过滤生效、文件 sink 缺失时仅终端
// ============================================================================

use super::*;
use log::Log;

#[test]
fn unmatched_target_falls_back_to_global_level() {
    let filter = LogDirectiveFilter::parse(DEFAULT_LOG_DIRECTIVES).expect("默认指令串应合法");

    assert!(filter.enabled("other_crate::module", log::Level::Debug));
    assert!(!filter.enabled("other_crate::module", log::Level::Trace));
}

#[test]
fn target_directive_overrides_global_level() {
    let filter = LogDirectiveFilter::parse(DEFAULT_LOG_DIRECTIVES).expect("默认指令串应合法");

    assert!(filter.enabled("tauri::app::run", log::Level::Info));
    assert!(!filter.enabled("tauri::app::run", log::Level::Debug));
    assert!(filter.enabled("sqlx::query", log::Level::Warn));
    assert!(!filter.enabled("sqlx::query", log::Level::Info));
    assert!(filter.enabled("sing_box_windows::app::core", log::Level::Debug));
}

#[test]
fn longest_prefix_wins() {
    let filter = LogDirectiveFilter::parse("sing_box_windows=info,sing_box_windows::app=trace")
        .expect("指令串应合法");

    // 命中更长的 `sing_box_windows::app`（trace）而非更短的 `sing_box_windows`（info）
    assert!(filter.enabled("sing_box_windows::app::core", log::Level::Debug));
    assert!(filter.enabled("sing_box_windows::app::core", log::Level::Trace));
    // 未命中更长前缀时回落到短前缀（info）
    assert!(filter.enabled("sing_box_windows::other", log::Level::Info));
    assert!(!filter.enabled("sing_box_windows::other", log::Level::Debug));
}

#[test]
fn empty_spec_disables_all_output() {
    // 与 `EnvFilter::parse("")` 一致：空指令串不放行任何日志
    let filter = LogDirectiveFilter::parse("").expect("空串不报错");

    assert!(!filter.enabled("any", log::Level::Error));
}

#[test]
fn invalid_level_directive_fails_whole_spec() {
    // 与 `EnvFilter::parse` 一致：任一指令非法 → 整个指令串解析失败
    let result = LogDirectiveFilter::parse("foo=bar,warn");

    assert!(result.is_err());
}

#[test]
fn off_directive_disables_all_output() {
    let filter = LogDirectiveFilter::parse("off").expect("指令串应合法");

    assert!(!filter.enabled("any", log::Level::Error));
}

#[test]
fn bare_target_directive_is_treated_as_trace() {
    // 与 `EnvFilter` 一致：裸 token 不是合法级别时视为 target，级别等价 trace
    let filter = LogDirectiveFilter::parse("my_crate").expect("指令串应合法");

    assert!(filter.enabled("my_crate::module", log::Level::Trace));
    assert!(filter.enabled("my_crate::module", log::Level::Debug));
    assert!(!filter.enabled("other_crate::module", log::Level::Error));
}

#[test]
fn duplicate_target_directive_last_wins() {
    // 与 `EnvFilter` 一致：同一 target 重复出现时后者覆盖前者
    let filter = LogDirectiveFilter::parse("my_crate=info,my_crate=debug").expect("指令串应合法");

    assert!(filter.enabled("my_crate::module", log::Level::Debug));
}

#[test]
fn parse_or_default_falls_back_when_spec_is_invalid() {
    // 非法指令串 → 回退默认指令（与 `build_env_filter` 的兜底一致），
    // 避免出现「零指令 → 全静默」的状态
    let filter = LogDirectiveFilter::parse_or_default("sqlx=verbose");

    assert!(filter.enabled("other_crate::module", log::Level::Debug));
    assert!(!filter.enabled("tauri::app::run", log::Level::Debug));
}

#[test]
fn format_log_line_matches_fmt_layer_style() {
    let line = format_log_line(
        "2026-10-30T16:06:12.345678Z",
        log::Level::Info,
        "ThreadId(1)",
        Some("core.rs"),
        Some(42),
        "内核已启动",
    );

    assert_eq!(
        line,
        "2026-10-30T16:06:12.345678Z  INFO ThreadId(1) core.rs:42 内核已启动"
    );
}

#[test]
fn format_log_line_omits_location_when_missing() {
    let line = format_log_line(
        "2026-10-30T16:06:12.345678Z",
        log::Level::Error,
        "ThreadId(2)",
        None,
        None,
        "失败",
    );

    assert_eq!(line, "2026-10-30T16:06:12.345678Z ERROR ThreadId(2) 失败");
}

#[test]
fn format_record_line_includes_file_line_and_level() {
    let args = format_args!("内核已启动");
    let record = log::Record::builder()
        .args(args)
        .level(log::Level::Info)
        .target("sing_box_windows::app")
        .file(Some("core.rs"))
        .line(Some(42))
        .build();

    let line = format_record_line(&record);

    assert!(line.contains(" INFO "));
    assert!(line.contains("core.rs:42"));
    assert!(line.ends_with("内核已启动"));
}

#[test]
fn format_record_line_uses_utc_timestamp() {
    // 与 fmt 层（`fmt::time::SystemTime`）一致：UTC RFC3339，微秒精度，带 Z 后缀
    let args = format_args!("时间戳");
    let record = log::Record::builder()
        .args(args)
        .level(log::Level::Info)
        .target("sing_box_windows::app")
        .build();

    let line = format_record_line(&record);
    let timestamp = line.split_whitespace().next().unwrap_or_default();

    assert!(timestamp.ends_with('Z'), "时间戳应为 UTC：{timestamp}");
    assert_eq!(timestamp.len(), "2026-10-30T16:06:12.345678Z".len());
}

#[test]
fn render_log_payload_ends_with_single_newline() {
    let payload = render_log_payload("2026-10-30T16:06:12.345678Z  INFO 行内容");

    assert!(payload.ends_with('\n'));
    assert_eq!(payload.matches('\n').count(), 1);
    assert!(payload.starts_with("2026-10-30T16:06:12.345678Z  INFO 行内容"));
}

#[test]
fn bridge_logger_writes_to_file() {
    let dir = std::env::temp_dir().join(format!("sbw-bridge-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let (sink, guard) =
        crate::utils::log_util::create_log_sink_in(dir.clone()).expect("应创建日志 sink");
    let logger = FileConsoleLogger::new(
        sink.writer().clone(),
        LogDirectiveFilter::parse("info,sing_box_windows=debug").expect("指令串应合法"),
    );

    let args = format_args!("桥接输出");
    let record = log::Record::builder()
        .args(args)
        .level(log::Level::Info)
        .target("sing_box_windows::app")
        .file(Some("core.rs"))
        .line(Some(7))
        .build();
    logger.log(&record);

    drop(guard);

    let entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("日志目录应存在")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(entries.len(), 1);

    let content = std::fs::read_to_string(entries[0].path()).expect("日志文件应可读");
    assert!(content.contains("桥接输出"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn bridge_logger_writes_one_line_per_record() {
    let dir = std::env::temp_dir().join(format!("sbw-bridge-lines-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let (sink, guard) =
        crate::utils::log_util::create_log_sink_in(dir.clone()).expect("应创建日志 sink");
    let logger = FileConsoleLogger::new(
        sink.writer().clone(),
        LogDirectiveFilter::parse("info").expect("指令串应合法"),
    );

    for index in 0..50 {
        let args = format_args!("第 {index} 条");
        let record = log::Record::builder()
            .args(args)
            .level(log::Level::Info)
            .target("sing_box_windows::app")
            .build();
        logger.log(&record);
    }

    drop(guard);

    let entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("日志目录应存在")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(entries.len(), 1);

    let content = std::fs::read_to_string(entries[0].path()).expect("日志文件应可读");
    let lines: Vec<&str> = content.lines().filter(|line| !line.is_empty()).collect();
    assert_eq!(lines.len(), 50);
    assert!(lines[0].contains("第 0 条"));
    assert!(lines[49].contains("第 49 条"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn bridge_logger_respects_filter() {
    let dir = std::env::temp_dir().join(format!("sbw-bridge-filter-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let (sink, guard) =
        crate::utils::log_util::create_log_sink_in(dir.clone()).expect("应创建日志 sink");
    let logger = FileConsoleLogger::new(
        sink.writer().clone(),
        LogDirectiveFilter::parse("warn").expect("指令串应合法"),
    );

    let args = format_args!("不该出现");
    let record = log::Record::builder()
        .args(args)
        .level(log::Level::Info)
        .target("any")
        .build();

    assert!(!logger.enabled(record.metadata()));

    // 被过滤掉的记录不应落盘
    logger.log(&record);
    drop(guard);

    let entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("日志目录应存在")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(entries.len(), 1);

    let content = std::fs::read_to_string(entries[0].path()).expect("日志文件应可读");
    assert!(!content.contains("不该出现"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn console_only_logger_keeps_working_without_file_sink() {
    // 文件 sink 创建失败时的降级路径：至少还能输出到终端
    let logger =
        FileConsoleLogger::console_only(LogDirectiveFilter::parse("info").expect("指令串应合法"));

    let args = format_args!("仅终端");
    let record = log::Record::builder()
        .args(args)
        .level(log::Level::Info)
        .target("sing_box_windows::app")
        .build();

    assert!(logger.enabled(record.metadata()));
    logger.log(&record);
}
