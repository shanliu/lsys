use std::collections::HashMap;
use std::hash::Hash;

use sqlx::{FromRow, MySql, Pool};

use crate::db::{SqlExpr, SqlQuote, TableMeta};

pub async fn fetch_one<M>(db: &Pool<MySql>, where_sql: impl AsRef<str>) -> Result<M, sqlx::Error>
where
    M: TableMeta + Send + Unpin,
    for<'r> M: FromRow<'r, sqlx::mysql::MySqlRow>,
{
    let sql = crate::sql_format!(
        "select * from {} where {} ",
        M::table_name(),
        SqlExpr(where_sql.as_ref())
    );
    sqlx::query_as::<_, M>(&sql).fetch_one(db).await
}

pub async fn fetch_vec<M>(
    db: &Pool<MySql>,
    where_sql: impl AsRef<str>,
) -> Result<Vec<M>, sqlx::Error>
where
    M: TableMeta + Send + Unpin,
    for<'r> M: FromRow<'r, sqlx::mysql::MySqlRow>,
{
    let sql = crate::sql_format!(
        "select * from {} where {} ",
        M::table_name(),
        SqlExpr(where_sql.as_ref())
    );
    sqlx::query_as::<_, M>(&sql).fetch_all(db).await
}

pub async fn fetch_map<M, K, F>(
    db: &Pool<MySql>,
    where_sql: impl AsRef<str>,
    key_by: F,
) -> Result<HashMap<K, M>, sqlx::Error>
where
    M: TableMeta + Send + Unpin,
    for<'r> M: FromRow<'r, sqlx::mysql::MySqlRow>,
    K: Eq + Hash,
    F: Fn(&M) -> K,
{
    let data = fetch_vec::<M>(db, where_sql).await?;
    Ok(build_unique_map(data, key_by))
}

pub async fn fetch_group<M, K, F>(
    db: &Pool<MySql>,
    where_sql: impl AsRef<str>,
    key_by: F,
) -> Result<HashMap<K, Vec<M>>, sqlx::Error>
where
    M: TableMeta + Send + Unpin,
    for<'r> M: FromRow<'r, sqlx::mysql::MySqlRow>,
    K: Eq + Hash,
    F: Fn(&M) -> K,
{
    let data = fetch_vec::<M>(db, where_sql).await?;
    let mut hash = HashMap::new();
    for data_ in data.into_iter() {
        hash.entry(key_by(&data_)).or_insert(vec![]).push(data_);
    }
    Ok(hash)
}

#[inline]
fn build_unique_map<M, K, F>(data: Vec<M>, key_by: F) -> HashMap<K, M>
where
    K: Eq + Hash,
    F: Fn(&M) -> K,
{
    let mut hash = HashMap::with_capacity(data.len());
    for data_ in data.into_iter() {
        hash.entry(key_by(&data_)).or_insert(data_);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::build_unique_map;

    #[derive(Clone)]
    struct TmpModel {
        id: u64,
        v: &'static str,
    }

    #[test]
    fn test_build_unique_map_keep_first() {
        let data = vec![
            TmpModel { id: 1, v: "a" },
            TmpModel { id: 1, v: "b" },
            TmpModel { id: 2, v: "c" },
        ];
        let map = build_unique_map(data, |m| m.id);
        assert_eq!(map.len(), 2);
        assert_eq!(map.get(&1).map(|m| m.v), Some("a"));
        assert_eq!(map.get(&2).map(|m| m.v), Some("c"));
    }
}
