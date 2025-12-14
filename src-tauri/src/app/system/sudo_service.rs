use serde::Serialize;

/// 统一给前端/调用方识别的错误码前缀（避免依赖具体文案）。
/// 约定：Rust 端返回 `SUDO_PASSWORD_REQUIRED` / `SUDO_PASSWORD_INVALID` 等，
/// 前端可据此弹出“请输入系统密码”的窗口。
pub const SUDO_PASSWORD_REQUIRED: &str = "SUDO_PASSWORD_REQUIRED";
pub const SUDO_PASSWORD_INVALID: &str = "SUDO_PASSWORD_INVALID";
pub const SUDO_UNSUPPORTED: &str = "SUDO_UNSUPPORTED";

#[derive(Debug, Clone, Serialize)]
pub struct SudoPasswordStatus {
    pub supported: bool,
    pub has_saved: bool,
}

/// 查询当前平台是否支持“保存并复用 sudo 密码”能力，以及是否已保存。
#[tauri::command]
pub fn sudo_password_status() -> Result<SudoPasswordStatus, String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let has_saved = has_saved_password()?;
        Ok(SudoPasswordStatus {
            supported: true,
            has_saved,
        })
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Ok(SudoPasswordStatus {
            supported: false,
            has_saved: false,
        })
    }
}

/// 设置/更新 sudo 密码：会先校验密码是否正确，正确才写入系统密钥环。
#[tauri::command]
pub fn sudo_set_password(_password: String) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        // 必要的安全措施：不保存无效密码，避免后续启动卡死/失败。
        validate_sudo_password(&_password)?;
        set_saved_password(&_password)?;
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err(SUDO_UNSUPPORTED.to_string())
    }
}

/// 清除已保存的 sudo 密码（例如用户修改了系统密码后需要重新设置）。
#[tauri::command]
pub fn sudo_clear_password() -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        delete_saved_password()?;
        Ok(())
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err(SUDO_UNSUPPORTED.to_string())
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn keyring_entry() -> Result<keyring::Entry, String> {
    // service 名尽量稳定，避免不同版本之间互相覆盖/找不到
    const SERVICE: &str = "sing-box-windows.sudo";
    // user 作为 keyring 的 “账户名”，用当前登录用户更直观
    let user = std::env::var("USER")
        .or_else(|_| std::env::var("USERNAME"))
        .unwrap_or_else(|_| "default".to_string());

    keyring::Entry::new(SERVICE, &user).map_err(|e| format!("初始化系统密钥环失败: {}", e))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn has_saved_password() -> Result<bool, String> {
    let entry = keyring_entry()?;
    match entry.get_password() {
        Ok(pwd) => Ok(!pwd.is_empty()),
        Err(err) => {
            // NoEntry/NotFound 等错误在不同后端实现上文案不同，这里统一视为“未保存”
            let msg = err.to_string().to_lowercase();
            if msg.contains("no entry")
                || msg.contains("not found")
                || msg.contains("no password")
                || msg.contains("credential not found")
            {
                Ok(false)
            } else {
                Err(format!("读取系统密钥环失败: {}", err))
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn get_saved_password() -> Result<Option<String>, String> {
    let entry = keyring_entry()?;
    match entry.get_password() {
        Ok(pwd) if !pwd.is_empty() => Ok(Some(pwd)),
        Ok(_) => Ok(None),
        Err(err) => {
            let msg = err.to_string().to_lowercase();
            if msg.contains("no entry")
                || msg.contains("not found")
                || msg.contains("no password")
                || msg.contains("credential not found")
            {
                Ok(None)
            } else {
                Err(format!("读取系统密钥环失败: {}", err))
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn set_saved_password(password: &str) -> Result<(), String> {
    let entry = keyring_entry()?;
    entry
        .set_password(password)
        .map_err(|e| format!("写入系统密钥环失败: {}", e))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn delete_saved_password() -> Result<(), String> {
    let entry = keyring_entry()?;
    match entry.delete_credential() {
        Ok(_) => Ok(()),
        Err(err) => {
            let msg = err.to_string().to_lowercase();
            // 未保存时删除也视为成功
            if msg.contains("no entry")
                || msg.contains("not found")
                || msg.contains("no password")
                || msg.contains("credential not found")
            {
                Ok(())
            } else {
                Err(format!("清除系统密钥环失败: {}", err))
            }
        }
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn validate_sudo_password(password: &str) -> Result<(), String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    // 说明：
    // - `-S`：从 stdin 读取密码（不依赖 TTY）
    // - `-k`：强制重新认证，确保我们真的验证了当前密码是否正确
    // - `-p ''`：禁用提示符，避免输出干扰
    // - `-v`：仅校验/刷新凭据，不执行命令
    let mut child = Command::new("sudo")
        .args(["-S", "-k", "-p", "", "-v"])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::piped())
        .spawn()
        .map_err(|e| format!("执行 sudo 校验失败: {}", e))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(password.as_bytes())
            .and_then(|_| stdin.write_all(b"\n"))
            .map_err(|e| format!("写入 sudo 密码失败: {}", e))?;
    }

    let output = child
        .wait_with_output()
        .map_err(|e| format!("等待 sudo 校验失败: {}", e))?;

    if output.status.success() {
        return Ok(());
    }

    let stderr = String::from_utf8_lossy(&output.stderr).to_lowercase();
    if stderr.contains("sorry")
        || stderr.contains("incorrect")
        || stderr.contains("authentication failure")
        || stderr.contains("try again")
    {
        return Err(SUDO_PASSWORD_INVALID.to_string());
    }

    Err(format!("sudo 校验失败: {}", stderr.trim()))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn can_run_sudo_non_interactive() -> bool {
    use std::process::Command;
    // `-n`：非交互模式，若需要密码则直接失败
    Command::new("sudo")
        .args(["-n", "true"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Linux/macOS: 读取已保存密码并用 sudo 提权启动内核。
///
/// 设计目标：
/// - 第一次使用由前端弹窗输入系统密码（本函数在未保存时返回 `SUDO_PASSWORD_REQUIRED`）
/// - 每次启动前先用 `sudo -S -k -v` 校验/刷新凭据
/// - 尽量用 `sudo -n` 启动内核，避免把密码写进内核 stdin
#[cfg(any(target_os = "linux", target_os = "macos"))]
pub fn spawn_kernel_with_saved_password(
    kernel_path: &str,
    work_dir: &str,
    config_path: &str,
) -> Result<std::process::Child, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let Some(password) = get_saved_password()? else {
        return Err(SUDO_PASSWORD_REQUIRED.to_string());
    };

    // 如果用户改了系统密码，这里会失败：我们要清除旧密码并提示重新设置。
    if let Err(err) = validate_sudo_password(&password) {
        // 密码不对时，避免后续反复失败，直接清空缓存。
        if err == SUDO_PASSWORD_INVALID {
            let _ = delete_saved_password();
            return Err(SUDO_PASSWORD_INVALID.to_string());
        }
        return Err(err);
    }

    // 首选：非交互 sudo（更安全，避免密码进入内核 stdin）
    if can_run_sudo_non_interactive() {
        let mut cmd = Command::new("sudo");
        cmd.args(["-n", "--", kernel_path, "run", "-D", work_dir, "-c", config_path])
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());

        return cmd
            .spawn()
            .map_err(|e| format!("sudo 启动内核失败: {}", e));
    }

    // 回退：策略要求每次都输入密码（例如 timestamp_timeout=0）
    // 这里用 `-S -k` 强制 sudo 读取密码，因此密码不会泄露给内核 stdin。
    let mut cmd = Command::new("sudo");
    cmd.args(["-S", "-k", "-p", "", "--", kernel_path, "run", "-D", work_dir, "-c", config_path])
        .stdin(Stdio::piped())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    let mut child = cmd
        .spawn()
        .map_err(|e| format!("sudo 启动内核失败: {}", e))?;

    if let Some(stdin) = child.stdin.as_mut() {
        stdin
            .write_all(password.as_bytes())
            .and_then(|_| stdin.write_all(b"\n"))
            .map_err(|e| format!("写入 sudo 密码失败: {}", e))?;
    }

    Ok(child)
}
