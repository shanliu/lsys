use crate::db::sqlx::update::Update;
use crate::db::sqlx::utils::{push_field_value_or_default, push_set_clause_to, push_values_to};

use super::field::Field;
use super::table::TableMeta;
use super::value::{FieldValue, IntoFieldValue, StoredValue};
use sqlx::{Database, Error, Executor};
use std::marker::PhantomData;

// 使用公共的数据库类型检测函数
#[cfg(feature = "db-mysql")]
#[allow(unused)]
use super::db_type::is_mysql_db;
#[cfg(feature = "db-postgres")]
#[allow(unused)]
use super::db_type::is_postgres_db;
#[cfg(feature = "db-sqlite")]
#[allow(unused)]
use super::db_type::is_sqlite_db;

/// INSERT 构建器
pub struct Insert<DB: Database, M: TableMeta> {
    pub(crate) fields: Vec<(String, StoredValue<DB>)>,
    _marker: PhantomData<M>,
}

impl<DB: Database, M: TableMeta> Insert<DB, M> {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<DB: Database, M: TableMeta> Default for Insert<DB, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<DB: Database, M: TableMeta> Insert<DB, M> {
    /// 设置字段值
    pub fn set<T, V>(mut self, field: Field<T>, value: V) -> Self
    where
        T: Clone + Send + Sync + 'static,
        for<'q> T: sqlx::Encode<'q, DB> + sqlx::Type<DB>,
        V: IntoFieldValue<DB, T>,
    {
        let col = field.column.to_string();
        let field_value = value.into_field_value();

        let existing_idx = self.fields.iter().position(|(c, _)| c == &col);

        match field_value {
            FieldValue::Value(v) => {
                let stored = StoredValue::Bind(Box::new(v));
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
            FieldValue::Dynamic(f) => {
                let stored = StoredValue::Dynamic(f);
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

    // 泛型 execute 方法已移至文件末尾的特定数据库实现
    // 以避免 QueryBuilder 的生命周期问题

    // execute_update_mysql 方法已移至文件末尾的特定数据库实现
    // 以避免 QueryBuilder 的生命周期问题

    // execute_update_postgres 方法已移至文件末尾的特定数据库实现

    // execute_update_sqlite 方法已移至文件末尾的特定数据库实现

    // execute_update 方法已移至文件末尾的特定数据库实现
}

pub struct BatchInsert<DB: Database, M: TableMeta> {
    rows: Vec<Insert<DB, M>>,
    _marker: PhantomData<M>,
}

impl<DB: Database, M: TableMeta> BatchInsert<DB, M> {
    pub fn new() -> Self {
        Self {
            rows: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<DB: Database, M: TableMeta> Default for BatchInsert<DB, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<DB: Database, M: TableMeta> BatchInsert<DB, M> {
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            rows: Vec::with_capacity(capacity),
            _marker: PhantomData,
        }
    }

    pub fn push(mut self, row: Insert<DB, M>) -> Self {
        self.rows.push(row);
        self
    }

    pub fn is_empty(&self) -> bool {
        self.rows.is_empty()
    }

    // 泛型 execute 方法已移至文件末尾的特定数据库实现
    // 以避免 QueryBuilder 的生命周期问题

    // BatchInsert 的 execute_update_* 方法已移至文件末尾的特定数据库实现

    // execute_update 方法已移至文件末尾的特定数据库实现
}

// ============================================================================
// MySQL 特定实现
// ============================================================================

#[cfg(feature = "db-mysql")]
impl<M: TableMeta> Insert<sqlx::MySql, M> {
    /// 执行 INSERT
    pub async fn execute<'e, E>(self, executor: E) -> Result<sqlx::mysql::MySqlQueryResult, Error>
    where
        E: Executor<'e, Database = sqlx::MySql>,
    {
        let table = M::table_name().quoted();

        if self.fields.is_empty() {
            let sql = format!("INSERT INTO {} () VALUES ()", table);
            return sqlx::query(&sql).execute(executor).await;
        }

        let columns: Vec<&str> = self.fields.iter().map(|(col, _)| col.as_str()).collect();
        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));

        {
            let mut sep = qb.separated(", ");
            for col in &columns {
                sep.push(*col);
            }
        }

        qb.push(") VALUES (");
        push_values_to(&self.fields, &mut qb);
        qb.push(")");

        qb.build().execute(executor).await
    }

    /// 执行 INSERT ... ON DUPLICATE KEY UPDATE
    pub async fn execute_update<'e, E>(
        self,
        on_duplicate: Update<sqlx::MySql, M>,
        executor: E,
    ) -> Result<sqlx::mysql::MySqlQueryResult, Error>
    where
        E: Executor<'e, Database = sqlx::MySql>,
    {
        if on_duplicate.is_empty() {
            return Err(Error::Protocol("ON DUPLICATE KEY UPDATE is empty".into()));
        }

        let table = M::table_name().quoted();

        if self.fields.is_empty() {
            let mut qb = sqlx::QueryBuilder::new(format!(
                "INSERT INTO {} () VALUES () ON DUPLICATE KEY UPDATE ",
                table
            ));
            push_set_clause_to(&on_duplicate.fields, &mut qb);
            return qb.build().execute(executor).await;
        }

        let columns: Vec<&str> = self.fields.iter().map(|(col, _)| col.as_str()).collect();
        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));

        {
            let mut sep = qb.separated(", ");
            for col in &columns {
                sep.push(*col);
            }
        }

        qb.push(") VALUES (");
        push_values_to(&self.fields, &mut qb);
        qb.push(") ON DUPLICATE KEY UPDATE ");
        push_set_clause_to(&on_duplicate.fields, &mut qb);

        qb.build().execute(executor).await
    }
}

#[cfg(feature = "db-mysql")]
impl<M: TableMeta> BatchInsert<sqlx::MySql, M> {
    /// 执行批量 INSERT
    pub async fn execute<'e, E>(self, executor: E) -> Result<sqlx::mysql::MySqlQueryResult, Error>
    where
        E: Executor<'e, Database = sqlx::MySql>,
    {
        if self.rows.is_empty() {
            return Ok(sqlx::mysql::MySqlQueryResult::default());
        }

        let table = M::table_name().quoted();
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

        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));

        {
            let mut sep = qb.separated(", ");
            for col in &all_columns {
                sep.push(col.as_str());
            }
        }

        qb.push(") VALUES ");

        let mut first_row = true;
        for row in &self.rows {
            if !first_row {
                qb.push(", ");
            }
            first_row = false;

            qb.push("(");
            let mut first_col = true;
            for col in &all_columns {
                if !first_col {
                    qb.push(", ");
                }
                first_col = false;
                push_field_value_or_default(&row.fields, col, &mut qb, "DEFAULT");
            }
            qb.push(")");
        }

        qb.build().execute(executor).await
    }

    /// 执行批量 INSERT ... ON DUPLICATE KEY UPDATE
    pub async fn execute_update<'e, E>(
        self,
        on_duplicate: Update<sqlx::MySql, M>,
        executor: E,
    ) -> Result<sqlx::mysql::MySqlQueryResult, Error>
    where
        E: Executor<'e, Database = sqlx::MySql>,
    {
        if self.rows.is_empty() {
            return Ok(sqlx::mysql::MySqlQueryResult::default());
        }

        if on_duplicate.is_empty() {
            return Err(Error::Protocol("ON DUPLICATE KEY UPDATE is empty".into()));
        }

        let table = M::table_name().quoted();
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
            let mut qb = sqlx::QueryBuilder::new(format!(
                "INSERT INTO {} () VALUES {} ON DUPLICATE KEY UPDATE ",
                table,
                value_groups.join(", ")
            ));
            push_set_clause_to(&on_duplicate.fields, &mut qb);
            return qb.build().execute(executor).await;
        }

        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));

        {
            let mut sep = qb.separated(", ");
            for col in &all_columns {
                sep.push(col.as_str());
            }
        }

        qb.push(") VALUES ");

        let mut first_row = true;
        for row in &self.rows {
            if !first_row {
                qb.push(", ");
            }
            first_row = false;

            qb.push("(");
            let mut first_col = true;
            for col in &all_columns {
                if !first_col {
                    qb.push(", ");
                }
                first_col = false;
                push_field_value_or_default(&row.fields, col, &mut qb, "DEFAULT");
            }
            qb.push(")");
        }

        qb.push(" ON DUPLICATE KEY UPDATE ");
        push_set_clause_to(&on_duplicate.fields, &mut qb);

        qb.build().execute(executor).await
    }
}

// ============================================================================
// PostgreSQL 特定实现
// ============================================================================

#[cfg(feature = "db-postgres")]
impl<M: TableMeta> Insert<sqlx::Postgres, M> {
    /// 执行 INSERT
    pub async fn execute<'e, E>(self, executor: E) -> Result<sqlx::postgres::PgQueryResult, Error>
    where
        E: Executor<'e, Database = sqlx::Postgres>,
    {
        let table = M::table_name().quoted();

        if self.fields.is_empty() {
            let sql = format!("INSERT INTO {} () VALUES ()", table);
            return sqlx::query(&sql).execute(executor).await;
        }

        let columns: Vec<&str> = self.fields.iter().map(|(col, _)| col.as_str()).collect();
        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));

        {
            let mut sep = qb.separated(", ");
            for col in &columns {
                sep.push(*col);
            }
        }

        qb.push(") VALUES (");
        push_values_to(&self.fields, &mut qb);
        qb.push(")");

        qb.build().execute(executor).await
    }

    /// 执行 INSERT ... ON CONFLICT DO UPDATE
    pub async fn execute_update<'e, E>(
        self,
        on_duplicate: Update<sqlx::Postgres, M>,
        executor: E,
    ) -> Result<sqlx::postgres::PgQueryResult, Error>
    where
        E: Executor<'e, Database = sqlx::Postgres>,
    {
        if on_duplicate.is_empty() {
            return Err(Error::Protocol("ON CONFLICT DO UPDATE is empty".into()));
        }

        let table = M::table_name().quoted();

        if self.fields.is_empty() {
            let conflict_column = on_duplicate
                .fields
                .first()
                .map(|(col, _)| col.as_str())
                .ok_or_else(|| {
                    Error::Protocol("At least one field required for conflict detection".into())
                })?;

            let mut qb = sqlx::QueryBuilder::new(format!(
                "INSERT INTO {} () VALUES () ON CONFLICT ({}) DO UPDATE SET ",
                table, conflict_column
            ));
            push_set_clause_to(&on_duplicate.fields, &mut qb);
            return qb.build().execute(executor).await;
        }

        let columns: Vec<&str> = self.fields.iter().map(|(col, _)| col.as_str()).collect();
        let conflict_column = columns.first().ok_or_else(|| {
            Error::Protocol("At least one insert field required for conflict detection".into())
        })?;

        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));

        {
            let mut sep = qb.separated(", ");
            for col in &columns {
                sep.push(*col);
            }
        }

        qb.push(") VALUES (");
        push_values_to(&self.fields, &mut qb);
        qb.push(format!(
            ") ON CONFLICT ({}) DO UPDATE SET ",
            conflict_column
        ));
        push_set_clause_to(&on_duplicate.fields, &mut qb);

        qb.build().execute(executor).await
    }
}

#[cfg(feature = "db-postgres")]
impl<M: TableMeta> BatchInsert<sqlx::Postgres, M> {
    /// 执行批量 INSERT
    pub async fn execute<'e, E>(self, executor: E) -> Result<sqlx::postgres::PgQueryResult, Error>
    where
        E: Executor<'e, Database = sqlx::Postgres>,
    {
        if self.rows.is_empty() {
            return Ok(sqlx::postgres::PgQueryResult::default());
        }

        let table = M::table_name().quoted();
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

        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));

        {
            let mut sep = qb.separated(", ");
            for col in &all_columns {
                sep.push(col.as_str());
            }
        }

        qb.push(") VALUES ");

        let mut first_row = true;
        for row in &self.rows {
            if !first_row {
                qb.push(", ");
            }
            first_row = false;

            qb.push("(");
            let mut first_col = true;
            for col in &all_columns {
                if !first_col {
                    qb.push(", ");
                }
                first_col = false;
                push_field_value_or_default(&row.fields, col, &mut qb, "DEFAULT");
            }
            qb.push(")");
        }

        qb.build().execute(executor).await
    }

    /// 执行批量 INSERT ... ON CONFLICT DO UPDATE
    pub async fn execute_update<'e, E>(
        self,
        on_duplicate: Update<sqlx::Postgres, M>,
        executor: E,
    ) -> Result<sqlx::postgres::PgQueryResult, Error>
    where
        E: Executor<'e, Database = sqlx::Postgres>,
    {
        if self.rows.is_empty() {
            return Ok(sqlx::postgres::PgQueryResult::default());
        }

        if on_duplicate.is_empty() {
            return Err(Error::Protocol("ON CONFLICT DO UPDATE is empty".into()));
        }

        let table = M::table_name().quoted();
        let mut all_columns: Vec<String> = Vec::new();
        for row in &self.rows {
            for (col, _) in &row.fields {
                if !all_columns.contains(col) {
                    all_columns.push(col.clone());
                }
            }
        }

        if all_columns.is_empty() {
            let conflict_column = on_duplicate
                .fields
                .first()
                .map(|(col, _)| col.as_str())
                .ok_or_else(|| {
                    Error::Protocol("At least one field required for conflict detection".into())
                })?;

            let value_groups: Vec<&str> = self.rows.iter().map(|_| "()").collect();
            let mut qb = sqlx::QueryBuilder::new(format!(
                "INSERT INTO {} () VALUES {} ON CONFLICT ({}) DO UPDATE SET ",
                table,
                value_groups.join(", "),
                conflict_column
            ));
            push_set_clause_to(&on_duplicate.fields, &mut qb);
            return qb.build().execute(executor).await;
        }

        let conflict_column = all_columns.first().map(|s| s.as_str()).ok_or_else(|| {
            Error::Protocol("At least one column required for conflict detection".into())
        })?;

        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));

        {
            let mut sep = qb.separated(", ");
            for col in &all_columns {
                sep.push(col.as_str());
            }
        }

        qb.push(") VALUES ");

        let mut first_row = true;
        for row in &self.rows {
            if !first_row {
                qb.push(", ");
            }
            first_row = false;

            qb.push("(");
            let mut first_col = true;
            for col in &all_columns {
                if !first_col {
                    qb.push(", ");
                }
                first_col = false;
                push_field_value_or_default(&row.fields, col, &mut qb, "DEFAULT");
            }
            qb.push(")");
        }

        qb.push(format!(" ON CONFLICT ({}) DO UPDATE SET ", conflict_column));
        push_set_clause_to(&on_duplicate.fields, &mut qb);

        qb.build().execute(executor).await
    }
}

// ============================================================================
// SQLite 特定实现
// ============================================================================

#[cfg(feature = "db-sqlite")]
impl<M: TableMeta> Insert<sqlx::Sqlite, M> {
    /// 执行 INSERT
    pub async fn execute<'e, E>(self, executor: E) -> Result<sqlx::sqlite::SqliteQueryResult, Error>
    where
        E: Executor<'e, Database = sqlx::Sqlite>,
    {
        let table = M::table_name().quoted();

        if self.fields.is_empty() {
            let sql = format!("INSERT INTO {} () VALUES ()", table);
            return sqlx::query(&sql).execute(executor).await;
        }

        let columns: Vec<&str> = self.fields.iter().map(|(col, _)| col.as_str()).collect();
        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));

        {
            let mut sep = qb.separated(", ");
            for col in &columns {
                sep.push(*col);
            }
        }

        qb.push(") VALUES (");
        push_values_to(&self.fields, &mut qb);
        qb.push(")");

        qb.build().execute(executor).await
    }

    /// 执行 INSERT ... ON CONFLICT DO UPDATE
    pub async fn execute_update<'e, E>(
        self,
        on_duplicate: Update<sqlx::Sqlite, M>,
        executor: E,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, Error>
    where
        E: Executor<'e, Database = sqlx::Sqlite>,
    {
        if on_duplicate.is_empty() {
            return Err(Error::Protocol("ON CONFLICT DO UPDATE is empty".into()));
        }

        let table = M::table_name().quoted();

        if self.fields.is_empty() {
            let conflict_column = on_duplicate
                .fields
                .first()
                .map(|(col, _)| col.as_str())
                .ok_or_else(|| {
                    Error::Protocol("At least one field required for conflict detection".into())
                })?;

            let mut qb = sqlx::QueryBuilder::new(format!(
                "INSERT INTO {} () VALUES () ON CONFLICT ({}) DO UPDATE SET ",
                table, conflict_column
            ));
            push_set_clause_to(&on_duplicate.fields, &mut qb);
            return qb.build().execute(executor).await;
        }

        let columns: Vec<&str> = self.fields.iter().map(|(col, _)| col.as_str()).collect();
        let conflict_column = columns.first().ok_or_else(|| {
            Error::Protocol("At least one insert field required for conflict detection".into())
        })?;

        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));

        {
            let mut sep = qb.separated(", ");
            for col in &columns {
                sep.push(*col);
            }
        }

        qb.push(") VALUES (");
        push_values_to(&self.fields, &mut qb);
        qb.push(format!(
            ") ON CONFLICT ({}) DO UPDATE SET ",
            conflict_column
        ));
        push_set_clause_to(&on_duplicate.fields, &mut qb);

        qb.build().execute(executor).await
    }
}

#[cfg(feature = "db-sqlite")]
impl<M: TableMeta> BatchInsert<sqlx::Sqlite, M> {
    /// 执行批量 INSERT
    pub async fn execute<'e, E>(self, executor: E) -> Result<sqlx::sqlite::SqliteQueryResult, Error>
    where
        E: Executor<'e, Database = sqlx::Sqlite>,
    {
        if self.rows.is_empty() {
            return Ok(sqlx::sqlite::SqliteQueryResult::default());
        }

        let table = M::table_name().quoted();
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

        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));

        {
            let mut sep = qb.separated(", ");
            for col in &all_columns {
                sep.push(col.as_str());
            }
        }

        qb.push(") VALUES ");

        let mut first_row = true;
        for row in &self.rows {
            if !first_row {
                qb.push(", ");
            }
            first_row = false;

            qb.push("(");
            let mut first_col = true;
            for col in &all_columns {
                if !first_col {
                    qb.push(", ");
                }
                first_col = false;
                push_field_value_or_default(&row.fields, col, &mut qb, "DEFAULT");
            }
            qb.push(")");
        }

        qb.build().execute(executor).await
    }

    /// 执行批量 INSERT ... ON CONFLICT DO UPDATE
    pub async fn execute_update<'e, E>(
        self,
        on_duplicate: Update<sqlx::Sqlite, M>,
        executor: E,
    ) -> Result<sqlx::sqlite::SqliteQueryResult, Error>
    where
        E: Executor<'e, Database = sqlx::Sqlite>,
    {
        if self.rows.is_empty() {
            return Ok(sqlx::sqlite::SqliteQueryResult::default());
        }

        if on_duplicate.is_empty() {
            return Err(Error::Protocol("ON CONFLICT DO UPDATE is empty".into()));
        }

        let table = M::table_name().quoted();
        let mut all_columns: Vec<String> = Vec::new();
        for row in &self.rows {
            for (col, _) in &row.fields {
                if !all_columns.contains(col) {
                    all_columns.push(col.clone());
                }
            }
        }

        if all_columns.is_empty() {
            let conflict_column = on_duplicate
                .fields
                .first()
                .map(|(col, _)| col.as_str())
                .ok_or_else(|| {
                    Error::Protocol("At least one field required for conflict detection".into())
                })?;

            let value_groups: Vec<&str> = self.rows.iter().map(|_| "()").collect();
            let mut qb = sqlx::QueryBuilder::new(format!(
                "INSERT INTO {} () VALUES {} ON CONFLICT ({}) DO UPDATE SET ",
                table,
                value_groups.join(", "),
                conflict_column
            ));
            push_set_clause_to(&on_duplicate.fields, &mut qb);
            return qb.build().execute(executor).await;
        }

        let conflict_column = all_columns.first().map(|s| s.as_str()).ok_or_else(|| {
            Error::Protocol("At least one column required for conflict detection".into())
        })?;

        let mut qb = sqlx::QueryBuilder::new(format!("INSERT INTO {} (", table));

        {
            let mut sep = qb.separated(", ");
            for col in &all_columns {
                sep.push(col.as_str());
            }
        }

        qb.push(") VALUES ");

        let mut first_row = true;
        for row in &self.rows {
            if !first_row {
                qb.push(", ");
            }
            first_row = false;

            qb.push("(");
            let mut first_col = true;
            for col in &all_columns {
                if !first_col {
                    qb.push(", ");
                }
                first_col = false;
                push_field_value_or_default(&row.fields, col, &mut qb, "DEFAULT");
            }
            qb.push(")");
        }

        qb.push(format!(" ON CONFLICT ({}) DO UPDATE SET ", conflict_column));
        push_set_clause_to(&on_duplicate.fields, &mut qb);

        qb.build().execute(executor).await
    }
}
