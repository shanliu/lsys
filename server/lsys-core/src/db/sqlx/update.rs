use crate::db::SqlQuote;

use super::field::Field;
use super::table::TableMeta;
use super::value::{FieldValue, IntoFieldValue, SqlSuffix, StoredValue};
use sqlx::mysql::{MySqlArguments, MySqlQueryResult};
use sqlx::query::Query;
use sqlx::{Error, Executor, MySql};
use std::marker::PhantomData;

/// UPDATE 构建器
pub struct Update<'a, M: TableMeta> {
    pub(crate) fields: Vec<(String, StoredValue<'a>)>,
    _marker: PhantomData<M>,
}

impl<'a, M: TableMeta> Update<'a, M> {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// 设置字段值
    pub fn set<T, V>(mut self, field: Field<T>, value: V) -> Self
    where
        T: for<'q> sqlx::Encode<'q, MySql> + sqlx::Type<MySql> + Send + Sync + 'a,
        V: IntoFieldValue<'a, T>,
    {
        let col = field.column.to_string();
        let field_value = value.into_field_value();

        let existing_idx = self.fields.iter().position(|(c, _)| c == &col);

        match field_value {
            FieldValue::Bind(v) => {
                let stored = StoredValue::Bind(v);
                match existing_idx {
                    Some(idx) => self.fields[idx] = (col, stored),
                    None => self.fields.push((col, stored)),
                }
            }
            FieldValue::Expr(e) => {
                let stored = StoredValue::Expr(e);
                match existing_idx {
                    Some(idx) => self.fields[idx] = (col, stored),
                    None => self.fields.push((col, stored)),
                }
            }
            FieldValue::Skip => {
                if let Some(idx) = existing_idx {
                    self.fields.remove(idx);
                }
            }
        }
        self
    }

    pub fn is_empty(&self) -> bool {
        self.fields.is_empty()
    }

    /// 生成 SET 子句
    pub(crate) fn to_set_clause(&self) -> String {
        self.fields
            .iter()
            .map(|(col, v)| match v {
                StoredValue::Bind(_) => format!("`{}` = ?", col),
                StoredValue::Expr(e) => format!("`{}` = {}", col, e),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// 绑定参数值
    ///
    /// 由于 Query<'q> 生命周期不变性，我们先收集所有参数到 Arguments，
    /// 然后用 query_with 重新构建 Query
    pub(crate) fn bind_values<'q>(&self, sql: &'q str) -> Query<'q, MySql, MySqlArguments> {
        let mut args = MySqlArguments::default();
        for (_, value) in &self.fields {
            if let StoredValue::Bind(b) = value {
                b.add_to_args(&mut args);
            }
        }

        sqlx::query_with(sql, args)
    }

    /// 执行 UPDATE
    ///
    /// 如果没有字段需要更新，直接返回成功（rows_affected = 0）
    pub async fn execute<'e, E>(
        self,
        suffix: SqlSuffix<'_>,
        executor: E,
    ) -> Result<MySqlQueryResult, Error>
    where
        E: Executor<'e, Database = MySql>,
    {
        if self.fields.is_empty() {
            // 没有字段需要更新，返回成功
            return Ok(MySqlQueryResult::default());
        }

        let table = M::table_name().sql_quote();

        let set_clause: Vec<String> = self
            .fields
            .iter()
            .map(|(col, v)| match v {
                StoredValue::Bind(_) => format!("`{}` = ?", col),
                StoredValue::Expr(e) => format!("`{}` = {}", col, e),
            })
            .collect();

        let sql = format!(
            "UPDATE {} SET {}{}",
            table,
            set_clause.join(", "),
            suffix.to_sql()
        );

        let query = self.bind_values(&sql);
        query.execute(executor).await
    }
}

impl<'a, M: TableMeta> Default for Update<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}
