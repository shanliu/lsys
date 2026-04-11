mod sql_tools;
pub use sql_tools::*;

#[cfg(feature = "db")]
pub mod sqlx;

#[cfg(feature = "db")]
mod sqlx_utils;

/// 统一导出 sql_tools + sqlx_utils
pub mod utils {
    // FetchField 字段查询工具（支持 MySQL / PostgreSQL / SQLite）
    //
    // 使用方式：
    //   use lsys_core::db::utils::FetchField;
    //
    //   // 初始化缓存（应用启动时调用一次）
    //   FetchField::<sqlx::MySql>::init_cache(remote_notify, true).await;
    //
    //   // 链式调用查询字段最大长度
    //   let max_len =FetchField::new(&pool).string_max::<UserModel>(&UserModel::NAME).await;
    //
    #[cfg(feature = "db")]
    pub use super::sqlx_utils::{
        FetchField, FetchFieldError, FetchFieldStringMaxResult, FetchFieldStringQuery,
        FetchFieldStringType,
    };

    #[cfg(feature = "redis")]
    pub use super::sqlx_utils::fetch_field_init;

    // Fetch 工具（支持 MySQL、PostgreSQL、SQLite）
    //
    // 使用方式：
    //   use lsys_core::db::utils::Fetch;
    //   let result = Fetch::<MySql, MyModel>::one(&pool, |qb| {
    //       qb.field_eq("id", 1);
    //   }).await?;

    #[cfg(feature = "db")]
    pub use super::sqlx_utils::Fetch;
}

#[cfg(feature = "db")]
mod sqlx_mod {
    // ============== 字段相关 ==============
    pub use super::sqlx::field::{Field, FieldMeta};

    // ============== 值与表达式 ==============
    pub use super::sqlx::value::{FieldValue, IntoFieldValue};

    // ============== SQL 构建辅助 ==============
    pub use super::sqlx::builder::{QueryBuilderExt, WhereClause};

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
