use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct LogConfig {
    pub level: String,
    pub dir: String,
    pub file_name_prefix: String,
    pub rotation: String, // "hourly", "daily", "never"
    pub max_file_size: u64,
    pub max_files: u32,
}

#[derive(Debug, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub password: Option<String>,
    pub pool_max_open: u64,
    pub pool_max_idle: u64,
    pub pool_timeout_seconds: u64,
}

#[derive(Debug, Deserialize)]
pub struct Config {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub jwt: JwtConfig,
    pub redis: RedisConfig,
    pub rate_limit: RateLimitConfig,
    pub log: LogConfig,
}

#[derive(Debug, Deserialize)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Debug, Deserialize)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
}

#[derive(Debug, Deserialize)]
pub struct JwtConfig {
    pub secret: String,
    pub expiration: i64,
}

#[derive(Debug, Deserialize)]
pub struct RateLimitConfig {
    pub window_secs: u64,
    pub max_requests: u64,
}

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            level: "info".to_string(),
            dir: "logs".to_string(),
            file_name_prefix: "app".to_string(),
            rotation: "daily".to_string(),
            max_file_size: 100,
            max_files: 30,
        }
    }
}
