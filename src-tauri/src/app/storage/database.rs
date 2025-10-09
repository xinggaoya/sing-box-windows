use sqlx::{sqlite::SqlitePool, Row, migrate::MigrateDatabase, sqlite::Sqlite};
use serde::{Deserialize, Serialize};
use chrono::Utc;
use crate::app::storage::state_model::{AppConfig, ThemeConfig, LocaleConfig, WindowConfig, UpdateConfig};
use super::error::StorageError;

#[derive(Debug, Clone)]
pub struct DatabaseService {
    pool: SqlitePool,
}

impl DatabaseService {
    pub async fn new(database_path: &str) -> Result<Self, StorageError> {
        tracing::info!("🗄️ 初始化数据库: {}", database_path);
        let database_url = format!("sqlite:{}", database_path);
        
        // 创建数据库
        if !Sqlite::database_exists(&database_url).await? {
            tracing::info!("📁 创建新数据库");
            Sqlite::create_database(&database_url).await?;
        } else {
            tracing::info!("📁 数据库已存在");
        }
        
        let pool = SqlitePool::connect(&database_url).await?;
        tracing::info!("✅ 数据库连接成功");
        
        // 创建表结构
        Self::create_tables(&pool).await?;
        tracing::info!("✅ 数据库表创建完成");
        
        Ok(Self { pool })
    }
    
    async fn create_tables(pool: &SqlitePool) -> Result<(), StorageError> {
        // 应用配置表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS app_config (
                id INTEGER PRIMARY KEY,
                auto_start_kernel BOOLEAN DEFAULT FALSE,
                prefer_ipv6 BOOLEAN DEFAULT FALSE,
                proxy_port INTEGER DEFAULT 12080,
                api_port INTEGER DEFAULT 12081,
                proxy_mode TEXT DEFAULT 'manual',
                tray_instance_id TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;
        
        // 主题配置表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS theme_config (
                id INTEGER PRIMARY KEY,
                is_dark BOOLEAN DEFAULT FALSE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;
        
        // 语言配置表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS locale_config (
                id INTEGER PRIMARY KEY,
                locale TEXT DEFAULT 'zh-CN',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;
        
        // 窗口配置表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS window_config (
                id INTEGER PRIMARY KEY,
                is_maximized BOOLEAN DEFAULT FALSE,
                width INTEGER DEFAULT 1000,
                height INTEGER DEFAULT 700,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;
        
        // 更新配置表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS update_config (
                id INTEGER PRIMARY KEY,
                auto_check BOOLEAN DEFAULT TRUE,
                last_check INTEGER DEFAULT 0,
                last_version TEXT,
                skip_version TEXT,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;
        
        // 通用配置表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS generic_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;
        
        Ok(())
    }
    
    // 应用配置
    pub async fn load_app_config(&self) -> Result<Option<AppConfig>, StorageError> {
        let row = sqlx::query(
            "SELECT * FROM app_config WHERE id = 1"
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            Ok(Some(AppConfig {
                auto_start_kernel: row.get("auto_start_kernel"),
                prefer_ipv6: row.get("prefer_ipv6"),
                proxy_port: row.get("proxy_port"),
                api_port: row.get("api_port"),
                proxy_mode: row.get("proxy_mode"),
                tray_instance_id: row.get("tray_instance_id"),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn save_app_config(&self, config: &AppConfig) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO app_config 
            (id, auto_start_kernel, prefer_ipv6, proxy_port, api_port, proxy_mode, tray_instance_id, updated_at)
            VALUES (1, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(config.auto_start_kernel)
        .bind(config.prefer_ipv6)
        .bind(config.proxy_port)
        .bind(config.api_port)
        .bind(&config.proxy_mode)
        .bind(&config.tray_instance_id)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // 主题配置
    pub async fn load_theme_config(&self) -> Result<Option<ThemeConfig>, StorageError> {
        let row = sqlx::query(
            "SELECT * FROM theme_config WHERE id = 1"
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            Ok(Some(ThemeConfig {
                is_dark: row.get("is_dark"),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn save_theme_config(&self, config: &ThemeConfig) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO theme_config 
            (id, is_dark, updated_at)
            VALUES (1, ?, ?)
            "#,
        )
        .bind(config.is_dark)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // 语言配置
    pub async fn load_locale_config(&self) -> Result<Option<LocaleConfig>, StorageError> {
        let row = sqlx::query(
            "SELECT * FROM locale_config WHERE id = 1"
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            Ok(Some(LocaleConfig {
                locale: row.get("locale"),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn save_locale_config(&self, config: &LocaleConfig) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO locale_config 
            (id, locale, updated_at)
            VALUES (1, ?, ?)
            "#,
        )
        .bind(&config.locale)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // 窗口配置
    pub async fn load_window_config(&self) -> Result<Option<WindowConfig>, StorageError> {
        let row = sqlx::query(
            "SELECT * FROM window_config WHERE id = 1"
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            Ok(Some(WindowConfig {
                is_maximized: row.get("is_maximized"),
                width: row.get("width"),
                height: row.get("height"),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn save_window_config(&self, config: &WindowConfig) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO window_config 
            (id, is_maximized, width, height, updated_at)
            VALUES (1, ?, ?, ?, ?)
            "#,
        )
        .bind(config.is_maximized)
        .bind(config.width)
        .bind(config.height)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // 更新配置
    pub async fn load_update_config(&self) -> Result<Option<UpdateConfig>, StorageError> {
        let row = sqlx::query(
            "SELECT * FROM update_config WHERE id = 1"
        )
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            Ok(Some(UpdateConfig {
                auto_check: row.get("auto_check"),
                last_check: row.get("last_check"),
                last_version: row.get("last_version"),
                skip_version: row.get("skip_version"),
            }))
        } else {
            Ok(None)
        }
    }
    
    pub async fn save_update_config(&self, config: &UpdateConfig) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO update_config 
            (id, auto_check, last_check, last_version, skip_version, updated_at)
            VALUES (1, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(config.auto_check)
        .bind(config.last_check)
        .bind(&config.last_version)
        .bind(&config.skip_version)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    // 通用配置保存方法
    pub async fn save_config<T>(&self, key: &str, value: &T) -> Result<(), StorageError>
    where
        T: Serialize + ?Sized,
    {
        let json = serde_json::to_string(value)?;
        
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS generic_config (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#
        )
        .execute(&self.pool)
        .await?;
        
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO generic_config 
            (key, value, updated_at)
            VALUES (?, ?, ?)
            "#
        )
        .bind(key)
        .bind(json)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;
        
        Ok(())
    }
    
    pub async fn load_config<T>(&self, key: &str) -> Result<Option<T>, StorageError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let row = sqlx::query(
            "SELECT value FROM generic_config WHERE key = ?"
        )
        .bind(key)
        .fetch_optional(&self.pool)
        .await?;
        
        if let Some(row) = row {
            let value: String = row.get("value");
            let config: T = serde_json::from_str(&value)?;
            Ok(Some(config))
        } else {
            Ok(None)
        }
    }
    
    pub async fn close(&self) -> Result<(), StorageError> {
        self.pool.close().await;
        Ok(())
    }
}