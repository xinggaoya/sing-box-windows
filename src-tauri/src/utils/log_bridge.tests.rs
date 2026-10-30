use super::*;
use log::Log;

#[test]
fn unmatched_target_falls_back_to_global_level() {
    let filter = LogDirectiveFilter::parse(DEFAULT_LOG_DIRECTIVES);

    assert!(filter.enabled("other_crate::module", log::Level::Debug));
    assert!(!filter.enabled("other_crate::module", log::Level::Trace));
}

#[test]
fn target_directive_overrides_global_level() {
    let filter = LogDirectiveFilter::parse(DEFAULT_LOG_DIRECTIVES);

    assert!(filter.enabled("tauri::app::run", log::Level::Info));
    assert!(!filter.enabled("tauri::app::run", log::Level::Debug));
    assert!(filter.enabled("sqlx::query", log::Level::Warn));
    assert!(!filter.enabled("sqlx::query", log::Level::Info));
    assert!(filter.enabled("sing_box_windows::app::core", log::Level::Debug));
}

#[test]
fn longest_prefix_wins() {
    let filter = LogDirectiveFilter::parse("sing_box_windows=info,sing_box_windows::app=trace");

    // 命中更长的 `sing_box_windows::app`（trace）而非更短的 `sing_box_windows`（info）
    assert!(filter.enabled("sing_box_windows::app::core", log::Level::Debug));
    assert!(filter.enabled("sing_box_windows::app::core", log::Level::Trace));
    // 未命中更长前缀时回落到短前缀（info）
    assert!(filter.enabled("sing_box_windows::other", log::Level::Info));
    assert!(!filter.enabled("sing_box_windows::other", log::Level::Debug));
}

#[test]
fn empty_spec_disables_all_output() {
    let filter = LogDirectiveFilter::parse("");

    assert!(!filter.enabled("any", log::Level::Error));
}

#[test]
fn invalid_level_directive_is_ignored() {
    let filter = LogDirectiveFilter::parse("foo=bar,warn");

    assert!(filter.enabled("any", log::Level::Warn));
    assert!(!filter.enabled("any", log::Level::Info));
}

#[test]
fn off_directive_disables_all_output() {
    let filter = LogDirectiveFilter::parse("off");

    assert!(!filter.enabled("any", log::Level::Error));
}

#[test]
fn format_log_line_matches_fmt_layer_style() {
    let line = format_log_line(
        "2026-10-30T23:06:12.345",
        log::Level::Info,
        "ThreadId(1)",
        Some("core.rs"),
        Some(42),
        "内核已启动",
    );

    assert_eq!(
        line,
        "2026-10-30T23:06:12.345  INFO ThreadId(1) core.rs:42 内核已启动"
    );
}

#[test]
fn format_log_line_omits_location_when_missing() {
    let line = format_log_line(
        "2026-10-30T23:06:12.345",
        log::Level::Error,
        "ThreadId(2)",
        None,
        None,
        "失败",
    );

    assert_eq!(line, "2026-10-30T23:06:12.345 ERROR ThreadId(2) 失败");
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
fn bridge_logger_writes_to_file() {
    let dir = std::env::temp_dir().join(format!("sbw-bridge-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let (sink, guard) =
        crate::utils::log_util::create_log_sink_in(dir.clone()).expect("应创建日志 sink");
    let logger = FileConsoleLogger::new(
        sink.writer().clone(),
        LogDirectiveFilter::parse("info,sing_box_windows=debug"),
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
fn render_log_payload_ends_with_single_newline() {
    let args = format_args!("载荷换行");
    let record = log::Record::builder()
        .args(args)
        .level(log::Level::Info)
        .target("sing_box_windows::app")
        .file(Some("core.rs"))
        .line(Some(7))
        .build();

    let payload = render_log_payload(&record);

    assert!(payload.ends_with('\n'));
    assert_eq!(payload.matches('\n').count(), 1);
    assert!(payload.contains("载荷换行"));
}

#[test]
fn bridge_logger_writes_one_line_per_record() {
    let dir = std::env::temp_dir().join(format!("sbw-bridge-lines-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let (sink, guard) =
        crate::utils::log_util::create_log_sink_in(dir.clone()).expect("应创建日志 sink");
    let logger = FileConsoleLogger::new(sink.writer().clone(), LogDirectiveFilter::parse("info"));

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
    let logger = FileConsoleLogger::new(sink.writer().clone(), LogDirectiveFilter::parse("warn"));

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
