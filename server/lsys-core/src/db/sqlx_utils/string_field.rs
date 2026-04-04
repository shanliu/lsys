//! 字符串字段最大长度查询
//!
//! 通过数据库查询字符串字段的最大长度，并提供缓存支持（需启用 redis feature）
//!
//! 支持的数据库：
//! - MySQL（需启用 `db-mysql` feature）
//! - PostgreSQL（需启用 `db-postgres` feature）
//! - SQLite（需启用 `db-sqlite` feature）

use std::collections::HashMap;
use std::ops::Deref;

use super::super::sqlx::field::Field;
use super::super::sqlx::table::{TableMeta, TableName};
#[cfg(feature = "redis")]
use crate::remote_notify::RemoteNotify;
#[cfg(feature = "redis")]
use std::sync::Arc;

// ============== 错误与结果类型 ==============

/// 字符串字段长度查询错误
#[derive(Debug, Clone)]
pub enum FetchFieldError {
    /// 非字符串类型
    NotString,
    /// 字段不存在
    NotFound,
    /// 查询错误
    DbError(String),
}

impl std::fmt::Display for FetchFieldError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FetchFieldError::NotString => write!(f, "field is not a string type"),
            FetchFieldError::NotFound => write!(f, "field not found in table"),
            FetchFieldError::DbError(msg) => write!(f, "database error: {}", msg),
        }
    }
}

impl std::error::Error for FetchFieldError {}

/// 字符串字段最大长度结果
#[derive(Debug, Clone)]
pub struct FetchFieldStringMaxResult(Result<u64, FetchFieldError>);

impl FetchFieldStringMaxResult {
    /// LONGTEXT / TEXT（无限制）最大长度常量
    pub const LONGTEXT_MAX: u64 = 4_294_967_295;

    /// 创建成功结果
    pub fn ok(len: u64) -> Self {
        Self(Ok(len))
    }

    /// 创建错误结果
    pub fn err(e: FetchFieldError) -> Self {
        Self(Err(e))
    }

    /// 获取用于验证的最大长度值
    ///
    /// - `Ok(n)` -> 返回 n
    /// - `Err(_)` -> 返回 default
    pub fn len_or(&self, default: u64) -> u64 {
        match &self.0 {
            Ok(len) => *len,
            Err(_) => default,
        }
    }
}

// 通过 Deref 可以直接使用 Result 的方法，如 is_ok(), is_err(), unwrap() 等
impl Deref for FetchFieldStringMaxResult {
    type Target = Result<u64, FetchFieldError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Result<u64, FetchFieldError>> for FetchFieldStringMaxResult {
    fn from(result: Result<u64, FetchFieldError>) -> Self {
        Self(result)
    }
}

/// 标记 trait：字符串类型字段
/// 用于编译时限制只有 String 类型的 Field 可以调用 string_max
pub trait FetchFieldStringType {}

impl FetchFieldStringType for String {}
impl FetchFieldStringType for Option<String> {}

// ============== 数据库适配 trait ==============

/// 数据库字符串字段查询适配
///
/// 为不同数据库提供统一的字符串字段最大长度查询接口。
/// 每种数据库通过对 `Pool<DB>` 实现此 trait 来提供各自的查询逻辑。
///
/// 已支持：
/// - `Pool<MySql>` —— `SHOW COLUMNS FROM`（需 `db-mysql` feature）
/// - `Pool<Postgres>` —— `information_schema.columns`（需 `db-postgres` feature）
/// - `Pool<Sqlite>` —— `PRAGMA table_info`（需 `db-sqlite` feature）
#[async_trait::async_trait]
pub trait FetchFieldStringQuery: Sync + Send {
    /// 查询单个字段的最大长度
    async fn query_single_field_max(
        &self,
        table: &TableName,
        column: &str,
    ) -> FetchFieldStringMaxResult;

    /// 查询表的所有字符串字段并返回映射（字段名 -> 最大长度）
    async fn query_all_string_fields(
        &self,
        table: &TableName,
    ) -> Result<HashMap<String, u64>, FetchFieldError>;
}

// ============== 缓存实现（仅 redis feature 启用时） ==============

#[cfg(feature = "redis")]
mod cache {
    use super::*;
    use crate::cache::{LocalCache, LocalCacheConfig};
    use std::collections::HashSet;
    use tokio::sync::{Mutex, OnceCell};

    /// 表字段类型映射：字段名 -> 最大长度
    pub type TableFieldsMap = HashMap<String, u64>;

    /// 全局缓存实例：缓存 key -> 所有字符串字段的长度映射
    /// 缓存 key 格式：{pool_addr}:{db_type}:{table_name}
    static STRING_FIELD_CACHE: OnceCell<LocalCache<String, TableFieldsMap>> = OnceCell::const_new();

    /// 已初始化缓存的 Pool 地址集合
    /// 只有在这个集合中的 Pool 才会使用缓存
    static INITED_POOLS: OnceCell<Mutex<HashSet<usize>>> = OnceCell::const_new();

    /// 初始化全局缓存实例
    ///
    /// 应在应用启动时调用一次，初始化缓存基础设施。
    /// 之后每个需要缓存的 Pool 需要单独调用 `init_pool_cache`。
    ///
    /// # Arguments
    /// * `remote_notify` - 远程通知实例
    /// * `use_cache` - 是否启用缓存，false 时缓存大小设为 0
    #[allow(unused)]
    pub async fn init_cache(remote_notify: Arc<RemoteNotify>, use_cache: bool) {
        STRING_FIELD_CACHE
            .get_or_init(|| async {
                LocalCache::new(
                    remote_notify,
                    LocalCacheConfig::new(
                        "string_field_max",
                        if use_cache { Some(500) } else { Some(0) },
                        Some(3600),
                    ),
                )
            })
            .await;

        // 初始化已 init 的 Pool 集合
        INITED_POOLS.get_or_init(|| async { Mutex::new(HashSet::new()) }).await;
    }

    /// 为特定 Pool 初始化缓存
    ///
    /// 每个 Pool 需要单独调用此方法才能启用缓存。
    /// 未调用此方法的 Pool 不会使用缓存，每次查询都会直接访问数据库。
    ///
    /// # Arguments
    /// * `pool_addr` - Pool 的内存地址（通过 `pool as *const _ as usize` 获取）
    ///
    /// # 示例
    /// ```ignore
    /// // 初始化全局缓存（应用启动时调用一次）
    /// FetchField::<sqlx::MySql>::init_cache(remote_notify, true).await;
    ///
    /// // 为特定 Pool 启用缓存
    /// FetchField::init_pool_cache(&pool).await;
    /// ```
    pub async fn init_pool_cache(pool_addr: usize) {
        let inited_pools = INITED_POOLS.get_or_init(|| async { Mutex::new(HashSet::new()) }).await;
        inited_pools.lock().await.insert(pool_addr);
    }

    /// 检查 Pool 是否已初始化缓存
    pub async fn is_pool_inited(pool_addr: usize) -> bool {
        let inited_pools = INITED_POOLS.get_or_init(|| async { Mutex::new(HashSet::new()) }).await;
        inited_pools.lock().await.contains(&pool_addr)
    }

    /// 生成缓存 key
    /// 格式：{pool_addr}:{db_type}:{table_name}
    pub(super) fn cache_key(pool_addr: usize, db_type: &str, table: &TableName) -> String {
        format!("{}:{}:{}", pool_addr, db_type, table.full_name())
    }

    /// 从缓存获取表的字段映射
    /// 只有已初始化的 Pool 才会使用缓存
    pub(super) async fn get_table(
        pool_addr: usize,
        db_type: &str,
        table: &TableName,
    ) -> Option<TableFieldsMap> {
        // 检查 Pool 是否已初始化
        if !is_pool_inited(pool_addr).await {
            return None;
        }

        if let Some(cache) = STRING_FIELD_CACHE.get() {
            let key = cache_key(pool_addr, db_type, table);
            cache.get(&key).await
        } else {
            None
        }
    }

    /// 缓存表的所有字符串字段映射
    /// 只有已初始化的 Pool 才会使用缓存
    pub(super) async fn set_table(
        pool_addr: usize,
        db_type: &str,
        table: &TableName,
        fields: TableFieldsMap,
    ) {
        // 检查 Pool 是否已初始化
        if !is_pool_inited(pool_addr).await {
            return;
        }

        if let Some(cache) = STRING_FIELD_CACHE.get() {
            let key = cache_key(pool_addr, db_type, table);
            cache.set(key, fields, 0).await;
        }
    }
}

// ============== MySQL 类型解析与实现 ==============

/// 解析 MySQL 类型字符串，提取最大长度
///
/// 支持的类型：
/// - char(n), varchar(n) -> n
/// - tinytext -> 255
/// - text -> 65535
/// - mediumtext -> 16777215
/// - longtext -> 4294967295
#[cfg(feature = "db-mysql")]
fn parse_mysql_type_max_length(type_str: &str) -> Option<u64> {
    let type_lower = type_str.to_lowercase();

    // varchar(n) 或 char(n)
    if type_lower.starts_with("varchar") || type_lower.starts_with("char") {
        if let Some(start) = type_lower.find('(')
            && let Some(end) = type_lower.find(')')
            && let Ok(len) = type_lower[start + 1..end].parse::<u64>()
        {
            return Some(len);
        }
        return None;
    }

    // text 类型
    match type_lower.as_str() {
        "tinytext" => Some(255),
        "text" => Some(65535),
        "mediumtext" => Some(16_777_215),
        "longtext" => Some(FetchFieldStringMaxResult::LONGTEXT_MAX),
        _ => None,
    }
}

#[cfg(feature = "db-mysql")]
#[async_trait::async_trait]
impl FetchFieldStringQuery for sqlx::Pool<sqlx::MySql> {
    async fn query_single_field_max(
        &self,
        table: &TableName,
        column: &str,
    ) -> FetchFieldStringMaxResult {
        use sqlx::Row;

        let db_part = table.db_part();
        let db_name = db_part.trim_end_matches('.');
        let sql = if db_name.is_empty() {
            format!("SHOW COLUMNS FROM `{}` LIKE '{}'", table.raw_name(), column)
        } else {
            format!(
                "SHOW COLUMNS FROM `{}`.`{}` LIKE '{}'",
                db_name,
                table.raw_name(),
                column
            )
        };

        match sqlx::query(&sql).fetch_optional(self).await {
            Ok(Some(row)) => {
                let type_str: String = row.try_get("Type").unwrap_or_default();
                match parse_mysql_type_max_length(&type_str) {
                    Some(len) => FetchFieldStringMaxResult::ok(len),
                    None => FetchFieldStringMaxResult::err(FetchFieldError::NotString),
                }
            }
            Ok(None) => FetchFieldStringMaxResult::err(FetchFieldError::NotFound),
            Err(e) => FetchFieldStringMaxResult::err(FetchFieldError::DbError(e.to_string())),
        }
    }

    async fn query_all_string_fields(
        &self,
        table: &TableName,
    ) -> Result<HashMap<String, u64>, FetchFieldError> {
        use sqlx::Row;

        let db_part = table.db_part();
        let db_name = db_part.trim_end_matches('.');
        let sql = if db_name.is_empty() {
            format!("SHOW COLUMNS FROM `{}`", table.raw_name())
        } else {
            format!("SHOW COLUMNS FROM `{}`.`{}`", db_name, table.raw_name())
        };

        match sqlx::query(&sql).fetch_all(self).await {
            Ok(rows) => {
                let mut fields_map = HashMap::new();
                for row in rows {
                    let field_name: String = row.try_get("Field").unwrap_or_default();
                    let type_str: String = row.try_get("Type").unwrap_or_default();
                    if let Some(len) = parse_mysql_type_max_length(&type_str) {
                        fields_map.insert(field_name, len);
                    }
                }
                Ok(fields_map)
            }
            Err(e) => Err(FetchFieldError::DbError(e.to_string())),
        }
    }
}

// ============== PostgreSQL 类型解析与实现 ==============

/// 从 TableName 中提取 PostgreSQL schema 名称
///
/// - `db_part()` 为空时，默认使用 "public"
/// - 否则去除尾部的 '.' 作为 schema 名
#[cfg(feature = "db-postgres")]
fn extract_pg_schema(table: &TableName) -> String {
    let db_part = table.db_part();
    if db_part.is_empty() {
        "public".to_string()
    } else {
        db_part.trim_end_matches('.').to_string()
    }
}

/// 解析 PostgreSQL 类型信息，提取最大长度
///
/// 支持的类型：
/// - character varying(n) / varchar(n) -> n
/// - character(n) / char(n) -> n
/// - character varying / varchar（无长度） -> LONGTEXT_MAX
/// - text -> LONGTEXT_MAX
#[cfg(feature = "db-postgres")]
fn parse_postgres_type_max_length(data_type: &str, char_max_len: Option<i32>) -> Option<u64> {
    match data_type.to_lowercase().as_str() {
        "character varying" | "character" | "char" | "varchar" => Some(
            char_max_len
                .map(|l| l as u64)
                .unwrap_or(FetchFieldStringMaxResult::LONGTEXT_MAX),
        ),
        "text" => Some(FetchFieldStringMaxResult::LONGTEXT_MAX),
        _ => None,
    }
}

#[cfg(feature = "db-postgres")]
#[async_trait::async_trait]
impl FetchFieldStringQuery for sqlx::Pool<sqlx::Postgres> {
    async fn query_single_field_max(
        &self,
        table: &TableName,
        column: &str,
    ) -> FetchFieldStringMaxResult {
        use sqlx::Row;

        let schema = extract_pg_schema(table);
        let sql = "SELECT data_type, character_maximum_length \
                    FROM information_schema.columns \
                    WHERE table_schema = $1 AND table_name = $2 AND column_name = $3";

        match sqlx::query(sql)
            .bind(&schema)
            .bind(table.raw_name())
            .bind(column)
            .fetch_optional(self)
            .await
        {
            Ok(Some(row)) => {
                let data_type: String = row.try_get("data_type").unwrap_or_default();
                let char_max_len: Option<i32> =
                    row.try_get("character_maximum_length").unwrap_or(None);
                match parse_postgres_type_max_length(&data_type, char_max_len) {
                    Some(len) => FetchFieldStringMaxResult::ok(len),
                    None => FetchFieldStringMaxResult::err(FetchFieldError::NotString),
                }
            }
            Ok(None) => FetchFieldStringMaxResult::err(FetchFieldError::NotFound),
            Err(e) => FetchFieldStringMaxResult::err(FetchFieldError::DbError(e.to_string())),
        }
    }

    async fn query_all_string_fields(
        &self,
        table: &TableName,
    ) -> Result<HashMap<String, u64>, FetchFieldError> {
        use sqlx::Row;

        let schema = extract_pg_schema(table);
        let sql = "SELECT column_name, data_type, character_maximum_length \
                    FROM information_schema.columns \
                    WHERE table_schema = $1 AND table_name = $2";

        match sqlx::query(sql)
            .bind(&schema)
            .bind(table.raw_name())
            .fetch_all(self)
            .await
        {
            Ok(rows) => {
                let mut fields_map = HashMap::new();
                for row in rows {
                    let col_name: String = row.try_get("column_name").unwrap_or_default();
                    let data_type: String = row.try_get("data_type").unwrap_or_default();
                    let char_max_len: Option<i32> =
                        row.try_get("character_maximum_length").unwrap_or(None);
                    if let Some(len) = parse_postgres_type_max_length(&data_type, char_max_len) {
                        fields_map.insert(col_name, len);
                    }
                }
                Ok(fields_map)
            }
            Err(e) => Err(FetchFieldError::DbError(e.to_string())),
        }
    }
}

// ============== SQLite 类型解析与实现 ==============

/// 解析 SQLite 类型字符串，提取最大长度
///
/// SQLite 使用类型亲和性（type affinity），本身不强制长度约束，
/// 但声明类型中的长度信息仍可用于应用层验证。
///
/// 支持的类型：
/// - varchar(n), char(n), character(n), nchar(n), nvarchar(n) -> n
/// - 上述类型无长度声明 -> LONGTEXT_MAX
/// - text, clob -> LONGTEXT_MAX
#[cfg(feature = "db-sqlite")]
fn parse_sqlite_type_max_length(type_str: &str) -> Option<u64> {
    let type_lower = type_str.to_lowercase();
    let type_trimmed = type_lower.trim();

    // varchar(n), char(n), character(n), nchar(n), nvarchar(n), character varying(n)
    if type_trimmed.starts_with("varchar")
        || type_trimmed.starts_with("character")
        || type_trimmed.starts_with("char")
        || type_trimmed.starts_with("nchar")
        || type_trimmed.starts_with("nvarchar")
    {
        if let Some(start) = type_trimmed.find('(') {
            if let Some(end) = type_trimmed.find(')') {
                if let Ok(len) = type_trimmed[start + 1..end].trim().parse::<u64>() {
                    return Some(len);
                }
            }
        }
        // 无长度声明，视为无限制
        return Some(FetchFieldStringMaxResult::LONGTEXT_MAX);
    }

    // text, clob
    if type_trimmed == "text" || type_trimmed == "clob" {
        return Some(FetchFieldStringMaxResult::LONGTEXT_MAX);
    }

    None
}

#[cfg(feature = "db-sqlite")]
#[async_trait::async_trait]
impl FetchFieldStringQuery for sqlx::Pool<sqlx::Sqlite> {
    async fn query_single_field_max(
        &self,
        table: &TableName,
        column: &str,
    ) -> FetchFieldStringMaxResult {
        use sqlx::Row;

        // SQLite 使用 PRAGMA table_info 查询列信息
        let db_part = table.db_part();
        let db_name = db_part.trim_end_matches('.');
        let sql = if db_name.is_empty() {
            format!("PRAGMA table_info(\"{}\")", table.raw_name())
        } else {
            format!(
                "PRAGMA \"{}\".table_info(\"{}\")",
                db_name,
                table.raw_name()
            )
        };

        match sqlx::query(&sql).fetch_all(self).await {
            Ok(rows) => {
                for row in rows {
                    let name: String = row.try_get("name").unwrap_or_default();
                    if name == column {
                        let type_str: String = row.try_get("type").unwrap_or_default();
                        return match parse_sqlite_type_max_length(&type_str) {
                            Some(len) => FetchFieldStringMaxResult::ok(len),
                            None => FetchFieldStringMaxResult::err(FetchFieldError::NotString),
                        };
                    }
                }
                FetchFieldStringMaxResult::err(FetchFieldError::NotFound)
            }
            Err(e) => FetchFieldStringMaxResult::err(FetchFieldError::DbError(e.to_string())),
        }
    }

    async fn query_all_string_fields(
        &self,
        table: &TableName,
    ) -> Result<HashMap<String, u64>, FetchFieldError> {
        use sqlx::Row;

        let db_part = table.db_part();
        let db_name = db_part.trim_end_matches('.');
        let sql = if db_name.is_empty() {
            format!("PRAGMA table_info(\"{}\")", table.raw_name())
        } else {
            format!(
                "PRAGMA \"{}\".table_info(\"{}\")",
                db_name,
                table.raw_name()
            )
        };

        match sqlx::query(&sql).fetch_all(self).await {
            Ok(rows) => {
                let mut fields_map = HashMap::new();
                for row in rows {
                    let name: String = row.try_get("name").unwrap_or_default();
                    let type_str: String = row.try_get("type").unwrap_or_default();
                    if let Some(len) = parse_sqlite_type_max_length(&type_str) {
                        fields_map.insert(name, len);
                    }
                }
                Ok(fields_map)
            }
            Err(e) => Err(FetchFieldError::DbError(e.to_string())),
        }
    }
}

// ============== 数据库类型前缀 ==============

use super::super::sqlx::db_type::{is_mysql_db, is_postgres_db, is_sqlite_db};
use sqlx::Database;

/// 获取数据库类型前缀（用于缓存 key）
fn get_db_type_prefix<DB: Database>() -> &'static str {
    if is_mysql_db::<DB>() {
        "mysql"
    } else if is_postgres_db::<DB>() {
        "postgres"
    } else if is_sqlite_db::<DB>() {
        "sqlite"
    } else {
        "unknown"
    }
}

// ============== FetchField 结构体 ==============

/// 字段信息查询工具
///
/// 提供字符串字段最大长度查询功能，支持 MySQL、PostgreSQL、SQLite。
/// 缓存键使用 Pool 的内存地址作为唯一标识，自动区分不同的数据库连接实例，
/// 无需用户手动指定命名空间。
///
/// # 地址稳定性说明
///
/// 缓存键使用 `Pool` 引用的内存地址作为唯一标识。为确保缓存键稳定：
///
/// **推荐实践**：
/// - 使用 `Arc<Pool<DB>>` 存储 Pool，确保 Pool 地址在整个应用生命周期内稳定
/// - 在应用启动时创建 Pool，并长期持有引用
/// - 从同一个 Pool 引用创建 FetchField 实例
///
/// **注意事项**：
/// - Pool 被 `clone()` 后，新 handle 有不同的地址，但共享内部连接池状态
/// - 如果需要在不同 handle 间共享缓存，应使用原始 Pool 引用
/// - Pool 被移动到新变量后，地址会变化（应避免这种情况）
///
/// # 示例
/// ```ignore
/// use std::sync::Arc;
/// use lsys_core::db::utils::FetchField;
///
/// // 初始化缓存（应用启动时调用一次）
/// FetchField::<sqlx::MySql>::init_cache(remote_notify, true).await;
///
/// // 推荐：使用 Arc 存储 Pool
/// let pool_arc = Arc::new(pool);
///
/// // 创建实例并查询
/// let fetch_field = FetchField::new(&*pool_arc);
/// let max_len1 = fetch_field.string_max::<UserModel>(&UserModel::NAME).await;
/// let max_len2 = fetch_field.string_max::<UserModel>(&UserModel::EMAIL).await;
/// ```
pub struct FetchField<'a, DB>
where
    DB: Database,
{
    pool: &'a sqlx::Pool<DB>,
}

impl<'a, DB> FetchField<'a, DB>
where
    DB: Database,
{
    /// 创建 FetchField 实例
    ///
    /// 接受 Pool 引用，直接持有而不克隆。
    /// 同一函数中多次查询时，建议只创建一次实例然后复用。
    ///
    /// **缓存说明**：
    /// - 只有调用过 `init_pool_cache` 的 Pool 才会使用缓存
    /// - 未初始化缓存的 Pool 每次查询都会直接访问数据库
    /// - 不同的 Pool 实例不会产生缓存冲突
    pub fn new(pool: &'a sqlx::Pool<DB>) -> Self {
        Self { pool }
    }

    /// 获取数据库类型前缀
    fn db_prefix() -> &'static str {
        get_db_type_prefix::<DB>()
    }

    /// 获取 Pool 的内存地址
    fn pool_addr(&self) -> usize {
        self.pool as *const _ as usize
    }
}

impl<'a, DB> FetchField<'a, DB>
where
    DB: Database,
    sqlx::Pool<DB>: FetchFieldStringQuery,
{
    /// 查询字符串字段的最大长度
    ///
    /// # 类型参数
    /// - `M`: 表元信息，提供表名
    ///
    /// # 参数
    /// - `field`: 字符串类型的字段标识符
    ///
    /// # 缓存行为
    /// - 启用 `redis` feature 且调用了 `init_cache` 时：查询整表并缓存
    /// - 未启用时：仅查询指定字段
    ///
    /// # 示例
    /// ```ignore
    /// // 创建一次实例，多次查询
    /// let fetch_field = FetchField::new(&pool);
    /// let max_len1 = fetch_field.string_max::<UserModel>(&UserModel::NAME).await;
    /// let max_len2 = fetch_field.string_max::<UserModel>(&UserModel::EMAIL).await;
    /// ```
    pub async fn string_max<M: TableMeta>(
        &self,
        field: &Field<impl FetchFieldStringType>,
    ) -> FetchFieldStringMaxResult {
        let table = M::table_name();
        let column = field.column.as_ref();


        #[cfg(feature = "redis")]
        {
            let db_prefix = Self::db_prefix();
            self.query_with_cache(&table, column, db_prefix).await
        }

        #[cfg(not(feature = "redis"))]
        {
            self.pool.query_single_field_max(&table, column).await
        }
    }

    /// 带缓存的查询实现
    /// 只有已初始化缓存的 Pool 才会使用缓存
    #[cfg(feature = "redis")]
    async fn query_with_cache(
        &self,
        table: &TableName,
        column: &str,
        db_prefix: &str,
    ) -> FetchFieldStringMaxResult {
        let pool_addr = self.pool_addr();

        // 检查 Pool 是否已初始化缓存
        if !cache::is_pool_inited(pool_addr).await {
            // Pool 未初始化缓存，直接查询数据库
            return self.pool.query_single_field_max(table, column).await;
        }

        // 先查缓存
        if let Some(cached_fields) = cache::get_table(pool_addr, db_prefix, table).await {
            return match cached_fields.get(column) {
                Some(&len) => FetchFieldStringMaxResult::ok(len),
                None => FetchFieldStringMaxResult::err(FetchFieldError::NotString),
            };
        }

        // 查询整表并缓存
        let fields_map = match self.pool.query_all_string_fields(table).await {
            Ok(map) => map,
            Err(e) => return FetchFieldStringMaxResult::err(e),
        };
        let result = match fields_map.get(column) {
            Some(&len) => FetchFieldStringMaxResult::ok(len),
            None => FetchFieldStringMaxResult::err(FetchFieldError::NotString),
        };
        cache::set_table(pool_addr, db_prefix, table, fields_map).await;
        result
    }
}

#[cfg(feature = "redis")]
impl<DB: Database> FetchField<'_, DB> {
    /// 为特定 Pool 初始化缓存
    ///
    /// 每个 Pool 需要单独调用此方法才能启用缓存。
    /// 未调用此方法的 Pool 不会使用缓存，每次查询都会直接访问数据库。
    ///
    /// 这样设计的好处：
    /// - 可以灵活控制哪些 Pool 启用缓存
    /// - 临时查询的 Pool 可以不启用缓存，避免不必要的开销
    /// - 不同 Pool 的缓存互不干扰
    ///
    /// # Arguments
    /// * `pool` - 需要启用缓存的 Pool 引用
    ///
    /// # 示例
    /// ```ignore
    /// // 主库 Pool 启用缓存
    /// FetchField::init_pool_cache(&main_pool).await;
    ///
    /// // 从库 Pool 也可以启用缓存（独立缓存）
    /// FetchField::init_pool_cache(&replica_pool).await;
    ///
    /// // 临时查询 Pool 不启用缓存，直接查询数据库
    /// let temp_fetch = FetchField::new(&temp_pool); // 无缓存
    /// ```
    pub async fn init_cache(pool: &sqlx::Pool<DB>) {
        let pool_addr = pool as *const _ as usize;
        cache::init_pool_cache(pool_addr).await;
    }

    /// 检查 Pool 是否已初始化缓存
    ///
    /// # Arguments
    /// * `pool` - Pool 引用
    ///
    /// # Returns
    /// * `bool` - 是否已初始化缓存
    pub async fn is_cache_inited(pool: &sqlx::Pool<DB>) -> bool {
        let pool_addr = pool as *const _ as usize;
        cache::is_pool_inited(pool_addr).await
    }
}

/// 初始化全局缓存基础设施
///
/// 应在应用启动时调用一次，初始化缓存基础设施。
/// 之后每个需要缓存的 Pool 需要单独调用 `FetchField::init_cache`。
///
/// # Arguments
/// * `remote_notify` - 远程通知实例
/// * `use_cache` - 是否启用缓存，false 时缓存大小设为 0
#[cfg(feature = "redis")]
pub use cache::init_cache as fetch_field_init;
