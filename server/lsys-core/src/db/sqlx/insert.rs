use crate::db::sqlx::update::Update;
use crate::db::SqlQuote;

use super::field::Field;
use super::table::TableMeta;
use super::value::{FieldValue, IntoFieldValue, StoredValue};
use sqlx::query::Query;
use sqlx::{Database, Error, Executor};
use std::marker::PhantomData;

/// INSERT 构建器
pub struct Insert<'a, DB: Database, M: TableMeta> {
    pub(crate) fields: Vec<(String, StoredValue<'a, DB>)>,
    _marker: PhantomData<M>,
}

impl<'a, DB: Database, M: TableMeta> Insert<'a, DB, M> {
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

    /// 将字段值绑定到 Query
    fn bind_fields<'q>(&'q self, sql: &'q str, mut args: <DB as Database>::Arguments<'q>) -> Query<'q, DB, <DB as Database>::Arguments<'q>> 
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

    /// 执行 INSERT
    pub async fn execute<'e, E>(self, executor: E) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'a_> <DB as Database>::Arguments<'a_>: sqlx::Arguments<'a_> + sqlx::IntoArguments<'a_, DB>,
        E: Executor<'e, Database = DB>,
    {
        let table = M::table_name().sql_quote();

        let sql = if self.fields.is_empty() {
            format!("INSERT INTO {} () VALUES ()", table)
        } else {
            let columns: Vec<&str> = self.fields.iter().map(|(col, _)| col.as_str()).collect();

            let placeholders: Vec<String> = self
                .fields
                .iter()
                .map(|(_, v)| match v {
                    StoredValue::Bind(_) => "?".to_string(), // In PostgreSQL it uses $1 but ? works for MySQL, SQLite.
                    StoredValue::Expr(e) => e.clone(),
                })
                .collect();

            format!(
                "INSERT INTO {} ({}) VALUES ({})",
                table,
                columns.join(", "),
                placeholders.join(", ")
            )
        };

        let args = <DB as Database>::Arguments::default();
        let query = self.bind_fields(&sql, args);
        query.execute(executor).await
    }

    /// 执行 INSERT ... ON DUPLICATE KEY UPDATE / ON CONFLICT DO UPDATE
    #[cfg(feature = "db-mysql")]
    pub async fn execute_update<'e, 'b, E>(
        self,
        on_duplicate: Update<'b, DB, M>,
        executor: E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'a_> <DB as Database>::Arguments<'a_>: sqlx::Arguments<'a_> + sqlx::IntoArguments<'a_, DB>,
        E: Executor<'e, Database = DB>,
    {
        if on_duplicate.is_empty() {
            return Err(Error::Protocol("ON DUPLICATE KEY UPDATE is empty".into()));
        }

        let table = M::table_name().sql_quote();
        let update_clause = on_duplicate.to_set_clause();

        let sql = if self.fields.is_empty() {
            format!(
                "INSERT INTO {} () VALUES () ON DUPLICATE KEY UPDATE {}",
                table, update_clause
            )
        } else {
            let columns: Vec<&str> = self.fields.iter().map(|(col, _)| col.as_str()).collect();

            let placeholders: Vec<String> = self
                .fields
                .iter()
                .map(|(_, v)| match v {
                    StoredValue::Bind(_) => "?".to_string(),
                    StoredValue::Expr(e) => e.clone(),
                })
                .collect();

            format!(
                "INSERT INTO {} ({}) VALUES ({}) ON DUPLICATE KEY UPDATE {}",
                table,
                columns.join(", "),
                placeholders.join(", "),
                update_clause
            )
        };

        let mut args = <DB as Database>::Arguments::default();

        for (_, value) in &self.fields {
            if let StoredValue::Bind(b) = value {
                b.add_to_args_dyn(&mut args);
            }
        }

        for (_, value) in &on_duplicate.fields {
            if let StoredValue::Bind(b) = value {
                b.add_to_args_dyn(&mut args);
            }
        }

        let query = sqlx::query_with(&sql, args);
        query.execute(executor).await
    }
}

impl<'a, DB: Database, M: TableMeta> Default for Insert<'a, DB, M> {
    fn default() -> Self {
        Self::new()
    }
}

// ============== BatchInsert ==============

/// 批量 INSERT 构建器
pub struct BatchInsert<'a, DB: Database, M: TableMeta> {
    rows: Vec<Insert<'a, DB, M>>,
    _marker: PhantomData<M>,
}

impl<'a, DB: Database, M: TableMeta> BatchInsert<'a, DB, M> {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            _marker: PhantomData,
        }
    }

    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rows: Vec::with_capacity(capacity),
            _marker: PhantomData,
        }
    }

    pub fn push(mut self, row: Insert<'a, DB, M>) -> Self {
        self.rows.push(row);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    pub async fn execute<'e, E>(self, executor: E) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'a_> <DB as Database>::Arguments<'a_>: sqlx::Arguments<'a_> + sqlx::IntoArguments<'a_, DB>,
        E: Executor<'e, Database = DB>,
    {
        if self.rows.is_empty() {
            return Ok(<DB as Database>::QueryResult::default());
        }

        let table = M::table_name().sql_quote();

        let mut all_columns: Vec<String> = Vec::new();
        for row in &self.rows {
            for (col, _) in &row.fields {
                if !all_columns.contains(col) {
                    all_columns.push(col.clone());
                }
            }
        }

        if all_columns.is_empty() {
            let value_groups: Vec<&str> = self.rows.iter().map(|_| "()").collect();
            let sql = format!(
                "INSERT INTO {} () VALUES {}",
                table,
                value_groups.join(", ")
            );
            return sqlx::query(&sql).execute(executor).await;
        }

        let mut value_groups: Vec<String> = Vec::new();
        let mut bind_values: Vec<&StoredValue<'a, DB>> = Vec::new();

        for row in &self.rows {
            let mut placeholders: Vec<String> = Vec::new();

            for col in &all_columns {
                if let Some((_, stored)) = row.fields.iter().find(|(c, _)| c == col) {
                    match stored {
                        StoredValue::Bind(_) => {
                            placeholders.push("?".to_string());
                            bind_values.push(stored);
                        }
                        StoredValue::Expr(e) => {
                            placeholders.push(e.clone());
                        }
                    }
                } else {
                    placeholders.push("DEFAULT".to_string());
                }
            }
            value_groups.push(format!("({})", placeholders.join(", ")));
        }

        let sql = format!(
            "INSERT INTO {} ({}) VALUES {}",
            table,
            all_columns.join(", "),
            value_groups.join(", ")
        );

        let mut args = <DB as Database>::Arguments::default();
        for stored in bind_values {
            if let StoredValue::Bind(b) = stored {
                b.add_to_args_dyn(&mut args);
            }
        }

        let query = sqlx::query_with(&sql, args);
        query.execute(executor).await
    }

    #[cfg(feature = "db-mysql")]
    pub async fn execute_update<'e, 'b, E>(
        self,
        on_duplicate: Update<'b, DB, M>,
        executor: E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'a_> <DB as Database>::Arguments<'a_>: sqlx::Arguments<'a_> + sqlx::IntoArguments<'a_, DB>,
        E: Executor<'e, Database = DB>,
    {
        if self.rows.is_empty() {
            return Ok(<DB as Database>::QueryResult::default());
        }

        if on_duplicate.is_empty() {
            return Err(Error::Protocol("ON DUPLICATE KEY UPDATE is empty".into()));
        }

        let table = M::table_name().sql_quote();
        let update_clause = on_duplicate.to_set_clause();

        let mut all_columns: Vec<String> = Vec::new();
        for row in &self.rows {
            for (col, _) in &row.fields {
                if !all_columns.contains(col) {
                    all_columns.push(col.clone());
                }
            }
        }

        if all_columns.is_empty() {
            let value_groups: Vec<&str> = self.rows.iter().map(|_| "()").collect();
            let sql = format!(
                "INSERT INTO {} () VALUES {} ON DUPLICATE KEY UPDATE {}",
                table,
                value_groups.join(", "),
                update_clause
            );

            let mut args = <DB as Database>::Arguments::default();
            for (_, value) in &on_duplicate.fields {
                if let StoredValue::Bind(b) = value {
                    b.add_to_args_dyn(&mut args);
                }
            }

            let query = sqlx::query_with(&sql, args);
            return query.execute(executor).await;
        }

        let mut value_groups: Vec<String> = Vec::new();
        let mut bind_values: Vec<&StoredValue<'a, DB>> = Vec::new();

        for row in &self.rows {
            let mut placeholders: Vec<String> = Vec::new();

            for col in &all_columns {
                if let Some((_, stored)) = row.fields.iter().find(|(c, _)| c == col) {
                    match stored {
                        StoredValue::Bind(_) => {
                            placeholders.push("?".to_string());
                            bind_values.push(stored);
                        }
                        StoredValue::Expr(e) => {
                            placeholders.push(e.clone());
                        }
                    }
                } else {
                    placeholders.push("DEFAULT".to_string());
                }
            }
            value_groups.push(format!("({})", placeholders.join(", ")));
        }

        let sql = format!(
            "INSERT INTO {} ({}) VALUES {} ON DUPLICATE KEY UPDATE {}",
            table,
            all_columns.join(", "),
            value_groups.join(", "),
            update_clause
        );

        let mut args = <DB as Database>::Arguments::default();

        for stored in bind_values {
            if let StoredValue::Bind(b) = stored {
                b.add_to_args_dyn(&mut args);
            }
        }

        for (_, value) in &on_duplicate.fields {
            if let StoredValue::Bind(b) = value {
                b.add_to_args_dyn(&mut args);
            }
        }

        let query = sqlx::query_with(&sql, args);
        query.execute(executor).await
    }
}

impl<'a, DB: Database, M: TableMeta> Default for BatchInsert<'a, DB, M> {
    fn default() -> Self {
        Self::new()
    }
}
