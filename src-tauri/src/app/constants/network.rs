//! 网络服务相关常量
//! 
//! 包含网络配置、API、订阅等网络功能相关的常量

/// 网络配置常量
pub mod network_config {
    /// 默认监听地址
    pub const DEFAULT_LISTEN_ADDRESS: &str = "0.0.0.0";

    /// 默认代理端口
    pub const DEFAULT_PROXY_PORT: u16 = 12080;

    /// 默认 Clash API 地址
    pub const DEFAULT_CLASH_API_ADDRESS: &str = "127.0.0.1";

    /// 默认 Clash API 端口
    pub const DEFAULT_CLASH_API_PORT: u16 = 12081;

    /// 默认 API Token
    pub const DEFAULT_API_TOKEN: &str = "";

    /// 网络请求超时时间（秒）
    pub const HTTP_TIMEOUT_SECONDS: u64 = 30;
}

/// API 常量
pub mod api {
    /// GitHub API URL
    pub const GITHUB_API_URL: &str =
        "https://api.github.com/repos/xinggaoya/sing-box-windows/releases/latest";

    /// 用户代理
    pub const USER_AGENT: &str = "sing-box-windows";
}

/// 服务器默认配置
pub mod server {
    /// 默认主机地址
    pub const DEFAULT_HOST: &str = "127.0.0.1";

    /// 默认端口
    pub const DEFAULT_PORT: u16 = 8080;
}

/// 速率限制配置
pub mod rate_limit {
    /// 默认窗口时间(秒)
    pub const DEFAULT_WINDOW_SECS: u64 = 60;

    /// 默认最大请求数
    pub const DEFAULT_MAX_REQUESTS: u64 = 100;
}
