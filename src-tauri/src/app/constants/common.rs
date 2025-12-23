//! 通用常量
//!
//! 包含消息、日志等通用功能相关的常量

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

    /// 默认最大文件数量 —— 每天一个文件，仅保留最近 5 天
    pub const DEFAULT_MAX_FILES: u32 = 5;
}
