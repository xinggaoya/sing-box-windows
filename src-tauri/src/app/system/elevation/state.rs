//! 提权状态模型与合并规则。
//!
//! 把「如何探测当前进程是否已提权」拆成两层：
//! - 本文件（平台无关）：状态枚举 + 合并规则，**纯函数、可跨平台单测**；
//! - `windows.rs` / `unix.rs`（平台相关）：具体探测实现。

/// 提权状态探测结果。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ElevationState {
    /// 已提权（Administrator / root）。
    Elevated,
    /// 未提权。
    NotElevated,
    /// 无法判定（例如 API 调用失败且无可用回退）。
    Unknown,
}

impl ElevationState {
    /// 是否可确认为「已提权」。
    ///
    /// `Unknown` 视为**未提权** —— 安全侧默认：无法判定时不得跳过必要的提权流程。
    pub fn is_elevated(self) -> bool {
        matches!(self, ElevationState::Elevated)
    }
}

/// 合并 token 检测与回退检测的结果。
///
/// **token 检测是权威来源**：它直接读进程 token，没有环境依赖；
/// 回退检测（Windows 上是 `net session`）依赖 Server 服务可用性，会假阴性。
/// 因此只有 token 结果为 `Unknown` 时才采信回退结果。
pub fn merge_elevation(token: ElevationState, fallback: ElevationState) -> ElevationState {
    match token {
        ElevationState::Unknown => fallback,
        authoritative => authoritative,
    }
}

#[cfg(test)]
#[path = "state.tests.rs"]
mod tests;
