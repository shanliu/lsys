mod sql_tools;
pub use sql_tools::*;

#[cfg(feature = "db")]
pub mod sqlx;

#[cfg(feature = "db")]
mod sqlx_utils;

/// 统一导出 sql_tools + sqlx_utils
pub mod utils {
    #[cfg(feature = "db-mysql")]
    pub use super::sqlx_utils::fetch_string_field_max;
    #[cfg(all(feature = "db-mysql", feature = "redis"))]
    pub use super::sqlx_utils::init_string_field_cache;
    #[cfg(feature = "db-mysql")]
    pub use super::sqlx_utils::{fetch_group, fetch_map, fetch_one, fetch_vec};
    #[cfg(feature = "db-mysql")]
    pub use super::sqlx_utils::{StringFieldMaxError, StringFieldMaxResult, StringFieldType};
}

#[cfg(feature = "db")]
mod sqlx_mod {
    // ============== 字段相关 ==============
    pub use super::sqlx::field::{Field, FieldMeta};

    // ============== 值与表达式 ==============
    pub use super::sqlx::value::{FieldValue, IntoFieldValue, Skip, SqlSuffix};

    // ============== 表相关 ==============
    pub use super::sqlx::table::{TableMeta, TableName};

    // ============== CURD 构建器 ==============
    /// CURD 操作构建器
    pub use super::sqlx::insert::{BatchInsert, Insert};
    pub use super::sqlx::update::Update;
    //辅助
    pub use super::sqlx::executor::OptionTxExecutor;
    pub use lsys_macros_db::lsys_model;
    pub use lsys_macros_db::lsys_model_status;
}

#[cfg(feature = "db")]
pub use sqlx_mod::*;
