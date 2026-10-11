//! 托盘图标与菜单构建、菜单事件分发
//!
//! 同时持有全局运行态存储 `TRAY_RUNTIME_STATE` —— 图标渲染、菜单文案、
//! 窗口显隐、退出判定读写的都是这一份。

use super::super::icon;
use super::super::model::{
    menu_ids, TrayCloseBehavior, TrayRuntimeStateInput, TrayToggleProxyFeaturePayload, TRAY_ICON_ID,
};
use super::super::state::TrayRuntimeState;
use super::actions::{
    apply_system_proxy_toggle_from_tray, apply_tun_toggle_from_tray, request_app_exit,
    restart_kernel_from_tray,
};
use super::runtime_state::{with_state_read, with_state_write};
use super::texts::{compose_tooltip, mode_summary_text, tray_text_for_locale, TrayText};
use super::window::show_main_window;
use lazy_static::lazy_static;
use std::sync::RwLock;
use tauri::menu::{CheckMenuItemBuilder, MenuBuilder, MenuItemBuilder, SubmenuBuilder};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::AppHandle;
use tracing::{debug, info, warn};

lazy_static! {
    pub(super) static ref TRAY_RUNTIME_STATE: RwLock<TrayRuntimeState> =
        RwLock::new(TrayRuntimeState::default());
}

pub(super) fn resolve_tray_icon(
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

pub(super) fn build_tray_menu(
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

pub(super) fn handle_proxy_toggle_menu_event(app: &AppHandle, feature: &str, enabled: bool) {
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

pub(super) fn handle_menu_event(app: &AppHandle, menu_id: &str) {
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

pub(super) fn handle_tray_icon_event(tray: &tauri::tray::TrayIcon, event: TrayIconEvent) {
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

pub(super) fn create_or_replace_tray_icon(
    app: &AppHandle,
    state: &TrayRuntimeState,
) -> Result<(), String> {
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
