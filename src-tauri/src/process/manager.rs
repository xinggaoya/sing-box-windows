use super::{ProcessError, Result};
use crate::app::constants::{messages, paths};
use crate::utils::proxy_util::disable_system_proxy;

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;
use std::process::{Child, Command, Stdio};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{debug, error, info, warn};

pub struct ProcessManager {
    process: Arc<RwLock<Option<Child>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            process: Arc::new(RwLock::new(None)),
        }
    }

    // 启动进程（带系统环境检查和重试机制）
    // tun_enabled: 是否启用 TUN 模式，在 Linux/macOS 上需要特殊权限提升
    pub async fn start(&self, config_path: &std::path::Path, tun_enabled: bool) -> Result<()> {
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
                                true
                            }
                        }
                    }
                    Ok(Some(status)) => {
                        info!("发现已退出的内核进程，退出状态: {}", status);
                        *process_guard = None;
                        true
                    }
                    Err(e) => {
                        warn!("检查内核进程状态失败: {}", e);
                        *process_guard = None;
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
                .try_start_kernel_process(&kernel_path, &kernel_work_dir, config_path, tun_enabled)
                .await
            {
                Ok(child) => {
                    // 保存进程句柄
                    {
                        let mut process_guard = self.process.write().await;
                        *process_guard = Some(child);
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

        // 检查是否有足够的系统资源
        #[cfg(windows)]
        {
            // 检查系统启动时间，如果是刚启动，可能需要等待更长时间
            match std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH) {
                Ok(uptime) => {
                    let uptime_minutes = uptime.as_secs() / 60;
                    if uptime_minutes < 2 {
                        info!("⏰ 系统刚启动{}分钟，增加启动等待时间", uptime_minutes);
                        tokio::time::sleep(Duration::from_secs(5)).await;
                    }
                }
                Err(e) => {
                    warn!("无法获取系统启动时间: {}", e);
                }
            }
        }

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
            let _ = (tun_enabled, kernel_str); // Windows 不使用这些参数，由应用整体权限控制
            let mut cmd = Command::new(kernel_path);
            cmd.args(&["run", "-D", work_dir_str, "-c", config_str]);
            cmd.stdout(Stdio::null()).stderr(Stdio::null());
            cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);

            let child = cmd
                .spawn()
                .map_err(|e| ProcessError::StartFailed(format!("启动内核进程失败: {}", e)))?;
            return Ok(child);
        }

        // Linux: TUN 模式使用 sudo + 系统密钥环提权（由前端首次收集系统密码）
        #[cfg(target_os = "linux")]
        {
            if tun_enabled {
                info!("🔐 TUN 模式启用，使用 sudo 提升内核权限");
                return crate::app::system::sudo_service::spawn_kernel_with_saved_password(
                    kernel_str,
                    work_dir_str,
                    config_str,
                )
                .map_err(ProcessError::StartFailed);
            } else {
                let mut cmd = Command::new(kernel_path);
                cmd.args(&["run", "-D", work_dir_str, "-c", config_str]);
                cmd.stdout(Stdio::null()).stderr(Stdio::null());

                let child = cmd
                    .spawn()
                    .map_err(|e| ProcessError::StartFailed(format!("启动内核进程失败: {}", e)))?;
                return Ok(child);
            }
        }

        // macOS: TUN 模式使用 sudo + 系统钥匙串提权（由前端首次收集系统密码）
        #[cfg(target_os = "macos")]
        {
            if tun_enabled {
                info!("🔐 TUN 模式启用，使用 sudo 提升内核权限");
                return crate::app::system::sudo_service::spawn_kernel_with_saved_password(
                    kernel_str,
                    work_dir_str,
                    config_str,
                )
                .map_err(ProcessError::StartFailed);
            } else {
                let mut cmd = Command::new(kernel_path);
                cmd.args(&["run", "-D", work_dir_str, "-c", config_str]);
                cmd.stdout(Stdio::null()).stderr(Stdio::null());

                let child = cmd
                    .spawn()
                    .map_err(|e| ProcessError::StartFailed(format!("启动内核进程失败: {}", e)))?;
                return Ok(child);
            }
        }

        // 其他平台回退
        #[cfg(not(any(target_os = "windows", target_os = "linux", target_os = "macos")))]
        {
            let _ = tun_enabled;
            let mut cmd = Command::new(kernel_path);
            cmd.args(&["run", "-D", work_dir_str, "-c", config_str]);
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

        // 多次检查，确保真正启动成功
        for i in 1..=5 {
            tokio::time::sleep(Duration::from_secs(1)).await;

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
        Ok(())
    }

    // 检查并终止系统中已存在的sing-box进程
    pub async fn kill_existing_processes(&self) -> std::io::Result<()> {
        info!("检查系统中是否有sing-box进程在运行");

        let kernel_name = crate::platform::get_kernel_executable_name();
        
        // 检查是否有进程运行
        match crate::platform::is_process_running(kernel_name).await {
            Ok(true) => {
                info!("发现已有 {} 进程，正在终止", kernel_name);
                
                // 终止进程
                if let Err(e) = crate::platform::kill_processes_by_name(kernel_name).await {
                    warn!("终止 {} 进程可能失败: {}", kernel_name, e);
                } else {
                    info!("成功终止所有 {} 进程", kernel_name);
                }
                
                // 等待进程完全终止
                sleep(Duration::from_millis(500)).await;
            }
            Ok(false) => {
                info!("未发现已有 {} 进程", kernel_name);
            }
            Err(e) => {
                warn!("无法检查 {} 进程状态: {}", kernel_name, e);
            }
        }

        Ok(())
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
            // 强制终止进程
            match child.kill() {
                Ok(_) => {
                    info!("{}", messages::INFO_PROCESS_STOPPED);
                    // 等待进程退出
                    match child.wait() {
                        Ok(status) => info!("内核进程已终止，退出状态: {}", status),
                        Err(e) => warn!("等待内核进程终止失败: {}", e),
                    }
                }
                Err(e) => {
                    warn!("终止内核进程失败: {}", e);
                    // 尝试使用更强力的方式终止
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
        } else {
            info!("没有正在运行的内核进程");
        }

        // 确保系统中所有sing-box进程都被终止
        if let Err(e) = self.kill_existing_processes().await {
            warn!("清理系统中的sing-box进程失败: {}", e);
        }

        Ok(())
    }

    // 重启进程
    pub async fn restart(&self, config_path: &std::path::Path, tun_enabled: bool) -> Result<()> {
        info!("正在重启内核进程，TUN模式: {}", tun_enabled);
        self.stop().await?;
        sleep(Duration::from_millis(1000)).await;
        self.start(config_path, tun_enabled).await?;
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
                        false
                    }
                    Err(_) => {
                        // 检查失败，清理状态
                        *process_guard = None;
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
    crate::platform::kill_process_by_pid(pid)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
}
