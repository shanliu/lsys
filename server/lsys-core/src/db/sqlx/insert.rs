use crate::db::sqlx::update::Update;
use crate::db::SqlQuote;

use super::field::Field;
use super::table::TableMeta;
use super::value::{FieldValue, IntoFieldValue, StoredValue};
use sqlx::mysql::{MySqlArguments, MySqlQueryResult};
use sqlx::query::Query;
use sqlx::{Error, Executor, MySql};
use std::marker::PhantomData;

/// INSERT 构建器
pub struct Insert<'a, M: TableMeta> {
    pub(crate) fields: Vec<(String, StoredValue<'a>)>,
    _marker: PhantomData<M>,
}

impl<'a, M: TableMeta> Insert<'a, M> {
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

    /// 将字段值绑定到 Query
    fn bind_fields<'q>(&self, sql: &'q str) -> Query<'q, MySql, MySqlArguments> {
        let mut args = MySqlArguments::default();

        for (_, value) in &self.fields {
            if let StoredValue::Bind(b) = value {
                b.add_to_args(&mut args);
            }
        }

        sqlx::query_with(sql, args)
    }

    /// 执行 INSERT
    ///
    /// 支持空 INSERT（如只有自增ID的表）
    pub async fn execute<'e, E>(self, executor: E) -> Result<MySqlQueryResult, Error>
    where
        E: Executor<'e, Database = MySql>,
    {
        let table = M::table_name().sql_quote();

        let sql = if self.fields.is_empty() {
            // 支持空 INSERT: INSERT INTO `table` () VALUES ()
            format!("INSERT INTO {} () VALUES ()", table)
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
                "INSERT INTO {} ({}) VALUES ({})",
                table,
                columns
                    .iter()
                    .map(|c| format!("`{}`", c))
                    .collect::<Vec<_>>()
                    .join(", "),
                placeholders.join(", ")
            )
        };

        let query = self.bind_fields(&sql);
        query.execute(executor).await
    }

    /// 执行 INSERT ... ON DUPLICATE KEY UPDATE
    pub async fn execute_update<'e, 'b, E>(
        self,
        on_duplicate: Update<'b, M>,
        executor: E,
    ) -> Result<MySqlQueryResult, Error>
    where
        E: Executor<'e, Database = MySql>,
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
                columns
                    .iter()
                    .map(|c| format!("`{}`", c))
                    .collect::<Vec<_>>()
                    .join(", "),
                placeholders.join(", "),
                update_clause
            )
        };

        // 收集所有参数：INSERT 部分 + UPDATE 部分
        let mut args = MySqlArguments::default();

        // INSERT 的参数
        for (_, value) in &self.fields {
            if let StoredValue::Bind(b) = value {
                b.add_to_args(&mut args);
            }
        }

        // UPDATE 的参数
        for (_, value) in &on_duplicate.fields {
            if let StoredValue::Bind(b) = value {
                b.add_to_args(&mut args);
            }
        }

        let query = sqlx::query_with(&sql, args);
        query.execute(executor).await
    }
}

impl<'a, M: TableMeta> Default for Insert<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}

// ============== BatchInsert ==============

/// 批量 INSERT 构建器
pub struct BatchInsert<'a, M: TableMeta> {
    rows: Vec<Insert<'a, M>>,
    _marker: PhantomData<M>,
}

impl<'a, M: TableMeta> BatchInsert<'a, M> {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            _marker: PhantomData,
        }
    }

    /// 创建预分配容量的批量插入构建器
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rows: Vec::with_capacity(capacity),
            _marker: PhantomData,
        }
    }

    /// 添加一行数据
    pub fn push(mut self, row: Insert<'a, M>) -> Self {
        self.rows.push(row);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    /// 执行批量插入
    ///
    /// 支持空 INSERT（如只有自增ID的表）
    /// 如果批次为空，返回成功（rows_affected = 0）
    pub async fn execute<'e, E>(self, executor: E) -> Result<MySqlQueryResult, Error>
    where
        E: Executor<'e, Database = MySql>,
    {
        if self.rows.is_empty() {
            return Ok(MySqlQueryResult::default());
        }

        let table = M::table_name().sql_quote();

        // 收集所有列
        let mut all_columns: Vec<String> = Vec::new();
        for row in &self.rows {
            for (col, _) in &row.fields {
                if !all_columns.contains(col) {
                    all_columns.push(col.clone());
                }
            }
        }

        // 如果所有行都是空的（只有自增ID的表）
        if all_columns.is_empty() {
            let value_groups: Vec<&str> = self.rows.iter().map(|_| "()").collect();
            let sql = format!(
                "INSERT INTO {} () VALUES {}",
                table,
                value_groups.join(", ")
            );
            return sqlx::query(&sql).execute(executor).await;
        }

        // 构建 VALUES 子句
        let mut value_groups: Vec<String> = Vec::new();
        let mut bind_values: Vec<&StoredValue> = Vec::new();

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
            all_columns
                .iter()
                .map(|c| format!("`{}`", c))
                .collect::<Vec<_>>()
                .join(", "),
            value_groups.join(", ")
        );

        // 收集所有参数
        let mut args = MySqlArguments::default();
        for stored in bind_values {
            if let StoredValue::Bind(b) = stored {
                b.add_to_args(&mut args);
            }
        }

        let query = sqlx::query_with(&sql, args);
        query.execute(executor).await
    }

    /// 执行批量 INSERT ON DUPLICATE KEY UPDATE
    ///
    /// 如果批次为空，返回成功（rows_affected = 0）
    pub async fn execute_update<'e, 'b, E>(
        self,
        on_duplicate: Update<'b, M>,
        executor: E,
    ) -> Result<MySqlQueryResult, Error>
    where
        E: Executor<'e, Database = MySql>,
    {
        if self.rows.is_empty() {
            return Ok(MySqlQueryResult::default());
        }

        if on_duplicate.is_empty() {
            return Err(Error::Protocol("ON DUPLICATE KEY UPDATE is empty".into()));
        }

        let table = M::table_name().sql_quote();
        let update_clause = on_duplicate.to_set_clause();

        // 收集所有列
        let mut all_columns: Vec<String> = Vec::new();
        for row in &self.rows {
            for (col, _) in &row.fields {
                if !all_columns.contains(col) {
                    all_columns.push(col.clone());
                }
            }
        }

        // 如果所有行都是空的（只有自增ID的表）
        if all_columns.is_empty() {
            let value_groups: Vec<&str> = self.rows.iter().map(|_| "()").collect();
            let sql = format!(
                "INSERT INTO {} () VALUES {} ON DUPLICATE KEY UPDATE {}",
                table,
                value_groups.join(", "),
                update_clause
            );

            // 只有 UPDATE 部分的参数
            let mut args = MySqlArguments::default();
            for (_, value) in &on_duplicate.fields {
                if let StoredValue::Bind(b) = value {
                    b.add_to_args(&mut args);
                }
            }

            let query = sqlx::query_with(&sql, args);
            return query.execute(executor).await;
        }

        // 构建 VALUES 子句
        let mut value_groups: Vec<String> = Vec::new();
        let mut bind_values: Vec<&StoredValue> = Vec::new();

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
            all_columns
                .iter()
                .map(|c| format!("`{}`", c))
                .collect::<Vec<_>>()
                .join(", "),
            value_groups.join(", "),
            update_clause
        );

        // 收集所有参数：VALUES 部分 + UPDATE 部分
        let mut args = MySqlArguments::default();

        // VALUES 部分的参数
        for stored in bind_values {
            if let StoredValue::Bind(b) = stored {
                b.add_to_args(&mut args);
            }
        }

        // UPDATE 部分的参数
        for (_, value) in &on_duplicate.fields {
            if let StoredValue::Bind(b) = value {
                b.add_to_args(&mut args);
            }
        }

        let query = sqlx::query_with(&sql, args);
        query.execute(executor).await
    }
}

impl<'a, M: TableMeta> Default for BatchInsert<'a, M> {
    fn default() -> Self {
        Self::new()
    }
}
