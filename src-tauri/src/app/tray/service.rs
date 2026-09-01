use super::icon;
use super::model::{
    events, menu_ids, TrayCloseBehavior, TrayNavigatePayload, TrayRuntimeStateInput,
    TrayToggleProxyFeaturePayload, TRAY_ICON_ID,
};
use super::state::TrayRuntimeState;
use crate::app::core::kernel_service::runtime::{apply_proxy_settings, kernel_restart_fast};
use crate::app::core::kernel_service::status::is_kernel_running;
use crate::app::storage::enhanced_storage_service::db_save_app_config_internal;
use crate::app::storage::state_model::AppConfig;
use lazy_static::lazy_static;
use std::sync::RwLock;
use std::time::Duration;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, WebviewWindow, WebviewWindowBuilder};
use tracing::{debug, info, warn};

lazy_static! {
    static ref TRAY_RUNTIME_STATE: RwLock<TrayRuntimeState> =
        RwLock::new(TrayRuntimeState::default());
}

#[derive(Debug, Clone, Copy)]
struct TrayText {
    show_window: &'static str,
    rebuild_window: &'static str,
    kernel_menu: &'static str,
    restart_kernel: &'static str,
    status_running: &'static str,
    status_stopped: &'static str,
    proxy_controls: &'static str,
    current_status: &'static str,
    mode_system: &'static str,
    mode_tun: &'static str,
    mode_manual: &'static str,
    quit: &'static str,
    tooltip_kernel: &'static str,
    tooltip_mode: &'static str,
    tooltip_subscription: &'static str,
}

const TRAY_TEXT_ZH_CN: TrayText = TrayText {
    show_window: "显示主界面",
    rebuild_window: "重建窗口",
    kernel_menu: "内核",
    restart_kernel: "重启内核",
    status_running: "运行中",
    status_stopped: "已停止",
    proxy_controls: "代理开关",
    current_status: "当前状态：",
    mode_system: "系统代理",
    mode_tun: "TUN 模式",
    mode_manual: "手动模式",
    quit: "退出",
    tooltip_kernel: "内核: ",
    tooltip_mode: "模式: ",
    tooltip_subscription: "订阅: ",
};

const TRAY_TEXT_EN_US: TrayText = TrayText {
    show_window: "Show Main Window",
    rebuild_window: "Rebuild Window",
    kernel_menu: "Kernel",
    restart_kernel: "Restart Kernel",
    status_running: "Running",
    status_stopped: "Stopped",
    proxy_controls: "Proxy Controls",
    current_status: "Current Status:",
    mode_system: "System",
    mode_tun: "TUN",
    mode_manual: "Manual",
    quit: "Quit",
    tooltip_kernel: "Kernel: ",
    tooltip_mode: "Mode: ",
    tooltip_subscription: "Subscription: ",
};

const TRAY_TEXT_JA_JP: TrayText = TrayText {
    show_window: "メイン画面を表示",
    rebuild_window: "ウィンドウを再構築",
    kernel_menu: "カーネル",
    restart_kernel: "カーネルを再起動",
    status_running: "稼働中",
    status_stopped: "停止中",
    proxy_controls: "プロキシ切替",
    current_status: "現在の状態：",
    mode_system: "システム",
    mode_tun: "TUN",
    mode_manual: "手動",
    quit: "終了",
    tooltip_kernel: "カーネル: ",
    tooltip_mode: "モード: ",
    tooltip_subscription: "サブスクリプション: ",
};

const TRAY_TEXT_RU_RU: TrayText = TrayText {
    show_window: "Показать окно",
    rebuild_window: "Пересоздать окно",
    kernel_menu: "Ядро",
    restart_kernel: "Перезапустить ядро",
    status_running: "Запущено",
    status_stopped: "Остановлено",
    proxy_controls: "Прокси-переключатели",
    current_status: "Текущее состояние:",
    mode_system: "Системный",
    mode_tun: "TUN",
    mode_manual: "Ручной",
    quit: "Выход",
    tooltip_kernel: "Ядро: ",
    tooltip_mode: "Режим: ",
    tooltip_subscription: "Подписка: ",
};

fn with_state_read<T>(f: impl FnOnce(&TrayRuntimeState) -> T) -> T {
    let guard = TRAY_RUNTIME_STATE
        .read()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&guard)
}

fn with_state_write<T>(f: impl FnOnce(&mut TrayRuntimeState) -> T) -> T {
    let mut guard = TRAY_RUNTIME_STATE
        .write()
        .unwrap_or_else(|poisoned| poisoned.into_inner());
    f(&mut guard)
}

fn tray_text_for_locale(locale: &str) -> TrayText {
    let normalized = locale.trim().to_lowercase();
    if normalized.starts_with("zh") {
        TRAY_TEXT_ZH_CN
    } else if normalized.starts_with("ja") {
        TRAY_TEXT_JA_JP
    } else if normalized.starts_with("ru") {
        TRAY_TEXT_RU_RU
    } else {
        TRAY_TEXT_EN_US
    }
}

fn mode_summary_text(state: &TrayRuntimeState, text: &TrayText) -> String {
    match (state.system_proxy_enabled, state.tun_enabled) {
        (true, true) => format!("{} + {}", text.mode_system, text.mode_tun),
        (true, false) => text.mode_system.to_string(),
        (false, true) => text.mode_tun.to_string(),
        (false, false) => text.mode_manual.to_string(),
    }
}

fn compose_tooltip(state: &TrayRuntimeState, text: &TrayText) -> String {
    let kernel_status = if state.kernel_running {
        text.status_running
    } else {
        text.status_stopped
    };
    let mode = mode_summary_text(state, text);

    let mut tooltip = format!(
        "sing-box-window - {}{}, {}{}",
        text.tooltip_kernel, kernel_status, text.tooltip_mode, mode
    );

    if let Some(subscription_name) = state.active_subscription_name.as_ref() {
        tooltip.push_str(&format!(
            ", {}{}",
            text.tooltip_subscription, subscription_name
        ));
    }

    tooltip
}

fn resolve_tray_icon(
    app: &AppHandle,
    state: &TrayRuntimeState,
) -> Option<tauri::image::Image<'static>> {
    if let Some(icon) = app.default_window_icon() {
        if let Some(recolored) = icon::recolor_icon_for_mode(icon, state.display_mode()) {
            return Some(recolored);
        }

        return Some(icon.clone().to_owned());
    }

    None
}

fn build_tray_menu(
    app: &AppHandle,
    state: &TrayRuntimeState,
    text: &TrayText,
) -> Result<tauri::menu::Menu<tauri::Wry>, String> {
    let primary_window_action = if state.window_visible {
        text.show_window
    } else {
        text.rebuild_window
    };

    let show_window_item = MenuItemBuilder::with_id(menu_ids::SHOW_WINDOW, primary_window_action)
        .build(app)
        .map_err(|e| format!("创建托盘菜单项失败: {}", e))?;

    let kernel_status_item = MenuItemBuilder::with_id(
        menu_ids::KERNEL_STATUS,
        if state.kernel_running {
            text.status_running
        } else {
            text.status_stopped
        },
    )
    .enabled(false)
    .build(app)
    .map_err(|e| format!("创建内核状态菜单项失败: {}", e))?;

    let kernel_restart_item =
        MenuItemBuilder::with_id(menu_ids::KERNEL_RESTART, text.restart_kernel)
            .enabled(state.kernel_running)
            .build(app)
            .map_err(|e| format!("创建重启菜单项失败: {}", e))?;

    let kernel_submenu = SubmenuBuilder::with_id(app, menu_ids::KERNEL_SUBMENU, text.kernel_menu)
        .item(&kernel_status_item)
        .item(&kernel_restart_item)
        .build()
        .map_err(|e| format!("创建内核子菜单失败: {}", e))?;

    let current_mode_item = MenuItemBuilder::with_id(
        menu_ids::PROXY_CURRENT,
        format!("{} {}", text.current_status, mode_summary_text(state, text)),
    )
    .enabled(false)
    .build(app)
    .map_err(|e| format!("创建当前模式菜单项失败: {}", e))?;

    let proxy_system_item = CheckMenuItemBuilder::with_id(menu_ids::PROXY_SYSTEM, text.mode_system)
        .checked(state.system_proxy_enabled)
        .enabled(true)
        .build(app)
        .map_err(|e| format!("创建系统代理菜单项失败: {}", e))?;

    let proxy_tun_item = CheckMenuItemBuilder::with_id(menu_ids::PROXY_TUN, text.mode_tun)
        .checked(state.tun_enabled)
        .enabled(true)
        .build(app)
        .map_err(|e| format!("创建TUN菜单项失败: {}", e))?;

    let proxy_submenu = SubmenuBuilder::with_id(app, menu_ids::PROXY_SUBMENU, text.proxy_controls)
        .item(&current_mode_item)
        .separator()
        .item(&proxy_system_item)
        .item(&proxy_tun_item)
        .build()
        .map_err(|e| format!("创建代理模式子菜单失败: {}", e))?;

    let quit_item = MenuItemBuilder::with_id(menu_ids::QUIT, text.quit)
        .build(app)
        .map_err(|e| format!("创建退出菜单项失败: {}", e))?;

    MenuBuilder::new(app)
        .items(&[
            &show_window_item,
            &kernel_submenu,
            &proxy_submenu,
            &quit_item,
        ])
        .build()
        .map_err(|e| format!("创建托盘菜单失败: {}", e))
}

fn handle_proxy_toggle_menu_event(app: &AppHandle, feature: &str, enabled: bool) {
    let app_handle = app.clone();
    let feature = feature.to_string();
    tauri::async_runtime::spawn(async move {
        let result = if feature == "systemProxy" {
            apply_system_proxy_toggle_from_tray(&app_handle, enabled).await
        } else {
            apply_tun_toggle_from_tray(&app_handle, enabled).await
        };

        if let Err(err) = result {
            warn!("托盘代理切换失败: {}", err);
        }
    });
}

fn handle_menu_event(app: &AppHandle, menu_id: &str) {
    match menu_id {
        menu_ids::SHOW_WINDOW => {
            if let Err(err) = show_main_window(app, true) {
                warn!("托盘显示窗口失败: {}", err);
            }
        }
        menu_ids::KERNEL_RESTART => {
            let app_handle = app.clone();
            tauri::async_runtime::spawn(async move {
                if let Err(err) = restart_kernel_from_tray(&app_handle).await {
                    warn!("托盘重启内核失败: {}", err);
                }
            });
        }
        menu_ids::PROXY_SYSTEM => {
            let enabled = with_state_read(|state| !state.system_proxy_enabled);
            handle_proxy_toggle_menu_event(app, "systemProxy", enabled)
        }
        menu_ids::PROXY_TUN => {
            let enabled = with_state_read(|state| !state.tun_enabled);
            handle_proxy_toggle_menu_event(app, "tun", enabled)
        }
        menu_ids::QUIT => {
            if let Err(err) = request_app_exit(app) {
                warn!("托盘退出流程失败: {}", err);
            }
        }
        _ => {
            debug!("忽略未处理的托盘菜单事件: {}", menu_id);
        }
    }
}

fn handle_tray_icon_event(tray: &tauri::tray::TrayIcon, event: TrayIconEvent) {
    if let TrayIconEvent::Click {
        button: MouseButton::Left,
        button_state: MouseButtonState::Up,
        ..
    } = event
    {
        if let Err(err) = show_main_window(tray.app_handle(), true) {
            warn!("托盘左键恢复窗口失败: {}", err);
        }
    }
}

fn create_or_replace_tray_icon(app: &AppHandle, state: &TrayRuntimeState) -> Result<(), String> {
    if app.remove_tray_by_id(TRAY_ICON_ID).is_some() {
        info!("已移除旧托盘实例，准备重建");
    }

    let text = tray_text_for_locale(&state.locale);
    let menu = build_tray_menu(app, state, &text)?;
    let tooltip = compose_tooltip(state, &text);
    let icon = resolve_tray_icon(app, state);

    let mut builder = TrayIconBuilder::with_id(TRAY_ICON_ID)
        .menu(&menu)
        .tooltip(&tooltip)
        .show_menu_on_left_click(false)
        .on_menu_event(|app, event| {
            let menu_id = event.id().as_ref().to_string();
            handle_menu_event(app, &menu_id);
        })
        .on_tray_icon_event(|tray, event| {
            handle_tray_icon_event(tray, event);
        });

    if let Some(icon) = icon {
        builder = builder.icon(icon);
    }

    builder
        .build(app)
        .map(|_| ())
        .map_err(|e| format!("创建托盘图标失败: {}", e))
}

pub fn init_tray(app: &AppHandle) -> Result<(), String> {
    let state = with_state_read(|state| state.clone());
    create_or_replace_tray_icon(app, &state)
}

pub fn refresh_tray(app: &AppHandle) -> Result<(), String> {
    let state = with_state_read(|state| state.clone());
    let text = tray_text_for_locale(&state.locale);
    let menu = build_tray_menu(app, &state, &text)?;
    let tooltip = compose_tooltip(&state, &text);
    let icon = resolve_tray_icon(app, &state);

    if let Some(tray) = app.tray_by_id(TRAY_ICON_ID) {
        if let Err(err) = tray.set_menu(Some(menu)) {
            warn!("更新托盘菜单失败，尝试重建托盘: {}", err);
            return create_or_replace_tray_icon(app, &state);
        }
        if let Err(err) = tray.set_tooltip(Some(tooltip.as_str())) {
            debug!("更新托盘提示失败（可忽略的平台差异）: {}", err);
        }
        if let Err(err) = tray.set_icon(icon) {
            warn!("更新托盘图标失败，尝试重建托盘: {}", err);
            return create_or_replace_tray_icon(app, &state);
        }
        return Ok(());
    }

    info!("未找到托盘实例，尝试重新创建");
    create_or_replace_tray_icon(app, &state)
}

pub fn sync_tray_state(app: &AppHandle, payload: TrayRuntimeStateInput) -> Result<(), String> {
    let changed = with_state_write(|state| state.apply_sync_payload(payload));
    if !changed {
        return Ok(());
    }
    refresh_tray(app)
}

pub fn set_last_visible_route(path: &str) {
    with_state_write(|state| {
        state.set_last_visible_route(path);
    });
}

pub fn consume_pending_proxy_toggle() -> Option<TrayToggleProxyFeaturePayload> {
    with_state_write(|state| state.take_pending_proxy_toggle())
}

pub fn apply_startup_preferences(close_behavior: TrayCloseBehavior, window_visible: bool) {
    with_state_write(|state| {
        state.close_behavior = close_behavior;
        state.window_visible = window_visible;
        state.keep_alive_without_windows = false;
        state.allow_app_exit = false;
    });
}

fn create_main_window(app: &AppHandle) -> Result<(), String> {
    let window_config = app
        .config()
        .app
        .windows
        .iter()
        .find(|config| config.label == "main")
        .cloned()
        .ok_or_else(|| "未找到主窗口配置".to_string())?;
    let app_handle = app.clone();

    std::thread::spawn(move || {
        WebviewWindowBuilder::from_config(&app_handle, &window_config)
            .map_err(|e| format!("创建主窗口构建器失败: {}", e))?
            .build()
            .map(|_| ())
            .map_err(|e| format!("重建主窗口失败: {}", e))
    })
    .join()
    .map_err(|_| "重建主窗口线程异常退出".to_string())?
}

fn ensure_main_window(app: &AppHandle) -> Result<(WebviewWindow, bool), String> {
    if let Some(window) = app.get_webview_window("main") {
        return Ok((window, false));
    }

    create_main_window(app)?;
    let window = app
        .get_webview_window("main")
        .ok_or_else(|| "重建主窗口后未找到实例".to_string())?;
    Ok((window, true))
}

pub fn show_main_window(app: &AppHandle, emit_events: bool) -> Result<(), String> {
    let (main_window, recreated) = ensure_main_window(app)?;

    let _ = main_window.unminimize();
    main_window
        .show()
        .map_err(|e| format!("显示主窗口失败: {}", e))?;
    main_window
        .set_focus()
        .map_err(|e| format!("聚焦主窗口失败: {}", e))?;

    with_state_write(|state| {
        state.set_window_visible(true);
        state.keep_alive_without_windows = false;
        state.allow_app_exit = false;
        if !recreated {
            state.pending_restore_route = None;
        }
    });

    if emit_events {
        let route = with_state_read(|state| state.last_visible_route.clone());
        let route = if route.trim().is_empty() {
            "/".to_string()
        } else {
            route
        };

        let _ = app.emit(events::ACTION_SHOW_WINDOW, ());
        if recreated {
            with_state_write(|state| {
                state.set_pending_restore_route(&route);
            });
        } else {
            let _ = app.emit(
                events::ACTION_NAVIGATE_LAST_ROUTE,
                TrayNavigatePayload { path: route },
            );
        }
    }

    Ok(())
}

pub fn hide_main_window(app: &AppHandle, emit_events: bool) -> Result<(), String> {
    let main_window = app
        .get_webview_window("main")
        .ok_or_else(|| "未找到主窗口".to_string())?;

    main_window
        .hide()
        .map_err(|e| format!("隐藏主窗口失败: {}", e))?;

    with_state_write(|state| {
        state.set_window_visible(false);
        state.allow_app_exit = false;
    });

    if emit_events {
        let _ = app.emit(events::ACTION_HIDE_WINDOW, ());
    }

    refresh_runtime_state_in_background(app);

    Ok(())
}

pub fn close_main_window(app: &AppHandle) -> Result<(), String> {
    match with_state_read(|state| state.close_behavior) {
        TrayCloseBehavior::Hide => hide_main_window(app, true),
        TrayCloseBehavior::Lightweight => destroy_main_window_for_tray(app),
    }
}

pub fn enter_startup_background_mode(app: &AppHandle, lightweight: bool) -> Result<(), String> {
    if !lightweight {
        return hide_main_window(app, false);
    }

    destroy_main_window_for_tray(app)
}

pub fn consume_pending_restore_route() -> Option<TrayNavigatePayload> {
    with_state_write(|state| {
        state
            .take_pending_restore_route()
            .map(|path| TrayNavigatePayload { path })
    })
}

pub fn should_prevent_exit() -> bool {
    with_state_read(|state| state.keep_alive_without_windows && !state.allow_app_exit)
}

fn destroy_main_window_for_tray(app: &AppHandle) -> Result<(), String> {
    let main_window = app
        .get_webview_window("main")
        .ok_or_else(|| "未找到主窗口".to_string())?;
    let route = with_state_read(|state| state.last_visible_route.clone());

    with_state_write(|state| {
        state.set_window_visible(false);
        state.keep_alive_without_windows = true;
        state.allow_app_exit = false;
        state.set_pending_restore_route(&route);
    });

    if let Err(err) = main_window.destroy() {
        with_state_write(|state| {
            state.keep_alive_without_windows = false;
        });
        return Err(format!("销毁主窗口失败: {}", err));
    }

    refresh_runtime_state_in_background(app);

    Ok(())
}

fn refresh_runtime_state_in_background(app: &AppHandle) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(err) = refresh_runtime_state_from_backend(&app_handle, true).await {
            warn!("窗口关闭后刷新托盘运行态失败: {}", err);
        }
    });
}

pub fn request_app_exit(app: &AppHandle) -> Result<(), String> {
    let _ = app.emit(events::ACTION_EXIT_REQUESTED, ());
    with_state_write(|state| {
        state.allow_app_exit = true;
        state.keep_alive_without_windows = false;
    });

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        match tokio::time::timeout(
            Duration::from_secs(4),
            crate::app::core::kernel_service::runtime::stop_kernel(None),
        )
        .await
        {
            Ok(Ok(message)) => info!("退出前停止内核成功: {}", message),
            Ok(Err(err)) => warn!("退出前停止内核失败，继续退出: {}", err),
            Err(_) => warn!("退出前停止内核超时，继续退出"),
        }

        app_handle.exit(0);
    });

    Ok(())
}

async fn restart_kernel_from_tray(app: &AppHandle) -> Result<(), String> {
    let result = kernel_restart_fast(
        app.clone(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
    )
    .await?;

    if !result
        .get("success")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return Err(result
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("重启内核失败")
            .to_string());
    }

    refresh_runtime_state_from_backend(app, false).await
}

async fn apply_system_proxy_toggle_from_tray(app: &AppHandle, enabled: bool) -> Result<(), String> {
    let result = apply_proxy_settings(app.clone(), Some(enabled), None).await?;
    if !result
        .get("success")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return Err(result
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("应用系统代理配置失败")
            .to_string());
    }

    let mut app_config =
        crate::app::storage::enhanced_storage_service::db_get_app_config_internal(app).await?;
    app_config.system_proxy_enabled = enabled;
    app_config.proxy_mode =
        derive_proxy_mode(app_config.system_proxy_enabled, app_config.tun_enabled);
    db_save_app_config_internal(app_config, app).await?;
    refresh_runtime_state_from_backend(app, false).await
}

async fn apply_tun_toggle_from_tray(app: &AppHandle, enabled: bool) -> Result<(), String> {
    if !enabled {
        let apply_result = apply_proxy_settings(app.clone(), None, Some(false)).await?;
        if !apply_result
            .get("success")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            return Err(apply_result
                .get("message")
                .and_then(|value| value.as_str())
                .unwrap_or("应用 TUN 关闭配置失败")
                .to_string());
        }

        let restart_result = kernel_restart_fast(
            app.clone(),
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            None,
            Some(false),
        )
        .await?;
        if !restart_result
            .get("success")
            .and_then(|value| value.as_bool())
            .unwrap_or(false)
        {
            return Err(restart_result
                .get("message")
                .and_then(|value| value.as_str())
                .unwrap_or("关闭 TUN 后重启内核失败")
                .to_string());
        }

        let mut app_config =
            crate::app::storage::enhanced_storage_service::db_get_app_config_internal(app).await?;
        app_config.tun_enabled = false;
        app_config.proxy_mode =
            derive_proxy_mode(app_config.system_proxy_enabled, app_config.tun_enabled);
        db_save_app_config_internal(app_config, app).await?;
        return refresh_runtime_state_from_backend(app, false).await;
    }

    #[cfg(target_os = "windows")]
    {
        if !crate::app::system::system_service::check_admin() {
            return queue_proxy_toggle_for_frontend(app, "tun", true);
        }
    }

    #[cfg(any(target_os = "linux", target_os = "macos"))]
    {
        let sudo_status =
            crate::app::system::sudo_service::sudo_password_status(app.clone()).await?;
        if !sudo_status.supported {
            return Err("当前平台暂不支持该操作".to_string());
        }

        if !sudo_status.has_saved {
            return queue_proxy_toggle_for_frontend(app, "tun", true);
        }
    }

    let apply_result = apply_proxy_settings(app.clone(), None, Some(true)).await?;
    if !apply_result
        .get("success")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        return Err(apply_result
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("应用 TUN 配置失败")
            .to_string());
    }

    let restart_result = kernel_restart_fast(
        app.clone(),
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        None,
        Some(true),
    )
    .await?;
    if !restart_result
        .get("success")
        .and_then(|value| value.as_bool())
        .unwrap_or(false)
    {
        let message = restart_result
            .get("message")
            .and_then(|value| value.as_str())
            .unwrap_or("启用 TUN 后重启内核失败")
            .to_string();

        #[cfg(any(target_os = "linux", target_os = "macos"))]
        if message.contains(crate::app::system::sudo_service::SUDO_PASSWORD_REQUIRED)
            || message.contains(crate::app::system::sudo_service::SUDO_PASSWORD_INVALID)
        {
            return queue_proxy_toggle_for_frontend(app, "tun", true);
        }

        return Err(message);
    }

    let mut app_config =
        crate::app::storage::enhanced_storage_service::db_get_app_config_internal(app).await?;
    app_config.tun_enabled = true;
    app_config.proxy_mode =
        derive_proxy_mode(app_config.system_proxy_enabled, app_config.tun_enabled);
    db_save_app_config_internal(app_config, app).await?;
    refresh_runtime_state_from_backend(app, false).await
}

fn derive_proxy_mode(system_proxy_enabled: bool, tun_enabled: bool) -> String {
    if tun_enabled {
        "tun".to_string()
    } else if system_proxy_enabled {
        "system".to_string()
    } else {
        "manual".to_string()
    }
}

fn apply_backend_runtime_snapshot(
    state: &mut TrayRuntimeState,
    app_config: &AppConfig,
    kernel_running: bool,
) -> bool {
    let mut changed = false;
    let close_behavior = TrayCloseBehavior::from_raw(&app_config.tray_close_behavior);

    if state.system_proxy_enabled != app_config.system_proxy_enabled {
        state.system_proxy_enabled = app_config.system_proxy_enabled;
        changed = true;
    }
    if state.tun_enabled != app_config.tun_enabled {
        state.tun_enabled = app_config.tun_enabled;
        changed = true;
    }
    if state.kernel_running != kernel_running {
        state.kernel_running = kernel_running;
        changed = true;
    }
    if state.close_behavior != close_behavior {
        state.close_behavior = close_behavior;
        changed = true;
    }

    changed
}

pub async fn refresh_runtime_state_from_backend(
    app: &AppHandle,
    force_refresh: bool,
) -> Result<(), String> {
    let app_config =
        crate::app::storage::enhanced_storage_service::db_get_app_config_internal(app).await?;
    let kernel_running = is_kernel_running().await.unwrap_or(false);
    let changed = with_state_write(|state| {
        apply_backend_runtime_snapshot(state, &app_config, kernel_running)
    });

    if changed || force_refresh {
        refresh_tray(app)?;
    }

    let _ = app.emit(events::RUNTIME_STATE_UPDATED, ());
    Ok(())
}

fn queue_proxy_toggle_for_frontend(
    app: &AppHandle,
    feature: &str,
    enabled: bool,
) -> Result<(), String> {
    let payload = TrayToggleProxyFeaturePayload {
        feature: feature.to_string(),
        enabled,
    };

    if app.get_webview_window("main").is_some() {
        show_main_window(app, true)?;
        app.emit(events::ACTION_SWITCH_PROXY_MODE, payload)
            .map_err(|e| format!("发送托盘代理切换事件失败: {}", e))?;
        return Ok(());
    }

    with_state_write(|state| {
        state.set_pending_proxy_toggle(payload);
    });
    show_main_window(app, true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::app::storage::state_model::AppConfig;

    fn sample_app_config() -> AppConfig {
        AppConfig {
            system_proxy_enabled: true,
            tun_enabled: false,
            tray_close_behavior: "lightweight".to_string(),
            ..AppConfig::default()
        }
    }

    #[test]
    fn backend_runtime_sync_updates_proxy_related_fields() {
        let mut state = TrayRuntimeState::default();
        let config = sample_app_config();

        let changed = apply_backend_runtime_snapshot(&mut state, &config, true);

        assert!(changed);
        assert!(state.system_proxy_enabled);
        assert!(!state.tun_enabled);
        assert!(state.kernel_running);
        assert_eq!(state.close_behavior, TrayCloseBehavior::Lightweight);
    }

    #[test]
    fn backend_runtime_sync_preserves_window_lifecycle_fields() {
        let mut state = TrayRuntimeState {
            window_visible: false,
            last_visible_route: "/settings".to_string(),
            pending_restore_route: Some("/settings".to_string()),
            keep_alive_without_windows: true,
            allow_app_exit: true,
            ..TrayRuntimeState::default()
        };
        let config = AppConfig {
            system_proxy_enabled: false,
            tun_enabled: true,
            tray_close_behavior: "hide".to_string(),
            ..AppConfig::default()
        };

        let changed = apply_backend_runtime_snapshot(&mut state, &config, false);

        assert!(changed);
        assert!(!state.window_visible);
        assert_eq!(state.last_visible_route, "/settings");
        assert_eq!(state.pending_restore_route.as_deref(), Some("/settings"));
        assert!(state.keep_alive_without_windows);
        assert!(state.allow_app_exit);
        assert!(state.tun_enabled);
        assert_eq!(state.close_behavior, TrayCloseBehavior::Hide);
    }
}
