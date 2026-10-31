//! sing-box 配置相关的公共逻辑
//!
//! 这里集中放置“配置生成 / 设置同步 / 节点注入”等能力，避免散落在订阅模块里做模板替换。

pub mod common;
pub mod config_generator;
pub mod config_intent;
pub mod config_render;
pub mod config_rules;
mod config_schema;
pub mod config_validator;
pub mod ownership;
pub mod settings_patch;
