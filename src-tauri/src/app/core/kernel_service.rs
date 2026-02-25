use crate::process::manager::ProcessManager;
use std::sync::Arc;

lazy_static::lazy_static! {
    pub(super) static ref PROCESS_MANAGER: Arc<ProcessManager> =
        Arc::new(ProcessManager::new());
}

pub mod download;
pub mod embedded;
pub mod event;
pub mod guard;
pub mod orchestrator;
pub mod runtime;
pub mod state;
pub mod status;
pub mod utils;
pub mod versioning;

pub use download::download_kernel;
pub use runtime::{
    apply_proxy_settings, kernel_restart_fast, kernel_start_enhanced, kernel_stop_enhanced,
    orchestrated_restart_kernel, orchestrated_start_kernel, orchestrated_stop_kernel,
    resolve_proxy_runtime_state, start_kernel_with_state, stop_kernel, ProxyOverrides,
    ResolvedProxyState,
};
pub use orchestrator::current_state_version;
pub use status::{
    get_system_uptime, is_kernel_running, kernel_check_health, kernel_get_snapshot,
    kernel_get_status_enhanced,
};
pub use versioning::{check_config_validity, check_kernel_version, get_latest_kernel_version_cmd};
pub use state::{KernelRuntimeConfig, KernelState, KernelStateManager, KERNEL_STATE};
