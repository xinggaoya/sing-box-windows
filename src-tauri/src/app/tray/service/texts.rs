//! 托盘文案与 tooltip 组装
//!
//! 托盘菜单与 tooltip 的文案在此按语言内联（它们不经过前端 vue-i18n，
//! 因此不属于 `pnpm test:i18n` 的校验范围；每次构建菜单时按当前语言重新取值）。

use super::super::state::TrayRuntimeState;

#[derive(Debug, Clone, Copy)]
pub(super) struct TrayText {
    pub(super) show_window: &'static str,
    pub(super) rebuild_window: &'static str,
    pub(super) kernel_menu: &'static str,
    pub(super) restart_kernel: &'static str,
    pub(super) status_running: &'static str,
    pub(super) status_stopped: &'static str,
    pub(super) proxy_controls: &'static str,
    pub(super) current_status: &'static str,
    pub(super) mode_system: &'static str,
    pub(super) mode_tun: &'static str,
    pub(super) mode_manual: &'static str,
    pub(super) quit: &'static str,
    pub(super) tooltip_kernel: &'static str,
    pub(super) tooltip_mode: &'static str,
    pub(super) tooltip_subscription: &'static str,
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

pub(super) fn tray_text_for_locale(locale: &str) -> TrayText {
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

pub(super) fn mode_summary_text(state: &TrayRuntimeState, text: &TrayText) -> String {
    match (state.system_proxy_enabled, state.tun_enabled) {
        (true, true) => format!("{} + {}", text.mode_system, text.mode_tun),
        (true, false) => text.mode_system.to_string(),
        (false, true) => text.mode_tun.to_string(),
        (false, false) => text.mode_manual.to_string(),
    }
}

pub(super) fn compose_tooltip(state: &TrayRuntimeState, text: &TrayText) -> String {
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
