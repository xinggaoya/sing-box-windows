use crate::app::core::kernel_service::log_rotation;
use crate::app::core::kernel_service::runtime::{
    resolve_proxy_runtime_state, start_kernel_impl, ProxyOverrides,
};
use crate::app::core::kernel_service::state::KERNEL_STATE;
use crate::app::core::kernel_service::status::is_kernel_running_ref;
use crate::app::core::kernel_service::utils::{
    emit_kernel_error_with_context, emit_kernel_stopped,
};
use crate::app::singbox_api::client::ApiError;
use crate::app::singbox_api::{ApiClientConfig, ApiClientHandle};
use crate::app::storage::enhanced_storage_service::db_get_app_config;
use std::sync::atomic::{AtomicBool, AtomicU16, Ordering};
use std::time::Duration;
use std::time::Instant;
use tauri::AppHandle;
use tokio::sync::Mutex;
use tokio::task::JoinHandle;
use tracing::{debug, info, warn};

static KEEP_ALIVE_ENABLED: AtomicBool = AtomicBool::new(false);
static GUARDED_API_PORT: AtomicU16 = AtomicU16::new(0);
static GUARDED_PROXY_PORT: AtomicU16 = AtomicU16::new(0);
static GUARDED_TUN_ENABLED: AtomicBool = AtomicBool::new(false);

/// 连通性自愈：连续失败达到该阈值即触发一次恢复动作。
const CONNECTIVITY_FAIL_THRESHOLD: u8 = 3;
/// 自愈的初始冷却时间，避免启动后立即触发。
const SELF_HEAL_WARMUP_SECS: u64 = 20;
/// 守护循环周期。
const GUARD_TICK_SECS: u64 = 8;
/// 日志周期滚动检查间隔（用循环累计计时，不新增定时器）。
const LOG_ROTATION_INTERVAL_SECS: u64 = 6 * 60 * 60;
/// L1/L2 软恢复动作后的观察冷却：给内核时间生效再决定是否升级。
const RECOVERY_OBSERVE_SECS: u64 = 20;
/// 连续重启内核的上限，达到后熔断（停止重启、仅保留轻量自愈）。
const MAX_CONSECUTIVE_RESTARTS: u32 = 2;
/// 熔断后的长冷却：期间只做轻量自愈与探测，冷却结束允许再试一轮完整恢复。
const CIRCUIT_COOLDOWN_SECS: u64 = 10 * 60;
/// 守护内 gRPC 动作的单次限时，避免拖垮守护循环节奏。
const GRPC_ACTION_TIMEOUT_SECS: u64 = 8;

/// 连通性故障的分级恢复动作，按代价从低到高逐级尝试。
///
/// 设计动机：代理探测失败 + 直连正常有两种病因——内核内状态损坏（隧道黑洞、
/// API hang，重启可修）与当前选中节点本身不可用（服务器故障/被墙/订阅过期，
/// 重启无解）。一律重启会对后者陷入"重启→失败→再重启"的循环，
/// 因此先做无感知的软恢复，再尝试切换出站，最后才重启，并以熔断兜底。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecoveryStage {
    /// L1：关闭内核全部连接，废弃可能黑洞化的隧道。新请求会重建出站连接，
    /// 对 QUIC 隧道静默死亡（最常见的假活病因）通常已足够，且用户无感知。
    CloseConnections,
    /// L2：手动选中的具体节点不可用时，把"手动切换"组切到"自动选择"
    /// （urltest 组会自动剔除坏节点）。
    SwitchToAuto,
    /// L3：重启内核，处理进程内部状态损坏。
    RestartKernel,
}

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

fn make_api_client(api_port: u16) -> Result<ApiClientHandle, ApiError> {
    ApiClientHandle::new(ApiClientConfig::localhost(api_port))
}

/// L1 软恢复：关闭内核全部连接，废弃黑洞化的隧道。
///
/// 返回 false 表示 gRPC 调用失败（内核假死更严重，应直接升级到重启）。
async fn recovery_close_all_connections(api_port: u16) -> bool {
    let client = match make_api_client(api_port) {
        Ok(client) => client,
        Err(error) => {
            tracing::warn!("L1 自愈：创建 gRPC 客户端失败: {}", error);
            return false;
        }
    };
    match tokio::time::timeout(
        Duration::from_secs(GRPC_ACTION_TIMEOUT_SECS),
        client.close_all_connections(),
    )
    .await
    {
        Ok(Ok(())) => {
            info!("自愈 L1：已关闭内核全部连接，等待新连接重建隧道");
            true
        }
        Ok(Err(err)) => {
            warn!("自愈 L1：关闭连接失败: {}", err);
            false
        }
        Err(_) => {
            warn!("自愈 L1：关闭连接超时");
            false
        }
    }
}

/// L2：若"手动切换"组当前选中的是具体节点（而非自动选择），触发自动选择组
/// 测速并切换过去。返回被切换掉的原节点 tag；返回 None 表示无事可做
/// （已是自动选择 / 组不存在 / gRPC 失败）。
async fn recovery_switch_to_auto(app_handle: &AppHandle, api_port: u16) -> Option<String> {
    let client = match make_api_client(api_port) {
        Ok(client) => client,
        Err(error) => {
            tracing::warn!("L2 自愈：创建 gRPC 客户端失败: {}", error);
            return None;
        }
    };
    let groups = match tokio::time::timeout(
        Duration::from_secs(GRPC_ACTION_TIMEOUT_SECS),
        client.get_groups_snapshot(),
    )
    .await
    {
        Ok(Ok(groups)) => groups,
        Ok(Err(err)) => {
            warn!("自愈 L2：获取节点组失败: {}", err);
            return None;
        }
        Err(_) => {
            warn!("自愈 L2：获取节点组超时");
            return None;
        }
    };

    let manual = groups
        .group
        .iter()
        .find(|g| g.tag == crate::app::singbox::common::TAG_MANUAL)?;
    if manual.selected == crate::app::singbox::common::TAG_AUTO {
        debug!("自愈 L2：当前已是自动选择，跳过切换");
        return None;
    }

    let previous = manual.selected.clone();
    // 先触发自动选择组测速，确保切换过去时它已挑选出可用节点。
    if let Err(err) = tokio::time::timeout(
        Duration::from_secs(GRPC_ACTION_TIMEOUT_SECS),
        client.url_test(crate::app::singbox::common::TAG_AUTO),
    )
    .await
    {
        warn!("自愈 L2：触发自动选择测速失败（继续尝试切换）: {:?}", err);
    }
    match tokio::time::timeout(
        Duration::from_secs(GRPC_ACTION_TIMEOUT_SECS),
        client.select_outbound(
            crate::app::singbox::common::TAG_MANUAL,
            crate::app::singbox::common::TAG_AUTO,
        ),
    )
    .await
    {
        Ok(Ok(())) => {
            info!("自愈 L2：节点 [{}] 疑似不可用，已切换到自动选择", previous);
            emit_kernel_error_with_context(
                app_handle,
                "KERNEL_GUARD_SWITCHED_TO_AUTO",
                &format!("节点 [{}] 不可用，已自动切换到「自动选择」", previous),
                None,
                Some("kernel.guard.self_heal"),
                true,
            );
            Some(previous)
        }
        Ok(Err(err)) => {
            warn!("自愈 L2：切换到自动选择失败: {}", err);
            None
        }
        Err(_) => {
            warn!("自愈 L2：切换到自动选择超时");
            None
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
    GUARDED_PROXY_PORT.store(0, Ordering::Relaxed);
    GUARDED_TUN_ENABLED.store(false, Ordering::Relaxed);
}

pub(super) async fn enable_kernel_guard(
    app_handle: AppHandle,
    api_port: u16,
    proxy_port: u16,
    tun_enabled: bool,
) {
    GUARDED_API_PORT.store(api_port, Ordering::Relaxed);
    GUARDED_PROXY_PORT.store(proxy_port, Ordering::Relaxed);
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
        // 分级恢复状态机：探测恢复成功时全部重置。
        let mut recovery_stage = RecoveryStage::CloseConnections;
        let mut consecutive_restarts: u32 = 0;
        let mut circuit_open_notified = false;

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
                let log_path =
                    std::path::PathBuf::from(crate::app::singbox::common::kernel_log_output_path());
                log_rotation::rotate_if_needed(&log_path);
                last_log_rotation_at = Instant::now();
            }

            match is_kernel_running_ref(&app_handle).await {
                Ok(true) => {
                    // 所有代理模式都做连通性自愈：进程活着但假死时也能恢复。
                    let policy = load_self_heal_policy(&app_handle).await;
                    if !policy.enabled {
                        connectivity_failures = 0;
                        next_self_heal_at =
                            Instant::now() + Duration::from_secs(SELF_HEAL_WARMUP_SECS);
                        continue;
                    }

                    let proxy_port = GUARDED_PROXY_PORT.load(Ordering::Relaxed);
                    if proxy_port == 0 {
                        // 未登记 mixed 端口（异常兜底）：退回直连探测，行为同旧版。
                        match crate::app::system::system_service::check_network_connectivity(Some(
                            false,
                        ))
                        .await
                        {
                            Ok(true) => {
                                if connectivity_failures > 0 {
                                    info!("连通性已恢复，清空失败计数");
                                }
                                connectivity_failures = 0;
                            }
                            Ok(false) | Err(_) => {
                                connectivity_failures = connectivity_failures.saturating_add(1);
                                warn!(
                                    "连通性检测失败，计数: {}/{}",
                                    connectivity_failures, CONNECTIVITY_FAIL_THRESHOLD
                                );
                            }
                        }
                        continue;
                    }

                    // 主探测：走内核 mixed 入站，能发现"进程存活但代理隧道死亡"的假活。
                    if crate::app::system::system_service::perform_proxy_probe(proxy_port).await {
                        if connectivity_failures > 0 {
                            info!("代理链路已恢复，清空失败计数");
                        }
                        connectivity_failures = 0;
                        if recovery_stage != RecoveryStage::CloseConnections
                            || consecutive_restarts > 0
                            || circuit_open_notified
                        {
                            info!("连通性已恢复，重置自愈状态机与重启计数");
                        }
                        recovery_stage = RecoveryStage::CloseConnections;
                        consecutive_restarts = 0;
                        circuit_open_notified = false;
                        continue;
                    }

                    connectivity_failures = connectivity_failures.saturating_add(1);
                    warn!(
                        "代理链路检测失败，计数: {}/{}",
                        connectivity_failures, CONNECTIVITY_FAIL_THRESHOLD
                    );

                    if connectivity_failures < CONNECTIVITY_FAIL_THRESHOLD {
                        continue;
                    }

                    // 对照探测：区分"内核隧道死亡"与"本机/上游断网"。
                    // 直连也失败时说明网络本身不可用，重启内核无济于事，
                    // 清零计数等待网络恢复，避免断网期间反复重启内核。
                    match crate::app::system::system_service::check_network_connectivity(Some(
                        false,
                    ))
                    .await
                    {
                        Ok(true) => {}
                        _ => {
                            warn!("直连对照探测同样失败，判定为系统断网，跳过内核自愈");
                            connectivity_failures = 0;
                            continue;
                        }
                    }

                    if Instant::now() < next_self_heal_at {
                        continue;
                    }

                    // 分级恢复：L1 关连接 → L2 切自动选择 → L3 重启内核，逐级升级。
                    // 避免"节点本身不可用"（重启无解）时陷入无限重启循环。
                    let api_port = GUARDED_API_PORT.load(Ordering::Relaxed);
                    connectivity_failures = 0;

                    match recovery_stage {
                        RecoveryStage::CloseConnections => {
                            let succeeded =
                                api_port != 0 && recovery_close_all_connections(api_port).await;
                            // L1 失败说明 gRPC 已不通（内核假死较深），直接升级重启；
                            // 成功则观察一段时间，仍失败再尝试切换出站。
                            recovery_stage = if succeeded {
                                RecoveryStage::SwitchToAuto
                            } else {
                                RecoveryStage::RestartKernel
                            };
                            next_self_heal_at =
                                Instant::now() + Duration::from_secs(RECOVERY_OBSERVE_SECS);
                        }
                        RecoveryStage::SwitchToAuto => {
                            // 已是自动选择时本步骤为空操作，同样进入观察期后升级重启。
                            if api_port != 0 {
                                recovery_switch_to_auto(&app_handle, api_port).await;
                            }
                            recovery_stage = RecoveryStage::RestartKernel;
                            next_self_heal_at =
                                Instant::now() + Duration::from_secs(RECOVERY_OBSERVE_SECS);
                        }
                        RecoveryStage::RestartKernel => {
                            if consecutive_restarts >= MAX_CONSECUTIVE_RESTARTS {
                                // 熔断：重启解决不了（典型为节点全部不可用），暂停重启、
                                // 发一次通知；轻量自愈（关连接/切组）与探测继续，
                                // 节点或订阅恢复后探测成功会自动解除熔断。
                                if !circuit_open_notified {
                                    warn!(
                                        "连续 {} 次重启自愈无效，进入熔断：暂停重启内核，仅保留轻量自愈",
                                        consecutive_restarts
                                    );
                                    emit_kernel_error_with_context(
                                        &app_handle,
                                        "KERNEL_GUARD_CIRCUIT_OPEN",
                                        "多次自动恢复无效，当前节点可能均不可用；已暂停自动重启，请手动切换节点或检查订阅",
                                        None,
                                        Some("kernel.guard.self_heal"),
                                        true,
                                    );
                                    circuit_open_notified = true;
                                }
                                recovery_stage = RecoveryStage::CloseConnections;
                                next_self_heal_at =
                                    Instant::now() + Duration::from_secs(CIRCUIT_COOLDOWN_SECS);
                                continue;
                            }

                            let mode_label = if GUARDED_TUN_ENABLED.load(Ordering::Relaxed) {
                                "tun-connectivity"
                            } else {
                                "system-connectivity"
                            };
                            let succeeded = heal_restart(&app_handle, mode_label).await;
                            // 无论成败都计数并进入冷却窗口，避免抖动。
                            consecutive_restarts = consecutive_restarts.saturating_add(1);
                            next_self_heal_at =
                                Instant::now() + Duration::from_secs(policy.cooldown_secs);
                            // 重启后若仍失败，从 L1 重新走一遍（关连接对死隧道仍有意义）。
                            recovery_stage = RecoveryStage::CloseConnections;
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
                    recovery_stage = RecoveryStage::CloseConnections;
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
    GUARDED_PROXY_PORT.store(0, Ordering::Relaxed);
    GUARDED_TUN_ENABLED.store(false, Ordering::Relaxed);
    let mut handle_slot = KERNEL_GUARD_HANDLE.lock().await;
    if let Some(handle) = handle_slot.take() {
        handle.abort();
    }
}
