// App module - Main application services
pub mod constants;

// Core services
pub mod core {
    pub mod kernel_service;
    pub mod proxy_service;
}

// Network services
pub mod network {
    pub mod subscription_service;
}

// System services
pub mod system {
    pub mod system_service;
    pub mod update_service;
    pub mod config_service;
}

// Re-exports for backward compatibility
pub use core::kernel_service;
pub use core::proxy_service;
pub use network::subscription_service;
pub use system::system_service;
pub use system::update_service;
pub use system::config_service;

// Re-export constants for backward compatibility
pub use constants::common::log;
pub use constants::common::messages;
pub use constants::core::config;
pub use constants::core::paths;
pub use constants::core::process;
pub use constants::network::api;
pub use constants::network::network_config;
pub use constants::system::registry;
