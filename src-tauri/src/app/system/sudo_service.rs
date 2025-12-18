use serde::Serialize;
use tauri::{AppHandle, Manager};

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
pub async fn sudo_password_status(app: AppHandle) -> Result<SudoPasswordStatus, String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let has_saved = has_saved_password(&app).await?;
        return Ok(SudoPasswordStatus {
            supported: true,
            has_saved,
        });
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Ok(SudoPasswordStatus {
            supported: false,
            has_saved: false,
        })
    }
}

/// 设置/更新 sudo 密码：会先校验密码是否正确，正确才加密写入数据库。
#[tauri::command]
pub async fn sudo_set_password(password: String, app: AppHandle) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        // 必要的安全措施：不保存无效密码，避免后续启动卡死/失败。
        validate_sudo_password(&password)?;
        save_password(&app, &password).await?;
        return Ok(());
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err(SUDO_UNSUPPORTED.to_string())
    }
}

/// 清除已保存的 sudo 密码（例如用户修改了系统密码后需要重新设置）。
#[tauri::command]
pub async fn sudo_clear_password(app: AppHandle) -> Result<(), String> {
    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        delete_saved_password(&app).await?;
        return Ok(());
    }

    #[cfg(not(any(target_os = "linux", target_os = "macos")))]
    {
        Err(SUDO_UNSUPPORTED.to_string())
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
use {
    base64::engine::general_purpose::STANDARD as BASE64_ENGINE,
    base64::Engine,
    aes_gcm::{
        aead::{Aead, KeyInit},
        Aes256Gcm, Nonce,
    },
    rand::RngCore,
    sha2::{Digest, Sha256},
    tracing::warn,
};

#[cfg(any(target_os = "linux", target_os = "macos"))]
const SUDO_PASSWORD_KEY: &str = "sudo_password_cipher_v1";
#[cfg(any(target_os = "linux", target_os = "macos"))]
const NONCE_LEN: usize = 12;

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn derive_crypto_key(app: &AppHandle) -> Result<[u8; 32], String> {
    let data_dir = app
        .path()
        .app_data_dir()
        .map_err(|e| format!("无法定位应用数据目录: {}", e))?;

    let mut hasher = Sha256::new();
    hasher.update(data_dir.to_string_lossy().as_bytes());
    hasher.update(b"|sing-box-windows|sudo|v1");
    let digest = hasher.finalize();

    let mut key = [0u8; 32];
    key.copy_from_slice(&digest);
    Ok(key)
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn encrypt_password(app: &AppHandle, password: &str) -> Result<String, String> {
    let key = derive_crypto_key(app)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| format!("初始化加密器失败: {}", e))?;

    let mut nonce_bytes = [0u8; NONCE_LEN];
    rand::thread_rng().fill_bytes(&mut nonce_bytes);
    let nonce = Nonce::from_slice(&nonce_bytes);

    let ciphertext = cipher
        .encrypt(nonce, password.as_bytes())
        .map_err(|e| format!("加密密码失败: {}", e))?;

    let mut combined = Vec::with_capacity(NONCE_LEN + ciphertext.len());
    combined.extend_from_slice(&nonce_bytes);
    combined.extend_from_slice(&ciphertext);

    Ok(BASE64_ENGINE.encode(combined))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
fn decrypt_password(app: &AppHandle, encoded: &str) -> Result<String, String> {
    let raw = BASE64_ENGINE
        .decode(encoded)
        .map_err(|e| format!("解码密码失败: {}", e))?;
    if raw.len() <= NONCE_LEN {
        return Err("保存的密码数据已损坏，请重新输入".to_string());
    }

    let (nonce_bytes, cipher_bytes) = raw.split_at(NONCE_LEN);
    let key = derive_crypto_key(app)?;
    let cipher =
        Aes256Gcm::new_from_slice(&key).map_err(|e| format!("初始化解密器失败: {}", e))?;

    let plaintext = cipher
        .decrypt(Nonce::from_slice(nonce_bytes), cipher_bytes)
        .map_err(|e| format!("解密密码失败: {}", e))?;

    String::from_utf8(plaintext).map_err(|e| format!("解密后的密码不是有效 UTF-8: {}", e))
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn has_saved_password(app: &AppHandle) -> Result<bool, String> {
    Ok(load_saved_password(app).await?.is_some())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn load_saved_password(app: &AppHandle) -> Result<Option<String>, String> {
    use crate::app::storage::enhanced_storage_service::get_enhanced_storage;

    let storage = get_enhanced_storage(app).await?;
    let cipher: Option<String> = storage
        .get_config(SUDO_PASSWORD_KEY)
        .await
        .map_err(|e| e.to_string())?;

    if let Some(cipher) = cipher {
        match decrypt_password(app, &cipher) {
            Ok(pwd) if !pwd.is_empty() => Ok(Some(pwd)),
            Ok(_) => Ok(None),
            Err(err) => {
                warn!("保存的 sudo 密码解密失败，清除缓存: {}", err);
                let _ = storage.remove_config(SUDO_PASSWORD_KEY).await;
                Ok(None)
            }
        }
    } else {
        Ok(None)
    }
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn save_password(app: &AppHandle, password: &str) -> Result<(), String> {
    use crate::app::storage::enhanced_storage_service::get_enhanced_storage;

    let cipher = encrypt_password(app, password)?;
    let storage = get_enhanced_storage(app).await?;
    storage
        .save_config(SUDO_PASSWORD_KEY, &cipher)
        .await
        .map_err(|e| e.to_string())
}

#[cfg(any(target_os = "linux", target_os = "macos"))]
async fn delete_saved_password(app: &AppHandle) -> Result<(), String> {
    use crate::app::storage::enhanced_storage_service::get_enhanced_storage;

    let storage = get_enhanced_storage(app).await?;
    storage
        .remove_config(SUDO_PASSWORD_KEY)
        .await
        .map_err(|e| e.to_string())
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
pub async fn spawn_kernel_with_saved_password(
    app_handle: &AppHandle,
    kernel_path: &str,
    work_dir: &str,
    config_path: &str,
) -> Result<std::process::Child, String> {
    use std::io::Write;
    use std::process::{Command, Stdio};

    let saved = load_saved_password(app_handle).await?;
    let Some(password) = saved else {
        return Err(SUDO_PASSWORD_REQUIRED.to_string());
    };

    // 如果用户改了系统密码，这里会失败：我们要清除旧密码并提示重新设置。
    if let Err(err) = validate_sudo_password(&password) {
        // 密码不对时，避免后续反复失败，直接清空缓存。
        if err == SUDO_PASSWORD_INVALID {
            let _ = delete_saved_password(app_handle).await;
            return Err(format!(
                "{}: saved password cleared, please re-enter",
                SUDO_PASSWORD_INVALID
            ));
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
