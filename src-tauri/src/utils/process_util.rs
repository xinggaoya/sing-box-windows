/// 进程工具函数
///
/// 提供跨平台的进程创建辅助函数，确保Windows下不显示控制台窗口

#[cfg(target_os = "windows")]
use std::os::windows::process::CommandExt;

/// 创建隐藏窗口的命令
///
/// # Arguments
///
/// * `program` - 要执行的程序
///
/// # Returns
///
/// 返回配置好的 `std::process::Command`，Windows下会隐藏控制台窗口
pub fn create_hidden_command(program: &str) -> std::process::Command {
    let mut cmd = std::process::Command::new(program);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);

    cmd
}

/// 创建隐藏窗口的异步命令
///
/// # Arguments
///
/// * `program` - 要执行的程序
///
/// # Returns
///
/// 返回配置好的 `tokio::process::Command`，Windows下会隐藏控制台窗口
pub fn create_hidden_async_command(program: &str) -> tokio::process::Command {
    let mut cmd = tokio::process::Command::new(program);

    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);

    cmd
}

/// 为现有命令设置隐藏窗口标志
///
/// 这是一个辅助函数，用于确保现有的命令不会显示控制台窗口
///
/// # Arguments
///
/// * `cmd` - 可变的命令引用
///
/// # Examples
///
/// ```
/// let mut cmd = tokio::process::Command::new("tasklist");
/// ensure_hidden_window(&mut cmd);
/// ```
pub fn ensure_hidden_window<T: CommandExt>(cmd: &mut T) {
    #[cfg(target_os = "windows")]
    cmd.creation_flags(crate::app::constants::core::process::CREATE_NO_WINDOW);
}

/// Windows 系统进程创建辅助函数
///
/// 专门用于Windows系统命令，确保隐藏控制台窗口
#[cfg(target_os = "windows")]
pub mod windows {
    use super::*;

    /// 创建 tasklist 命令
    pub fn create_tasklist_command() -> tokio::process::Command {
        create_hidden_async_command("tasklist")
    }

    /// 创建 taskkill 命令
    pub fn create_taskkill_command() -> tokio::process::Command {
        create_hidden_async_command("taskkill")
    }

    /// 创建 wmic 命令
    pub fn create_wmic_command() -> tokio::process::Command {
        create_hidden_async_command("wmic")
    }

    /// 创建 PowerShell 命令
    pub fn create_powershell_command() -> tokio::process::Command {
        create_hidden_async_command("powershell")
    }

    /// 创建 cmd 命令
    pub fn create_cmd_command() -> tokio::process::Command {
        create_hidden_async_command("cmd")
    }

    /// 创建 wscript 命令
    pub fn create_wscript_command() -> std::process::Command {
        create_hidden_command("wscript")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_hidden_command() {
        let cmd = create_hidden_command("echo");
        #[cfg(target_os = "windows")]
        {
            if let Ok(flags) = cmd.get_creation_flags() {
                assert_eq!(flags, crate::app::constants::core::process::CREATE_NO_WINDOW);
            }
        }
    }

    #[test]
    fn test_create_hidden_async_command() {
        let cmd = create_hidden_async_command("echo");
        #[cfg(target_os = "windows")]
        {
            if let Ok(flags) = cmd.get_creation_flags() {
                assert_eq!(flags, crate::app::constants::core::process::CREATE_NO_WINDOW);
            }
        }
    }

    #[cfg(target_os = "windows")]
    #[test]
    fn test_windows_commands() {
        use windows::*;

        let tasklist = create_tasklist_command();
        if let Ok(flags) = tasklist.get_creation_flags() {
            assert_eq!(flags, crate::app::constants::core::process::CREATE_NO_WINDOW);
        }

        let taskkill = create_taskkill_command();
        if let Ok(flags) = taskkill.get_creation_flags() {
            assert_eq!(flags, crate::app::constants::core::process::CREATE_NO_WINDOW);
        }

        let wmic = create_wmic_command();
        if let Ok(flags) = wmic.get_creation_flags() {
            assert_eq!(flags, crate::app::constants::core::process::CREATE_NO_WINDOW);
        }

        let powershell = create_powershell_command();
        if let Ok(flags) = powershell.get_creation_flags() {
            assert_eq!(flags, crate::app::constants::core::process::CREATE_NO_WINDOW);
        }
    }
}