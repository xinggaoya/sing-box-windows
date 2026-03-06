use super::*;

#[test]
fn test_create_hidden_command() {
    let cmd = create_hidden_command("echo");
    #[cfg(target_os = "windows")]
    {
        // 在Windows平台上，验证命令可以被创建
        assert_eq!(cmd.get_program(), "echo");
    }
    #[cfg(not(target_os = "windows"))]
    {
        // 在非Windows平台上，只是验证命令可以被创建
        assert_eq!(cmd.get_program(), "echo");
    }
}

#[test]
fn test_create_hidden_async_command() {
    let _cmd = create_hidden_async_command("echo");
    // 只是验证命令可以被创建，不检查内部属性
    // tokio::process::Command 没有提供 get_program 方法
}

#[cfg(target_os = "windows")]
#[test]
fn test_windows_commands() {
    use windows::*;

    let _tasklist = create_tasklist_command();
    let _taskkill = create_taskkill_command();
    let _wmic = create_wmic_command();
}
