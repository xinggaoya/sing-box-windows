use super::{ProcessError, Result};
use crate::app::constants::{messages, paths};
use crate::utils::proxy_util::disable_system_proxy;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
#[cfg(target_os = "macos")]
use std::path::Path;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tauri::AppHandle;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

pub struct ProcessManager {
    process: Arc<RwLock<Option<Child>>>,
}

impl Default for ProcessManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            process: Arc::new(RwLock::new(None)),
        }
    }

    fn managed_pid_file() -> std::path::PathBuf {
        paths::get_kernel_work_dir().join(".managed-kernel.pid")
    }

    fn persist_managed_pid(&self, pid: u32) -> std::io::Result<()> {
        let pid_file = Self::managed_pid_file();
        if let Some(parent) = pid_file.parent() {
            std::fs::create_dir_all(parent)?;
        }
        std::fs::write(pid_file, pid.to_string())
    }

    fn read_managed_pid(&self) -> Option<u32> {
        let pid_file = Self::managed_pid_file();
        let content = std::fs::read_to_string(pid_file).ok()?;
        content.trim().parse::<u32>().ok()
    }

    fn clear_managed_pid(&self) {
        let pid_file = Self::managed_pid_file();
        if let Err(e) = std::fs::remove_file(&pid_file) {
            if e.kind() != std::io::ErrorKind::NotFound {
                warn!("清理托管 PID 文件失败 {:?}: {}", pid_file, e);
            }
        }
    }

    fn is_pid_matching_kernel_name(&self, pid: u32, kernel_name: &str) -> bool {
        #[cfg(target_os = "linux")]
        {
            let comm_path = format!("/proc/{}/comm", pid);
            if let Ok(name) = std::fs::read_to_string(&comm_path) {
                if name.trim() == kernel_name {
                    return true;
                }
            }

            let exe_path = format!("/proc/{}/exe", pid);
            if let Ok(target) = std::fs::read_link(&exe_path) {
                return target
                    .file_name()
                    .and_then(|f| f.to_str())
                    .map(|f| f == kernel_name)
                    .unwrap_or(false);
            }

            return false;
        }

        #[cfg(target_os = "macos")]
        {
            let output = std::process::Command::new("ps")
                .args(["-p", &pid.to_string(), "-o", "comm="])
                .output();
            if let Ok(output) = output {
                if output.status.success() {
                    let comm = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    let cmd_base = Path::new(&comm)
                        .file_name()
                        .and_then(|f| f.to_str())
                        .unwrap_or(comm.as_str());
                    return cmd_base == kernel_name;
                }
            }
            return false;
        }

        #[cfg(target_os = "windows")]
        {
            let output = std::process::Command::new("tasklist")
                .args(["/FI", &format!("PID eq {}", pid), "/FO", "CSV", "/NH"])
                .output();

            if let Ok(output) = output {
                if output.status.success() {
                    let stdout = String::from_utf8_lossy(&output.stdout);
                    for line in stdout.lines() {
                        let parts: Vec<&str> = line
                            .split('"')
                            .filter(|s| !s.is_empty() && *s != ",")
                            .collect();
                        if let Some(image_name) = parts.first() {
                            return image_name.eq_ignore_ascii_case(kernel_name);
                        }
                    }
                }
            }
            return false;
        }

        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            let _ = (pid, kernel_name);
            false
        }
    }

    // 启动进程（带系统环境检查和重试机制）
    // tun_enabled: 是否启用 TUN 模式，在 Linux/macOS 上需要特殊权限提升
    pub async fn start(
        &self,
        app_handle: &AppHandle,
        config_path: &std::path::Path,
        tun_enabled: bool,
    ) -> Result<()> {
        info!("🚀 开始启动内核进程... TUN模式: {}", tun_enabled);

        // 验证配置文件有效性
        self.validate_config(config_path).await?;

        // 先检查本地是否有sing-box进程在运行，如果有则先终止
        if let Err(e) = self.kill_existing_processes().await {
            warn!("终止已有sing-box进程失败: {}", e);
        }

        // 检查本实例中是否已经有进程在运行
        let should_restart = {
            let mut process_guard = self.process.write().await;
            if let Some(ref mut proc) = *process_guard {
                // 尝试获取进程状态，如果可以获取则说明进程还在运行
                match proc.try_wait() {
                    Ok(None) => {
                        // 进程在运行，需要先停止
                        info!("内核已经在运行中，将重新启动");
                        match proc.kill() {
                            Ok(_) => {
                                info!("已终止现有内核进程");
                                match proc.wait() {
                                    Ok(status) => info!("内核进程已终止，退出状态: {}", status),
                                    Err(e) => warn!("等待内核进程终止失败: {}", e),
                                }
                                *process_guard = None;
                                self.clear_managed_pid();
                                true
                            }
                            Err(e) => {
                                warn!("终止现有内核进程失败: {}", e);
                                // 尝试使用更强力的方式终止
                                let pid = proc.id();
                                if let Err(e) = kill_process_by_pid(pid) {
                                    error!("强制终止进程失败: {}", e);
                                }
                                *process_guard = None;
                                self.clear_managed_pid();
                                true
                            }
                        }
                    }
                    Ok(Some(status)) => {
                        info!("发现已退出的内核进程，退出状态: {}", status);
                        *process_guard = None;
                        self.clear_managed_pid();
                        true
                    }
                    Err(e) => {
                        warn!("检查内核进程状态失败: {}", e);
                        *process_guard = None;
                        self.clear_managed_pid();
                        true
                    }
                }
            } else {
                true
            }
        };

        if !should_restart {
            return Ok(());
        }

        // 获取内核路径和配置路径
        let kernel_path = paths::get_kernel_path();
        let kernel_work_dir = paths::get_kernel_work_dir();

        // 检查系统环境，特别是在开机自启动时
        self.check_system_environment().await?;

        // 多次尝试启动进程
        let max_attempts = 3;
        let mut last_error = ProcessError::StartFailed("未知错误".to_string());

        for attempt in 1..=max_attempts {
            info!("🔧 尝试启动内核进程，第 {}/{} 次", attempt, max_attempts);

            match self
                .try_start_kernel_process(
                    app_handle,
                    &kernel_path,
                    &kernel_work_dir,
                    config_path,
                    tun_enabled,
                )
                .await
            {
                Ok(child) => {
                    let child_pid = child.id();
                    // 保存进程句柄
                    {
                        let mut process_guard = self.process.write().await;
                        *process_guard = Some(child);
                    }
                    if let Err(e) = self.persist_managed_pid(child_pid) {
                        warn!("记录托管内核 PID 失败 (pid={}): {}", child_pid, e);
                    }

                    // 更稳健的启动检查
                    if self.verify_startup().await {
                        info!("✅ 内核进程启动成功并验证通过");
                        return Ok(());
                    } else {
                        last_error =
                            ProcessError::StartFailed("内核进程启动后验证失败".to_string());
                        warn!("❌ 第{}次启动后验证失败", attempt);

                        // 清理失败的进程
                        if let Err(e) = self.cleanup_failed_process().await {
                            error!("清理失败进程时出错: {}", e);
                        }
                    }
                }
                Err(e) => {
                    last_error = e;
                    error!("❌ 第{}次启动失败: {}", attempt, last_error);
                }
            }

            // 如果不是最后一次尝试，等待后重试
            if attempt < max_attempts {
                let delay = Duration::from_secs(2 * attempt as u64);
                warn!("⏳ 第{}次启动失败，{}秒后重试...", attempt, delay.as_secs());
                tokio::time::sleep(delay).await;
            }
        }

        Err(last_error)
    }

    // 检查系统环境
    async fn check_system_environment(&self) -> Result<()> {
        info!("🔍 检查系统环境...");

        // 检查内核文件是否可执行
        let kernel_path = paths::get_kernel_path();
        if !kernel_path.exists() {
            return Err(ProcessError::ConfigError(format!(
                "内核文件不存在: {}",
                kernel_path.to_str().unwrap_or("unknown")
            )));
        }

        // 检查工作目录
        let kernel_work_dir = paths::get_kernel_work_dir();
        if !kernel_work_dir.exists() {
            if let Err(e) = tokio::fs::create_dir_all(&kernel_work_dir).await {
                return Err(ProcessError::SystemError(format!(
                    "无法创建工作目录: {}",
                    e
                )));
            }
        }

        info!("✅ 系统环境检查完成");
        Ok(())
    }

    // 尝试启动内核进程
    // tun_enabled 参数用于在 Linux/macOS 上启用 TUN 时进行权限提升
    async fn try_start_kernel_process(
        &self,
        app_handle: &AppHandle,
        kernel_path: &std::path::Path,
        kernel_work_dir: &std::path::Path,
        config_path: &std::path::Path,
        tun_enabled: bool,
    ) -> Result<std::process::Child> {
        let kernel_str = kernel_path
            .to_str()
            .ok_or_else(|| ProcessError::StartFailed("内核路径包含无效字符".to_string()))?;
        let work_dir_str = kernel_work_dir
            .to_str()
            .ok_or_else(|| ProcessError::StartFailed("工作目录路径包含无效字符".to_string()))?;
        let config_str = config_path
            .to_str()
            .ok_or_else(|| ProcessError::StartFailed("配置文件路径包含无效字符".to_string()))?;

        // Windows: 直接启动（假设应用已以管理员权限运行）
        #[cfg(target_os = "windows")]
        {
            let _ = (tun_enabled, kernel_str, app_handle); // Windows 不使用这些参数，由应用整体权限控制
            let mut cmd = Command::new(kernel_path);
            cmd.args(["run", "-D", work_dir_str, "-c", config_str]);
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
            cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);

            let child = cmd
                .spawn()
                .map_err(|e| ProcessError::StartFailed(format!("启动内核进程失败: {}", e)))?;
            Ok(child)
        }

        // Linux: TUN 模式使用 sudo + 系统密钥环提权（由前端首次收集系统密码）
        #[cfg(target_os = "linux")]
        {
            if tun_enabled {
                info!("🔐 TUN 模式启用，使用 sudo 提升内核权限");
                return crate::app::system::sudo_service::spawn_kernel_with_saved_password(
                    app_handle,
                    kernel_str,
                    work_dir_str,
                    config_str,
                )
                .await
                .map_err(ProcessError::StartFailed);
            } else {
                let mut cmd = Command::new(kernel_path);
                cmd.args(["run", "-D", work_dir_str, "-c", config_str]);
                cmd.stdout(Stdio::null()).stderr(Stdio::null());

                let child = cmd
                    .spawn()
                    .map_err(|e| ProcessError::StartFailed(format!("启动内核进程失败: {}", e)))?;
                Ok(child)
            }
        }

        // macOS: TUN 模式使用 sudo + 系统钥匙串提权（由前端首次收集系统密码）
        #[cfg(target_os = "macos")]
        {
            if tun_enabled {
                info!("🔐 TUN 模式启用，使用 sudo 提升内核权限");
                return crate::app::system::sudo_service::spawn_kernel_with_saved_password(
                    app_handle,
                    kernel_str,
                    work_dir_str,
                    config_str,
                )
                .await
                .map_err(ProcessError::StartFailed);
            } else {
                let mut cmd = Command::new(kernel_path);
                cmd.args(["run", "-D", work_dir_str, "-c", config_str]);
                cmd.stdout(Stdio::null()).stderr(Stdio::null());

                let child = cmd
                    .spawn()
                    .map_err(|e| ProcessError::StartFailed(format!("启动内核进程失败: {}", e)))?;
                Ok(child)
            }
        }

        // 其他平台回退
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            let _ = (tun_enabled, app_handle);
            let mut cmd = Command::new(kernel_path);
            cmd.args(["run", "-D", work_dir_str, "-c", config_str]);
            cmd.stdout(Stdio::null()).stderr(Stdio::null());

            let child = cmd
                .spawn()
                .map_err(|e| ProcessError::StartFailed(format!("启动内核进程失败: {}", e)))?;
            Ok(child)
        }
    }

    // 说明：旧版 Linux(pkexec)/macOS(osascript) 提权方案已替换为 sudo + 密钥环保存密码，
    // 以满足“首次弹窗输入密码、后续自动提权”的产品需求。
    // 验证启动是否成功
    async fn verify_startup(&self) -> bool {
        info!("🔍 验证内核启动状态...");

        // 短轮询快速确认，不长时间阻塞启动流程
        for i in 1..=3 {
            tokio::time::sleep(Duration::from_millis(500)).await;

            if self.is_running().await {
                info!("✅ 内核状态验证通过（第{}次检查）", i);
                return true;
            } else {
                debug!("⏳ 内核尚未就绪，第{}次检查", i);
            }
        }

        error!("❌ 内核启动验证失败，多次检查都未通过");
        false
    }

    // 清理失败的进程
    async fn cleanup_failed_process(&self) -> Result<()> {
        let mut process_guard = self.process.write().await;
        if let Some(mut child) = process_guard.take() {
            if let Err(e) = child.kill() {
                warn!("清理失败进程时出错: {}", e);
                // 尝试强制终止
                #[cfg(windows)]
                {
                    let pid = child.id();
                    if let Err(e) = kill_process_by_pid(pid) {
                        error!("强制终止进程失败: {}", e);
                    }
                }
            }
        }
        self.clear_managed_pid();
        Ok(())
    }

    // 仅清理本程序托管过的内核 PID，避免误杀用户自行运行的 sing-box 进程。
    pub async fn kill_existing_processes(&self) -> std::io::Result<()> {
        let kernel_name = crate::platform::get_kernel_executable_name();
        let Some(pid) = self.read_managed_pid() else {
            info!("未发现托管 PID 记录，跳过内核进程清理");
            return Ok(());
        };

        if !self.is_pid_matching_kernel_name(pid, kernel_name) {
            warn!(
                "托管 PID({}) 与当前内核进程名({})不匹配，已跳过清理并清除记录",
                pid, kernel_name
            );
            self.clear_managed_pid();
            return Ok(());
        }

        info!("发现托管内核进程 PID: {}，开始清理", pid);
        if let Err(e) = kill_process_by_pid(pid) {
            warn!("终止托管内核进程失败 (pid={}): {}", pid, e);
        }
        self.clear_managed_pid();
        sleep(Duration::from_millis(300)).await;

        Ok(())
    }

    // 按进程名强制清理所有内核进程。
    // 用于“检测到旧内核残留导致启动冲突”场景，优先保证新启动流程可恢复。
    pub async fn force_kill_kernel_processes_by_name(&self) -> std::result::Result<(), String> {
        let kernel_name = crate::platform::get_kernel_executable_name();
        info!("按进程名强制清理内核进程: {}", kernel_name);

        crate::platform::kill_processes_by_name(kernel_name)
            .await
            .map_err(|e| format!("按进程名终止内核进程失败: {}", e))?;

        // 清理本地句柄与 PID 记录，避免后续状态仍指向被外部终止的旧进程。
        {
            let mut process_guard = self.process.write().await;
            *process_guard = None;
        }
        self.clear_managed_pid();

        sleep(Duration::from_millis(350)).await;
        match crate::platform::is_process_running(kernel_name).await {
            Ok(true) => Err(format!(
                "强制清理后仍检测到 {} 进程在运行，可能存在权限不足",
                kernel_name
            )),
            Ok(false) => Ok(()),
            Err(e) => {
                // 检测失败时不直接阻断：终止命令已成功执行，交由上层启动稳定性校验兜底。
                warn!("强制清理后状态复核失败，继续后续流程: {}", e);
                Ok(())
            }
        }
    }

    // 停止进程
    pub async fn stop(&self) -> Result<()> {
        // 尝试关闭系统代理
        if let Err(e) = disable_system_proxy() {
            warn!("关闭系统代理失败: {}", e);
        } else {
            info!("{}", messages::INFO_SYSTEM_PROXY_DISABLED);
        }

        // 提取进程并停止它
        let mut child_opt = {
            let mut process_guard = self.process.write().await;
            process_guard.take()
        };

        if let Some(mut child) = child_opt.take() {
            // Windows 优先使用强制终止，避免长时间等待
            #[cfg(windows)]
            {
                let pid = child.id();
                if let Err(e) = kill_process_by_pid(pid) {
                    warn!("强制终止内核进程失败: {}", e);
                } else {
                    info!("已强制终止内核进程 (pid={})", pid);
                }
            }

            // 其他平台或兜底再尝试优雅 kill
            match child.kill() {
                Ok(_) => {
                    info!("{}", messages::INFO_PROCESS_STOPPED);
                    if let Err(e) = child.wait() {
                        warn!("等待内核进程终止失败: {}", e);
                    }
                }
                Err(e) => {
                    warn!("终止内核进程失败: {}", e);
                    #[cfg(windows)]
                    {
                        let pid = child.id();
                        if let Err(e) = kill_process_by_pid(pid) {
                            error!("强制终止进程失败: {}", e);
                            return Err(ProcessError::StopFailed(format!(
                                "强制终止进程失败: {}",
                                e
                            )));
                        }
                    }
                }
            }
            self.clear_managed_pid();
        } else {
            info!("没有正在运行的内核进程");
        }

        // 兜底：尝试清理托管 PID 记录对应的进程
        if let Err(e) = self.kill_existing_processes().await {
            warn!("清理托管内核进程失败: {}", e);
        }

        Ok(())
    }

    // 重启进程
    pub async fn restart(
        &self,
        app_handle: &AppHandle,
        config_path: &std::path::Path,
        tun_enabled: bool,
    ) -> Result<()> {
        info!("正在重启内核进程，TUN模式: {}", tun_enabled);
        self.stop().await?;
        sleep(Duration::from_millis(1000)).await;
        self.start(app_handle, config_path, tun_enabled).await?;
        info!("内核进程重启完成");
        Ok(())
    }

    // 验证配置文件
    async fn validate_config(&self, config_path: &std::path::Path) -> Result<()> {
        if !config_path.exists() {
            return Err(ProcessError::ConfigError(format!(
                "配置文件不存在: {}",
                config_path.to_str().unwrap_or("unknown")
            )));
        }

        // 检查配置文件是否可读
        if let Err(e) = tokio::fs::metadata(config_path).await {
            return Err(ProcessError::ConfigError(format!(
                "无法访问配置文件: {}",
                e
            )));
        }

        // 启动前执行一次显式配置检查，避免内核启动后才暴露语法/迁移错误。
        let kernel_path = paths::get_kernel_path();
        if kernel_path.exists() {
            let config_str = config_path
                .to_str()
                .ok_or_else(|| ProcessError::ConfigError("配置路径包含无效字符".to_string()))?;

            let mut check_cmd = Command::new(&kernel_path);
            check_cmd.args(["check", "--config", config_str]);

            #[cfg(target_os = "windows")]
            check_cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);

            let output = check_cmd
                .output()
                .map_err(|e| ProcessError::ConfigError(format!("执行配置校验命令失败: {}", e)))?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                let stdout = String::from_utf8_lossy(&output.stdout).trim().to_string();
                let detail = if !stderr.is_empty() { stderr } else { stdout };

                if detail.contains("legacy DNS servers is deprecated")
                    || detail.contains("ENABLE_DEPRECATED_LEGACY_DNS_SERVERS")
                {
                    return Err(ProcessError::ConfigError(
                        "当前配置仍使用已弃用的 legacy DNS servers。请在订阅页刷新当前订阅配置，或关闭“按原始配置运行”后重新生成。".to_string(),
                    ));
                }
                if detail.contains("legacy domain strategy options is deprecated")
                    || detail.contains("ENABLE_DEPRECATED_LEGACY_DOMAIN_STRATEGY_OPTIONS")
                {
                    return Err(ProcessError::ConfigError(
                        "当前配置仍使用已弃用的 legacy domain strategy 选项。请在订阅页刷新当前订阅配置（或重新导入）后重试。".to_string(),
                    ));
                }
                if detail.contains("dns.servers") && detail.contains("unknown field \"strategy\"") {
                    return Err(ProcessError::ConfigError(
                        "当前配置包含已弃用字段 dns.servers[].strategy。请在订阅页手动刷新当前订阅配置后重试。".to_string(),
                    ));
                }

                return Err(ProcessError::ConfigError(format!(
                    "配置校验失败: {}",
                    detail
                )));
            }
        }

        Ok(())
    }

    // 检查进程是否运行（使用读锁，提升并发性能）
    pub async fn is_running(&self) -> bool {
        let process_guard = self.process.read().await;

        if let Some(ref _proc) = *process_guard {
            // 这里我们不能直接调用 try_wait，因为它需要可变引用
            // 我们需要在写锁中进行状态检查
            drop(process_guard);

            // 获取写锁进行状态检查
            let mut process_guard = self.process.write().await;
            if let Some(ref mut proc) = *process_guard {
                match proc.try_wait() {
                    Ok(None) => true, // 进程还在运行
                    Ok(Some(_)) => {
                        // 进程已退出，清理状态
                        *process_guard = None;
                        self.clear_managed_pid();
                        false
                    }
                    Err(_) => {
                        // 检查失败，清理状态
                        *process_guard = None;
                        self.clear_managed_pid();
                        false
                    }
                }
            } else {
                false
            }
        } else {
            false
        }
    }
}

// 使用PID强制终止进程
fn kill_process_by_pid(pid: u32) -> std::io::Result<()> {
    crate::platform::kill_process_by_pid(pid).map_err(std::io::Error::other)
}
