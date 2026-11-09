//! `installation` 模块单元测试。
//!
//! 安装态判定是「界面报未安装 vs 报真实原因」的分水岭：判错一次，用户就会
//! 去反复下载内核，而真正的原因（被杀毒软件隔离 / 配置文件非法）被藏起来。
//! 判定逻辑已抽成纯函数 `classify`，这里把四种状态组合全部钉死。

use super::{classify, InstallState};

#[test]
fn 文件存在且版本可读_判定为已安装() {
    assert_eq!(classify(true, true, true), InstallState::Installed);
    assert_eq!(classify(true, false, true), InstallState::Installed);
}

#[test]
fn 文件存在但版本读不到_判定为不可读() {
    assert_eq!(classify(true, false, false), InstallState::Unreadable);
    assert_eq!(classify(true, true, false), InstallState::Unreadable);
}

#[test]
fn 文件缺失但有历史记录_判定为文件丢失() {
    // 曾经装过、现在没了 —— 典型场景是杀毒软件把内核隔离了
    assert_eq!(classify(false, true, false), InstallState::FileLost);
}

#[test]
fn 文件缺失且无历史记录_判定为从未安装() {
    assert_eq!(classify(false, false, false), InstallState::Missing);
}

#[test]
fn 仅文件存在即视为已安装_无论版本能否读出() {
    // 「文件在磁盘上」本身就是「已安装」的事实；读不出版本是另一层问题
    // （无法执行 / 被拦截），不能退化成「未安装」。
    assert!(InstallState::Installed.is_installed());
    assert!(InstallState::Unreadable.is_installed());
    assert!(!InstallState::FileLost.is_installed());
    assert!(!InstallState::Missing.is_installed());
}
