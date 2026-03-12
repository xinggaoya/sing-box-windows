//! Linux 平台实现

use std::path::Path;
use tracing::{info, warn};

fn parse_pid_list(output: &str) -> Vec<u32> {
    output
        .split_whitespace()
        .filter_map(|item| item.trim().parse::<u32>().ok())
        .collect()
}

fn parse_proc_status_state(content: &str) -> Option<char> {
    content
        .lines()
        .find_map(|line| line.strip_prefix("State:"))
        .and_then(|state| state.split_whitespace().next())
        .and_then(|state| state.chars().next())
}

fn parse_proc_stat_state(content: &str) -> Option<char> {
    let (_, rest) = content.rsplit_once(") ")?;
    rest.chars().next()
}

fn read_process_state(proc_root: &Path, pid: u32) -> Option<char> {
    let status_path = proc_root.join(pid.to_string()).join("status");
    if let Ok(status) = std::fs::read_to_string(status_path) {
        if let Some(state) = parse_proc_status_state(&status) {
            return Some(state);
        }
    }

    let stat_path = proc_root.join(pid.to_string()).join("stat");
    std::fs::read_to_string(stat_path)
        .ok()
        .and_then(|stat| parse_proc_stat_state(&stat))
}

fn is_active_process_state(state: char) -> bool {
    !matches!(state, 'Z' | 'X')
}

fn is_pid_active(proc_root: &Path, pid: u32) -> bool {
    read_process_state(proc_root, pid)
        .map(is_active_process_state)
        .unwrap_or(false)
}

fn pid_matches_process_name(proc_root: &Path, pid: u32, process_name: &str) -> bool {
    let proc_path = proc_root.join(pid.to_string());

    let comm_path = proc_path.join("comm");
    if let Ok(name) = std::fs::read_to_string(comm_path) {
        if name.trim() == process_name {
            return true;
        }
    }

    let exe_path = proc_path.join("exe");
    if let Ok(target) = std::fs::read_link(exe_path) {
        return target
            .file_name()
            .and_then(|file| file.to_str())
            .map(|file| file == process_name)
            .unwrap_or(false);
    }

    false
}

fn filter_active_pids(
    proc_root: &Path,
    process_name: &str,
    pids: Vec<u32>,
    self_pid: u32,
) -> Vec<u32> {
    let mut active = Vec::new();

    for pid in pids {
        if pid == self_pid {
            continue;
        }

        if !pid_matches_process_name(proc_root, pid, process_name) {
            continue;
        }

        if is_pid_active(proc_root, pid) {
            active.push(pid);
        } else {
            info!("检测到 {} zombie 进程，已忽略: {}", process_name, pid);
        }
    }

    active.sort_unstable();
    active.dedup();
    active
}

fn read_active_pids_from_proc(proc_root: &Path, process_name: &str, self_pid: u32) -> Vec<u32> {
    let Ok(entries) = std::fs::read_dir(proc_root) else {
        warn!("无法遍历 {:?}，进程检测可能不准确", proc_root);
        return Vec::new();
    };

    let mut active = Vec::new();
    for entry in entries.flatten() {
        let pid = match entry.file_name().to_string_lossy().parse::<u32>() {
            Ok(pid) => pid,
            Err(_) => continue,
        };

        if pid == self_pid {
            continue;
        }

        if !pid_matches_process_name(proc_root, pid, process_name) {
            continue;
        }

        if is_pid_active(proc_root, pid) {
            active.push(pid);
        } else {
            info!(
                "通过 /proc 检测到 {} zombie 进程，已忽略: {}",
                process_name, pid
            );
        }
    }

    active.sort_unstable();
    active.dedup();
    active
}

fn collect_active_processes(proc_root: &Path, process_name: &str) -> Vec<u32> {
    let self_pid = std::process::id();

    if let Ok(output) = std::process::Command::new("pidof")
        .arg("-x")
        .arg(process_name)
        .output()
    {
        if output.status.success() {
            let active = filter_active_pids(
                proc_root,
                process_name,
                parse_pid_list(&String::from_utf8_lossy(&output.stdout)),
                self_pid,
            );
            if !active.is_empty() {
                info!("通过pidof检测到活跃进程: {:?}", active);
                return active;
            }
        }
    }

    if let Ok(output) = std::process::Command::new("pgrep")
        .args(["-x", process_name])
        .output()
    {
        if output.status.success() {
            let active = filter_active_pids(
                proc_root,
                process_name,
                parse_pid_list(&String::from_utf8_lossy(&output.stdout)),
                self_pid,
            );
            if !active.is_empty() {
                info!("通过pgrep检测到活跃进程: {:?}", active);
                return active;
            }
        }
    }

    read_active_pids_from_proc(proc_root, process_name, self_pid)
}

pub async fn platform_list_active_processes_by_name(
    process_name: &str,
) -> Result<Vec<u32>, String> {
    Ok(collect_active_processes(Path::new("/proc"), process_name))
}

/// 检测进程是否运行（Linux）
pub async fn platform_is_process_running(process_name: &str) -> Result<bool, String> {
    let active_pids = platform_list_active_processes_by_name(process_name).await?;
    if active_pids.is_empty() {
        info!("未检测到活跃 {} 进程", process_name);
        return Ok(false);
    }

    info!("检测到活跃 {} 进程: {:?}", process_name, active_pids);
    Ok(true)
}

/// 杀死指定名称的进程（Linux）
pub async fn platform_kill_processes_by_name(process_name: &str) -> Result<(), String> {
    let output = std::process::Command::new("pkill")
        .args(["-9", "-x", process_name])
        .output()
        .map_err(|e| format!("执行pkill失败: {}", e))?;
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() || output.status.code() == Some(1) {
        if !stderr.is_empty() {
            warn!("pkill {} 输出警告: {}", process_name, stderr);
        }
        info!("成功终止进程: {}", process_name);
        Ok(())
    } else {
        Err(format!("终止进程失败: {}", stderr))
    }
}

/// 杀死指定 PID 的进程（Linux）
pub fn platform_kill_process_by_pid(pid: u32) -> Result<(), String> {
    let output = std::process::Command::new("kill")
        .args(["-9", &pid.to_string()])
        .output()
        .map_err(|e| format!("执行kill失败: {}", e))?;
    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();

    if output.status.success() {
        if !stderr.is_empty() {
            warn!("kill PID {} 输出警告: {}", pid, stderr);
        }
        info!("成功终止进程 PID: {}", pid);
        Ok(())
    } else {
        Err(format!("终止进程失败: {}", stderr))
    }
}

/// 获取系统架构（Linux）
pub fn platform_get_system_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" => "amd64",
        "x86" => "386",
        "aarch64" => "arm64",
        "arm" => "armv7",
        _ => "amd64",
    }
}

/// 获取平台名称（Linux）
pub fn platform_get_platform_name() -> &'static str {
    "linux"
}

/// 获取内核可执行文件名（Linux）
pub fn platform_get_kernel_executable_name() -> &'static str {
    "sing-box"
}

/// 获取系统运行时间（Linux）
pub async fn platform_get_system_uptime_ms() -> Result<u64, String> {
    match std::fs::read_to_string("/proc/uptime") {
        Ok(content) => {
            let uptime_seconds: f64 = content
                .split_whitespace()
                .next()
                .unwrap_or("0")
                .parse()
                .unwrap_or(0.0);
            Ok((uptime_seconds * 1000.0) as u64)
        }
        Err(_) => Ok(0),
    }
}

/// 配置 tokio 进程命令（Linux - 无操作）
pub fn platform_configure_process_command(_command: &mut tokio::process::Command) {
    // Linux 不需要特殊配置
}

/// 配置标准库进程命令（Linux - 无操作）
pub fn platform_configure_std_command(_command: &mut std::process::Command) {
    // Linux 不需要特殊配置
}

#[cfg(test)]
mod tests {
    use super::{
        filter_active_pids, is_active_process_state, parse_pid_list, parse_proc_stat_state,
        parse_proc_status_state, read_active_pids_from_proc,
    };
    use std::fs;
    use std::path::{Path, PathBuf};
    use std::time::{SystemTime, UNIX_EPOCH};

    fn unique_test_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("{}_{}", name, nanos))
    }

    fn write_proc_entry(
        proc_root: &Path,
        pid: u32,
        process_name: &str,
        state: char,
        exe_name: &str,
    ) {
        let pid_dir = proc_root.join(pid.to_string());
        fs::create_dir_all(&pid_dir).unwrap();
        fs::write(pid_dir.join("comm"), format!("{}\n", process_name)).unwrap();
        fs::write(
            pid_dir.join("status"),
            format!("Name:\t{}\nState:\t{} (test)\n", process_name, state),
        )
        .unwrap();
        fs::write(
            pid_dir.join("stat"),
            format!("{pid} ({process_name}) {state} 1 1 1 0"),
        )
        .unwrap();
        #[cfg(unix)]
        std::os::unix::fs::symlink(exe_name, pid_dir.join("exe")).unwrap();
    }

    #[test]
    fn parse_pid_list_works_for_mixed_whitespace() {
        assert_eq!(parse_pid_list("12  34\n56"), vec![12, 34, 56]);
    }

    #[test]
    fn parse_process_state_prefers_expected_char() {
        assert_eq!(
            parse_proc_status_state("Name:\tsing-box\nState:\tS (sleeping)\n"),
            Some('S')
        );
        assert_eq!(parse_proc_stat_state("123 (sing-box) Z 1 1 1 0"), Some('Z'));
    }

    #[test]
    fn zombie_state_is_not_considered_active() {
        assert!(is_active_process_state('S'));
        assert!(!is_active_process_state('Z'));
        assert!(!is_active_process_state('X'));
    }

    #[test]
    fn filter_active_pids_ignores_zombie_and_self() {
        let proc_root = unique_test_dir("linux_proc_filter");
        fs::create_dir_all(&proc_root).unwrap();

        let self_pid = std::process::id();
        write_proc_entry(&proc_root, 41001, "sing-box", 'S', "sing-box");
        write_proc_entry(&proc_root, 41002, "sing-box", 'Z', "sing-box");
        write_proc_entry(&proc_root, self_pid, "sing-box", 'S', "sing-box");

        let active = filter_active_pids(
            &proc_root,
            "sing-box",
            vec![41001, 41002, self_pid],
            self_pid,
        );
        assert_eq!(active, vec![41001]);

        fs::remove_dir_all(proc_root).unwrap();
    }

    #[test]
    fn read_active_pids_from_proc_only_returns_live_matching_processes() {
        let proc_root = unique_test_dir("linux_proc_scan");
        fs::create_dir_all(&proc_root).unwrap();

        write_proc_entry(&proc_root, 42001, "sing-box", 'S', "sing-box");
        write_proc_entry(&proc_root, 42002, "sing-box", 'Z', "sing-box");
        write_proc_entry(&proc_root, 42003, "bash", 'S', "bash");

        let active = read_active_pids_from_proc(&proc_root, "sing-box", std::process::id());
        assert_eq!(active, vec![42001]);

        fs::remove_dir_all(proc_root).unwrap();
    }
}
