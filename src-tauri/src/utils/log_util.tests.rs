use super::*;
use std::io::Write;

#[test]
fn sink_writes_content_to_file() {
    let dir = std::env::temp_dir().join(format!("sbw-log-sink-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let (sink, guard) = create_log_sink_in(dir.clone()).expect("应创建日志 sink");
    {
        let mut writer = sink.writer().clone();
        writer.write_all(b"hello bridge\n").expect("写入应成功");
    }
    // NonBlocking::flush 是空操作，必须 drop guard 让 worker 退出并刷盘
    drop(guard);

    // rolling::daily 生成的文件名带日期后缀（app.log.YYYY-MM-DD），按前缀扫描更稳
    let entries: Vec<_> = std::fs::read_dir(&dir)
        .expect("日志目录应存在")
        .filter_map(Result::ok)
        .collect();
    assert_eq!(entries.len(), 1);

    let content = std::fs::read_to_string(entries[0].path()).expect("日志文件应可读");
    assert!(content.contains("hello bridge"));

    let _ = std::fs::remove_dir_all(&dir);
}

#[test]
fn sink_exposes_log_directory() {
    let dir = std::env::temp_dir().join(format!("sbw-log-dir-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);

    let (sink, guard) = create_log_sink_in(dir.clone()).expect("应创建日志 sink");

    assert_eq!(sink.dir(), dir.as_path());

    drop(guard);
    let _ = std::fs::remove_dir_all(&dir);
}
