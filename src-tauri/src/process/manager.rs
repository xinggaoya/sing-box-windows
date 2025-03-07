use super::{ProcessError, ProcessInfo, ProcessStatus, Result};
use crate::utils::app_util::get_work_dir;
use std::os::windows::process::CommandExt;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use crate::utils::proxy_util::disable_system_proxy;
use crate::app::constants::{paths, process, messages};

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

    // 检查是否存在其他 sing-box 进程
    async fn check_other_sing_box_process(&self) -> Option<u32> {
        // 获取自己的PID以排除
        let self_pid = {
            let info = self.process_info.read().await;
            info.pid
        };
        
        match std::process::Command::new("tasklist")
            .arg("/FI")
            .arg("IMAGENAME eq sing-box.exe")
            .arg("/FO")
            .arg("CSV")
            .arg("/NH")
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout);
                if output_str.trim().is_empty() {
                    return None; // 没有sing-box进程
                }
                
                // 解析所有sing-box进程的PID
                for line in output_str.lines() {
                    let parts: Vec<&str> = line.split(',').collect();
                    if parts.len() >= 2 {
                        // 提取PID并转换为u32
                        if let Ok(pid) = parts[1].trim_matches('"').parse::<u32>() {
                            // 排除自己的PID
                            if self_pid != Some(pid) {
                                info!("发现其他sing-box进程: PID={}", pid);
                                return Some(pid);
                            }
                        }
                    }
                }
                None
            }
            Err(e) => {
                error!("检查其他sing-box进程失败: {}", e);
                None
            }
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
        self._is_running(false).await
    }

    // 启动前检查
    async fn pre_start_check(&self) -> Result<()> {
        // 强制检查进程状态，确保状态与实际一致
        let force_check = true;
        let is_running = self._is_running(force_check).await;
        
        // 如果当前实例的进程在运行，先尝试停止它
        if is_running {
            info!("检测到应用内核已运行，尝试强制停止");
            match self.force_stop().await {
                Ok(_) => info!("成功停止当前运行的内核"),
                Err(e) => {
                    warn!("无法停止当前运行的内核: {}", e);
                    // 即使当前无法停止，仍然继续尝试下一步
                }
            }
            
            // 重置状态，以确保干净的启动环境
            self.reset_process_state().await;
        }

        // 检查是否有其他sing-box进程在运行（通过进程名称匹配）
        if self.is_process_running_by_name("sing-box.exe").await {
            info!("检测到其他sing-box进程正在运行，尝试强制停止所有实例");
            if let Err(e) = self.kill_process_by_name("sing-box.exe").await {
                warn!("无法停止部分sing-box进程: {}", e);
            }
            
            // 等待进程完全停止
            sleep(Duration::from_secs(1)).await;
            
            // 再次检查进程是否已全部停止
            if self.is_process_running_by_name("sing-box.exe").await {
                warn!("仍有sing-box进程在运行，将继续尝试启动");
            }
        }

        // 检查配置文件
        self.check_config().await?;

        Ok(())
    }

    // 通过进程名称检查进程是否在运行
    async fn is_process_running_by_name(&self, process_name: &str) -> bool {
        let query = format!("IMAGENAME eq {}", process_name);
        
        match std::process::Command::new("tasklist")
            .arg("/FI")
            .arg(query)
            .arg("/FO")
            .arg("CSV")
            .arg("/NH")
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()
        {
            Ok(output) => {
                let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                !output_str.is_empty() && output_str.contains(process_name)
            }
            Err(e) => {
                error!("检查进程名称失败: {}", e);
                false
            }
        }
    }

    // 通过进程名称终止所有匹配的进程
    async fn kill_process_by_name(&self, process_name: &str) -> std::io::Result<()> {
        // 使用taskkill /IM 命令强制终止所有匹配的进程
        let output = std::process::Command::new("taskkill")
            .arg("/F") // 强制终止
            .arg("/IM")
            .arg(process_name)
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()?;
        
        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            if error.contains("没有运行的任务") {
                // 忽略"没有运行的任务"错误，这意味着没有找到匹配的进程
                return Ok(());
            }
            error!("终止进程失败: {}", error);
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other, 
                format!("终止进程失败: {}", error)
            ));
        }
        
        info!("已终止所有 {} 进程", process_name);
        Ok(())
    }

    // 检查配置文件
    async fn check_config(&self) -> Result<()> {
        info!("当前工作目录: {}", get_work_dir());
        
        // 检查配置文件是否存在
        let config_path = paths::get_config_path();
        if !config_path.exists() {
            return Err(ProcessError::ConfigError(messages::ERR_CONFIG_READ_FAILED.to_string()));
        }

        // 验证配置文件
        let config_str = std::fs::read_to_string(&config_path)
            .map_err(|e| ProcessError::ConfigError(format!("{}: {}", messages::ERR_CONFIG_READ_FAILED, e)))?;

        // 解析JSON
        let json_result: serde_json::Result<serde_json::Value> = serde_json::from_str(&config_str);
        if let Err(e) = json_result {
            return Err(ProcessError::ConfigError(format!("配置文件JSON格式错误: {}", e)));
        }

        // 验证配置有效性 - 使用sing-box自带的验证功能
        let kernel_path = paths::get_kernel_path();
        let output = std::process::Command::new(&kernel_path)
            .arg("check")
            .arg("-c")
            .arg(&config_path)
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()
            .map_err(|e| ProcessError::ConfigError(format!("无法验证配置: {}", e)))?;

        if !output.status.success() {
            let error_output = String::from_utf8_lossy(&output.stderr);
            return Err(ProcessError::ConfigError(format!("配置无效: {}", error_output)));
        }

        info!("配置文件检查通过");
        
        // 如果输出为空，则配置有效
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

        // 获取工作目录和内核路径
        let kernel_work_dir = paths::get_kernel_work_dir();
        let kernel_path = paths::get_kernel_path();

        // 启动进程
        let child = match Command::new(kernel_path.to_str().unwrap())
            .arg("run")
            .arg("-D")
            .arg(kernel_work_dir.to_str().unwrap())
            .creation_flags(process::CREATE_NO_WINDOW)
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

        info!("{}", messages::INFO_PROCESS_STARTED);
        Ok(())
    }

    // 停止进程
    pub async fn stop(&self) -> Result<()> {
        // 检查进程状态
        let status = self.get_status().await.status;
        if matches!(status, ProcessStatus::Stopped) {
            return Ok(());
        }

        // 更新状态为停止中
        {
            let mut info = self.process_info.write().await;
            info.status = ProcessStatus::Stopping;
        }

        // 首先尝试优雅地停止进程
        // let stop_result = self.graceful_stop().await;
        
        // 如果优雅停止失败，则强制终止
        // if stop_result.is_err() {
            warn!("进程停止超时，尝试强制终止");
            self.force_stop().await?;
        // }
        
        // 关闭系统代理
        if let Err(e) = disable_system_proxy() {
            warn!("关闭系统代理失败: {}", e);
        } else {
            info!("{}", messages::INFO_SYSTEM_PROXY_DISABLED);
        }

        // 更新进程状态
        {
            let mut info = self.process_info.write().await;
            info.status = ProcessStatus::Stopped;
            info.pid = None;
        }

        info!("{}", messages::INFO_PROCESS_STOPPED);
        Ok(())
    }

    // 发送停止信号
    fn send_signal(&self, pid: u32) -> std::io::Result<()> {
        std::process::Command::new("taskkill")
            .arg("/PID")
            .arg(pid.to_string())
            .creation_flags(process::CREATE_NO_WINDOW)
            .output()?;
        Ok(())
    }

    // 强制结束进程
    fn kill_process(&self, pid: u32) -> std::io::Result<()> {
        std::process::Command::new("taskkill")
            .arg("/F")
            .arg("/PID")
            .arg(pid.to_string())
            .creation_flags(process::CREATE_NO_WINDOW)
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
    async fn handle_error(&self, err: ProcessError) -> Result<()> {
        let mut info = self.process_info.write().await;
        info.status = ProcessStatus::Failed(err.to_string());
        info.last_error = Some(err.to_string());
        error!("进程错误: {}", err);
        Ok(())
    }

    // 优雅停止进程
    async fn graceful_stop(&self) -> Result<()> {
        let pid = {
            let info = self.process_info.read().await;
            info.pid.ok_or(ProcessError::NotRunning)?
        };

        // 尝试发送正常停止信号
        if let Err(e) = self.send_signal(pid) {
            return Err(ProcessError::StopFailed(format!("发送停止信号失败: {}", e)));
        }

        // 等待进程停止的超时时间
        let timeout = Duration::from_secs(process::GRACEFUL_TIMEOUT);
        let start = std::time::Instant::now();

        // 等待进程停止
        while self.check_process_exists(Some(pid)).await {
            if start.elapsed() > timeout {
                return Err(ProcessError::StopFailed("进程停止超时".to_string()));
            }
            sleep(Duration::from_millis(100)).await;
        }

        Ok(())
    }

    // 强制停止进程
    async fn force_stop(&self) -> Result<()> {
        let pid = {
            let info = self.process_info.read().await;
            info.pid.ok_or(ProcessError::NotRunning)?
        };

        // 强制结束进程
        if let Err(e) = self.kill_process(pid) {
            return Err(ProcessError::StopFailed(format!("强制停止失败: {}", e)));
        }

        // 短暂等待确保进程已终止
        sleep(Duration::from_millis(500)).await;
        
        // 检查进程是否仍存在
        if self.check_process_exists(Some(pid)).await {
            return Err(ProcessError::StopFailed("强制停止失败，进程仍在运行".to_string()));
        }

        Ok(())
    }

    // 检查进程是否存在
    async fn check_process_exists(&self, pid: Option<u32>) -> bool {
        if let Some(pid) = pid {
            // 使用具体的PID格式化查询，确保准确匹配
            let query = format!("PID eq {}", pid);
            
            match std::process::Command::new("tasklist")
                .arg("/FI")
                .arg(query)
                .arg("/FO")
                .arg("CSV")
                .arg("/NH") // 不显示标题行
                .creation_flags(process::CREATE_NO_WINDOW)
                .output()
            {
                Ok(output) => {
                    let output_str = String::from_utf8_lossy(&output.stdout).trim().to_string();
                    // 检查输出是否包含PID
                    // 输出格式应该是 "sing-box.exe","PID",...
                    if output_str.is_empty() {
                        return false;
                    }
                    
                    // 更严格地检查PID是否匹配
                    output_str.contains(&format!(",\"{}\"", pid))
                }
                Err(_) => {
                    error!("查询进程状态失败");
                    false
                }
            }
        } else {
            false
        }
    }

    // 内部是否运行中检查函数，可以强制检查实际进程
    async fn _is_running(&self, force_check: bool) -> bool {
        // 首先检查进程信息
        let info = self.process_info.read().await;
        let status_running = matches!(
            info.status,
            ProcessStatus::Running | ProcessStatus::Starting
        );

        // 如果状态显示为非运行，且不强制检查，直接返回false
        if !status_running && !force_check {
            return false;
        }

        // 如果没有PID，说明进程未运行
        if info.pid.is_none() {
            if status_running {
                // 状态不一致，需要重置
                drop(info); // 释放读锁
                self.reset_process_state().await;
                warn!("进程状态显示运行中，但没有PID，已重置状态");
            }
            return false;
        }

        // 检查进程是否实际存在
        let pid = info.pid.unwrap(); // 安全，因为已经检查了is_none
        let exists = self.check_process_exists(Some(pid)).await;
        
        if !exists && status_running {
            // 进程不存在但状态显示运行中，重置状态
            drop(info); // 释放读锁
            self.reset_process_state().await;
            warn!("进程状态显示运行中 (PID: {})，但实际进程不存在，已重置状态", pid);
            return false;
        }

        // 返回实际进程存在状态
        exists
    }
}

