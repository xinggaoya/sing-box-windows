use super::error::StorageError;
use crate::app::core::tun_profile::normalize_persisted_tun_route_exclude_address;
use crate::app::storage::state_model::{
    AppConfig, LocaleConfig, ThemeConfig, UpdateConfig, WindowConfig,
};
use chrono::Utc;
use serde::{Deserialize, Serialize};
use sqlx::{
    migrate::MigrateDatabase,
    sqlite::Sqlite,
    sqlite::SqlitePool,
    sqlite::{SqliteConnectOptions, SqliteJournalMode, SqlitePoolOptions, SqliteSynchronous},
    Row,
};
use std::str::FromStr;
use std::time::Duration;

/// SQLite 写锁等待超时。应用存在多条并发写路径（内核守护每8s 回写自愈策略、
/// 自动保存、订阅切换等），无busy_timeout 时并发写会直接返回 SQLITE_BUSY。
const SQLITE_BUSY_TIMEOUT: Duration = Duration::from_secs(5);
/// 连接池上限。SQLite 单写者模型，过多连接只会增加锁竞争。
const SQLITE_MAX_CONNECTIONS: u32 = 4;
/// 连接获取超时，避免池耗尽时调用方无限等待。
const SQLITE_ACQUIRE_TIMEOUT: Duration = Duration::from_secs(10);
/// 进程退出时回收连接的最长等待时间。
const SQLITE_IDLE_TIMEOUT: Duration = Duration::from_secs(300);

/// 当前 schema 版本，写入 SQLite 的 `user_version`。
///
/// 之前没有版本概念，迁移失败也完全不可观测。现在版本号与实际列集合共同构成
/// “数据库处于什么状态”的唯一真相，可在日志与测试中直接断言。
/// 新增字段时：递增此值，并在对应表的 `ensure_columns` 列表中补列。
pub const SCHEMA_VERSION: i64 = 1;

/// 读取现有列名集合。
///
/// `PRAGMA table_info` 不支持参数化占位符，表名只能拼接 —— 调用方均传本文件内的
/// 字面量常量（非外部输入），因此不存在注入面。
async fn existing_columns(
    pool: &SqlitePool,
    table: &str,
) -> Result<std::collections::HashSet<String>, StorageError> {
    let rows = sqlx::query(&format!("PRAGMA table_info({})", table))
        .fetch_all(pool)
        .await?;
    let mut names = std::collections::HashSet::new();
    for row in rows {
        names.insert(row.try_get::<String, _>("name")?);
    }
    Ok(names)
}

/// 幂等补列：已存在的列直接跳过，缺失才执行 `ALTER TABLE ADD COLUMN`。
///
/// 早期实现对每条 `ALTER` 直接 `.ok()` 丢弃错误，结果是迁移失败在日志里完全不可见，
/// 线上仅表现为“某项设置不生效”且无法自查；`StorageError::Migration` 也因此从未被构造。
/// 现改为：显式查列 -> 缺失才 ALTER -> 失败则带上完整 SQL 上抛。
async fn ensure_columns(
    pool: &SqlitePool,
    table: &str,
    columns: &[(&str, &str)],
) -> Result<(), StorageError> {
    let present = existing_columns(pool, table).await?;
    let mut applied = 0usize;

    for (column, definition) in columns {
        if present.contains(*column) {
            continue;
        }
        let sql = format!("ALTER TABLE {} ADD COLUMN {} {}", table, column, definition);
        sqlx::query(&sql)
            .execute(pool)
            .await
            .map_err(|e| StorageError::Migration(format!("{} -> {}", sql.trim(), e)))?;
        applied += 1;
        tracing::debug!("⬆️ schema 补列: {}.{}", table, column);
    }

    if applied > 0 {
        tracing::info!("⬆️ {}.* 补齐 {} 个缺失列", table, applied);
    }
    Ok(())
}

/// 读取 `PRAGMA user_version`。
pub async fn read_schema_version(pool: &SqlitePool) -> Result<i64, StorageError> {
    let row: (i64,) = sqlx::query_as("PRAGMA user_version")
        .fetch_one(pool)
        .await?;
    Ok(row.0)
}

/// 写入 `PRAGMA user_version`（schema 建立完成后的最后一步）。
async fn write_schema_version(pool: &SqlitePool) -> Result<(), StorageError> {
    sqlx::query(&format!("PRAGMA user_version = {}", SCHEMA_VERSION))
        .execute(pool)
        .await?;
    tracing::info!("🗃️ schema 版本已标记为 {}", SCHEMA_VERSION);
    Ok(())
}

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
            tracing::info!("🆕 创建新数据库");
            Sqlite::create_database(&database_url).await?;
        } else {
            tracing::info!("📂 数据库已存在");
        }

        let pool = Self::build_pool(&database_url).await?;
        tracing::info!(
            "✅ 数据库连接成功 (WAL, busy_timeout={}s)",
            SQLITE_BUSY_TIMEOUT.as_secs()
        );

        // 建表/迁移按“库”而非“进程”执行：不同数据库路径（如测试用的临时库）
        // 必须各自完成一次 schema 初始化，否则会静默缺表。
        Self::ensure_schema(&pool).await?;
        tracing::info!("✅ 数据库表结构就绪");

        Ok(Self { pool })
    }

    /// 构造连接池：WAL + busy_timeout + 连接数上限。
    ///
    /// 之前使用 `SqlitePool::connect` 裸连，走的是 SQLite 默认的 rollback journal
    /// 与 `busy_timeout=0`，并发写会直接报 SQLITE_BUSY，WAL 缺失还会阻塞读。
    async fn build_pool(database_url: &str) -> Result<SqlitePool, StorageError> {
        let options = SqliteConnectOptions::from_str(database_url)?
            // WAL：读写不互相阻塞，是多任务并发访问 SQLite 的前提。
            .journal_mode(SqliteJournalMode::Wal)
            // NORMAL 在 WAL 下兼顾安全与吞吐（断电时最多丢最近一次事务）。
            .synchronous(SqliteSynchronous::Normal)
            // 外键约束默认关闭，显式开启以免声明的外键形同虚设。
            .foreign_keys(true)
            .busy_timeout(SQLITE_BUSY_TIMEOUT)
            .create_if_missing(true);

        SqlitePoolOptions::new()
            .max_connections(SQLITE_MAX_CONNECTIONS)
            .acquire_timeout(SQLITE_ACQUIRE_TIMEOUT)
            .idle_timeout(SQLITE_IDLE_TIMEOUT)
            .connect_with(options)
            .await
            .map_err(StorageError::Database)
    }

    /// 确保 schema 就绪：先建表，再补齐增量列。
    ///
    /// 不使用进程级 `OnceCell`，因为它会让“第二个不同路径的数据库”静默不建表。
    /// 代价是重复建表（全部 `CREATE TABLE IF NOT EXISTS`），但那在 SQLite 上
    /// 是幂等的、耗时可忽略，换来的是每个库都可靠。
    async fn ensure_schema(pool: &SqlitePool) -> Result<(), StorageError> {
        Self::create_tables(pool).await?;
        write_schema_version(pool).await
    }

    async fn create_tables(pool: &SqlitePool) -> Result<(), StorageError> {
        // 应用配置表
        sqlx::query(
            r#"
            CREATE TABLE IF NOT EXISTS app_config (
                id INTEGER PRIMARY KEY,
                auto_start_kernel BOOLEAN DEFAULT FALSE,
                auto_start_app BOOLEAN DEFAULT FALSE,
                auto_hide_to_tray_on_autostart BOOLEAN DEFAULT TRUE,
                tray_close_behavior TEXT DEFAULT 'hide',
                prefer_ipv6 BOOLEAN DEFAULT FALSE,
                allow_lan_access BOOLEAN DEFAULT FALSE,
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
                tun_route_exclude_address TEXT,
                active_config_path TEXT,
                installed_kernel_version TEXT,
                singbox_dns_proxy TEXT DEFAULT 'https://1.1.1.1/dns-query',
                singbox_dns_cn TEXT DEFAULT 'https://dns.alidns.com/dns-query',
                singbox_dns_resolver TEXT DEFAULT '114.114.114.114',
                singbox_urltest_url TEXT DEFAULT 'http://cp.cloudflare.com/generate_204',
                singbox_default_proxy_outbound TEXT DEFAULT 'manual',
                singbox_block_ads BOOLEAN DEFAULT TRUE,
                singbox_download_detour TEXT DEFAULT 'direct',
                singbox_dns_hijack BOOLEAN DEFAULT TRUE,
                singbox_fake_dns_enabled BOOLEAN DEFAULT FALSE,
                singbox_fake_dns_ipv4_range TEXT DEFAULT '198.18.0.0/15',
                singbox_fake_dns_ipv6_range TEXT DEFAULT 'fc00::/18',
                singbox_fake_dns_filter_mode TEXT DEFAULT 'proxy_only',
                singbox_enable_app_groups BOOLEAN DEFAULT TRUE,
                tun_self_heal_enabled BOOLEAN DEFAULT TRUE,
                tun_self_heal_cooldown_secs INTEGER DEFAULT 90,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        // 旧库升级兼容：补齐缺失列。列已存在则跳过（显式查表，而非靠错误字符串判断）。
        ensure_columns(
            pool,
            "app_config",
            &[
                ("auto_start_app", "BOOLEAN DEFAULT FALSE"),
                ("auto_hide_to_tray_on_autostart", "BOOLEAN DEFAULT TRUE"),
                ("tray_close_behavior", "TEXT DEFAULT 'hide'"),
                ("allow_lan_access", "BOOLEAN DEFAULT FALSE"),
                ("system_proxy_bypass", "TEXT DEFAULT 'localhost;127.*;10.*;172.16.*;172.17.*;172.18.*;172.19.*;172.20.*;172.21.*;172.22.*;172.23.*;172.24.*;172.25.*;172.26.*;172.27.*;172.28.*;172.29.*;172.30.*;172.31.*;192.168.*'"),
                ("tun_auto_route", "BOOLEAN DEFAULT TRUE"),
                ("tun_strict_route", "BOOLEAN DEFAULT TRUE"),
                ("tun_mtu", "INTEGER DEFAULT 1500"),
                ("tun_ipv4", "TEXT DEFAULT '172.19.0.1/30'"),
                ("tun_ipv6", "TEXT DEFAULT 'fdfe:dcba:9876::1/126'"),
                ("tun_stack", "TEXT DEFAULT 'mixed'"),
                ("tun_enable_ipv6", "BOOLEAN DEFAULT FALSE"),
                ("tun_route_exclude_address", "TEXT"),
                ("system_proxy_enabled", "BOOLEAN DEFAULT FALSE"),
                ("tun_enabled", "BOOLEAN DEFAULT FALSE"),
                ("active_config_path", "TEXT"),
                ("installed_kernel_version", "TEXT"),
                ("singbox_dns_proxy", "TEXT DEFAULT 'https://1.1.1.1/dns-query'"),
                ("singbox_dns_cn", "TEXT DEFAULT 'https://dns.alidns.com/dns-query'"),
                ("singbox_dns_resolver", "TEXT DEFAULT '114.114.114.114'"),
                ("singbox_urltest_url", "TEXT DEFAULT 'http://cp.cloudflare.com/generate_204'"),
                ("singbox_default_proxy_outbound", "TEXT DEFAULT 'manual'"),
                ("singbox_block_ads", "BOOLEAN DEFAULT TRUE"),
                ("singbox_download_detour", "TEXT DEFAULT 'direct'"),
                ("singbox_dns_hijack", "BOOLEAN DEFAULT TRUE"),
                ("singbox_fake_dns_enabled", "BOOLEAN DEFAULT FALSE"),
                ("singbox_fake_dns_ipv4_range", "TEXT DEFAULT '198.18.0.0/15'"),
                ("singbox_fake_dns_ipv6_range", "TEXT DEFAULT 'fc00::/18'"),
                ("singbox_fake_dns_filter_mode", "TEXT DEFAULT 'proxy_only'"),
                ("singbox_enable_app_groups", "BOOLEAN DEFAULT TRUE"),
                ("tun_self_heal_enabled", "BOOLEAN DEFAULT TRUE"),
                ("tun_self_heal_cooldown_secs", "INTEGER DEFAULT 90"),
                // === sing-box 1.14 升级新增字段 ===
                ("kernel_update_track", "TEXT DEFAULT 'stable'"),
                ("singbox_dns_optimistic_cache", "BOOLEAN DEFAULT FALSE"),
                ("singbox_dns_optimistic_timeout", "TEXT DEFAULT '30m'"),
                ("singbox_dns_timeout", "TEXT DEFAULT '10s'"),
                ("singbox_dns_final_strategy", "TEXT DEFAULT 'cn'"),
                ("singbox_dns_use_mdns", "BOOLEAN DEFAULT TRUE"),
                ("singbox_enable_tls_spoof", "BOOLEAN DEFAULT FALSE"),
                ("tun_dns_mode", "TEXT DEFAULT 'hijack'"),
                ("tun_include_macs", "TEXT DEFAULT '[]'"),
                ("tun_exclude_macs", "TEXT DEFAULT '[]'"),
                ("hysteria2_disable_chrome_parrot", "BOOLEAN DEFAULT FALSE"),
                ("hysteria2_obfs_type", "TEXT DEFAULT 'salamander'"),
                ("clash_mode", "TEXT DEFAULT 'rule'"),
                ("enable_web_dashboard", "BOOLEAN DEFAULT FALSE"),
                ("enable_tailscale_endpoint", "BOOLEAN DEFAULT FALSE"),
                ("tailscale_run_ssh_server", "BOOLEAN DEFAULT FALSE"),
                ("tailscale_taildrop_directory", "TEXT DEFAULT 'Taildrop'"),
                // 1.13 早期 schema 可能缺失的基础列
                ("proxy_mode", "TEXT DEFAULT 'manual'"),
                ("created_at", "DATETIME DEFAULT CURRENT_TIMESTAMP"),
                ("updated_at", "DATETIME DEFAULT CURRENT_TIMESTAMP"),
            ],
        )
        .await?;

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
        ensure_columns(
            pool,
            "theme_config",
            &[
                ("mode", "TEXT DEFAULT 'system'"),
                ("accent_color", "TEXT DEFAULT '#6366f1'"),
                ("compact_mode", "BOOLEAN DEFAULT FALSE"),
            ],
        )
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
                accept_prerelease BOOLEAN DEFAULT FALSE,
                update_channel TEXT DEFAULT 'stable',
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                updated_at DATETIME DEFAULT CURRENT_TIMESTAMP
            )
            "#,
        )
        .execute(pool)
        .await?;

        // 更新配置兼容字段补充（确保旧表增加新增字段）
        ensure_columns(
            pool,
            "update_config",
            &[
                ("accept_prerelease", "BOOLEAN DEFAULT FALSE"),
                ("update_channel", "TEXT DEFAULT 'stable'"),
            ],
        )
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
        let row = sqlx::query("SELECT * FROM app_config WHERE id = 1")
            .fetch_optional(&self.pool)
            .await?;

        if let Some(row) = row {
            let default_config = AppConfig::default();
            Ok(Some(AppConfig {
                auto_start_kernel: row.get("auto_start_kernel"),
                auto_start_app: row.get("auto_start_app"),
                auto_hide_to_tray_on_autostart: row
                    .try_get("auto_hide_to_tray_on_autostart")
                    .unwrap_or(default_config.auto_hide_to_tray_on_autostart),
                tray_close_behavior: row
                    .try_get("tray_close_behavior")
                    .unwrap_or_else(|_| default_config.tray_close_behavior.clone()),
                prefer_ipv6: row.get("prefer_ipv6"),
                allow_lan_access: row
                    .try_get("allow_lan_access")
                    .unwrap_or(default_config.allow_lan_access),
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
                tun_route_exclude_address: parse_tun_route_exclude_address_column(
                    row.try_get("tun_route_exclude_address").unwrap_or(None),
                ),
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
                singbox_fake_dns_enabled: row
                    .try_get("singbox_fake_dns_enabled")
                    .unwrap_or(default_config.singbox_fake_dns_enabled),
                singbox_fake_dns_ipv4_range: row
                    .try_get("singbox_fake_dns_ipv4_range")
                    .unwrap_or_else(|_| default_config.singbox_fake_dns_ipv4_range.clone()),
                singbox_fake_dns_ipv6_range: row
                    .try_get("singbox_fake_dns_ipv6_range")
                    .unwrap_or_else(|_| default_config.singbox_fake_dns_ipv6_range.clone()),
                singbox_fake_dns_filter_mode: row
                    .try_get("singbox_fake_dns_filter_mode")
                    .unwrap_or_else(|_| default_config.singbox_fake_dns_filter_mode.clone()),
                singbox_enable_app_groups: row
                    .try_get("singbox_enable_app_groups")
                    .unwrap_or(default_config.singbox_enable_app_groups),
                tun_self_heal_enabled: row
                    .try_get("tun_self_heal_enabled")
                    .unwrap_or(default_config.tun_self_heal_enabled),
                tun_self_heal_cooldown_secs: row
                    .try_get("tun_self_heal_cooldown_secs")
                    .unwrap_or(default_config.tun_self_heal_cooldown_secs),
                // === sing-box 1.14 升级新增字段（SQLite 老库无列，使用 try_get + default 兜底） ===
                kernel_update_track: row
                    .try_get("kernel_update_track")
                    .unwrap_or_else(|_| default_config.kernel_update_track.clone()),
                singbox_dns_optimistic_cache: row
                    .try_get("singbox_dns_optimistic_cache")
                    .unwrap_or(default_config.singbox_dns_optimistic_cache),
                singbox_dns_optimistic_timeout: row
                    .try_get("singbox_dns_optimistic_timeout")
                    .unwrap_or_else(|_| default_config.singbox_dns_optimistic_timeout.clone()),
                singbox_dns_timeout: row
                    .try_get("singbox_dns_timeout")
                    .unwrap_or_else(|_| default_config.singbox_dns_timeout.clone()),
                singbox_dns_final_strategy: row
                    .try_get("singbox_dns_final_strategy")
                    .unwrap_or_else(|_| default_config.singbox_dns_final_strategy.clone()),
                singbox_dns_use_mdns: row
                    .try_get("singbox_dns_use_mdns")
                    .unwrap_or(default_config.singbox_dns_use_mdns),
                singbox_enable_tls_spoof: row
                    .try_get("singbox_enable_tls_spoof")
                    .unwrap_or(default_config.singbox_enable_tls_spoof),
                tun_dns_mode: row
                    .try_get("tun_dns_mode")
                    .unwrap_or_else(|_| default_config.tun_dns_mode.clone()),
                tun_include_macs: parse_string_array_column(
                    row.try_get("tun_include_macs").unwrap_or(None),
                ),
                tun_exclude_macs: parse_string_array_column(
                    row.try_get("tun_exclude_macs").unwrap_or(None),
                ),
                hysteria2_disable_chrome_parrot: row
                    .try_get("hysteria2_disable_chrome_parrot")
                    .unwrap_or(default_config.hysteria2_disable_chrome_parrot),
                hysteria2_obfs_type: row
                    .try_get("hysteria2_obfs_type")
                    .unwrap_or_else(|_| default_config.hysteria2_obfs_type.clone()),
                clash_mode: row
                    .try_get("clash_mode")
                    .unwrap_or_else(|_| default_config.clash_mode.clone()),
                // === sing-box 1.14 升级新增字段（Web Dashboard + Tailscale） ===
                enable_web_dashboard: row
                    .try_get("enable_web_dashboard")
                    .unwrap_or(default_config.enable_web_dashboard),
                enable_tailscale_endpoint: row
                    .try_get("enable_tailscale_endpoint")
                    .unwrap_or(default_config.enable_tailscale_endpoint),
                tailscale_run_ssh_server: row
                    .try_get("tailscale_run_ssh_server")
                    .unwrap_or(default_config.tailscale_run_ssh_server),
                tailscale_taildrop_directory: row
                    .try_get("tailscale_taildrop_directory")
                    .unwrap_or_else(|_| default_config.tailscale_taildrop_directory.clone()),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_app_config(&self, config: &AppConfig) -> Result<(), StorageError> {
        // 注意：列顺序必须与 db 实际 schema 一致（同事在 1.13 → 1.14 升级时
        // 把 tun_route_exclude_address 移到 tun_self_heal_cooldown_secs 之后，
        // 同时在中间插入了 created_at / updated_at 时间戳列）。本 INSERT 严格按
        // PRAGMA table_info(app_config) 的实际顺序排列。
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO app_config
            (id, auto_start_kernel, auto_start_app, auto_hide_to_tray_on_autostart, tray_close_behavior, prefer_ipv6, allow_lan_access, proxy_port, api_port, proxy_mode, system_proxy_enabled, tun_enabled, tray_instance_id, system_proxy_bypass, tun_auto_route, tun_strict_route, tun_mtu, tun_ipv4, tun_ipv6, tun_stack, tun_enable_ipv6, active_config_path, installed_kernel_version, singbox_dns_proxy, singbox_dns_cn, singbox_dns_resolver, singbox_urltest_url, singbox_default_proxy_outbound, singbox_block_ads, singbox_download_detour, singbox_dns_hijack, singbox_fake_dns_enabled, singbox_fake_dns_ipv4_range, singbox_fake_dns_ipv6_range, singbox_fake_dns_filter_mode, singbox_enable_app_groups, tun_self_heal_enabled, tun_self_heal_cooldown_secs, created_at, updated_at, tun_route_exclude_address, kernel_update_track, singbox_dns_optimistic_cache, singbox_dns_optimistic_timeout, singbox_dns_timeout, singbox_dns_final_strategy, singbox_dns_use_mdns, singbox_enable_tls_spoof, tun_dns_mode, tun_include_macs, tun_exclude_macs, hysteria2_disable_chrome_parrot, hysteria2_obfs_type, clash_mode, enable_web_dashboard, enable_tailscale_endpoint, tailscale_run_ssh_server, tailscale_taildrop_directory)
            VALUES (1, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(config.auto_start_kernel)
        .bind(config.auto_start_app)
        .bind(config.auto_hide_to_tray_on_autostart)
        .bind(&config.tray_close_behavior)
        .bind(config.prefer_ipv6)
        .bind(config.allow_lan_access)
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
        .bind(config.singbox_fake_dns_enabled)
        .bind(&config.singbox_fake_dns_ipv4_range)
        .bind(&config.singbox_fake_dns_ipv6_range)
        .bind(&config.singbox_fake_dns_filter_mode)
        .bind(config.singbox_enable_app_groups)
        .bind(config.tun_self_heal_enabled)
        .bind(config.tun_self_heal_cooldown_secs)
        // === created_at / updated_at 时间戳 ===
        .bind(Utc::now())
        .bind(Utc::now())
        // === tun_route_exclude_address 位置在 updated_at 之后 ===
        .bind(serialize_optional_json(&config.tun_route_exclude_address)?)
        // === sing-box 1.14 升级新增字段（与 PRAGMA 顺序一致） ===
        .bind(&config.kernel_update_track)
        .bind(config.singbox_dns_optimistic_cache)
        .bind(&config.singbox_dns_optimistic_timeout)
        .bind(&config.singbox_dns_timeout)
        .bind(&config.singbox_dns_final_strategy)
        .bind(config.singbox_dns_use_mdns)
        .bind(config.singbox_enable_tls_spoof)
        .bind(&config.tun_dns_mode)
        .bind(serialize_string_vec(&config.tun_include_macs))
        .bind(serialize_string_vec(&config.tun_exclude_macs))
        .bind(config.hysteria2_disable_chrome_parrot)
        .bind(&config.hysteria2_obfs_type)
        .bind(&config.clash_mode)
        .bind(config.enable_web_dashboard)
        .bind(config.enable_tailscale_endpoint)
        .bind(config.tailscale_run_ssh_server)
        .bind(&config.tailscale_taildrop_directory)
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
                update_channel: row
                    .try_get("update_channel")
                    .unwrap_or(default_config.update_channel),
            }))
        } else {
            Ok(None)
        }
    }

    pub async fn save_update_config(&self, config: &UpdateConfig) -> Result<(), StorageError> {
        sqlx::query(
            r#"
            INSERT OR REPLACE INTO update_config 
            (id, auto_check, last_check, last_version, skip_version, accept_prerelease, update_channel, updated_at)
            VALUES (1, ?, ?, ?, ?, ?, ?, ?)
            "#,
        )
        .bind(config.auto_check)
        .bind(config.last_check)
        .bind(&config.last_version)
        .bind(&config.skip_version)
        .bind(config.accept_prerelease)
        .bind(&config.update_channel)
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

fn serialize_optional_json<T>(value: &Option<T>) -> Result<Option<String>, StorageError>
where
    T: Serialize,
{
    value
        .as_ref()
        .map(serde_json::to_string)
        .transpose()
        .map_err(StorageError::Serialization)
}

fn parse_tun_route_exclude_address_column(raw: Option<String>) -> Option<Vec<String>> {
    let raw = raw?;

    if raw.trim().is_empty() {
        return None;
    }

    match serde_json::from_str::<Vec<String>>(&raw) {
        Ok(values) => normalize_persisted_tun_route_exclude_address(Some(values)),
        Err(error) => {
            tracing::warn!(
                "检测到无效的已持久化 tun_route_exclude_address JSON，已回退为 None: {}",
                error
            );
            None
        }
    }
}

/// 通用字符串数组列解析（用于 1.14 新增的 `tun_include_macs` / `tun_exclude_macs`）。
/// 失败时回退为空 Vec 并 warn。
fn parse_string_array_column(raw: Option<String>) -> Vec<String> {
    let Some(raw) = raw else {
        return Vec::new();
    };
    if raw.trim().is_empty() {
        return Vec::new();
    }
    match serde_json::from_str::<Vec<String>>(&raw) {
        Ok(values) => values,
        Err(error) => {
            tracing::warn!("检测到无效的字符串数组 JSON，已回退为空数组: {}", error);
            Vec::new()
        }
    }
}

/// 把字符串数组序列化为 JSON 字符串写入 SQLite；空数组写 "[]"。
fn serialize_string_vec(values: &[String]) -> String {
    serde_json::to_string(values).unwrap_or_else(|_| "[]".to_string())
}

#[cfg(test)]
#[path = "database.tests.rs"]
mod tests;
