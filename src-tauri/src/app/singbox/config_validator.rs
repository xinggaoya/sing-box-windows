//! sing-box 配置文件校验器。
//!
//! 负责在订阅/手动配置写入磁盘前，用 `sing-box check --config <path>` 真正地
//! 让内核进程校验配置合法性。一旦失败，返回结构化错误给调用方，由调用方
//! 决定是中止写入（推荐）还是回退到原配置。
//!
//! 设计要点：
//! - **可降级**：内核二进制不存在时只打 warn、返回 `Ok(Skipped)`，不阻塞用户。
//! - **可中断**：通过自实现的 50ms 轮询限制 5s，避免大配置卡住 UI 线程。
//! - **跨平台**：Windows 下复用 `platform::configure_std_command` 避免弹黑窗。
//! - **错误信息友好化**：把 sing-box 英文报错翻译成用户能看的中文提示。
//!
//! 流程：
//! 1. `process/manager.rs` 启动内核前已有一份 `check` 逻辑，本模块是它的"前置版"，
//!    让订阅写入本身就能校验，不依赖内核启动流程。
//! 2. 写入流程是：写 `.tmp` → `validate_singbox_config(tmp)` → 原子 rename 到目标。

use crate::app::constants::paths;
use std::path::Path;
use std::process::{Command, Stdio};
use std::time::Duration;
use tracing::{info, warn};

/// 校验结果。
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationOutcome {
    /// 内核二进制不存在或不可执行 → 跳过校验（降级路径）。
    Skipped { reason: String },
    /// 内核明确报告配置 OK。
    Valid,
    /// 内核报告配置有问题，附带中文化的错误摘要。
    Invalid { summary: String, raw: String },
}

impl ValidationOutcome {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationOutcome::Valid | ValidationOutcome::Skipped { .. })
    }

    pub fn is_invalid(&self) -> bool {
        matches!(self, ValidationOutcome::Invalid { .. })
    }
}

/// sing-box `check` 子命令超时时间。多数配置校验应 < 1s，留 5s 余量。
const SINGBOX_CHECK_TIMEOUT: Duration = Duration::from_secs(5);

/// 调用 `sing-box check --config <path>` 校验配置文件。
///
/// 行为契约：
/// - 内核不存在/不可执行：返回 `Ok(Skipped)`，**不视为错误**（优雅降级）。
/// - 内核返回非 0：返回 `Ok(Invalid)`，由调用方决定如何处理。
/// - 进程启动本身失败（极少见，比如权限问题）：返回 `Err`。
pub fn validate_singbox_config(config_path: &Path) -> Result<ValidationOutcome, String> {
    let kernel_path = paths::get_kernel_path();

    if !kernel_path.exists() {
        warn!(
            "sing-box 内核不存在，跳过配置校验（降级路径）: {:?}",
            kernel_path
        );
        return Ok(ValidationOutcome::Skipped {
            reason: format!("sing-box 内核不存在: {}", kernel_path.display()),
        });
    }

    let config_str = config_path
        .to_str()
        .ok_or_else(|| format!("配置路径包含无效字符: {}", config_path.display()))?;

    let mut cmd = Command::new(&kernel_path);
    cmd.args(["check", "--config", config_str]);
    cmd.stdout(Stdio::piped()).stderr(Stdio::piped());

    // Windows 下隐藏黑窗
    #[cfg(target_os = "windows")]
    crate::platform::configure_std_command(&mut cmd);

    let output = match run_with_timeout(cmd, SINGBOX_CHECK_TIMEOUT) {
        Ok(output) => output,
        Err(e) => {
            // 超时或启动失败：降级（不阻塞订阅导入）
            warn!("sing-box check 执行失败（降级）: {}", e);
            return Ok(ValidationOutcome::Skipped { reason: e });
        }
    };

    if output.status.success() {
        info!("sing-box check 通过: {}", config_str);
        return Ok(ValidationOutcome::Valid);
    }

    // sing-box 错误优先在 stderr
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
    let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
    let raw = if !stderr.is_empty() {
        stderr
    } else {
        stdout
    };

    let summary = humanize_singbox_error(&raw);
    Ok(ValidationOutcome::Invalid {
        summary,
        raw: truncate_for_log(&raw, 512),
    })
}

/// 便捷封装：写 `target_path` 前先写到 `<target>.tmp` → sing-box check → 原子 rename。
///
/// 行为契约：
/// - `Invalid` → **不覆盖 target**，保留 `.tmp` 供排查；返回 `Err(summary)`。
/// - `Valid` / `Skipped` → 原子 rename 到 `target`，返回 `Ok(outcome)`。
/// - IO 错误 → `Err`。
///
/// 之所以 inline 在这里（而不是 `atomic_write_validated` + 调用方手写 .tmp 路径），
/// 是因为这是订阅写入流程的标准范式，重复使用率高，集中在这里减少出错面。
pub fn validate_singbox_config_inline(
    target_path: &Path,
    serialized: &str,
) -> Result<ValidationOutcome, String> {
    use std::io::Write;

    if let Some(parent) = target_path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("创建配置目录失败: {}", e))?;
    }

    // 备份现有配置（即使目标文件不存在也允许）
    let _ = crate::app::network::subscription_service::helpers::backup_existing_config(target_path);

    let tmp_path = target_path.with_extension("tmp");
    {
        let mut f = std::fs::File::create(&tmp_path)
            .map_err(|e| format!("创建临时配置失败 ({}): {}", tmp_path.display(), e))?;
        f.write_all(serialized.as_bytes())
            .map_err(|e| format!("写入临时配置失败: {}", e))?;
        // best-effort: 把数据落盘
        let _ = f.sync_all();
    }

    let outcome = validate_singbox_config(&tmp_path)?;
    if outcome.is_invalid() {
        // 不删 .tmp，方便排查；同时**绝不**覆盖 target
        if let ValidationOutcome::Invalid { summary, .. } = &outcome {
            return Err(format!(
                "配置未通过 sing-box 校验：{}\n临时文件保留在: {}",
                summary,
                tmp_path.display()
            ));
        }
    }

    // Valid / Skipped：原子替换
    if let Err(e) = std::fs::rename(&tmp_path, target_path) {
        // Windows 上 rename 到已存在文件会失败 → 退化为 remove + rename
        if target_path.exists() {
            std::fs::remove_file(target_path).ok();
        }
        std::fs::rename(&tmp_path, target_path).map_err(|e2| {
            format!(
                "原子重命名失败 ({}): {}",
                tmp_path.display(),
                e2
            )
        })?;
        // 抑制第一次 rename 的非空 e 警告
        let _ = e;
    }

    match &outcome {
        ValidationOutcome::Skipped { reason } => {
            warn!(
                "配置已写入（未经过内核校验）: {} ({})",
                target_path.display(),
                reason
            );
        }
        _ => {
            info!("配置已写入并通过内核校验: {}", target_path.display());
        }
    }

    Ok(outcome)
}

/// 把 sing-box 英文错误翻译成中文化摘要。
///
/// sing-box 的错误信息通常是 `FATAL[xxx] xxx/yyy: message` 这种格式。
/// 我们只取前几行 / 关键短语，保持简短。
fn humanize_singbox_error(raw: &str) -> String {
    let lowered = raw.to_ascii_lowercase();

    // 已知模式的友好化映射
    if lowered.contains("legacy dns servers is deprecated")
        || lowered.contains("enable_deprecated_legacy_dns_servers")
    {
        return "当前配置仍使用已弃用的 legacy DNS servers。请关闭“按原始配置运行”后重新生成。".to_string();
    }
    if lowered.contains("legacy domain strategy options is deprecated")
        || lowered.contains("enable_deprecated_legacy_domain_strategy_options")
    {
        return "当前配置仍使用已弃用的 legacy domain strategy 选项。请重新导入订阅后重试。".to_string();
    }
    if lowered.contains("dns.servers") && lowered.contains("unknown field \"strategy\"") {
        return "配置包含已弃用字段 dns.servers[].strategy。请重新导入订阅后重试。".to_string();
    }
    if lowered.contains("missing field") {
        return format!("配置缺少必填字段: {}", first_meaningful_line(raw));
    }
    if lowered.contains("unknown field") {
        return format!("配置包含未知字段: {}", first_meaningful_line(raw));
    }
    if lowered.contains("invalid") || lowered.contains("parse error") {
        return format!("配置解析失败: {}", first_meaningful_line(raw));
    }

    // 兜底：取第一行非空内容
    first_meaningful_line(raw)
}

fn first_meaningful_line(raw: &str) -> String {
    raw.lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .unwrap_or("(无错误详情)")
        .to_string()
}

fn truncate_for_log(raw: &str, max_bytes: usize) -> String {
    if raw.len() <= max_bytes {
        raw.to_string()
    } else {
        // 按字符边界截，避免 UTF-8 中间断开
        let mut end = max_bytes;
        while !raw.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}…(truncated)", &raw[..end])
    }
}

/// 在不引入额外依赖的前提下，给 `std::process::Command` 加一个 5s 超时。
///
/// 50ms 轮询粒度足以让"正常 100-500ms 的 check"不浪费 CPU；
/// 同时 5s 上限防止大配置 / 卡死 IO 把订阅流程拖死。
fn run_with_timeout(
    mut cmd: Command,
    timeout: Duration,
) -> Result<std::process::Output, String> {
    use std::io::Read;

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("启动 sing-box 失败: {}", e))?;

    let start = std::time::Instant::now();

    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let mut stdout = Vec::new();
                let mut stderr = Vec::new();
                if let Some(mut s) = child.stdout.take() {
                    let _ = s.read_to_end(&mut stdout);
                }
                if let Some(mut s) = child.stderr.take() {
                    let _ = s.read_to_end(&mut stderr);
                }
                let _ = child.wait();
                return Ok(std::process::Output {
                    status,
                    stdout,
                    stderr,
                });
            }
            Ok(None) => {
                if start.elapsed() > timeout {
                    let _ = child.kill();
                    let _ = child.wait();
                    return Err(format!("sing-box check 超时（>{:?}）", timeout));
                }
                std::thread::sleep(Duration::from_millis(50));
            }
            Err(e) => {
                let _ = child.kill();
                return Err(format!("sing-box check wait 失败: {}", e));
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn humanize_legacy_dns_deprecation() {
        let raw = "FATAL[config] legacy DNS servers is deprecated";
        let out = humanize_singbox_error(raw);
        assert!(out.contains("legacy DNS"));
    }

    #[test]
    fn humanize_missing_field() {
        let raw = "FATAL[config] missing field `tag` at `outbounds[3]`";
        let out = humanize_singbox_error(raw);
        assert!(out.contains("缺少必填字段"));
        assert!(out.contains("missing field"));
    }

    #[test]
    fn humanize_unknown_field() {
        let raw = "FATAL[config] unknown field `xxx` at `outbounds[0]`";
        let out = humanize_singbox_error(raw);
        assert!(out.contains("未知字段"));
    }

    #[test]
    fn humanize_fallback_first_line() {
        let raw = "some random error\nsecond line";
        let out = humanize_singbox_error(raw);
        assert_eq!(out, "some random error");
    }

    #[test]
    fn truncate_respects_utf8_boundary() {
        let s = "你好hello世界world"; // 14 chars
        let t = truncate_for_log(s, 7);
        // 7 字节在 "你" 末尾（"你" = 3 字节），所以回退到 6 字节
        // 期望以 "(truncated)" 结尾
        assert!(t.ends_with("(truncated)"));
    }

    #[test]
    fn outcome_helpers() {
        assert!(ValidationOutcome::Valid.is_valid());
        assert!(ValidationOutcome::Skipped { reason: "x".into() }.is_valid());
        assert!(!ValidationOutcome::Invalid {
            summary: "s".into(),
            raw: "r".into()
        }
        .is_valid());
        assert!(ValidationOutcome::Invalid {
            summary: "s".into(),
            raw: "r".into()
        }
        .is_invalid());
    }
}
