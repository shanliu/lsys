//! 数据库类型检测辅助模块
//!
//! 提供运行时数据库类型检测功能，支持同时启用多种数据库后端。
//! 即使同时启用 `db-mysql`、`db-postgres`、`db-sqlite` 多个 feature，
//! 也能在运行时正确识别实际的数据库类型。

use sqlx::Database;

/// 检测是否为 MySQL 数据库类型
///
/// - 启用 `db-mysql` feature 时：运行时检查类型 ID
/// - 未启用时：始终返回 false
#[cfg(feature = "db-mysql")]
#[allow(unused)]
pub fn is_mysql_db<DB: Database>() -> bool {
    use sqlx::MySql;
    std::any::TypeId::of::<DB>() == std::any::TypeId::of::<MySql>()
}

#[cfg(not(feature = "db-mysql"))]
#[allow(dead_code)]
pub fn is_mysql_db<DB: Database>() -> bool {
    false
}

/// 检测是否为 PostgreSQL 数据库类型
///
/// - 启用 `db-postgres` feature 时：运行时检查类型 ID
/// - 未启用时：始终返回 false
#[cfg(feature = "db-postgres")]
#[allow(unused)]
#[allow(clippy::extra_unused_type_parameters)]
pub fn is_postgres_db<DB: Database>() -> bool {
    use sqlx::Postgres;
    std::any::TypeId::of::<DB>() == std::any::TypeId::of::<Postgres>()
}

#[cfg(not(feature = "db-postgres"))]
#[allow(dead_code)]
#[allow(clippy::extra_unused_type_parameters)]
pub fn is_postgres_db<DB: Database>() -> bool {
    false
}

/// 检测是否为 SQLite 数据库类型
///
/// - 启用 `db-sqlite` feature 时：运行时检查类型 ID
/// - 未启用时：始终返回 false
#[cfg(feature = "db-sqlite")]
#[allow(unused)]
#[allow(clippy::extra_unused_type_parameters)]
pub fn is_sqlite_db<DB: Database>() -> bool {
    use sqlx::Sqlite;
    std::any::TypeId::of::<DB>() == std::any::TypeId::of::<Sqlite>()
}

#[cfg(not(feature = "db-sqlite"))]
#[allow(dead_code)]
#[allow(clippy::extra_unused_type_parameters)]
pub fn is_sqlite_db<DB: Database>() -> bool {
    false
}
