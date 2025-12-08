//! 平台抽象层
//! 
//! 提供跨平台的统一接口，封装各平台特定实现（进程检测、杀进程、系统架构等）。
//! 使用条件编译实现静态分发，避免运行时开销。

#[cfg(target_os = "windows")]
mod windows;
#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;

// 重导出当前平台的实现
#[cfg(target_os = "windows")]
pub use windows::*;
#[cfg(target_os = "linux")]
pub use linux::*;
#[cfg(target_os = "macos")]
pub use macos::*;

/// 检测指定名称的进程是否正在运行
pub async fn is_process_running(process_name: &str) -> Result<bool, String> {
    platform_is_process_running(process_name).await
}

/// 杀死指定名称的所有进程
pub async fn kill_processes_by_name(process_name: &str) -> Result<(), String> {
    platform_kill_processes_by_name(process_name).await
}

/// 杀死指定 PID 的进程
pub fn kill_process_by_pid(pid: u32) -> Result<(), String> {
    platform_kill_process_by_pid(pid)
}

/// 获取系统架构（用于内核下载）
pub fn get_system_arch() -> &'static str {
    platform_get_system_arch()
}

/// 获取当前平台名称（用于内核下载）
pub fn get_platform_name() -> &'static str {
    platform_get_platform_name()
}

/// 获取内核可执行文件名
pub fn get_kernel_executable_name() -> &'static str {
    platform_get_kernel_executable_name()
}

/// 获取系统运行时间（毫秒）
pub async fn get_system_uptime_ms() -> Result<u64, String> {
    platform_get_system_uptime_ms().await
}

/// 为 tokio 进程命令设置平台特定的配置（如隐藏窗口）
pub fn configure_process_command(command: &mut tokio::process::Command) {
    platform_configure_process_command(command);
}

/// 为标准库进程命令设置平台特定的配置
pub fn configure_std_command(command: &mut std::process::Command) {
    platform_configure_std_command(command);
}
