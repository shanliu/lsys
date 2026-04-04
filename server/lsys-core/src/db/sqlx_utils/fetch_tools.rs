// Fetch 工具，用于简化数据库查询操作
//
// 使用方式：
// ```rust
// use lsys_core::db::sqlx_utils::Fetch;
//
// // 查询单条记录
// let user = Fetch::<MySql, User>::one(&pool, |qb| {
//     qb.field_eq("id", 1);
// }).await?;
//
// // 查询多条记录
// let users = Fetch::<MySql, User>::vec(&pool, |qb| {
//     qb.field_eq("status", "active");
// }).await?;
//
// // 查询并转为 HashMap
// let user_map = Fetch::<MySql, User>::map(&pool, |qb| {
//     qb.push(" 1=1 ");
// }, |u| u.id).await?;
//
// // 查询并分组
// let user_groups = Fetch::<MySql, User>::group(&pool, |qb| {
//     qb.push(" 1=1 ");
// }, |u| u.role_id).await?;
// ```

use sqlx::{Database, FromRow, QueryBuilder};
use std::collections::HashMap;
use std::hash::Hash;
use std::marker::PhantomData;
use crate::db::TableMeta;

/// Fetch 工具结构体
pub struct Fetch<DB: Database, M: TableMeta> {
    _db: PhantomData<DB>,
    _model: PhantomData<M>,
}

/// 内部宏：为指定数据库类型生成 Fetch 实现
macro_rules! impl_fetch_for_db {
    ($db_type:ty, $row_type:ty) => {
        impl<M: TableMeta> Fetch<$db_type, M> {
            /// 查询单条记录
            pub async fn one<'e, E>(
                executor: E,
                build_where: impl FnOnce(&mut QueryBuilder<'static, $db_type>),
            ) -> Result<M, sqlx::Error>
            where
                E: sqlx::Executor<'e, Database = $db_type>,
                M: Send + Unpin,
                for<'r> M: FromRow<'r, $row_type>,
            {
                let mut qb = QueryBuilder::new(format!("SELECT * FROM {} WHERE ", M::table_name()));
                build_where(&mut qb);
                qb.build_query_as::<M>().fetch_one(executor).await
            }

            /// 查询多条记录
            pub async fn vec<'e, E>(
                executor: E,
                build_where: impl FnOnce(&mut QueryBuilder<'static, $db_type>),
            ) -> Result<Vec<M>, sqlx::Error>
            where
                E: sqlx::Executor<'e, Database = $db_type>,
                M: Send + Unpin,
                for<'r> M: FromRow<'r, $row_type>,
            {
                let mut qb = QueryBuilder::new(format!("SELECT * FROM {} WHERE ", M::table_name()));
                build_where(&mut qb);
                qb.build_query_as::<M>().fetch_all(executor).await
            }

            /// 查询并转为 HashMap
            pub async fn map<'e, E, K, F>(
                executor: E,
                build_where: impl FnOnce(&mut QueryBuilder<'static, $db_type>),
                key_by: F,
            ) -> Result<HashMap<K, M>, sqlx::Error>
            where
                E: sqlx::Executor<'e, Database = $db_type>,
                M: Send + Unpin,
                for<'r> M: FromRow<'r, $row_type>,
                K: Eq + Hash,
                F: Fn(&M) -> K,
            {
                let data = Self::vec(executor, build_where).await?;
                let mut hash = HashMap::with_capacity(data.len());
                for item in data {
                    hash.entry(key_by(&item)).or_insert(item);
                }
                Ok(hash)
            }

            /// 查询并分组
            pub async fn group<'e, E, K, F>(
                executor: E,
                build_where: impl FnOnce(&mut QueryBuilder<'static, $db_type>),
                key_by: F,
            ) -> Result<HashMap<K, Vec<M>>, sqlx::Error>
            where
                E: sqlx::Executor<'e, Database = $db_type>,
                M: Send + Unpin,
                for<'r> M: FromRow<'r, $row_type>,
                K: Eq + Hash,
                F: Fn(&M) -> K,
            {
                let data = Self::vec(executor, build_where).await?;
                let mut hash = HashMap::new();
                for item in data {
                    hash.entry(key_by(&item)).or_insert_with(Vec::new).push(item);
                }
                Ok(hash)
            }
        }
    };
}

// 为各数据库类型生成实现
#[cfg(feature = "db-mysql")]
impl_fetch_for_db!(sqlx::MySql, sqlx::mysql::MySqlRow);
#[cfg(feature = "db-postgres")]
impl_fetch_for_db!(sqlx::Postgres, sqlx::postgres::PgRow);
#[cfg(feature = "db-sqlite")]
impl_fetch_for_db!(sqlx::Sqlite, sqlx::sqlite::SqliteRow);
