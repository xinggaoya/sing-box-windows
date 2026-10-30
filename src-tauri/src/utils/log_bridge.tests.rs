use super::*;

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
