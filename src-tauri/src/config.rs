use serde::Deserialize;
use crate::app::constants::{log, server, database, jwt, rate_limit};

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
            level: log::DEFAULT_LEVEL.to_string(),
            dir: log::DEFAULT_DIR.to_string(),
            file_name_prefix: log::DEFAULT_FILE_PREFIX.to_string(),
            rotation: log::rotation::DEFAULT.to_string(),
            max_file_size: log::DEFAULT_MAX_FILE_SIZE,
            max_files: log::DEFAULT_MAX_FILES,
        }
    }
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            host: server::DEFAULT_HOST.to_string(),
            port: server::DEFAULT_PORT,
        }
    }
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            url: database::DEFAULT_URL.to_string(),
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            secret: jwt::DEFAULT_SECRET.to_string(),
            expiration: jwt::DEFAULT_EXPIRATION,
        }
    }
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            window_secs: rate_limit::DEFAULT_WINDOW_SECS,
            max_requests: rate_limit::DEFAULT_MAX_REQUESTS,
        }
    }
}
