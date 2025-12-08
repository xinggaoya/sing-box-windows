use crate::process::manager::ProcessManager;
use std::sync::Arc;

lazy_static::lazy_static! {
    pub(super) static ref PROCESS_MANAGER: Arc<ProcessManager> =
        Arc::new(ProcessManager::new());
}

pub mod download;
pub mod event;
pub mod guard;
pub mod runtime;
pub mod state;
pub mod status;
pub mod utils;
pub mod versioning;

pub use download::{download_kernel, install_kernel};
pub use runtime::{
    apply_proxy_settings, force_stop_and_exit, kernel_start_enhanced, kernel_stop_background,
    kernel_stop_enhanced, resolve_proxy_runtime_state, start_kernel_with_state, stop_kernel,
    ProxyOverrides, ResolvedProxyState,
};
pub use status::{
    get_system_uptime, is_kernel_running, kernel_check_health, kernel_get_status_enhanced,
};
pub use versioning::{check_config_validity, check_kernel_version, get_latest_kernel_version_cmd};
pub use state::{KernelRuntimeConfig, KernelState, KernelStateManager, KERNEL_STATE};
