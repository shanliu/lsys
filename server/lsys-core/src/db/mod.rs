mod utils;
pub use utils::*;

#[cfg(feature = "db")]
mod sqlx;

#[cfg(feature = "db")]
mod sqlx_utils;

#[cfg(feature = "db")]
mod sqlx_mod {
    // ============== 字段相关 ==============
    pub use super::sqlx::field::{Field, FieldMeta};

    // ============== 值与表达式 ==============
    pub use super::sqlx::value::{FieldValue, IntoFieldValue, Skip, SqlSuffix};

    // ============== 表相关 ==============
    pub use super::sqlx::table::{TableMeta, TableName};

    // ============== 字符串字段长度 ==============
    /// 初始化字符串字段缓存（需启用 redis feature）
    #[cfg(feature = "redis")]
    pub use super::sqlx_utils::string_field::init_string_field_cache;
    /// 内部使用，供宏生成的代码调用
    pub use super::sqlx_utils::string_field::query_string_field_max;
    /// 字符串字段最大长度结果类型及相关
    pub use super::sqlx_utils::string_field::{
        StringFieldMaxError, StringFieldMaxResult, StringFieldType,
    };

    // ============== CURD 构建器 ==============
    /// CURD 操作构建器
    pub use super::sqlx::insert::{BatchInsert, Insert};
    pub use super::sqlx::update::Update;
    //辅助
    pub use super::sqlx_utils::{DBOptionExecutorPool, DBOptionExecutorTransaction};
    pub use lsys_macros_db::lsys_model;
    pub use lsys_macros_db::lsys_model_status;
}

#[cfg(feature = "db")]
pub use sqlx_mod::*;
