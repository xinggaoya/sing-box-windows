use super::model::{
    events, menu_ids, TrayNavigatePayload, TrayProxyMode, TrayRuntimeStateInput,
    TraySwitchProxyModePayload, TRAY_ICON_ID,
};
use super::state::TrayRuntimeState;
use lazy_static::lazy_static;
use std::sync::RwLock;
use std::time::Duration;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Emitter, Manager, Runtime};
use tracing::{debug, info, warn};

lazy_static! {
    static ref TRAY_RUNTIME_STATE: RwLock<TrayRuntimeState> =
        RwLock::new(TrayRuntimeState::default());
}

#[derive(Debug, Clone, Copy)]
struct TrayText {
    show_window: &'static str,
    kernel_menu: &'static str,
    restart_kernel: &'static str,
    status_running: &'static str,
    status_stopped: &'static str,
    switch_proxy_mode: &'static str,
    current_mode: &'static str,
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
    kernel_menu: "内核",
    restart_kernel: "重启内核",
    status_running: "运行中",
    status_stopped: "已停止",
    switch_proxy_mode: "切换代理模式",
    current_mode: "当前模式：",
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
    kernel_menu: "Kernel",
    restart_kernel: "Restart Kernel",
    status_running: "Running",
    status_stopped: "Stopped",
    switch_proxy_mode: "Switch Proxy Mode",
    current_mode: "Current Mode:",
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
    kernel_menu: "カーネル",
    restart_kernel: "カーネルを再起動",
    status_running: "稼働中",
    status_stopped: "停止中",
    switch_proxy_mode: "プロキシモード切替",
    current_mode: "現在のモード：",
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
    kernel_menu: "Ядро",
    restart_kernel: "Перезапустить ядро",
    status_running: "Запущено",
    status_stopped: "Остановлено",
    switch_proxy_mode: "Режим прокси",
    current_mode: "Текущий режим:",
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

fn mode_text(mode: TrayProxyMode, text: &TrayText) -> &'static str {
    match mode {
        TrayProxyMode::System => text.mode_system,
        TrayProxyMode::Tun => text.mode_tun,
        TrayProxyMode::Manual => text.mode_manual,
    }
}

fn compose_tooltip(state: &TrayRuntimeState, text: &TrayText) -> String {
    let kernel_status = if state.kernel_running {
        text.status_running
    } else {
        text.status_stopped
    };
    let mode = mode_text(state.proxy_mode, text);

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

fn build_tray_menu<R: Runtime>(
    app: &AppHandle<R>,
    state: &TrayRuntimeState,
    text: &TrayText,
) -> Result<tauri::menu::Menu<R>, String> {
    let show_window_item = MenuItemBuilder::with_id(menu_ids::SHOW_WINDOW, text.show_window)
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
        format!(
            "{} {}",
            text.current_mode,
            mode_text(state.proxy_mode, text)
        ),
    )
    .enabled(false)
    .build(app)
    .map_err(|e| format!("创建当前模式菜单项失败: {}", e))?;

    let proxy_system_item = CheckMenuItemBuilder::with_id(menu_ids::PROXY_SYSTEM, text.mode_system)
        .checked(state.proxy_mode == TrayProxyMode::System)
        .enabled(state.proxy_mode != TrayProxyMode::System)
        .build(app)
        .map_err(|e| format!("创建系统代理菜单项失败: {}", e))?;

    let proxy_tun_item = CheckMenuItemBuilder::with_id(menu_ids::PROXY_TUN, text.mode_tun)
        .checked(state.proxy_mode == TrayProxyMode::Tun)
        .enabled(state.proxy_mode != TrayProxyMode::Tun)
        .build(app)
        .map_err(|e| format!("创建TUN菜单项失败: {}", e))?;

    let proxy_manual_item = CheckMenuItemBuilder::with_id(menu_ids::PROXY_MANUAL, text.mode_manual)
        .checked(state.proxy_mode == TrayProxyMode::Manual)
        .enabled(state.proxy_mode != TrayProxyMode::Manual)
        .build(app)
        .map_err(|e| format!("创建手动代理菜单项失败: {}", e))?;

    let proxy_submenu =
        SubmenuBuilder::with_id(app, menu_ids::PROXY_SUBMENU, text.switch_proxy_mode)
            .item(&current_mode_item)
            .separator()
            .item(&proxy_system_item)
            .item(&proxy_tun_item)
            .item(&proxy_manual_item)
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

fn handle_proxy_switch_menu_event<R: Runtime>(app: &AppHandle<R>, mode: TrayProxyMode) {
    let payload = TraySwitchProxyModePayload {
        mode: mode.as_str().to_string(),
    };
    if let Err(err) = app.emit(events::ACTION_SWITCH_PROXY_MODE, payload) {
        warn!("发送托盘代理切换事件失败: {}", err);
    }
}

fn handle_menu_event<R: Runtime>(app: &AppHandle<R>, menu_id: &str) {
    match menu_id {
        menu_ids::SHOW_WINDOW => {
            if let Err(err) = show_main_window(app, true) {
                warn!("托盘显示窗口失败: {}", err);
            }
        }
        menu_ids::KERNEL_RESTART => {
            if let Err(err) = app.emit(events::ACTION_RESTART_KERNEL, ()) {
                warn!("发送重启内核事件失败: {}", err);
            }
        }
        menu_ids::PROXY_SYSTEM => handle_proxy_switch_menu_event(app, TrayProxyMode::System),
        menu_ids::PROXY_TUN => handle_proxy_switch_menu_event(app, TrayProxyMode::Tun),
        menu_ids::PROXY_MANUAL => handle_proxy_switch_menu_event(app, TrayProxyMode::Manual),
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

fn handle_tray_icon_event<R: Runtime>(tray: &tauri::tray::TrayIcon<R>, event: TrayIconEvent) {
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

fn create_or_replace_tray_icon<R: Runtime>(
    app: &AppHandle<R>,
    state: &TrayRuntimeState,
) -> Result<(), String> {
    if app.remove_tray_by_id(TRAY_ICON_ID).is_some() {
        info!("已移除旧托盘实例，准备重建");
    }

    let text = tray_text_for_locale(&state.locale);
    let menu = build_tray_menu(app, state, &text)?;
    let tooltip = compose_tooltip(state, &text);

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

    if let Some(icon) = app.default_window_icon().cloned() {
        builder = builder.icon(icon);
    }

    builder
        .build(app)
        .map(|_| ())
        .map_err(|e| format!("创建托盘图标失败: {}", e))
}

pub fn init_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let state = with_state_read(|state| state.clone());
    create_or_replace_tray_icon(app, &state)
}

pub fn refresh_tray<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let state = with_state_read(|state| state.clone());
    let text = tray_text_for_locale(&state.locale);
    let menu = build_tray_menu(app, &state, &text)?;
    let tooltip = compose_tooltip(&state, &text);

    if let Some(tray) = app.tray_by_id(TRAY_ICON_ID) {
        if let Err(err) = tray.set_menu(Some(menu)) {
            warn!("更新托盘菜单失败，尝试重建托盘: {}", err);
            return create_or_replace_tray_icon(app, &state);
        }
        if let Err(err) = tray.set_tooltip(Some(tooltip.as_str())) {
            debug!("更新托盘提示失败（可忽略的平台差异）: {}", err);
        }
        return Ok(());
    }

    info!("未找到托盘实例，尝试重新创建");
    create_or_replace_tray_icon(app, &state)
}

pub fn sync_tray_state<R: Runtime>(
    app: &AppHandle<R>,
    payload: TrayRuntimeStateInput,
) -> Result<(), String> {
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

pub fn show_main_window<R: Runtime>(app: &AppHandle<R>, emit_events: bool) -> Result<(), String> {
    let main_window = app
        .get_webview_window("main")
        .ok_or_else(|| "未找到主窗口".to_string())?;

    let _ = main_window.unminimize();
    main_window
        .show()
        .map_err(|e| format!("显示主窗口失败: {}", e))?;
    main_window
        .set_focus()
        .map_err(|e| format!("聚焦主窗口失败: {}", e))?;

    with_state_write(|state| {
        state.set_window_visible(true);
    });

    if emit_events {
        let route = with_state_read(|state| state.last_visible_route.clone());
        let route = if route.trim().is_empty() {
            "/".to_string()
        } else {
            route
        };

        let _ = app.emit(events::ACTION_SHOW_WINDOW, ());
        let _ = app.emit(
            events::ACTION_NAVIGATE_LAST_ROUTE,
            TrayNavigatePayload { path: route },
        );
    }

    Ok(())
}

pub fn hide_main_window<R: Runtime>(app: &AppHandle<R>, emit_events: bool) -> Result<(), String> {
    let main_window = app
        .get_webview_window("main")
        .ok_or_else(|| "未找到主窗口".to_string())?;

    main_window
        .hide()
        .map_err(|e| format!("隐藏主窗口失败: {}", e))?;

    with_state_write(|state| {
        state.set_window_visible(false);
    });

    if emit_events {
        let _ = app.emit(events::ACTION_HIDE_WINDOW, ());
    }

    Ok(())
}

pub fn request_app_exit<R: Runtime>(app: &AppHandle<R>) -> Result<(), String> {
    let _ = app.emit(events::ACTION_EXIT_REQUESTED, ());

    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        match tokio::time::timeout(
            Duration::from_secs(4),
            crate::app::core::kernel_service::runtime::stop_kernel(),
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
