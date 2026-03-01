// ============== SQL 工具 ==============
mod sql_quote;
pub use self::sql_quote::*;
mod page;
pub use self::page::*;

// ============== sqlx 查询工具 ==============
#[cfg(feature = "db-mysql")]
pub use super::sqlx_utils::fetch_string_field_max;
#[cfg(all(feature = "db-mysql", feature = "redis"))]
pub use super::sqlx_utils::init_string_field_cache;
#[cfg(feature = "db-mysql")]
pub use super::sqlx_utils::{fetch_group, fetch_map, fetch_one, fetch_vec};
#[cfg(feature = "db-mysql")]
pub use super::sqlx_utils::{StringFieldMaxError, StringFieldMaxResult, StringFieldType};
