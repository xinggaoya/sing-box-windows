use super::{ProcessError, Result};
use crate::app::constants::{messages, paths};
use crate::utils::proxy_util::disable_system_proxy;
use std::os::windows::process::CommandExt;
use std::process::{Child, Command};
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};

pub struct ProcessManager {
    process: Arc<RwLock<Option<Child>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            process: Arc::new(RwLock::new(None)),
        }
    }

    // 启动进程
    pub async fn start(&self) -> Result<()> {
        // 验证配置文件有效性
        self.validate_config().await?;

        // 先检查本地是否有sing-box.exe进程在运行，如果有则先终止
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

        // 启动新进程
        let child = Command::new(kernel_path)
            .args(&[
                "run", 
                "-D", 
                kernel_work_dir.to_str().ok_or_else(|| {
                    ProcessError::StartFailed("工作目录路径包含无效字符".to_string())
                })?
            ])
            .creation_flags(crate::app::constants::process::CREATE_NO_WINDOW)
            .spawn()
            .map_err(|e| ProcessError::StartFailed(format!("启动内核进程失败: {}", e)))?;

        // 保存进程句柄
        {
            let mut process_guard = self.process.write().await;
            *process_guard = Some(child);
        }

        // 等待一段时间确保内核启动
        sleep(Duration::from_secs(1)).await;
        
        // 检查内核是否成功启动
        if !self.is_running().await {
            return Err(ProcessError::StartFailed("内核启动失败".to_string()));
        }

        info!("{}", messages::INFO_PROCESS_STARTED);
        Ok(())
    }

    // 检查并终止系统中已存在的sing-box进程
    async fn kill_existing_processes(&self) -> std::io::Result<()> {
        info!("检查系统中是否有sing-box进程在运行");
        
        #[cfg(windows)]
        {
            // 使用异步进程命令
            let output = tokio::process::Command::new("tasklist")
                .args(&["/FI", "IMAGENAME eq sing-box.exe", "/FO", "CSV", "/NH"])
                .creation_flags(crate::app::constants::process::CREATE_NO_WINDOW)
                .output()
                .await?;
            
            let stdout = String::from_utf8_lossy(&output.stdout);
            
            // 检查输出中是否包含sing-box.exe
            if stdout.contains("sing-box.exe") {
                info!("发现已有sing-box.exe进程，正在终止");
                
                // 使用异步进程命令终止所有sing-box.exe进程
                let kill_output = tokio::process::Command::new("taskkill")
                    .args(&["/F", "/IM", "sing-box.exe"])
                    .creation_flags(crate::app::constants::process::CREATE_NO_WINDOW)
                    .output()
                    .await?;
                
                if kill_output.status.success() {
                    info!("成功终止所有sing-box.exe进程");
                } else {
                    let error = String::from_utf8_lossy(&kill_output.stderr);
                    warn!("终止sing-box.exe进程可能失败: {}", error);
                }
                
                // 等待一段时间确保进程完全终止
                sleep(Duration::from_millis(500)).await;
            } else {
                info!("未发现已有sing-box.exe进程");
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
                            return Err(ProcessError::StopFailed(format!("强制终止进程失败: {}", e)));
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
    pub async fn restart(&self) -> Result<()> {
        info!("正在重启内核进程");
        self.stop().await?;
        sleep(Duration::from_millis(1000)).await;
        self.start().await?;
        info!("内核进程重启完成");
        Ok(())
    }

    // 验证配置文件
    async fn validate_config(&self) -> Result<()> {
        let config_path = paths::get_config_path();
        
        if !config_path.exists() {
            return Err(ProcessError::ConfigError(format!(
                "配置文件不存在: {}",
                config_path.to_str().unwrap_or("unknown")
            )));
        }

        // 检查配置文件是否可读
        if let Err(e) = tokio::fs::metadata(&config_path).await {
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
                    Ok(None) => true,  // 进程还在运行
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
    #[cfg(windows)]
    {
        use std::process::Command;
        
        let output = Command::new("taskkill")
            .args(&["/F", "/PID", &pid.to_string()])
            .creation_flags(crate::app::constants::process::CREATE_NO_WINDOW)
            .output()?;
        
        if output.status.success() {
            info!("成功使用PID {}强制终止进程", pid);
        } else {
            let error = String::from_utf8_lossy(&output.stderr);
            warn!("使用PID {}强制终止进程失败: {}", pid, error);
        }
    }
    
    Ok(())
}
