//! 应用常量定义
//!
//! 这个文件包含应用程序中使用的所有常量定义
//! 集中管理常量便于统一修改和维护

// 导出所有子模块
pub mod core;
pub mod network;
pub mod system;
pub mod common;

// 重新导出常用常量，保持向后兼容性
pub use core::process;
pub use core::paths;
pub use core::config;
pub use network::network_config;
pub use network::api;
pub use system::registry;
pub use common::messages;
pub use common::log;
