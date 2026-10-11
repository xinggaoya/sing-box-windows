//! 主窗口创建、显隐、关闭与托盘后台模式

use super::super::model::{events, TrayCloseBehavior, TrayNavigatePayload};
use super::runtime_state::{refresh_runtime_state_from_backend, with_state_read, with_state_write};

use tauri::{AppHandle, Emitter, Manager, WebviewWindow, WebviewWindowBuilder};
use tracing::warn;

pub(super) fn create_main_window(app: &AppHandle) -> Result<(), String> {
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

pub(super) fn ensure_main_window(app: &AppHandle) -> Result<(WebviewWindow, bool), String> {
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

pub(super) fn destroy_main_window_for_tray(app: &AppHandle) -> Result<(), String> {
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

pub(super) fn refresh_runtime_state_in_background(app: &AppHandle) {
    let app_handle = app.clone();
    tauri::async_runtime::spawn(async move {
        if let Err(err) = refresh_runtime_state_from_backend(&app_handle, true).await {
            warn!("窗口关闭后刷新托盘运行态失败: {}", err);
        }
    });
}
