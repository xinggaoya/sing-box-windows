use crate::app::core::kernel_service::log_rotation;
use crate::app::core::kernel_service::runtime::{
    resolve_proxy_runtime_state, start_kernel_impl, ProxyOverrides,
};
use crate::app::core::kernel_service::state::KERNEL_STATE;
use crate::app::core::kernel_service::status::is_kernel_running;
use crate::app::core::kernel_service::utils::{emit_kernel_error_with_context, emit_kernel_stopped};
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::time::Duration;
use std::time::Instant;
use tauri::AppHandle;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{info, warn};

static KEEP_ALIVE_ENABLED: AtomicBool = AtomicBool::new(false);
static GUARDED_API_PORT: AtomicU16 = AtomicU16::new(0);
static GUARDED_TUN_ENABLED: AtomicBool = AtomicBool::new(false);

/// 连通性自愈：连续失败达到该阈值即触发一次内核重启。
const CONNECTIVITY_FAIL_THRESHOLD: u8 = 3;
/// 自愈的初始冷却时间，避免启动后立即触发。
const SELF_HEAL_WARMUP_SECS: u64 = 20;
/// 守护循环周期。
const GUARD_TICK_SECS: u64 = 8;
/// 日志周期滚动检查间隔（用循环累计计时，不新增定时器）。
const LOG_ROTATION_INTERVAL_SECS: u64 = 6 * 60 * 60;

#[derive(Debug, Clone, Copy)]
struct SelfHealPolicy {
    enabled: bool,
    cooldown_secs: u64,
}

impl SelfHealPolicy {
    fn default_policy() -> Self {
        Self {
            enabled: true,
            cooldown_secs: 90,
        }
    }
}

lazy_static::lazy_static! {
    pub(super) static ref KERNEL_GUARD_HANDLE: Mutex<Option<JoinHandle<()>>> =
        Mutex::new(None);
}

/// 读取自愈策略。
///
/// 复用原 TUN 自愈配置项（`tun_self_heal_enabled` / `tun_self_heal_cooldown_secs`），
/// 现在适用于所有代理模式（system/manual/tun），避免引入新的配置项。
async fn load_self_heal_policy(app_handle: &AppHandle) -> SelfHealPolicy {
    match db_get_app_config(app_handle.clone()).await {
        Ok(config) => SelfHealPolicy {
            enabled: config.tun_self_heal_enabled,
            cooldown_secs: u64::from(config.tun_self_heal_cooldown_secs).clamp(15, 600),
        },
        Err(err) => {
            warn!("读取自愈策略失败，回退默认值: {}", err);
            SelfHealPolicy::default_policy()
        }
    }
}

/// 守护/自愈触发的统一重启入口。
///
/// 复用 `start_kernel_impl` 的完整启动逻辑（配置写入、端口就绪后开代理、
/// 稳定性校验、事件中继），保证自愈后内核与代理都可用，避免旧实现裸调
/// `PROCESS_MANAGER.start/restart` 导致"自愈后仍无法访问网络"。
///
/// 通过 `reactivate_guard=false` 调用：守护循环本身已在运行，不重建自身，
/// 也避免 `enable_kernel_guard` 返回的非 Send future 跨 spawn 的问题。
///
/// 返回 true 表示重启成功。sudo 密码失效等不可恢复错误由调用方处理。
async fn heal_restart(app_handle: &AppHandle, reason: &str) -> bool {
    info!("触发内核自愈重启（{}）", reason);

    let overrides = ProxyOverrides::default();
    let resolved = match resolve_proxy_runtime_state(app_handle, overrides).await {
        Ok(state) => state,
        Err(err) => {
            warn!("自愈重启：解析运行态失败: {}", err);
            KERNEL_STATE.mark_failed();
            emit_kernel_error_with_context(
                app_handle,
                "KERNEL_GUARD_SELF_HEAL_FAILED",
                "内核自愈重启失败",
                Some(&err),
                Some("kernel.guard.self_heal"),
                true,
            );
            return false;
        }
    };

    match start_kernel_impl(app_handle.clone(), &resolved, false).await {
        Ok(value) => {
            let success = value
                .get("success")
                .and_then(|v| v.as_bool())
                .unwrap_or(false);
            if success {
                KERNEL_STATE.record_restart(reason);
                info!("内核自愈重启完成（{}）", reason);
                true
            } else {
                let message = value
                    .get("message")
                    .and_then(|v| v.as_str())
                    .unwrap_or("自愈重启未成功")
                    .to_string();
                warn!("自愈重启未成功: {}", message);
                KERNEL_STATE.mark_failed();
                emit_kernel_error_with_context(
                    app_handle,
                    "KERNEL_GUARD_SELF_HEAL_FAILED",
                    "内核自愈重启失败",
                    Some(&message),
                    Some("kernel.guard.self_heal"),
                    true,
                );
                false
            }
        }
        Err(err) => {
            warn!("自愈重启异常: {}", err);
            KERNEL_STATE.mark_failed();
            emit_kernel_error_with_context(
                app_handle,
                "KERNEL_GUARD_SELF_HEAL_FAILED",
                "内核自愈重启失败",
                Some(&err),
                Some("kernel.guard.self_heal"),
                true,
            );
            false
        }
    }
}

/// 判断错误是否因 sudo 密码失效（不可恢复），需停止守护并提示用户。
fn is_sudo_failure(err_str: &str) -> bool {
    err_str.contains("SUDO_PASSWORD_REQUIRED") || err_str.contains("SUDO_PASSWORD_INVALID")
}

/// 关闭守护并清理其静态状态（用于 sudo 失效等需停止守护的场景）。
fn shutdown_guard() {
    KEEP_ALIVE_ENABLED.store(false, Ordering::Relaxed);
    GUARDED_API_PORT.store(0, Ordering::Relaxed);
    GUARDED_TUN_ENABLED.store(false, Ordering::Relaxed);
}

pub(super) async fn enable_kernel_guard(app_handle: AppHandle, api_port: u16, tun_enabled: bool) {
    GUARDED_API_PORT.store(api_port, Ordering::Relaxed);
    GUARDED_TUN_ENABLED.store(tun_enabled, Ordering::Relaxed);
    if KEEP_ALIVE_ENABLED.swap(true, Ordering::Relaxed) {
        return;
    }

    let guard_handle = spawn_guard_loop(app_handle);

    let mut handle_slot = KERNEL_GUARD_HANDLE.lock().await;
    *handle_slot = Some(guard_handle);
}

/// 启动守护循环任务。
///
/// 单独抽出，使 `enable_kernel_guard` 中锁的持有不与 spawn 混在同一个 await 链里，
/// 从而保证 `enable_kernel_guard` 返回的 future 满足 `Send`（可被自愈路径经
/// `start_kernel_impl` 在 `tokio::spawn` 中调用）。
fn spawn_guard_loop(app_handle: AppHandle) -> JoinHandle<()> {
    tokio::spawn(async move {
        info!("内核守护已启动");
        let mut connectivity_failures: u8 = 0;
        let mut next_self_heal_at = Instant::now() + Duration::from_secs(SELF_HEAL_WARMUP_SECS);
        let mut last_log_rotation_at = Instant::now();

        loop {
            if !KEEP_ALIVE_ENABLED.load(Ordering::Relaxed) {
                break;
            }

            tokio::time::sleep(Duration::from_secs(GUARD_TICK_SECS)).await;

            if !KEEP_ALIVE_ENABLED.load(Ordering::Relaxed) {
                break;
            }

            // 周期性检查内核日志大小并滚动，避免长期运行无限增长（不只在启动时滚动一次）。
            if last_log_rotation_at.elapsed() >= Duration::from_secs(LOG_ROTATION_INTERVAL_SECS) {
                let log_path = std::path::PathBuf::from(
                    crate::app::singbox::common::kernel_log_output_path(),
                );
                log_rotation::rotate_if_needed(&log_path);
                last_log_rotation_at = Instant::now();
            }

            match is_kernel_running().await {
                Ok(true) => {
                    // 所有代理模式都做连通性自愈：进程活着但假死时也能恢复。
                    let policy = load_self_heal_policy(&app_handle).await;
                    if !policy.enabled {
                        connectivity_failures = 0;
                        next_self_heal_at =
                            Instant::now() + Duration::from_secs(SELF_HEAL_WARMUP_SECS);
                        continue;
                    }

                    let mut should_attempt_self_heal = false;
                    match crate::app::system::system_service::check_network_connectivity(Some(false))
                        .await
                    {
                        Ok(true) => {
                            if connectivity_failures > 0 {
                                info!("连通性已恢复，清空失败计数");
                            }
                            connectivity_failures = 0;
                        }
                        Ok(false) => {
                            connectivity_failures = connectivity_failures.saturating_add(1);
                            warn!(
                                "连通性检测失败，计数: {}/{}",
                                connectivity_failures, CONNECTIVITY_FAIL_THRESHOLD
                            );
                            should_attempt_self_heal =
                                connectivity_failures >= CONNECTIVITY_FAIL_THRESHOLD;
                        }
                        Err(err) => {
                            connectivity_failures = connectivity_failures.saturating_add(1);
                            warn!(
                                "连通性检测异常，计数: {}/{}，错误: {}",
                                connectivity_failures, CONNECTIVITY_FAIL_THRESHOLD, err
                            );
                            should_attempt_self_heal =
                                connectivity_failures >= CONNECTIVITY_FAIL_THRESHOLD;
                        }
                    }

                    if should_attempt_self_heal && Instant::now() >= next_self_heal_at {
                        let mode_label = if GUARDED_TUN_ENABLED.load(Ordering::Relaxed) {
                            "tun-connectivity"
                        } else {
                            "system-connectivity"
                        };
                        let succeeded = heal_restart(&app_handle, mode_label).await;
                        // 复位计数与冷却；无论成败都进入冷却窗口避免抖动。
                        connectivity_failures = 0;
                        next_self_heal_at = Instant::now() + Duration::from_secs(policy.cooldown_secs);
                        if !succeeded {
                            // heal_restart 内部已标记 failed 并上报错误，此处不额外处理。
                            // 若是 sudo 失效等不可恢复错误，停止守护避免无意义重试。
                            let should_stop = KERNEL_STATE
                                .get_startup_diagnosis()
                                .map(|d| is_sudo_failure(&d.detail))
                                .unwrap_or(false);
                            if should_stop {
                                emit_kernel_error_with_context(
                                    &app_handle,
                                    "KERNEL_GUARD_SUDO_INVALID",
                                    "TUN 提权失败：sudo 密码无效，请重新输入系统密码后重启内核。",
                                    None,
                                    Some("kernel.guard.self_heal"),
                                    false,
                                );
                                shutdown_guard();
                                break;
                            }
                        }
                    }

                    continue;
                }
                _ => {
                    let port_value = GUARDED_API_PORT.load(Ordering::Relaxed);
                    let tun_enabled = GUARDED_TUN_ENABLED.load(Ordering::Relaxed);
                    info!(
                        "守护检测到内核停止，尝试自动重启: api_port={}, tun_enabled={}",
                        port_value, tun_enabled
                    );
                    KERNEL_STATE.mark_crashed();
                    emit_kernel_stopped(&app_handle);

                    let succeeded = heal_restart(&app_handle, "process-crashed").await;
                    if !succeeded {
                        // sudo 密码失效等不可恢复错误：停止守护，避免无意义的重试循环。
                        let should_stop = KERNEL_STATE
                            .get_startup_diagnosis()
                            .map(|d| is_sudo_failure(&d.detail))
                            .unwrap_or(false);
                        if should_stop {
                            emit_kernel_error_with_context(
                                &app_handle,
                                "KERNEL_GUARD_SUDO_INVALID",
                                "TUN 提权失败：sudo 密码无效，请重新输入系统密码后重启内核。",
                                None,
                                Some("kernel.guard.self_heal"),
                                false,
                            );
                            shutdown_guard();
                            break;
                        }
                    }

                    connectivity_failures = 0;
                    next_self_heal_at = Instant::now() + Duration::from_secs(SELF_HEAL_WARMUP_SECS);
                }
            }
        }

        info!("内核守护任务结束");
    })
}

pub(super) async fn disable_kernel_guard() {
    if !KEEP_ALIVE_ENABLED.swap(false, Ordering::Relaxed) {
        return;
    }

    GUARDED_API_PORT.store(0, Ordering::Relaxed);
    GUARDED_TUN_ENABLED.store(false, Ordering::Relaxed);
    let mut handle_slot = KERNEL_GUARD_HANDLE.lock().await;
    if let Some(handle) = handle_slot.take() {
        handle.abort();
    }
}
