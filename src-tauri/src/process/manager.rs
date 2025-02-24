use super::{ProcessError, ProcessInfo, ProcessStatus, Result};
use crate::utils::app_util::get_work_dir;
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use crate::utils::proxy_util::disable_system_proxy;

pub struct ProcessManager {
    process_info: Arc<RwLock<ProcessInfo>>,
    child_process: Arc<RwLock<Option<tokio::process::Child>>>,
}

impl ProcessManager {
    pub fn new() -> Self {
        Self {
            process_info: Arc::new(RwLock::new(ProcessInfo {
                pid: None,
                status: ProcessStatus::Stopped,
                last_error: None,
            })),
            child_process: Arc::new(RwLock::new(None)),
        }
    }

    // 获取进程状态
    pub async fn get_status(&self) -> ProcessInfo {
        self.process_info.read().await.clone()
    }

    // 检查进程是否真实存在
    async fn check_process_exists(&self, pid: Option<u32>) -> bool {
        if let Some(pid) = pid {
            match std::process::Command::new("tasklist")
                .arg("/FI")
                .arg(format!("PID eq {}", pid))
                .creation_flags(0x08000000)
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout);
                    output_str.contains(&pid.to_string())
                }
                Err(_) => false,
            }
        } else {
            false
        }
    }

    // 检查是否存在其他 sing-box 进程
    async fn check_other_sing_box_process(&self) -> Option<u32> {
        match std::process::Command::new("tasklist")
            .arg("/FI")
            .arg("IMAGENAME eq sing-box.exe")
            .arg("/FO")
            .arg("CSV")
            .arg("/NH")
            .creation_flags(0x08000000)
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                // CSV 格式: "sing-box.exe","1234",...
                if let Some(line) = output_str.lines().next() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        // 提取 PID
                        if let Some(pid_str) = parts[1].trim_matches('"').parse::<u32>().ok() {
                            return Some(pid_str);
                        }
                    }
                }
                None
            }
            Err(_) => None,
        }
    }

    // 重置进程状态
    async fn reset_process_state(&self) {
        let mut info = self.process_info.write().await;
        info.status = ProcessStatus::Stopped;
        info.pid = None;
        info.last_error = None;

        let mut process = self.child_process.write().await;
        *process = None;
    }

    // 检查进程是否在运行
    pub async fn is_running(&self) -> bool {
        let info = self.process_info.read().await;
        let status_running = matches!(
            info.status,
            ProcessStatus::Running | ProcessStatus::Starting
        );

        // 如果状态显示运行中，但实际进程不存在，则重置状态
        if status_running && !self.check_process_exists(info.pid).await {
            drop(info); // 释放读锁
            self.reset_process_state().await;
            return false;
        }

        status_running
    }

    // 启动前检查
    async fn pre_start_check(&self) -> Result<()> {
        // 检查是否有其他 sing-box 进程在运行
        if let Some(pid) = self.check_other_sing_box_process().await {
            // 尝试停止其他进程
            info!("检测到其他 sing-box 进程 (PID: {}), 尝试停止", pid);
            if let Err(e) = self.send_signal(pid) {
                warn!("停止其他进程失败: {}, 尝试强制停止", e);
                if let Err(e) = self.kill_process(pid) {
                    error!("强制停止其他进程失败: {}", e);
                }
            }
                    // 如果超时后进程仍然存在，强制终止
        if self.check_process_exists(Some(pid)).await {
            warn!("进程停止超时，尝试强制终止");
            if let Err(e) = self.kill_process(pid) {
                return Err(ProcessError::StopFailed(format!("强制停止失败: {}", e)));
            }
        }
            // 等待进程完全停止
            sleep(Duration::from_secs(1)).await;
        }

        let work_dir = get_work_dir();
        info!("当前工作目录: {}", work_dir);

        let sing_box_dir = Path::new(&work_dir).join("sing-box");
        let kernel_path = sing_box_dir.join("sing-box.exe");
        let _config_path = sing_box_dir.join("config.json");

        // 检查 sing-box 目录是否存在
        if !sing_box_dir.exists() {
            return Err(ProcessError::StartFailed("sing-box 目录不存在".to_string()));
        }

        // 检查内核文件是否存在
        if !kernel_path.exists() {
            return Err(ProcessError::StartFailed("内核文件不存在".to_string()));
        }

        Ok(())
    }

    // 启动进程
    pub async fn start(&self) -> Result<()> {
        // 更新状态为启动中
        {
            let mut info = self.process_info.write().await;
            info.status = ProcessStatus::Starting;
            info.last_error = None;
        }

        // 启动前检查
        if let Err(e) = self.pre_start_check().await {
            self.handle_error(e.clone()).await?;
            return Err(e);
        }

        // 获取工作目录
        let work_dir = get_work_dir();
        let kernel_work_dir = Path::new(&work_dir).join("sing-box");
        let kernel_path = kernel_work_dir.join("sing-box.exe");

        // 启动进程
        let child = match Command::new(kernel_path.to_str().unwrap())
            .arg("run")
            .arg("-D")
            .arg(kernel_work_dir.to_str().unwrap())
            .creation_flags(0x08000000)
            .spawn()
        {
            Ok(child) => child,
            Err(e) => {
                let err = ProcessError::StartFailed(format!("启动失败: {}", e));
                self.handle_error(err.clone()).await?;
                return Err(err);
            }
        };

        // 更新进程信息
        {
            let mut info = self.process_info.write().await;
            info.pid = Some(child.id().unwrap_or(0));
            info.status = ProcessStatus::Starting;
        }

        // 保存子进程
        {
            let mut process = self.child_process.write().await;
            *process = Some(child);
        }

        info!("进程启动成功");
        Ok(())
    }

    // 停止进程
    pub async fn stop(&self) -> Result<()> {
        // 检查进程状态
        if !self.is_running().await {
            return Ok(());
        }

        // 更新状态为停止中
        {
            let mut info = self.process_info.write().await;
            info.status = ProcessStatus::Stopping;
        }

        // 获取PID
        let pid = {
            let info = self.process_info.read().await;
            info.pid.ok_or(ProcessError::NotRunning)?
        };

        // 尝试优雅停止
        if let Err(e) = self.send_signal(pid) {
            warn!("优雅停止失败: {}, 尝试强制停止", e);
            if let Err(e) = self.kill_process(pid) {
                return Err(ProcessError::StopFailed(format!("强制停止失败: {}", e)));
            }
        }

        // 如果超时后进程仍然存在，强制终止
        if self.check_process_exists(Some(pid)).await {
            warn!("进程停止超时，尝试强制终止");
            if let Err(e) = self.kill_process(pid) {
                return Err(ProcessError::StopFailed(format!("强制停止失败: {}", e)));
            }
        }

        // 关闭系统代理
        if let Err(e) = disable_system_proxy() {
            warn!("关闭系统代理失败: {}", e);
        } else {
            info!("系统代理已关闭");
        }

        // 重置进程状态
        self.reset_process_state().await;

        info!("进程已停止");
        Ok(())
    }

    // 发送停止信号
    fn send_signal(&self, pid: u32) -> std::io::Result<()> {
        std::process::Command::new("taskkill")
            .arg("/PID")
            .arg(pid.to_string())
            .creation_flags(0x08000000)
            .output()?;
        Ok(())
    }

    // 强制结束进程
    fn kill_process(&self, pid: u32) -> std::io::Result<()> {
        std::process::Command::new("taskkill")
            .arg("/F")
            .arg("/PID")
            .arg(pid.to_string())
            .creation_flags(0x08000000)
            .output()?;
        Ok(())
    }

    // 重启进程 强制停止
    pub async fn restart(&self) -> Result<()> {
        self.stop().await?;
        // 休眠1s
        sleep(Duration::from_secs(1)).await;
        self.start().await?;
        Ok(())
    }

    // 错误处理
    async fn handle_error(&self, error: ProcessError) -> Result<()> {
        let mut info = self.process_info.write().await;
        info.status = ProcessStatus::Failed(error.to_string());
        info.last_error = Some(error.to_string());

        // 根据错误类型记录不同级别的日志
        match &error {
            ProcessError::AlreadyRunning => warn!("进程已在运行中: {}", error),
            ProcessError::NotRunning => warn!("进程未运行: {}", error),
            ProcessError::StartFailed(msg) => error!("启动失败: {}", msg),
            ProcessError::StopFailed(msg) => error!("停止失败: {}", msg),
            ProcessError::StatusCheckFailed(msg) => error!("状态检查失败: {}", msg),
            ProcessError::ConfigError(msg) => error!("配置错误: {}", msg),
            ProcessError::SystemError(msg) => error!("系统错误: {}", msg),
            ProcessError::PermissionError(msg) => error!("权限错误: {}", msg),
            ProcessError::NetworkError(msg) => error!("网络错误: {}", msg),
            ProcessError::Unknown(msg) => error!("未知错误: {}", msg),
        }

        // 记录额外的系统信息
        if let Some(pid) = info.pid {
            info!("相关进程PID: {}", pid);
        }

        Ok(())
    }
}
