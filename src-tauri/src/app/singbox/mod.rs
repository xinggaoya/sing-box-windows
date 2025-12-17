//! sing-box 配置相关的公共逻辑
//!
//! 这里集中放置“配置生成 / 设置同步 / 节点注入”等能力，避免散落在订阅模块里做模板替换。

pub mod config_generator;
pub mod settings_patch;
mod common;
mod config_schema;
