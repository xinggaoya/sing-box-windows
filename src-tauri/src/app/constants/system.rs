//! 系统服务相关常量
//!
//! 包含系统配置、注册表等系统功能相关的常量

/// 注册表常量
pub mod registry {
    /// Windows Internet 设置注册表路径
    pub const INTERNET_SETTINGS: &str =
        r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";

    /// 代理开关键名
    pub const PROXY_ENABLE: &str = "ProxyEnable";

    /// 代理服务器键名
    pub const PROXY_SERVER: &str = "ProxyServer";
}

/// 数据库默认配置
pub mod database {
    /// 默认数据库连接URL
    pub const DEFAULT_URL: &str = "sqlite://data.db";
}

/// JWT认证配置
pub mod jwt {
    /// 默认密钥(注意：生产环境应使用安全的随机密钥)
    pub const DEFAULT_SECRET: &str = "your-secret-key";

    /// 默认过期时间(秒)
    pub const DEFAULT_EXPIRATION: i64 = 86400; // 24小时
}
