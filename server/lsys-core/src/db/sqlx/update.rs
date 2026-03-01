use crate::db::SqlQuote;

use super::field::Field;
use super::table::TableMeta;
use super::value::{FieldValue, IntoFieldValue, SqlSuffix, StoredValue};
use sqlx::query::Query;
use sqlx::{Database, Error, Executor};
use std::marker::PhantomData;

/// UPDATE 构建器
pub struct Update<'a, DB: Database, M: TableMeta> {
    pub(crate) fields: Vec<(String, StoredValue<'a, DB>)>,
    _marker: PhantomData<M>,
}

impl<'a, DB: Database, M: TableMeta> Update<'a, DB, M> {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// 设置字段值
    pub fn set<T, V>(mut self, field: Field<T>, value: V) -> Self
    where
        for<'q> T: sqlx::Encode<'q, DB> + sqlx::Type<DB> + Send + Sync + 'a,
        V: IntoFieldValue<'a, DB, T>,
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
                StoredValue::Bind(_) => format!("{} = ?", col),
                StoredValue::Expr(e) => format!("{} = {}", col, e),
            })
            .collect::<Vec<_>>()
            .join(", ")
    }

    /// 绑定参数值
    pub(crate) fn bind_values<'q>(&'q self, sql: &'q str, mut args: <DB as Database>::Arguments<'q>) -> Query<'q, DB, <DB as Database>::Arguments<'q>> 
    where
        for<'a_> <DB as Database>::Arguments<'a_>: sqlx::Arguments<'a_> + sqlx::IntoArguments<'a_, DB>,
    {
        for (_, value) in &self.fields {
            if let StoredValue::Bind(b) = value {
                b.add_to_args_dyn(&mut args);
            }
        }

        sqlx::query_with(sql, args)
    }

    /// 执行 UPDATE
    pub async fn execute<'e, E>(
        self,
        suffix: SqlSuffix<'_>,
        executor: E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'a_> <DB as Database>::Arguments<'a_>: sqlx::Arguments<'a_> + sqlx::IntoArguments<'a_, DB>,
        E: Executor<'e, Database = DB>,
    {
        if self.fields.is_empty() {
            // 没有字段需要更新，返回成功
            return Ok(<DB as Database>::QueryResult::default());
        }

        let table = M::table_name().sql_quote();

        let set_clause: Vec<String> = self
            .fields
            .iter()
            .map(|(col, v)| match v {
                StoredValue::Bind(_) => format!("{} = ?", col),
                StoredValue::Expr(e) => format!("{} = {}", col, e),
            })
            .collect();

        let sql = format!(
            "UPDATE {} SET {}{}",
            table,
            set_clause.join(", "),
            suffix.to_sql()
        );

        let args = <DB as Database>::Arguments::default();
        let query = self.bind_values(&sql, args);
        query.execute(executor).await
    }
}

impl<'a, DB: Database, M: TableMeta> Default for Update<'a, DB, M> {
    fn default() -> Self {
        Self::new()
    }
}
