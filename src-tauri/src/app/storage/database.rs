use super::error::StorageError;
use crate::app::storage::state_model::{
    AppConfig, LocaleConfig, ThemeConfig, UpdateConfig, WindowConfig,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{migrate::MigrateDatabase, sqlite::Sqlite, sqlite::SqlitePool, Row};
use tokio::sync::OnceCell;

static SCHEMA_INIT: OnceCell<()> = OnceCell::const_new();

#[derive(Debug, Clone)]
pub struct DatabaseService {
    pool: SqlitePool,
}

impl DatabaseService {
    pub async fn new(database_path: &str) -> Result<Self, StorageError> {
        tracing::info!("??? 初始化数据库: {}", database_path);
        let database_url = format!("sqlite:{}", database_path);

        // 创建数据库
        if !Sqlite::database_exists(&database_url).await? {
            tracing::info!("?? 创建新数据库");
            Sqlite::create_database(&database_url).await?;
        } else {
            tracing::info!("?? 数据库已存在");
        }

        let pool = SqlitePool::connect(&database_url).await?;
        tracing::info!("? 数据库连接成功");

        // 创建表结构（仅执行一次）
        Self::create_tables_once(&pool).await?;
        tracing::info!("? 数据库表创建完成");

        Ok(Self { pool })
    }

    async fn create_tables_once(pool: &SqlitePool) -> Result<(), StorageError> {
        SCHEMA_INIT
            .get_or_try_init(|| async { Self::create_tables(pool).await })
            .await
            .map(|_| ())
    }

    async fn create_tables(pool: &SqlitePool) -> Result<(), StorageError> {
        // 应用配置表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS app_config (
                id INTEGER PRIMARY KEY,
                auto_start_kernel BOOLEAN DEFAULT FALSE,
                auto_start_app BOOLEAN DEFAULT FALSE,
                prefer_ipv6 BOOLEAN DEFAULT FALSE,
                proxy_port INTEGER DEFAULT 12080,
                api_port INTEGER DEFAULT 12081,
                proxy_mode TEXT DEFAULT 'manual',
                system_proxy_enabled BOOLEAN DEFAULT FALSE,
                tun_enabled BOOLEAN DEFAULT FALSE,
                tray_instance_id TEXT,
                system_proxy_bypass TEXT DEFAULT 'localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.20.*;172.21.*;172.22.*;172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;172.28.*;172.29.*;172.30.*;172.31.*;192.168.*',
                tun_auto_route BOOLEAN DEFAULT TRUE,
                tun_strict_route BOOLEAN DEFAULT TRUE,
                tun_mtu INTEGER DEFAULT 1500,
                tun_ipv4 TEXT DEFAULT '172.19.0.1/30',
                tun_ipv6 TEXT DEFAULT 'fdfe:dcba:9876::1/126',
                tun_stack TEXT DEFAULT 'mixed',
                tun_enable_ipv6 BOOLEAN DEFAULT FALSE,
                active_config_path TEXT,
                installed_kernel_version TEXT,
                singbox_dns_proxy TEXT DEFAULT 'https://1.1.1.1/dns-query',
                singbox_dns_cn TEXT DEFAULT 'h3://dns.alidns.com/dns-query',
                singbox_dns_resolver TEXT DEFAULT '114.114.114.114',
                singbox_urltest_url TEXT DEFAULT 'http://cp.cloudflare.com/generate_204',
                singbox_default_proxy_outbound TEXT DEFAULT 'manual',
                singbox_block_ads BOOLEAN DEFAULT TRUE,
                singbox_download_detour TEXT DEFAULT 'manual',
                singbox_dns_hijack BOOLEAN DEFAULT TRUE,
                singbox_enable_app_groups BOOLEAN DEFAULT TRUE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        // 检查并添加 legacy 缺失字段（升级兼容）
        let alter_statements = [
            "ALTER TABLE app_config ADD COLUMN auto_start_app BOOLEAN DEFAULT FALSE",
            "ALTER TABLE app_config ADD COLUMN system_proxy_bypass TEXT DEFAULT 'localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.20.*;172.21.*;172.22.*;172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;172.28.*;172.29.*;172.30.*;172.31.*;192.168.*'",
            "ALTER TABLE app_config ADD COLUMN tun_auto_route BOOLEAN DEFAULT TRUE",
            "ALTER TABLE app_config ADD COLUMN tun_strict_route BOOLEAN DEFAULT TRUE",
            "ALTER TABLE app_config ADD COLUMN tun_mtu INTEGER DEFAULT 1500",
            "ALTER TABLE app_config ADD COLUMN tun_ipv4 TEXT DEFAULT '172.19.0.1/30'",
            "ALTER TABLE app_config ADD COLUMN tun_ipv6 TEXT DEFAULT 'fdfe:dcba:9876::1/126'",
            "ALTER TABLE app_config ADD COLUMN tun_stack TEXT DEFAULT 'mixed'",
            "ALTER TABLE app_config ADD COLUMN tun_enable_ipv6 BOOLEAN DEFAULT FALSE",
            "ALTER TABLE app_config ADD COLUMN system_proxy_enabled BOOLEAN DEFAULT FALSE",
            "ALTER TABLE app_config ADD COLUMN tun_enabled BOOLEAN DEFAULT FALSE",
            "ALTER TABLE app_config ADD COLUMN active_config_path TEXT",
            "ALTER TABLE app_config ADD COLUMN installed_kernel_version TEXT",
            "ALTER TABLE app_config ADD COLUMN singbox_dns_proxy TEXT DEFAULT 'https://1.1.1.1/dns-query'",
            "ALTER TABLE app_config ADD COLUMN singbox_dns_cn TEXT DEFAULT 'h3://dns.alidns.com/dns-query'",
            "ALTER TABLE app_config ADD COLUMN singbox_dns_resolver TEXT DEFAULT '114.114.114.114'",
            "ALTER TABLE app_config ADD COLUMN singbox_urltest_url TEXT DEFAULT 'http://cp.cloudflare.com/generate_204'",
            "ALTER TABLE app_config ADD COLUMN singbox_default_proxy_outbound TEXT DEFAULT 'manual'",
            "ALTER TABLE app_config ADD COLUMN singbox_block_ads BOOLEAN DEFAULT TRUE",
            "ALTER TABLE app_config ADD COLUMN singbox_download_detour TEXT DEFAULT 'manual'",
            "ALTER TABLE app_config ADD COLUMN singbox_dns_hijack BOOLEAN DEFAULT TRUE",
            "ALTER TABLE app_config ADD COLUMN singbox_enable_app_groups BOOLEAN DEFAULT TRUE",
        ];

        for statement in alter_statements {
            sqlx::query(statement).execute(pool).await.ok();
        }

        // 主题配置表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS theme_config (
                id INTEGER PRIMARY KEY,
                is_dark BOOLEAN DEFAULT FALSE,
                mode TEXT DEFAULT 'system',
                accent_color TEXT DEFAULT '#6366f1',
                compact_mode BOOLEAN DEFAULT FALSE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        // 主题配置表兼容字段补充
        let alter_theme_statements = [
            "ALTER TABLE theme_config ADD COLUMN mode TEXT DEFAULT 'system'",
            "ALTER TABLE theme_config ADD COLUMN accent_color TEXT DEFAULT '#6366f1'",
            "ALTER TABLE theme_config ADD COLUMN compact_mode BOOLEAN DEFAULT FALSE",
        ];

        for statement in alter_theme_statements {
            sqlx::query(statement).execute(pool).await.ok();
        }

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
                accept_prerelease BOOLEAN DEFAULT FALSE,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        // 更新配置兼容字段补充（确保旧表增加接收预发布字段）
        let alter_update_statements =
            ["ALTER TABLE update_config ADD COLUMN accept_prerelease BOOLEAN DEFAULT FALSE"];
        for statement in alter_update_statements {
            sqlx::query(statement).execute(pool).await.ok();
        }

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
        let row = sqlx::query("SELECT * FROM app_config WHERE id = 1")
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let default_config = AppConfig::default();
            Ok(Some(AppConfig {
                auto_start_kernel: row.get("auto_start_kernel"),
                auto_start_app: row.get("auto_start_app"),
                prefer_ipv6: row.get("prefer_ipv6"),
                proxy_port: row.get("proxy_port"),
                api_port: row.get("api_port"),
                proxy_mode: row.get("proxy_mode"),
                system_proxy_enabled: row
                    .try_get("system_proxy_enabled")
                    .unwrap_or(default_config.system_proxy_enabled),
                tun_enabled: row
                    .try_get("tun_enabled")
                    .unwrap_or(default_config.tun_enabled),
                tray_instance_id: row.get("tray_instance_id"),
                system_proxy_bypass: row
                    .try_get("system_proxy_bypass")
                    .unwrap_or_else(|_| default_config.system_proxy_bypass.clone()),
                tun_auto_route: row
                    .try_get("tun_auto_route")
                    .unwrap_or(default_config.tun_auto_route),
                tun_strict_route: row
                    .try_get("tun_strict_route")
                    .unwrap_or(default_config.tun_strict_route),
                tun_mtu: row.try_get("tun_mtu").unwrap_or(default_config.tun_mtu),
                tun_ipv4: row
                    .try_get("tun_ipv4")
                    .unwrap_or_else(|_| default_config.tun_ipv4.clone()),
                tun_ipv6: row
                    .try_get("tun_ipv6")
                    .unwrap_or_else(|_| default_config.tun_ipv6.clone()),
                tun_stack: row
                    .try_get("tun_stack")
                    .unwrap_or_else(|_| default_config.tun_stack.clone()),
                tun_enable_ipv6: row
                    .try_get("tun_enable_ipv6")
                    .unwrap_or(default_config.tun_enable_ipv6),
                active_config_path: row.try_get("active_config_path").unwrap_or(None),
                installed_kernel_version: row.try_get("installed_kernel_version").unwrap_or(None),
                singbox_dns_proxy: row
                    .try_get("singbox_dns_proxy")
                    .unwrap_or_else(|_| default_config.singbox_dns_proxy.clone()),
                singbox_dns_cn: row
                    .try_get("singbox_dns_cn")
                    .unwrap_or_else(|_| default_config.singbox_dns_cn.clone()),
                singbox_dns_resolver: row
                    .try_get("singbox_dns_resolver")
                    .unwrap_or_else(|_| default_config.singbox_dns_resolver.clone()),
                singbox_urltest_url: row
                    .try_get("singbox_urltest_url")
                    .unwrap_or_else(|_| default_config.singbox_urltest_url.clone()),
                singbox_default_proxy_outbound: row
                    .try_get("singbox_default_proxy_outbound")
                    .unwrap_or_else(|_| default_config.singbox_default_proxy_outbound.clone()),
                singbox_block_ads: row
                    .try_get("singbox_block_ads")
                    .unwrap_or(default_config.singbox_block_ads),
                singbox_download_detour: row
                    .try_get("singbox_download_detour")
                    .unwrap_or_else(|_| default_config.singbox_download_detour.clone()),
                singbox_dns_hijack: row
                    .try_get("singbox_dns_hijack")
                    .unwrap_or(default_config.singbox_dns_hijack),
                singbox_enable_app_groups: row
                    .try_get("singbox_enable_app_groups")
                    .unwrap_or(default_config.singbox_enable_app_groups),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_app_config(&self, config: &AppConfig) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO app_config
            (id, auto_start_kernel, auto_start_app, prefer_ipv6, proxy_port, api_port, proxy_mode, system_proxy_enabled, tun_enabled, tray_instance_id, system_proxy_bypass, tun_auto_route, tun_strict_route, tun_mtu, tun_ipv4, tun_ipv6, tun_stack, tun_enable_ipv6, active_config_path, installed_kernel_version, singbox_dns_proxy, singbox_dns_cn, singbox_dns_resolver, singbox_urltest_url, singbox_default_proxy_outbound, singbox_block_ads, singbox_download_detour, singbox_dns_hijack, singbox_enable_app_groups, updated_at)
            VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(config.auto_start_kernel)
        .bind(config.auto_start_app)
        .bind(config.prefer_ipv6)
        .bind(config.proxy_port)
        .bind(config.api_port)
        .bind(&config.proxy_mode)
        .bind(config.system_proxy_enabled)
        .bind(config.tun_enabled)
        .bind(&config.tray_instance_id)
        .bind(&config.system_proxy_bypass)
        .bind(config.tun_auto_route)
        .bind(config.tun_strict_route)
        .bind(config.tun_mtu)
        .bind(&config.tun_ipv4)
        .bind(&config.tun_ipv6)
        .bind(&config.tun_stack)
        .bind(config.tun_enable_ipv6)
        .bind(&config.active_config_path)
        .bind(&config.installed_kernel_version)
        .bind(&config.singbox_dns_proxy)
        .bind(&config.singbox_dns_cn)
        .bind(&config.singbox_dns_resolver)
        .bind(&config.singbox_urltest_url)
        .bind(&config.singbox_default_proxy_outbound)
        .bind(config.singbox_block_ads)
        .bind(&config.singbox_download_detour)
        .bind(config.singbox_dns_hijack)
        .bind(config.singbox_enable_app_groups)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // 主题配置
    pub async fn load_theme_config(&self) -> Result<Option<ThemeConfig>, StorageError> {
        let row = sqlx::query("SELECT * FROM theme_config WHERE id = 1")
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let stored_mode: String = row.try_get("mode").unwrap_or_else(|_| "system".to_string());
            let normalized_mode = match stored_mode.as_str() {
                "light" | "dark" | "system" => stored_mode,
                _ => "system".to_string(),
            };

            let accent_color: String = row
                .try_get("accent_color")
                .unwrap_or_else(|_| "#6366f1".to_string());

            Ok(Some(ThemeConfig {
                is_dark: row.try_get("is_dark").unwrap_or(true),
                mode: normalized_mode,
                accent_color,
                compact_mode: row.try_get("compact_mode").unwrap_or(false),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_theme_config(&self, config: &ThemeConfig) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO theme_config 
            (id, is_dark, mode, accent_color, compact_mode, updated_at)
            VALUES (1, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(config.is_dark)
        .bind(&config.mode)
        .bind(&config.accent_color)
        .bind(config.compact_mode)
        .bind(Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    // 语言配置
    pub async fn load_locale_config(&self) -> Result<Option<LocaleConfig>, StorageError> {
        let row = sqlx::query("SELECT * FROM locale_config WHERE id = 1")
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
        let row = sqlx::query("SELECT * FROM window_config WHERE id = 1")
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
        let row = sqlx::query("SELECT * FROM update_config WHERE id = 1")
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let default_config = UpdateConfig::default();
            Ok(Some(UpdateConfig {
                auto_check: row.get("auto_check"),
                last_check: row.get("last_check"),
                last_version: row.get("last_version"),
                skip_version: row.get("skip_version"),
                accept_prerelease: row
                    .try_get("accept_prerelease")
                    .unwrap_or(default_config.accept_prerelease),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_update_config(&self, config: &UpdateConfig) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO update_config 
            (id, auto_check, last_check, last_version, skip_version, accept_prerelease, updated_at)
            VALUES (1, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(config.auto_check)
        .bind(config.last_check)
        .bind(&config.last_version)
        .bind(&config.skip_version)
        .bind(config.accept_prerelease)
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
            INSERT OR REPLACE INTO generic_config 
            (key, value, updated_at)
            VALUES (?, ?, ?)
            "#,
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
        let row = sqlx::query("SELECT value FROM generic_config WHERE key = ?")
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

    // 删除配置
    pub async fn remove_config(&self, key: &str) -> Result<(), StorageError> {
        sqlx::query("DELETE FROM generic_config WHERE key = ?1")
            .bind(key)
            .execute(&self.pool)
            .await
            .map_err(StorageError::Database)?;

        Ok(())
    }

    pub async fn close(&self) -> Result<(), StorageError> {
        self.pool.close().await;
        Ok(())
    }
}
