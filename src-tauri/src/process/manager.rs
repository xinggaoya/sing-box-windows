use super::{ProcessError, ProcessInfo, ProcessStatus, Result};
use crate::utils::app_util::get_work_dir;
use log::{error, info, warn};
use std::os::windows::process::CommandExt;
use std::path::Path;
use std::process::Child;
use std::sync::Arc;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};

pub struct ProcessManager {
    process_info: Arc<RwLock<ProcessInfo>>,
    child_process: Arc<RwLock<Option<Child>>>,
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

    // 重置进程状态
    async fn reset_process_state(&self) {
        let mut info = self.process_info.write().await;
        info.status = ProcessStatus::Stopped;
        info.pid = None;
        info.last_error = None;

        let mut process = self.child_process.write().await;
        *process = None;
    }

    // 检查进程是否在运行（修改后的版本）
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

    // 启动前检查（修改后的版本）
    async fn pre_start_check(&self) -> Result<()> {
        let info = self.process_info.read().await;

        // 检查状态和实际进程
        if matches!(
            info.status,
            ProcessStatus::Running | ProcessStatus::Starting
        ) {
            // 如果状态是运行中但进程不存在，重置状态
            if !self.check_process_exists(info.pid).await {
                drop(info); // 释放读锁
                self.reset_process_state().await;
            } else {
                return Err(ProcessError::AlreadyRunning);
            }
        }

        let work_dir = get_work_dir();
        info!("当前工作目录: {}", work_dir);

        let sing_box_dir = Path::new(&work_dir).join("sing-box");
        let kernel_path = sing_box_dir.join("sing-box");
        let config_path = sing_box_dir.join("config.json");

        // 检查 sing-box 目录是否存在
        if !sing_box_dir.exists() {
            return Err(ProcessError::StartFailed("sing-box 目录不存在".to_string()));
        }

        // 检查内核文件是否存在
        if !kernel_path.exists() {
            return Err(ProcessError::StartFailed("内核文件不存在".to_string()));
        }

        // 检查配置文件是否存在
        if !config_path.exists() {
            return Err(ProcessError::StartFailed("配置文件不存在".to_string()));
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
        let kernel_path = Path::new(&work_dir).join("sing-box/sing-box");
        let kernel_work_dir = Path::new(&work_dir).join("sing-box");

        // 启动进程
        let child = match std::process::Command::new(kernel_path.to_str().unwrap())
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
            info.pid = Some(child.id());
            info.status = ProcessStatus::Running;
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

        // 等待进程退出
        let timeout = Duration::from_secs(5);
        let start_time = std::time::Instant::now();

        'wait_loop: while start_time.elapsed() < timeout {
            // 检查进程是否还存在
            if !self.check_process_exists(Some(pid)).await {
                break 'wait_loop;
            }

            let mut process_guard = self.child_process.write().await;
            if let Some(child) = process_guard.as_mut() {
                match child.try_wait() {
                    Ok(Some(_)) => {
                        // 进程已退出
                        *process_guard = None;
                        break 'wait_loop;
                    }
                    Ok(None) => {
                        // 进程仍在运行，释放锁并等待
                        drop(process_guard);
                        sleep(Duration::from_millis(100)).await;
                        continue;
                    }
                    Err(e) => {
                        // 如果出错，检查进程是否真的还在运行
                        if !self.check_process_exists(Some(pid)).await {
                            break 'wait_loop;
                        }
                        return Err(ProcessError::StopFailed(e.to_string()));
                    }
                }
            } else {
                // 如果child_process为None，但进程仍在运行，尝试强制终止
                if self.check_process_exists(Some(pid)).await {
                    if let Err(e) = self.kill_process(pid) {
                        return Err(ProcessError::StopFailed(format!("强制停止失败: {}", e)));
                    }
                }
                break 'wait_loop;
            }
        }

        // 如果超时后进程仍然存在，强制终止
        if self.check_process_exists(Some(pid)).await {
            warn!("进程停止超时，尝试强制终止");
            if let Err(e) = self.kill_process(pid) {
                return Err(ProcessError::StopFailed(format!("强制停止失败: {}", e)));
            }
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

    // 错误处理
    async fn handle_error(&self, error: ProcessError) -> Result<()> {
        let mut info = self.process_info.write().await;
        info.status = ProcessStatus::Failed(error.to_string());
        info.last_error = Some(error.to_string());
        error!("进程错误: {}", error);
        Ok(())
    }
}
