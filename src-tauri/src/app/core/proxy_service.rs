//! 代理服务模块（聚合入口）
//!
//! 原先这四类职责全部塞在一个 900+ 行的文件里：26 条 Tauri 命令与配置写入逻辑混排，
//! 改一处要通读全文。现按职责拆为下面四个子模块：
//!
//! - [`inbound_writer`]：把代理模式翻译成 inbound 配置并落盘 + 操作 OS 代理设置
//!   （启动**前**的行为，与 gRPC 无关）
//! - [`grpc_bridge`]：前端命令 → sing-box 1.14 官方 gRPC API 的桥接
//!   （启动**后**向运行中内核发问）
//! - [`status`]：内核状态快照与网络质量测试
//! - [`custom_rule_crud`]：自定义规则的增删改查（走「写配置 + 重启内核」，不依赖 gRPC）
//!
//! 注意：Tauri 的 `#[tauri::command]` 会生成 `__cmd__xxx` 辅助符号，
//! **不会**被 glob 再导出传递，因此 `lib.rs` 中的命令注册直接引用子模块路径
//! （如 `proxy_service::grpc_bridge::get_groups`），而非经本文件转发。

pub mod custom_rule_crud;
pub mod grpc_bridge;
pub mod inbound_writer;
pub mod status;

// 供内部跨模块调用（非命令）再导出
pub use inbound_writer::*;
