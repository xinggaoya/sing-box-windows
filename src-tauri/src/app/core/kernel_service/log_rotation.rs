//! sing-box.log 按大小滚动。
//!
//! 背景：sing-box 内核不支持原生日志轮转，而应用把内核 `log.output` 固定指向
//! `<work_dir>/sing-box/sing-box.log`（见 `singbox/common.rs`），长期运行会无限增长。
//! 这里在每次启动内核前检查大小并按份滚动，保留近期 N 份用于排障。

use std::fs;
use std::path::{Path, PathBuf};
use tracing::{info, warn};

/// 单文件大小阈值（超过即触发滚动）。10MB。
pub const MAX_LOG_BYTES: u64 = 10 * 1024 * 1024;
/// 历史保留份数（不含当前文件）。即最多保留 sing-box.log + .1 + .2 + .3。
pub const ROTATE_KEEP: usize = 3;

/// 若当前日志文件达到阈值，按 `.1`、`.2`、… 的后缀滚动，保留 `ROTATE_KEEP` 份。
///
/// 实现要点：
/// - 先删除最旧的 `.N`，再从 `.N-1` 起依次向后重命名，最后把当前文件改名为 `.1`；
/// - 重命名完成后原路径文件不复存在，由内核下次写日志时自动重建——避免这里创建空文件
///   与内核打开同一文件产生句柄竞争。
/// - 任何 IO 错误都不向上抛：日志滚动失败不应阻塞内核启动，仅记录警告。
pub fn rotate_if_needed(log_path: &Path) {
    let size = match fs::metadata(log_path) {
        Ok(meta) => meta.len(),
        Err(_) => return, // 文件不存在，无需滚动
    };
    if size < MAX_LOG_BYTES {
        return;
    }

    info!(
        "sing-box.log 达到 {} 字节，开始滚动保留 {} 份",
        size, ROTATE_KEEP
    );

    if let Err(err) = rotate_inner(log_path, ROTATE_KEEP) {
        warn!("sing-box.log 滚动失败（已忽略，不影响启动）: {}", err);
    }
}

fn rotate_inner(log_path: &Path, keep: usize) -> std::io::Result<()> {
    // 删除最旧的 .N（如果存在）
    let oldest = rotated_path(log_path, keep);
    if oldest.exists() {
        fs::remove_file(&oldest)?;
    }

    // 从 .N-1 起依次向后续编号重命名：.N-1 -> .N, ..., .1 -> .2
    for idx in (1..keep).rev() {
        let from = rotated_path(log_path, idx);
        let to = rotated_path(log_path, idx + 1);
        if from.exists() {
            fs::rename(&from, &to)?;
        }
    }

    // 当前文件 -> .1
    let first = rotated_path(log_path, 1);
    fs::rename(log_path, &first)?;

    Ok(())
}

fn rotated_path(log_path: &Path, index: usize) -> PathBuf {
    let mut name = log_path.file_name().unwrap_or_default().to_os_string();
    name.push(format!(".{}", index));
    log_path.with_file_name(name)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Write;

    fn write_file(path: &Path, bytes: usize) {
        let mut f = fs::File::create(path).unwrap();
        f.write_all(&vec![b'x'; bytes]).unwrap();
    }

    #[test]
    fn does_not_rotate_when_under_threshold() {
        let dir = std::env::temp_dir().join(format!("logrot-{}", uuid_like()));
        fs::create_dir_all(&dir).unwrap();
        let log = dir.join("sing-box.log");
        write_file(&log, 100);

        rotate_if_needed(&log);

        assert!(log.exists());
        assert!(!rotated_path(&log, 1).exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn rotates_when_over_threshold_and_keeps_n() {
        let dir = std::env::temp_dir().join(format!("logrot-{}", uuid_like()));
        fs::create_dir_all(&dir).unwrap();
        let log = dir.join("sing-box.log");
        // 构造一个超过阈值的文件
        write_file(&log, (MAX_LOG_BYTES + 1) as usize);

        rotate_if_needed(&log);

        // 原文件已被重命名为 .1
        assert!(!log.exists());
        assert!(rotated_path(&log, 1).exists());
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn keeps_at_most_n_rotated_files() {
        let dir = std::env::temp_dir().join(format!("logrot-{}", uuid_like()));
        fs::create_dir_all(&dir).unwrap();
        let log = dir.join("sing-box.log");

        // 预置 .1 .2 .3（已满）+ 一个超过阈值的当前文件
        for i in 1..=ROTATE_KEEP {
            write_file(&rotated_path(&log, i), 10);
        }
        write_file(&log, (MAX_LOG_BYTES + 1) as usize);

        rotate_if_needed(&log);

        // 滚动后：旧 .3 被删除，新链为 .1(原当前) .2(原.1) .3(原.2)
        for i in 1..=ROTATE_KEEP {
            assert!(rotated_path(&log, i).exists(), "应存在 .{}", i);
        }
        assert!(
            !rotated_path(&log, ROTATE_KEEP + 1).exists(),
            "不应保留超出份数"
        );
        assert!(!log.exists(), "原文件应已被重命名");
        fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn missing_file_is_noop() {
        let dir = std::env::temp_dir().join(format!("logrot-{}", uuid_like()));
        fs::create_dir_all(&dir).unwrap();
        let log = dir.join("does-not-exist.log");
        // 不应 panic
        rotate_if_needed(&log);
        fs::remove_dir_all(&dir).ok();
    }

    // 简易唯一标识，避免并发测试目录冲突（不引入 uuid 依赖）。
    fn uuid_like() -> String {
        use std::time::{SystemTime, UNIX_EPOCH};
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        format!("{:x}", nanos)
    }
}
