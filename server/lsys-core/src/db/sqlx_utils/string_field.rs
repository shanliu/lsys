//! 字符串字段最大长度查询
//!
//! 通过数据库查询字符串字段的最大长度，并提供缓存支持（需启用 redis feature）

use std::ops::Deref;

use sqlx::{MySql, Pool, Row};

use super::super::sqlx::field::Field;
use super::super::sqlx::table::{TableMeta, TableName};

/// 字符串字段长度查询错误
#[derive(Debug, Clone)]
pub enum StringFieldMaxError {
    /// 非字符串类型
    NotString,
    /// 字段不存在
    NotFound,
    /// 查询错误
    DbError(String),
}

impl std::fmt::Display for StringFieldMaxError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            StringFieldMaxError::NotString => write!(f, "field is not a string type"),
            StringFieldMaxError::NotFound => write!(f, "field not found in table"),
            StringFieldMaxError::DbError(msg) => write!(f, "database error: {}", msg),
        }
    }
}

impl std::error::Error for StringFieldMaxError {}

/// 字符串字段最大长度结果
#[derive(Debug, Clone)]
pub struct StringFieldMaxResult(Result<u64, StringFieldMaxError>);

impl StringFieldMaxResult {
    /// LONGTEXT 最大长度常量
    pub const LONGTEXT_MAX: u64 = 4_294_967_295;

    /// 创建成功结果
    pub fn ok(len: u64) -> Self {
        Self(Ok(len))
    }

    /// 创建错误结果
    pub fn err(e: StringFieldMaxError) -> Self {
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
impl Deref for StringFieldMaxResult {
    type Target = Result<u64, StringFieldMaxError>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl From<Result<u64, StringFieldMaxError>> for StringFieldMaxResult {
    fn from(result: Result<u64, StringFieldMaxError>) -> Self {
        Self(result)
    }
}

/// 标记 trait：字符串类型字段
/// 用于编译时限制只有 String 类型的 Field 可以调用 string_field_max
pub trait StringFieldType {}

impl StringFieldType for String {}
impl StringFieldType for Option<String> {}

// ============== 缓存实现（仅 redis feature 启用时） ==============

#[cfg(feature = "redis")]
mod cache {
    use super::*;
    use crate::cache::{LocalCache, LocalCacheConfig};
    use crate::RemoteNotify;
    use std::collections::HashMap;
    use std::sync::Arc;
    use tokio::sync::OnceCell;

    /// 表字段类型映射：字段名 -> 最大长度
    pub type TableFieldsMap = HashMap<String, u64>;

    /// 全局缓存实例：表名 -> 所有字符串字段的长度映射
    static STRING_FIELD_CACHE: OnceCell<LocalCache<String, TableFieldsMap>> = OnceCell::const_new();

    /// 初始化缓存
    ///
    /// # Arguments
    /// * `remote_notify` - 远程通知实例
    /// * `use_cache` - 是否启用缓存，false 时缓存大小设为 0
    ///
    /// # Examples
    /// ```no_run
    /// # use lsys_core::RemoteNotify;
    /// # use std::sync::Arc;
    /// # use lsys_core::db::init_string_field_cache;
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let remote_notify = Arc::new(RemoteNotify::default());
    /// // 启用缓存
    /// init_string_field_cache(remote_notify.clone(), true).await;
    ///
    /// // 禁用缓存
    /// init_string_field_cache(remote_notify.clone(), false).await;
    /// # }
    /// ```
    pub async fn init_string_field_cache(remote_notify: Arc<RemoteNotify>, use_cache: bool) {
        let _ = STRING_FIELD_CACHE
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
    }

    /// 从缓存获取表的字段映射
    pub(super) async fn get_table(table: &TableName) -> Option<TableFieldsMap> {
        if let Some(cache) = STRING_FIELD_CACHE.get() {
            cache.get(&table.full_name()).await
        } else {
            None
        }
    }

    /// 缓存表的所有字符串字段映射
    pub(super) async fn set_table(table: &TableName, fields: TableFieldsMap) {
        if let Some(cache) = STRING_FIELD_CACHE.get() {
            cache.set(table.full_name(), fields, 0).await;
        }
    }
}

#[cfg(feature = "redis")]
pub use cache::init_string_field_cache;

// ============== 类型解析 ==============

/// 解析 MySQL 类型字符串，提取最大长度
///
/// 支持的类型：
/// - char(n), varchar(n) -> n
/// - tinytext -> 255
/// - text -> 65535
/// - mediumtext -> 16777215
/// - longtext -> 4294967295
fn parse_type_max_length(type_str: &str) -> Option<u64> {
    let type_lower = type_str.to_lowercase();

    // varchar(n) 或 char(n)
    if type_lower.starts_with("varchar") || type_lower.starts_with("char") {
        if let Some(start) = type_lower.find('(') {
            if let Some(end) = type_lower.find(')') {
                if let Ok(len) = type_lower[start + 1..end].parse::<u64>() {
                    return Some(len);
                }
            }
        }
        return None;
    }

    // text 类型
    match type_lower.as_str() {
        "tinytext" => Some(255),
        "text" => Some(65535),
        "mediumtext" => Some(16_777_215),
        "longtext" => Some(4_294_967_295),
        _ => None,
    }
}

// ============== 查询实现 ==============

/// 查询单个字段的最大长度（无缓存时使用）
///
/// 使用 `SHOW COLUMNS FROM table LIKE 'column'` 只查询指定字段
#[cfg(not(feature = "redis"))]
async fn query_single_field_max(
    pool: &Pool<MySql>,
    table: &TableName,
    column: &str,
) -> StringFieldMaxResult {
    let sql = format!(
        "SHOW COLUMNS FROM `{}` LIKE '{}'",
        table.full_name(),
        column
    );

    match sqlx::query(&sql).fetch_optional(pool).await {
        Ok(Some(row)) => {
            let type_str: String = row.try_get("Type").unwrap_or_default();
            match parse_type_max_length(&type_str) {
                Some(len) => StringFieldMaxResult::ok(len),
                None => StringFieldMaxResult::err(StringFieldMaxError::NotString),
            }
        }
        Ok(None) => StringFieldMaxResult::err(StringFieldMaxError::NotFound),
        Err(e) => StringFieldMaxResult::err(StringFieldMaxError::DbError(e.to_string())),
    }
}

/// 查询表的所有字符串字段并构建映射（有缓存时使用）
///
/// 使用 `SHOW COLUMNS FROM table` 查询所有列，解析字符串类型字段
#[cfg(feature = "redis")]
async fn query_table_string_fields(
    pool: &Pool<MySql>,
    table: &TableName,
) -> Result<std::collections::HashMap<String, u64>, StringFieldMaxError> {
    use std::collections::HashMap;

    let sql = format!("SHOW COLUMNS FROM `{}`", table.full_name());

    match sqlx::query(&sql).fetch_all(pool).await {
        Ok(rows) => {
            let mut fields_map = HashMap::new();
            for row in rows {
                let field_name: String = row.try_get("Field").unwrap_or_default();
                let type_str: String = row.try_get("Type").unwrap_or_default();
                // 只缓存字符串类型字段
                if let Some(len) = parse_type_max_length(&type_str) {
                    fields_map.insert(field_name, len);
                }
            }
            Ok(fields_map)
        }
        Err(e) => Err(StringFieldMaxError::DbError(e.to_string())),
    }
}

/// 从数据库查询字符串字段最大长度
///
/// # 缓存
/// - 启用 `redis` feature 且调用了 `init_string_field_cache` 时：查询整表并缓存所有字符串字段
/// - 未启用时：只查询指定字段，避免不必要的解析
pub async fn query_string_field_max<M: TableMeta>(
    pool: &Pool<MySql>,
    field: &Field<impl StringFieldType>,
) -> StringFieldMaxResult {
    let table = M::table_name();
    let column = field.column.as_ref();

    #[cfg(feature = "redis")]
    return query_with_cache(pool, &table, column).await;

    #[cfg(not(feature = "redis"))]
    query_single_field_max(pool, &table, column).await
}

/// 带缓存的查询实现
#[cfg(feature = "redis")]
async fn query_with_cache(
    pool: &Pool<MySql>,
    table: &TableName,
    column: &str,
) -> StringFieldMaxResult {
    // 先查缓存
    if let Some(cached_fields) = cache::get_table(table).await {
        return match cached_fields.get(column) {
            Some(&len) => StringFieldMaxResult::ok(len),
            None => StringFieldMaxResult::err(StringFieldMaxError::NotString),
        };
    }

    // 查询整表并缓存
    let fields_map = match query_table_string_fields(pool, table).await {
        Ok(map) => map,
        Err(e) => return StringFieldMaxResult::err(e),
    };
    let result = match fields_map.get(column) {
        Some(&len) => StringFieldMaxResult::ok(len),
        None => StringFieldMaxResult::err(StringFieldMaxError::NotString),
    };
    cache::set_table(table, fields_map).await;
    result
}

// ============== 测试 ==============

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_type_max_length() {
        assert_eq!(parse_type_max_length("varchar(255)"), Some(255));
        assert_eq!(parse_type_max_length("VARCHAR(100)"), Some(100));
        assert_eq!(parse_type_max_length("char(10)"), Some(10));
        assert_eq!(parse_type_max_length("CHAR(50)"), Some(50));
        assert_eq!(parse_type_max_length("tinytext"), Some(255));
        assert_eq!(parse_type_max_length("TEXT"), Some(65535));
        assert_eq!(parse_type_max_length("mediumtext"), Some(16_777_215));
        assert_eq!(parse_type_max_length("LONGTEXT"), Some(4_294_967_295));
        assert_eq!(parse_type_max_length("int"), None);
        assert_eq!(parse_type_max_length("bigint"), None);
    }

    #[test]
    fn test_string_field_max() {
        let result = StringFieldMaxResult::ok(255);
        assert!(result.is_ok());
        assert_eq!(*result.as_ref().unwrap(), 255);

        let result = StringFieldMaxResult::err(StringFieldMaxError::NotString);
        assert!(result.is_err());
    }

    #[test]
    fn test_max_len_or() {
        assert_eq!(StringFieldMaxResult::ok(255).len_or(100), 255);
        assert_eq!(
            StringFieldMaxResult::err(StringFieldMaxError::NotString).len_or(100),
            100
        );
        assert_eq!(
            StringFieldMaxResult::err(StringFieldMaxError::NotFound).len_or(200),
            200
        );
        assert_eq!(
            StringFieldMaxResult::err(StringFieldMaxError::DbError("test".to_string())).len_or(300),
            300
        );
    }
}
