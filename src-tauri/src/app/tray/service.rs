//! 托盘服务（聚合入口）
//!
//! 后端主导托盘：托盘图标与菜单、窗口显隐、托盘触发的内核/代理动作、
//! 以及运行态快照的同步，此前全部塞在一个 900+ 行的 `service.rs` 里。
//! 现按职责拆为：
//!
//! - [`texts`]：多语言文案与 tooltip 组装
//! - [`menu`]：图标/菜单构建与菜单事件分发（并持有全局运行态存储）
//! - [`window`]：主窗口创建、显隐、关闭、后台模式
//! - [`actions`]：托盘触发的业务动作（重启内核 / 切代理模式 / 切 TUN / 退出）
//! - [`runtime_state`]：运行态读写与后端快照同步
//!
//! 子模块之间交叉较多（菜单动作要改窗口、动作要刷新运行态），
//! 因此这里统一再导出，保持既有的 `crate::app::tray::xxx` 调用路径不变。

pub mod actions;
pub mod menu;
pub mod runtime_state;
pub mod texts;
pub mod window;

pub use actions::request_app_exit;
pub use menu::{
    apply_startup_preferences, consume_pending_proxy_toggle, init_tray, set_last_visible_route,
    sync_tray_state,
};
pub use runtime_state::refresh_runtime_state_from_backend;
pub use window::{
    close_main_window, consume_pending_restore_route, enter_startup_background_mode,
    hide_main_window, should_prevent_exit, show_main_window,
};

#[cfg(test)]
#[path = "service/service.tests.rs"]
mod tests;
