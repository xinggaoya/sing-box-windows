//! 应用常量定义
//! 
//! 这个文件包含应用程序中使用的所有常量定义
//! 集中管理常量便于统一修改和维护

/// 进程相关常量
pub mod process {
    /// Windows 创建进程时隐藏控制台窗口的标志
    pub const CREATE_NO_WINDOW: u32 = 0x08000000;
    
    /// 进程超时和延迟常量（秒）
    pub const GRACEFUL_TIMEOUT: u64 = 5;
    pub const HEALTH_CHECK_INTERVAL: u64 = 30;
    pub const MAX_RESTART_ATTEMPTS: u32 = 3;
    pub const RESTART_DELAY: u64 = 1;
}

/// 文件路径常量
pub mod paths {
    use std::path::PathBuf;
    use crate::utils::app_util::get_work_dir;

    /// 获取 Sing-Box 可执行文件路径
    pub fn get_kernel_path() -> PathBuf {
        let work_dir = get_work_dir();
        PathBuf::from(&work_dir).join("sing-box").join("sing-box.exe")
    }

    /// 获取 Sing-Box 工作目录
    pub fn get_kernel_work_dir() -> PathBuf {
        let work_dir = get_work_dir();
        PathBuf::from(&work_dir).join("sing-box")
    }

    /// 获取配置文件路径
    pub fn get_config_path() -> PathBuf {
        let work_dir = get_work_dir();
        PathBuf::from(&work_dir).join("sing-box").join("config.json")
    }
}

/// 网络常量
pub mod network {
    /// 默认监听地址
    pub const DEFAULT_LISTEN_ADDRESS: &str = "0.0.0.0";
    
    /// 默认代理端口
    pub const DEFAULT_PROXY_PORT: u16 = 12080;
    
    /// 默认 Clash API 地址
    pub const DEFAULT_CLASH_API_ADDRESS: &str = "127.0.0.1";
    
    /// 默认 Clash API 端口
    pub const DEFAULT_CLASH_API_PORT: u16 = 12081;
    
    /// 网络请求超时时间（秒）
    pub const HTTP_TIMEOUT_SECONDS: u64 = 30;
}

/// API 常量
pub mod api {
    /// GitHub API URL
    pub const GITHUB_API_URL: &str = "https://api.github.com/repos/xinggaoya/sing-box-windows/releases/latest";
    
    /// 用户代理
    pub const USER_AGENT: &str = "sing-box-windows";
}

/// 提示消息常量
pub mod messages {
    // 错误消息
    pub const ERR_KERNEL_NOT_FOUND: &str = "内核文件不存在";
    pub const ERR_VERSION_CHECK_FAILED: &str = "执行版本检查失败";
    pub const ERR_GET_VERSION_FAILED: &str = "获取版本信息失败";
    pub const ERR_CONFIG_READ_FAILED: &str = "读取配置文件失败";
    pub const ERR_DOWNLOAD_FAILED: &str = "下载失败";
    pub const ERR_SUBSCRIPTION_FAILED: &str = "下载订阅失败";
    pub const ERR_PROCESS_SUBSCRIPTION_FAILED: &str = "处理订阅内容失败";
    pub const ERR_GET_EXE_PATH_FAILED: &str = "获取当前程序路径失败";
    pub const ERR_RESTART_FAILED: &str = "重启失败";
    pub const ERR_INVALID_CONFIG: &str = "配置文件无效";
    pub const ERR_PROCESS_ALREADY_RUNNING: &str = "进程已在运行中";
    pub const ERR_PROCESS_NOT_RUNNING: &str = "进程未运行";
    pub const ERR_PROCESS_START_FAILED: &str = "进程启动失败";
    pub const ERR_PROCESS_STOP_FAILED: &str = "进程停止失败";
    pub const ERR_HTTP_CLIENT_FAILED: &str = "创建HTTP客户端失败";
    pub const ERR_REQUEST_FAILED: &str = "请求失败";
    pub const ERR_SERVER_ERROR: &str = "服务器返回错误状态码";
    pub const ERR_FILE_SIZE_UNKNOWN: &str = "无法获取文件大小";
    pub const ERR_CREATE_DIR_FAILED: &str = "创建目录失败";
    pub const ERR_CREATE_FILE_FAILED: &str = "创建文件失败";
    pub const ERR_OPEN_FILE_FAILED: &str = "打开文件失败";
    pub const ERR_READ_ARCHIVE_FAILED: &str = "读取归档失败";
    pub const ERR_EXTRACT_FILE_FAILED: &str = "解压文件失败";
    pub const ERR_INVALID_FILENAME: &str = "无效的文件名";
    pub const ERR_WRITE_FILE_FAILED: &str = "写入文件失败";
    pub const ERR_READ_FILE_FAILED: &str = "读取文件失败";
    pub const ERR_KEY_NOT_FOUND: &str = "未找到键";
    
    // 信息消息
    pub const INFO_PROCESS_STARTED: &str = "进程启动成功";
    pub const INFO_PROCESS_STOPPED: &str = "进程已停止";
    pub const INFO_SYSTEM_PROXY_DISABLED: &str = "系统代理已关闭";
    pub const INFO_CONFIG_CHECK_PASSED: &str = "配置文件检查通过";
    pub const INFO_PROXY_MODE_ENABLED: &str = "代理模式已启用";
    pub const INFO_DOWNLOAD_STARTED: &str = "开始下载文件";
    pub const INFO_UNZIP_STARTED: &str = "开始解压文件";
    pub const INFO_EXTRACTING_FILE: &str = "正在解压";
}

/// 配置常量
pub mod config {
    /// 默认的 Inbound 标签
    pub const DEFAULT_INBOUND_TAG: &str = "mixed-in";
    
    /// 默认的 Inbound 类型
    pub const DEFAULT_INBOUND_TYPE: &str = "mixed";
}

/// 日志常量
pub mod log {
    /// 日志级别
    pub const DEFAULT_LEVEL: &str = "debug";
    
    /// 日志目录
    pub const DEFAULT_DIR: &str = "logs";
    
    /// 日志文件名前缀
    pub const DEFAULT_FILE_PREFIX: &str = "app";
    
    /// 日志轮转类型
    pub mod rotation {
        pub const HOURLY: &str = "hourly";
        pub const DAILY: &str = "daily";
        pub const NEVER: &str = "never";
        pub const DEFAULT: &str = "daily";
    }
    
    /// 默认最大文件大小(MB)
    pub const DEFAULT_MAX_FILE_SIZE: u64 = 100;
    
    /// 默认最大文件数量
    pub const DEFAULT_MAX_FILES: u32 = 30;
}

/// 注册表常量
pub mod registry {
    /// Windows Internet 设置注册表路径
    pub const INTERNET_SETTINGS: &str = r"Software\Microsoft\Windows\CurrentVersion\Internet Settings";
    
    /// 代理开关键名
    pub const PROXY_ENABLE: &str = "ProxyEnable";
    
    /// 代理服务器键名
    pub const PROXY_SERVER: &str = "ProxyServer";
}

/// 服务器默认配置
pub mod server {
    /// 默认主机地址
    pub const DEFAULT_HOST: &str = "127.0.0.1";
    
    /// 默认端口
    pub const DEFAULT_PORT: u16 = 8080;
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

/// 速率限制配置
pub mod rate_limit {
    /// 默认窗口时间(秒)
    pub const DEFAULT_WINDOW_SECS: u64 = 60;
    
    /// 默认最大请求数
    pub const DEFAULT_MAX_REQUESTS: u64 = 100;
} 