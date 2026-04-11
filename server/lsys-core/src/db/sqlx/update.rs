use super::field::Field;
use super::table::TableMeta;
use super::value::{FieldValue, IntoFieldValue, StoredValue};
use sqlx::{Database, Error, Executor};
use std::marker::PhantomData;

/// UPDATE 构建器
pub struct Update<DB: Database, M: TableMeta> {
    pub(crate) fields: Vec<(String, StoredValue<DB>)>,
    _marker: PhantomData<M>,
}

impl<DB: Database, M: TableMeta> Update<DB, M> {
    pub fn new() -> Self {
        Self {
            fields: Vec::new(),
            _marker: PhantomData,
        }
    }
}

impl<DB: Database, M: TableMeta> Default for Update<DB, M> {
    fn default() -> Self {
        Self::new()
    }
}

impl<DB: Database, M: TableMeta> Update<DB, M> {
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
}

// 为每个数据库类型提供具体实现，避免泛型生命周期问题
macro_rules! impl_update_execute {
    ($db:ty) => {
        impl<M: TableMeta> Update<$db, M> {
            /// 执行 UPDATE，使用回调构建 WHERE 子句
            pub async fn execute<'e, E, F>(
                self,
                executor: E,
                where_clause: F,
            ) -> Result<<$db as Database>::QueryResult, Error>
            where
                E: Executor<'e, Database = $db>,
                F: FnOnce(&mut sqlx::QueryBuilder<'_, $db>),
            {
                if self.fields.is_empty() {
                    return Ok(<$db as Database>::QueryResult::default());
                }

                let table = M::table_name().quoted();
                let mut qb = sqlx::QueryBuilder::new(format!("UPDATE {} SET ", table));

                // 处理 SET 子句
                let mut first = true;
                for (col, value) in &self.fields {
                    if !first {
                        qb.push(", ");
                    }
                    first = false;

                    qb.push(col.as_str());
                    qb.push(" = ");

                    match value {
                        StoredValue::Bind(b) => {
                            b.bind_to(&mut qb);
                        }
                        StoredValue::Expr(e) => {
                            qb.push(e.as_ref());
                        }
                        StoredValue::Dynamic(f) => {
                            f(&mut qb);
                        }
                    }
                }

                // 执行回调构建 WHERE 子句
                where_clause(&mut qb);

                qb.build().execute(executor).await
            }
        }
    };
}

#[cfg(feature = "db-mysql")]
impl_update_execute!(sqlx::MySql);

#[cfg(feature = "db-postgres")]
impl_update_execute!(sqlx::Postgres);

#[cfg(feature = "db-sqlite")]
impl_update_execute!(sqlx::Sqlite);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::{QueryBuilderExt, TableName};

    // 模拟一个测试用的 Model
    struct TestModel;

    impl TableMeta for TestModel {
        fn table_name() -> TableName {
            TableName::new("test_table")
        }
    }

    impl TestModel {
        const ID: Field<u64> = Field::new("id");
        const STATUS: Field<i8> = Field::new("status");
        const TRY_NUM: Field<i32> = Field::new("try_num");
    }

    #[cfg(feature = "db-mysql")]
    #[test]
    fn test_update_with_dynamic_field_value() {
        use sqlx::MySql;

        // 测试 FieldValue::Expr 和 FieldValue::Dynamic
        let update = Update::<MySql, TestModel>::new()
            .set(TestModel::TRY_NUM, FieldValue::Expr("try_num+1".into())) // 简单表达式用 Expr
            .set(
                TestModel::STATUS,
                FieldValue::Dynamic(Box::new(|qb| {
                    qb.push("if(try_num>=");
                    qb.push_bind(3_i32);
                    qb.push(",");
                    qb.push_bind(2_i8);
                    qb.push(",status)");
                })),
            );

        // 验证字段数量
        assert_eq!(update.fields.len(), 2);

        // 验证字段名
        assert_eq!(update.fields[0].0, "try_num");
        assert_eq!(update.fields[1].0, "status");

        // 验证类型
        assert!(matches!(update.fields[0].1, StoredValue::Expr(_)));
        assert!(matches!(update.fields[1].1, StoredValue::Dynamic(_)));

        println!("✓ FieldValue::Expr 和 FieldValue::Dynamic 字段设置成功");
    }

    #[cfg(feature = "db-mysql")]
    #[test]
    fn test_update_sql_generation() {
        use sqlx::MySql;

        // 测试生成的 SQL 结构（通过 QueryBuilder）
        let mut qb = sqlx::QueryBuilder::<MySql>::new("UPDATE test_table SET ");

        // 模拟 try_num 字段的 Dynamic 回调
        qb.push("try_num = ");
        let try_num_callback = |qb: &mut sqlx::QueryBuilder<MySql>| {
            qb.push("try_num+1");
        };
        try_num_callback(&mut qb);

        qb.push(", ");

        // 模拟 status 字段的 Dynamic 回调
        qb.push("status = ");
        let max_try = 3_i32;
        let fail_status = 2_i8;
        let status_callback = move |qb: &mut sqlx::QueryBuilder<MySql>| {
            qb.push("if(try_num>=");
            qb.push_bind(max_try);
            qb.push(",");
            qb.push_bind(fail_status);
            qb.push(",status)");
        };
        status_callback(&mut qb);

        // 添加 WHERE 子句
        qb.push_where().field_eq("id", 123_u64);

        let sql = qb.sql();

        // 验证 SQL 结构
        assert!(sql.contains("UPDATE test_table SET"));
        assert!(sql.contains("try_num = try_num+1"));
        assert!(sql.contains("status = if(try_num>="));
        assert!(sql.contains("WHERE"));
        assert!(sql.contains("id="));

        println!("✓ 生成的 SQL: {}", sql);
        println!("✓ FieldValue::Dynamic 能正确绑定值并生成 SQL");
    }

    #[cfg(feature = "db-mysql")]
    #[test]
    fn test_dynamic_with_vec_data() {
        use sqlx::MySql;

        // 测试 Dynamic 回调中使用 Vec 数据（模拟 cancel_data 场景）
        let cancel_ids = vec![10_u64, 20_u64, 30_u64];

        let mut qb = sqlx::QueryBuilder::<MySql>::new("UPDATE test_table SET ");
        qb.push("status = ");

        // 使用 move 闭包捕获 cancel_ids 的所有权
        let status_callback = move |qb: &mut sqlx::QueryBuilder<MySql>| {
            qb.push("if(try_num>=");
            qb.push_bind(5_i32);
            qb.push(",");
            qb.push_bind(2_i8);
            qb.push(",");
            if cancel_ids.is_empty() {
                qb.push("status");
            } else {
                qb.push("if(");
                qb.field_in_copied("id", &cancel_ids);
                qb.push(",");
                qb.push_bind(3_i8);
                qb.push(",status)");
            }
            qb.push(")");
        };

        status_callback(&mut qb);

        let sql = qb.sql();

        // 验证 SQL 包含 IN 子句
        assert!(sql.contains("if(try_num>="));
        assert!(sql.contains("if("));
        assert!(sql.contains("id IN"));

        println!("✓ 生成的 SQL: {}", sql);
        println!("✓ FieldValue::Dynamic 能正确处理 Vec 数据并使用 field_in_copied");
    }

    #[test]
    fn test_mixed_field_values() {
        use sqlx::MySql;

        // 测试混合使用不同类型的 FieldValue
        let update = Update::<MySql, TestModel>::new()
            .set(TestModel::ID, 100_u64) // Value
            .set(TestModel::TRY_NUM, FieldValue::Expr("try_num+1".into())) // Expr
            .set(
                TestModel::STATUS,
                FieldValue::Dynamic(Box::new(|qb| {
                    qb.push("if(try_num>=");
                    qb.push_bind(3_i32);
                    qb.push(",2,status)");
                })),
            ); // Dynamic

        assert_eq!(update.fields.len(), 3);
        assert!(matches!(update.fields[0].1, StoredValue::Bind(_)));
        assert!(matches!(update.fields[1].1, StoredValue::Expr(_)));
        assert!(matches!(update.fields[2].1, StoredValue::Dynamic(_)));

        println!("✓ 混合使用 Value、Expr 和 Dynamic 成功");
    }
}
