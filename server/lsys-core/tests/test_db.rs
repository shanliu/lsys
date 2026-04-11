// Integration tests for the `db` module of lsys-core.

// =============================================================================
// 1. sql_tools — pagination types (no feature gate needed)
// =============================================================================

use lsys_core::db::{
    CursorPageDir, CursorPageSort, DEFAULT_TOTAL_COUNT_THRESHOLD, OffsetPageParam, OffsetPageValue,
    TotalParam, TotalRow,
};

// ── OffsetPageValue ──────────────────────────────────────────────────────────

#[test]
fn offset_page_value_page1() {
    let pv = OffsetPageValue::page(1, 10);
    assert_eq!(pv.offset, 0);
    assert_eq!(pv.limit, 10);
}

#[test]
fn offset_page_value_page2() {
    let pv = OffsetPageValue::page(2, 10);
    assert_eq!(pv.offset, 10);
    assert_eq!(pv.limit, 10);
}

#[test]
fn offset_page_value_page0_edge() {
    // page 0 should clamp offset to 0
    let pv = OffsetPageValue::page(0, 10);
    assert_eq!(pv.offset, 0);
    assert_eq!(pv.limit, 10);
}

#[test]
fn offset_page_value_large_page() {
    let pv = OffsetPageValue::page(100, 25);
    assert_eq!(pv.offset, (100 - 1) * 25);
    assert_eq!(pv.limit, 25);
}

#[test]
fn offset_page_value_new_direct() {
    let pv = OffsetPageValue::new(42, 7);
    assert_eq!(pv.offset, 42);
    assert_eq!(pv.limit, 7);
}

// ── OffsetPageParam ──────────────────────────────────────────────────────────

#[test]
fn offset_page_param_some() {
    let param = OffsetPageParam::new(Some(OffsetPageValue::page(3, 20)));
    let pv = param.page_value().unwrap();
    assert_eq!(pv.offset, 40);
    assert_eq!(pv.limit, 20);
}

#[test]
fn offset_page_param_none() {
    let param = OffsetPageParam::new(None);
    assert!(param.page_value().is_none());
}

// ── TotalRow / TotalParam ────────────────────────────────────────────────────

#[test]
fn total_row_exact() {
    let row = TotalRow::Exact(42);
    assert!(row.is_exact());
}

#[test]
fn total_row_over() {
    let row = TotalRow::Over(10000);
    assert!(!row.is_exact());
}

#[test]
fn total_param_default_is_threshold() {
    let param = TotalParam::default();
    match param {
        TotalParam::Threshold(v) => assert_eq!(v, DEFAULT_TOTAL_COUNT_THRESHOLD),
        _ => panic!("default should be Threshold"),
    }
}

#[test]
fn total_param_full() {
    let param = TotalParam::Full;
    let query = param.total_count_query();
    assert!(!query.is_threshold_mode());
    assert!(query.threshold_limit().is_none());

    let result = query.finalize(999);
    assert!(result.is_exact());
}

#[test]
fn total_param_threshold_under() {
    let param = TotalParam::Threshold(100);
    let query = param.total_count_query();
    assert!(query.is_threshold_mode());
    assert_eq!(query.threshold_limit(), Some(101));

    let result = query.finalize(50);
    assert!(result.is_exact());
}

#[test]
fn total_param_threshold_over() {
    let param = TotalParam::Threshold(100);
    let query = param.total_count_query();
    let result = query.finalize(101);
    assert!(!result.is_exact());
}

// ── CursorPageSort / CursorPageDir ───────────────────────────────────────────

#[test]
fn cursor_page_sort_sql() {
    assert_eq!(CursorPageSort::Asc.as_sql(), "asc");
    assert_eq!(CursorPageSort::Desc.as_sql(), "desc");
}

#[test]
fn cursor_page_dir_variants() {
    assert_eq!(CursorPageDir::Next, CursorPageDir::Next);
    assert_eq!(CursorPageDir::Prev, CursorPageDir::Prev);
    assert_ne!(CursorPageDir::Next, CursorPageDir::Prev);
}

// =============================================================================
// 2. Field / FieldMeta / TableName — require "db" feature
// =============================================================================

#[cfg(feature = "db")]
mod db_struct_tests {
    use lsys_core::db::{Field, FieldMeta, TableMeta, TableName};
    use std::ops::Deref;

    // ── FieldMeta ────────────────────────────────────────────────────────────

    #[test]
    fn field_meta_same() {
        let fm = FieldMeta::same("status");
        assert_eq!(fm.name.as_ref(), "status");
        assert_eq!(fm.column.as_ref(), "status");
    }

    #[test]
    fn field_meta_different_column() {
        let fm = FieldMeta::new("createdAt", "created_at");
        assert_eq!(fm.name.as_ref(), "createdAt");
        assert_eq!(fm.column.as_ref(), "created_at");
    }

    #[test]
    fn field_meta_clone_eq() {
        let a = FieldMeta::same("id");
        let b = a.clone();
        assert_eq!(a, b);
    }

    // ── Field<T> ─────────────────────────────────────────────────────────────

    #[test]
    fn field_new_name_equals_column() {
        let f: Field<i32> = Field::new("age");
        assert_eq!(f.name.as_ref(), "age");
        assert_eq!(f.column.as_ref(), "age");
    }

    #[test]
    fn field_with_column() {
        let f: Field<String> = Field::with_column("userName", "user_name");
        assert_eq!(f.name.as_ref(), "userName");
        assert_eq!(f.column.as_ref(), "user_name");
    }

    #[test]
    fn field_meta_accessor() {
        let f: Field<u64> = Field::new("id");
        let meta = f.meta();
        assert_eq!(meta.name.as_ref(), "id");
    }

    #[test]
    fn field_deref_to_meta() {
        let f: Field<bool> = Field::new("active");
        let meta: &FieldMeta = f.deref();
        assert_eq!(meta.column.as_ref(), "active");
    }

    #[test]
    fn field_display() {
        let f: Field<i64> = Field::with_column("myField", "my_column");
        assert_eq!(format!("{}", f), "my_column");
    }

    #[test]
    fn field_clone() {
        let f: Field<i32> = Field::new("x");
        let f2 = f.clone();
        assert_eq!(f.meta(), f2.meta());
    }

    #[test]
    fn field_from_meta() {
        let meta = FieldMeta::new("a", "b");
        let f: Field<String> = Field::from_meta(meta.clone());
        assert_eq!(f.meta(), &meta);
    }

    // ── TableName ────────────────────────────────────────────────────────────

    #[test]
    fn table_name_simple() {
        let tn = TableName::new("users");
        assert_eq!(tn.raw_name(), "users");
        assert_eq!(tn.db_part(), "");
        assert_eq!(tn.full_name(), "users");
    }

    #[test]
    fn table_name_with_db_dot() {
        let tn = TableName::new("mydb.users");
        assert_eq!(tn.raw_name(), "users");
        assert_eq!(tn.db_part(), "mydb.");
        assert_eq!(tn.full_name(), "mydb.users");
    }

    #[test]
    fn table_name_with_db_constructor() {
        let tn = TableName::with_db("other_db.", "items");
        assert_eq!(tn.raw_name(), "items");
        assert_eq!(tn.db_part(), "other_db.");
        assert_eq!(tn.full_name(), "other_db.items");
    }

    #[test]
    fn table_name_quoted_simple() {
        let tn = TableName::new("users");
        // simple identifier — no quoting
        assert_eq!(tn.quoted(), "users");
    }

    #[test]
    fn table_name_quoted_with_db() {
        let tn = TableName::new("mydb.users");
        // contains a dot, still valid simple identifier chars
        assert_eq!(tn.quoted(), "mydb.users");
    }

    #[test]
    fn table_name_display() {
        let tn = TableName::new("orders");
        assert_eq!(format!("{}", tn), "orders");
    }

    #[test]
    fn table_name_clone_eq() {
        let a = TableName::new("t");
        let b = a.clone();
        assert_eq!(a, b);
    }

    // ── TableMeta impl ───────────────────────────────────────────────────────

    struct DummyModel;
    impl TableMeta for DummyModel {
        fn table_name() -> TableName {
            TableName::new("dummy_table")
        }
    }

    #[test]
    fn table_meta_trait() {
        assert_eq!(DummyModel::table_name().raw_name(), "dummy_table");
    }
}

// =============================================================================
// 3. FieldValue — require "db" + a concrete DB backend
// =============================================================================

#[cfg(feature = "db-sqlite")]
mod field_value_tests {
    use lsys_core::db::{FieldValue, IntoFieldValue};
    use sqlx::Sqlite;

    #[test]
    fn field_value_skip() {
        let v: FieldValue<Sqlite, i32> = FieldValue::Skip;
        assert!(v.is_skip());
    }

    #[test]
    fn field_value_value_not_skip() {
        let v: FieldValue<Sqlite, i32> = FieldValue::Value(42);
        assert!(!v.is_skip());
    }

    #[test]
    fn field_value_expr() {
        let v: FieldValue<Sqlite, i32> = FieldValue::expr("NOW()");
        assert!(!v.is_skip());
    }

    #[test]
    fn into_field_value_i32() {
        let v: FieldValue<Sqlite, i32> = 42_i32.into_field_value();
        match v {
            FieldValue::Value(x) => assert_eq!(x, 42),
            _ => panic!("expected Value"),
        }
    }

    #[test]
    fn into_field_value_string() {
        let v: FieldValue<Sqlite, String> = "hello".into_field_value();
        match v {
            FieldValue::Value(s) => assert_eq!(s, "hello"),
            _ => panic!("expected Value"),
        }
    }

    #[test]
    fn into_field_value_option_none() {
        let v: FieldValue<Sqlite, Option<i32>> = None::<i32>.into_field_value();
        match v {
            FieldValue::Value(opt) => assert!(opt.is_none()),
            _ => panic!("expected Value(None)"),
        }
    }

    #[test]
    fn into_field_value_option_some() {
        let v: FieldValue<Sqlite, Option<i32>> = Some(7_i32).into_field_value();
        match v {
            FieldValue::Value(opt) => assert_eq!(opt, Some(7)),
            _ => panic!("expected Value(Some(7))"),
        }
    }

    #[test]
    fn field_value_debug() {
        let v: FieldValue<Sqlite, i32> = FieldValue::Skip;
        let dbg = format!("{:?}", v);
        assert!(dbg.contains("Skip"));
    }
}

// =============================================================================
// 4. QueryBuilderExt — SQL generation (requires a concrete DB backend)
// =============================================================================

#[cfg(feature = "db-sqlite")]
mod query_builder_ext_sqlite {
    use lsys_core::db::QueryBuilderExt;
    use sqlx::{QueryBuilder, Sqlite};

    #[test]
    fn push_where_and_field_eq() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM t");
        qb.push_where().field_eq("status", 1_i32);
        let sql = qb.into_sql();
        assert!(sql.contains("WHERE"), "sql: {sql}");
        assert!(sql.contains("status="), "sql: {sql}");
    }

    #[test]
    fn push_and_or() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM t");
        qb.push_where()
            .field_eq("a", 1_i32)
            .push_and()
            .field_ne("b", 2_i32)
            .push_or()
            .field_gt("c", 3_i32);
        let sql = qb.into_sql();
        assert!(sql.contains(" WHERE "), "sql: {sql}");
        assert!(sql.contains(" AND "), "sql: {sql}");
        assert!(sql.contains(" OR "), "sql: {sql}");
        assert!(sql.contains("a="), "sql: {sql}");
        assert!(sql.contains("b!="), "sql: {sql}");
        assert!(sql.contains("c>"), "sql: {sql}");
    }

    #[test]
    fn comparison_operators() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT 1");
        qb.push_where()
            .field_gte("x", 10_i64)
            .push_and()
            .field_lt("y", 20_i64)
            .push_and()
            .field_lte("z", 30_i64);
        let sql = qb.into_sql();
        assert!(sql.contains("x>="), "sql: {sql}");
        assert!(sql.contains("y<"), "sql: {sql}");
        assert!(sql.contains("z<="), "sql: {sql}");
    }

    #[test]
    fn field_like() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT 1");
        qb.push_where().field_like("name", "%test%".to_string());
        let sql = qb.into_sql();
        assert!(sql.contains("name LIKE "), "sql: {sql}");
    }

    #[test]
    fn field_in_values() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT 1");
        qb.push_where().field_in("id", vec![1_i32, 2, 3]);
        let sql = qb.into_sql();
        assert!(sql.contains("id IN ("), "sql: {sql}");
    }

    #[test]
    fn field_in_copied() {
        let ids = [10_i64, 20, 30];
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT 1");
        qb.push_where().field_in_copied("code", &ids);
        let sql = qb.into_sql();
        assert!(sql.contains("code IN ("), "sql: {sql}");
    }

    #[test]
    fn field_in_string() {
        let tags = ["alpha", "beta"];
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT 1");
        qb.push_where().field_in_string("tag", &tags);
        let sql = qb.into_sql();
        assert!(sql.contains("tag IN ("), "sql: {sql}");
    }

    #[test]
    fn field_not_in() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT 1");
        qb.push_where().field_not_in("id", vec![1_i32, 2]);
        let sql = qb.into_sql();
        assert!(sql.contains("id NOT IN ("), "sql: {sql}");
    }

    #[test]
    fn field_not_in_copied() {
        let ids = [5_i64, 6];
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT 1");
        qb.push_where().field_not_in_copied("x", &ids);
        let sql = qb.into_sql();
        assert!(sql.contains("x NOT IN ("), "sql: {sql}");
    }

    #[test]
    fn field_not_in_string() {
        let vals = ["a", "b"];
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT 1");
        qb.push_where().field_not_in_string("col", &vals);
        let sql = qb.into_sql();
        assert!(sql.contains("col NOT IN ("), "sql: {sql}");
    }

    #[test]
    fn field_is_null_and_not_null() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT 1");
        qb.push_where()
            .field_is_null("deleted_at")
            .push_and()
            .field_not_null("name");
        let sql = qb.into_sql();
        assert!(sql.contains("deleted_at IS NULL"), "sql: {sql}");
        assert!(sql.contains("name IS NOT NULL"), "sql: {sql}");
    }

    #[test]
    fn push_list() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT 1 WHERE id IN ");
        qb.push_list(vec![10_i32, 20, 30]);
        let sql = qb.into_sql();
        assert!(sql.contains("("), "sql: {sql}");
    }
}

// Repeat a subset for MySQL to verify cross-backend
#[cfg(feature = "db-mysql")]
mod query_builder_ext_mysql {
    use lsys_core::db::QueryBuilderExt;
    use sqlx::{MySql, QueryBuilder};

    #[test]
    fn field_eq_mysql() {
        let mut qb = QueryBuilder::<MySql>::new("SELECT * FROM t");
        qb.push_where().field_eq("id", 1_i64);
        let sql = qb.into_sql();
        assert!(sql.contains(" WHERE "), "sql: {sql}");
        assert!(sql.contains("id="), "sql: {sql}");
    }

    #[test]
    fn field_in_mysql() {
        let mut qb = QueryBuilder::<MySql>::new("SELECT 1");
        qb.push_where().field_in("role", vec![1_i32, 2, 3]);
        let sql = qb.into_sql();
        assert!(sql.contains("role IN ("), "sql: {sql}");
    }

    #[test]
    fn field_is_null_mysql() {
        let mut qb = QueryBuilder::<MySql>::new("SELECT 1");
        qb.push_where().field_is_null("col");
        let sql = qb.into_sql();
        assert!(sql.contains("col IS NULL"), "sql: {sql}");
    }
}

// Repeat a subset for Postgres
#[cfg(feature = "db-postgres")]
mod query_builder_ext_postgres {
    use lsys_core::db::QueryBuilderExt;
    use sqlx::{Postgres, QueryBuilder};

    #[test]
    fn field_eq_postgres() {
        let mut qb = QueryBuilder::<Postgres>::new("SELECT * FROM t");
        qb.push_where().field_eq("id", 1_i64);
        let sql = qb.into_sql();
        assert!(sql.contains(" WHERE "), "sql: {sql}");
        assert!(sql.contains("id="), "sql: {sql}");
    }

    #[test]
    fn field_not_null_postgres() {
        let mut qb = QueryBuilder::<Postgres>::new("SELECT 1");
        qb.push_where().field_not_null("email");
        let sql = qb.into_sql();
        assert!(sql.contains("email IS NOT NULL"), "sql: {sql}");
    }
}

// =============================================================================
// 5. WhereClause — auto WHERE/AND management
// =============================================================================

#[cfg(feature = "db-sqlite")]
mod where_clause_tests {
    use lsys_core::db::{QueryBuilderExt, WhereClause};
    use sqlx::{QueryBuilder, Sqlite};

    #[test]
    fn first_call_inserts_where() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM t");
        let mut wc = WhereClause::new(&mut qb);
        assert!(!wc.has_condition());
        wc.and().field_eq("a", 1_i32);
        assert!(wc.has_condition());
        let sql = wc.builder().sql().to_string();
        assert!(sql.contains(" WHERE "), "sql: {sql}");
    }

    #[test]
    fn second_call_inserts_and() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM t");
        {
            let mut wc = WhereClause::new(&mut qb);
            wc.and().field_eq("a", 1_i32);
            wc.and().field_eq("b", 2_i32);
        }
        let sql = qb.into_sql();
        assert!(sql.contains(" WHERE "), "sql: {sql}");
        assert!(sql.contains(" AND "), "sql: {sql}");
    }

    #[test]
    fn or_connector() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM t");
        {
            let mut wc = WhereClause::new(&mut qb);
            wc.and().field_eq("x", 1_i32);
            wc.or().field_eq("y", 2_i32);
        }
        let sql = qb.into_sql();
        assert!(sql.contains(" WHERE "), "sql: {sql}");
        assert!(sql.contains(" OR "), "sql: {sql}");
    }

    #[test]
    fn no_conditions_no_where() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM t");
        {
            let wc = WhereClause::new(&mut qb);
            assert!(!wc.has_condition());
        }
        let sql = qb.into_sql();
        assert!(!sql.contains("WHERE"), "sql: {sql}");
    }

    #[test]
    fn set_condition_presets_state() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM t WHERE 1=1");
        {
            let mut wc = WhereClause::new(&mut qb);
            wc.set_condition(true);
            // Next call should produce AND, not WHERE
            wc.and().field_eq("z", 5_i32);
        }
        let sql = qb.into_sql();
        // Should NOT have a second WHERE
        let count = sql.matches("WHERE").count();
        assert_eq!(count, 1, "sql: {sql}");
        assert!(sql.contains(" AND "), "sql: {sql}");
    }

    #[test]
    fn split_custom_separator() {
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM t");
        {
            let mut wc = WhereClause::new(&mut qb);
            wc.and().field_eq("a", 1_i32);
            wc.split(" OR ").field_eq("b", 2_i32);
        }
        let sql = qb.into_sql();
        assert!(sql.contains(" OR "), "sql: {sql}");
    }
}

// =============================================================================
// 6. Insert / Update — SQL generation (no real DB needed)
// =============================================================================

#[cfg(feature = "db-sqlite")]
mod insert_update_tests {
    use lsys_core::db::{Field, FieldValue, Insert, QueryBuilderExt, TableMeta, TableName, Update};
    use sqlx::Sqlite;

    struct TestModel;
    impl TableMeta for TestModel {
        fn table_name() -> TableName {
            TableName::new("test_items")
        }
    }

    impl TestModel {
        const ID: Field<i64> = Field::new("id");
        const NAME: Field<String> = Field::new("name");
        const STATUS: Field<i32> = Field::new("status");
    }

    // ── Insert ───────────────────────────────────────────────────────────────

    #[test]
    fn insert_is_empty_initially() {
        let ins = Insert::<Sqlite, TestModel>::new();
        assert!(ins.is_empty());
    }

    #[test]
    fn insert_set_adds_field() {
        let ins = Insert::<Sqlite, TestModel>::new()
            .set(TestModel::ID, 1_i64)
            .set(TestModel::NAME, "hello".to_string());
        assert!(!ins.is_empty());
    }

    #[test]
    fn insert_skip_removes_field() {
        let ins = Insert::<Sqlite, TestModel>::new()
            .set(TestModel::ID, 1_i64)
            .set(TestModel::ID, FieldValue::<Sqlite, i64>::Skip);
        assert!(ins.is_empty());
    }

    #[test]
    fn insert_set_overrides_same_field() {
        let ins = Insert::<Sqlite, TestModel>::new()
            .set(TestModel::STATUS, 1_i32)
            .set(TestModel::STATUS, 2_i32);
        // Should have exactly 1 field entry
        assert_eq!(ins.fields.len(), 1);
    }

    #[test]
    fn insert_expr_value() {
        let ins = Insert::<Sqlite, TestModel>::new()
            .set(TestModel::STATUS, FieldValue::<Sqlite, i32>::expr("0"));
        assert!(!ins.is_empty());
    }

    // ── Update ───────────────────────────────────────────────────────────────

    #[test]
    fn update_is_empty_initially() {
        let upd = Update::<Sqlite, TestModel>::new();
        assert!(upd.is_empty());
    }

    #[test]
    fn update_set_adds_field() {
        let upd = Update::<Sqlite, TestModel>::new().set(TestModel::NAME, "world".to_string());
        assert!(!upd.is_empty());
    }

    #[test]
    fn update_skip_removes_field() {
        let upd = Update::<Sqlite, TestModel>::new()
            .set(TestModel::STATUS, 1_i32)
            .set(TestModel::STATUS, FieldValue::<Sqlite, i32>::Skip);
        assert!(upd.is_empty());
    }

    #[test]
    fn update_set_overrides_same_field() {
        let upd = Update::<Sqlite, TestModel>::new()
            .set(TestModel::STATUS, 10_i32)
            .set(TestModel::STATUS, 20_i32);
        assert_eq!(upd.fields.len(), 1);
    }

    #[test]
    fn update_expr_field() {
        let upd = Update::<Sqlite, TestModel>::new().set(
            TestModel::STATUS,
            FieldValue::<Sqlite, i32>::expr("status+1"),
        );
        assert!(!upd.is_empty());
    }

    #[test]
    fn update_dynamic_field() {
        let upd = Update::<Sqlite, TestModel>::new().set(
            TestModel::STATUS,
            FieldValue::<Sqlite, i32>::dynamic(|qb| {
                qb.push("CASE WHEN status > 0 THEN 1 ELSE 0 END");
            }),
        );
        assert!(!upd.is_empty());
    }

    #[test]
    fn update_mixed_field_values() {
        let upd = Update::<Sqlite, TestModel>::new()
            .set(TestModel::ID, 100_i64) // Value
            .set(
                TestModel::NAME,
                FieldValue::<Sqlite, String>::expr("'default'"),
            ) // Expr
            .set(
                TestModel::STATUS,
                FieldValue::<Sqlite, i32>::dynamic(|qb| {
                    qb.push("0");
                }),
            ); // Dynamic
        assert_eq!(upd.fields.len(), 3);
    }

    // ── BatchInsert ──────────────────────────────────────────────────────────

    #[test]
    fn batch_insert_empty() {
        use lsys_core::db::BatchInsert;
        let batch = BatchInsert::<Sqlite, TestModel>::new();
        assert!(batch.is_empty());
    }

    #[test]
    fn batch_insert_push() {
        use lsys_core::db::BatchInsert;
        let row1 = Insert::<Sqlite, TestModel>::new()
            .set(TestModel::ID, 1_i64)
            .set(TestModel::NAME, "a".to_string());
        let row2 = Insert::<Sqlite, TestModel>::new()
            .set(TestModel::ID, 2_i64)
            .set(TestModel::NAME, "b".to_string());
        let batch = BatchInsert::<Sqlite, TestModel>::new()
            .push(row1)
            .push(row2);
        assert!(!batch.is_empty());
    }
}

// MySQL-specific Insert/Update tests
#[cfg(feature = "db-mysql")]
mod insert_update_mysql_tests {
    use lsys_core::db::{Field, Insert, TableMeta, TableName, Update};
    use sqlx::MySql;

    struct MysqlModel;
    impl TableMeta for MysqlModel {
        fn table_name() -> TableName {
            TableName::new("mysql_test")
        }
    }
    impl MysqlModel {
        const ID: Field<u64> = Field::new("id");
        const VAL: Field<String> = Field::new("val");
    }

    #[test]
    fn mysql_insert_build() {
        let ins = Insert::<MySql, MysqlModel>::new()
            .set(MysqlModel::ID, 1_u64)
            .set(MysqlModel::VAL, "test".to_string());
        assert!(!ins.is_empty());
    }

    #[test]
    fn mysql_update_build() {
        let upd = Update::<MySql, MysqlModel>::new().set(MysqlModel::VAL, "updated".to_string());
        assert!(!upd.is_empty());
    }
}

// =============================================================================
// 7. OffsetPageParam::push_limit (requires "db" feature + a backend)
// =============================================================================

#[cfg(feature = "db-sqlite")]
mod offset_page_push_limit_tests {
    use lsys_core::db::OffsetPageParam;
    use lsys_core::db::OffsetPageValue;
    use sqlx::{QueryBuilder, Sqlite};

    #[test]
    fn push_limit_some() {
        let param = OffsetPageParam::new(Some(OffsetPageValue::page(2, 10)));
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM t");
        param.push_limit(&mut qb);
        let sql = qb.into_sql();
        assert!(sql.contains("limit 10"), "sql: {sql}");
        assert!(sql.contains("offset 10"), "sql: {sql}");
    }

    #[test]
    fn push_limit_none() {
        let param = OffsetPageParam::new(None);
        let mut qb = QueryBuilder::<Sqlite>::new("SELECT * FROM t");
        param.push_limit(&mut qb);
        let sql = qb.into_sql();
        assert!(!sql.contains("limit"), "sql: {sql}");
    }
}

// =============================================================================
// 8. Integration tests — actual SQLite database
// =============================================================================

#[cfg(feature = "db-sqlite")]
mod sqlite_integration {
    use lsys_core::db::{
        Field, FieldValue, Insert, QueryBuilderExt, TableMeta, TableName, Update, WhereClause,
    };
    use sqlx::{Row, Sqlite, SqlitePool};

    struct Item;
    impl TableMeta for Item {
        fn table_name() -> TableName {
            TableName::new("test_db_items")
        }
    }
    impl Item {
        const ID: Field<i64> = Field::new("id");
        const NAME: Field<String> = Field::new("name");
        const STATUS: Field<i32> = Field::new("status");
    }

    async fn setup_pool() -> SqlitePool {
        let url =
            std::env::var("DATABASE_URL_SQLITE").unwrap_or_else(|_| "sqlite::memory:".to_string());
        SqlitePool::connect(&url).await.expect("connect sqlite")
    }

    #[tokio::test]
    async fn sqlite_insert_and_query() {
        let pool = setup_pool().await;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS test_db_items (id INTEGER PRIMARY KEY, name TEXT NOT NULL, status INTEGER NOT NULL DEFAULT 0)",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Insert
        let ins = Insert::<Sqlite, Item>::new()
            .set(Item::ID, 1_i64)
            .set(Item::NAME, "alpha".to_string())
            .set(Item::STATUS, 1_i32);
        ins.execute(&pool).await.unwrap();

        // Query
        let row = sqlx::query("SELECT id, name, status FROM test_db_items WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        let name: String = row.get("name");
        let status: i32 = row.get("status");
        assert_eq!(name, "alpha");
        assert_eq!(status, 1);

        // Cleanup
        sqlx::query("DROP TABLE IF EXISTS test_db_items")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn sqlite_update_and_verify() {
        let pool = setup_pool().await;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS test_db_upd (id INTEGER PRIMARY KEY, name TEXT NOT NULL, status INTEGER NOT NULL DEFAULT 0)",
        )
        .execute(&pool)
        .await
        .unwrap();

        // Seed data
        sqlx::query("INSERT INTO test_db_upd (id, name, status) VALUES (1, 'before', 0)")
            .execute(&pool)
            .await
            .unwrap();

        // Update using builder
        struct UpdModel;
        impl TableMeta for UpdModel {
            fn table_name() -> TableName {
                TableName::new("test_db_upd")
            }
        }

        let name_field: Field<String> = Field::new("name");
        let status_field: Field<i32> = Field::new("status");

        let upd = Update::<Sqlite, UpdModel>::new()
            .set(name_field, "after".to_string())
            .set(status_field, 99_i32);

        upd.execute(&pool, |qb| {
            qb.push_where().field_eq("id", 1_i64);
        })
        .await
        .unwrap();

        // Verify
        let row = sqlx::query("SELECT name, status FROM test_db_upd WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        let name: String = row.get("name");
        let status: i32 = row.get("status");
        assert_eq!(name, "after");
        assert_eq!(status, 99);

        // Cleanup
        sqlx::query("DROP TABLE IF EXISTS test_db_upd")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn sqlite_update_with_expr() {
        let pool = setup_pool().await;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS test_db_expr (id INTEGER PRIMARY KEY, counter INTEGER NOT NULL DEFAULT 0)",
        )
        .execute(&pool)
        .await
        .unwrap();

        sqlx::query("INSERT INTO test_db_expr (id, counter) VALUES (1, 10)")
            .execute(&pool)
            .await
            .unwrap();

        struct ExprModel;
        impl TableMeta for ExprModel {
            fn table_name() -> TableName {
                TableName::new("test_db_expr")
            }
        }

        let counter_field: Field<i32> = Field::new("counter");

        let upd = Update::<Sqlite, ExprModel>::new()
            .set(counter_field, FieldValue::<Sqlite, i32>::expr("counter+1"));

        upd.execute(&pool, |qb| {
            qb.push_where().field_eq("id", 1_i64);
        })
        .await
        .unwrap();

        let row = sqlx::query("SELECT counter FROM test_db_expr WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        let counter: i32 = row.get("counter");
        assert_eq!(counter, 11);

        sqlx::query("DROP TABLE IF EXISTS test_db_expr")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn sqlite_where_clause_integration() {
        let pool = setup_pool().await;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS test_db_wc (id INTEGER PRIMARY KEY, val INTEGER NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();

        for i in 1..=5_i64 {
            sqlx::query("INSERT INTO test_db_wc (id, val) VALUES (?, ?)")
                .bind(i)
                .bind(i * 10)
                .execute(&pool)
                .await
                .unwrap();
        }

        // Use WhereClause to build a query
        let mut qb = sqlx::QueryBuilder::<Sqlite>::new("SELECT * FROM test_db_wc");
        {
            let mut wc = WhereClause::new(&mut qb);
            wc.and().field_gte("val", 20_i64);
            wc.and().field_lte("val", 40_i64);
        }

        let rows = qb.build().fetch_all(&pool).await.unwrap();
        assert_eq!(rows.len(), 3); // val 20, 30, 40

        sqlx::query("DROP TABLE IF EXISTS test_db_wc")
            .execute(&pool)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn sqlite_batch_insert() {
        use lsys_core::db::BatchInsert;
        let pool = setup_pool().await;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS test_db_batch (id INTEGER PRIMARY KEY, name TEXT NOT NULL)",
        )
        .execute(&pool)
        .await
        .unwrap();

        let row1 = Insert::<Sqlite, Item>::new()
            .set(Item::ID, 10_i64)
            .set(Item::NAME, "r1".to_string())
            .set(Item::STATUS, 0_i32);
        let row2 = Insert::<Sqlite, Item>::new()
            .set(Item::ID, 11_i64)
            .set(Item::NAME, "r2".to_string())
            .set(Item::STATUS, 0_i32);

        // Use a separate model that matches the batch table
        struct BatchModel;
        impl TableMeta for BatchModel {
            fn table_name() -> TableName {
                TableName::new("test_db_batch")
            }
        }

        let id_f: Field<i64> = Field::new("id");
        let name_f: Field<String> = Field::new("name");

        let b1 = Insert::<Sqlite, BatchModel>::new()
            .set(id_f.clone(), 10_i64)
            .set(name_f.clone(), "r1".to_string());
        let b2 = Insert::<Sqlite, BatchModel>::new()
            .set(id_f, 11_i64)
            .set(name_f, "r2".to_string());

        let batch = BatchInsert::<Sqlite, BatchModel>::new().push(b1).push(b2);
        batch.execute(&pool).await.unwrap();

        let count: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM test_db_batch")
            .fetch_one(&pool)
            .await
            .unwrap();
        assert_eq!(count, 2);

        sqlx::query("DROP TABLE IF EXISTS test_db_batch")
            .execute(&pool)
            .await
            .unwrap();
    }
}

// =============================================================================
// 9. Integration tests — actual MySQL database (gated)
// =============================================================================

#[cfg(feature = "db-mysql")]
mod mysql_integration {
    use lsys_core::db::{Field, Insert, QueryBuilderExt, TableMeta, TableName, Update};
    use sqlx::{MySql, MySqlPool, Row};

    struct MItem;
    impl TableMeta for MItem {
        fn table_name() -> TableName {
            TableName::new("test_db_mysql_item")
        }
    }
    impl MItem {
        const ID: Field<u64> = Field::new("id");
        const NAME: Field<String> = Field::new("name");
        const STATUS: Field<i32> = Field::new("status");
    }

    async fn mysql_pool() -> Option<MySqlPool> {
        let url = std::env::var("DATABASE_URL_MYSQL").ok()?;
        MySqlPool::connect(&url).await.ok()
    }

    #[tokio::test]
    async fn mysql_insert_update_query() {
        let Some(pool) = mysql_pool().await else {
            eprintln!("Skipping MySQL test: DATABASE_URL_MYSQL not set");
            return;
        };

        sqlx::query("CREATE TABLE IF NOT EXISTS test_db_mysql_item (id BIGINT UNSIGNED PRIMARY KEY, name VARCHAR(100) NOT NULL, status INT NOT NULL DEFAULT 0)")
            .execute(&pool)
            .await
            .unwrap();

        // Insert
        let ins = Insert::<MySql, MItem>::new()
            .set(MItem::ID, 1_u64)
            .set(MItem::NAME, "mysql_test".to_string())
            .set(MItem::STATUS, 0_i32);
        ins.execute(&pool).await.unwrap();

        // Update
        let upd = Update::<MySql, MItem>::new().set(MItem::STATUS, 42_i32);
        upd.execute(&pool, |qb| {
            qb.push_where().field_eq("id", 1_u64);
        })
        .await
        .unwrap();

        // Verify
        let row = sqlx::query("SELECT status FROM test_db_mysql_item WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        let status: i32 = row.get("status");
        assert_eq!(status, 42);

        // Cleanup
        sqlx::query("DROP TABLE IF EXISTS test_db_mysql_item")
            .execute(&pool)
            .await
            .unwrap();
    }
}

// =============================================================================
// 10. Integration tests — actual PostgreSQL database (gated)
// =============================================================================

#[cfg(feature = "db-postgres")]
mod postgres_integration {
    use lsys_core::db::{Field, Insert, QueryBuilderExt, TableMeta, TableName, Update};
    use sqlx::{PgPool, Postgres, Row};

    struct PItem;
    impl TableMeta for PItem {
        fn table_name() -> TableName {
            TableName::new("test_db_pg_item")
        }
    }
    impl PItem {
        const ID: Field<i64> = Field::new("id");
        const NAME: Field<String> = Field::new("name");
        const STATUS: Field<i32> = Field::new("status");
    }

    async fn pg_pool() -> Option<PgPool> {
        let url = std::env::var("DATABASE_URL_PG").ok()?;
        PgPool::connect(&url).await.ok()
    }

    #[tokio::test]
    async fn pg_insert_update_query() {
        let Some(pool) = pg_pool().await else {
            eprintln!("Skipping PostgreSQL test: DATABASE_URL_PG not set");
            return;
        };

        sqlx::query("CREATE TABLE IF NOT EXISTS test_db_pg_item (id BIGINT PRIMARY KEY, name VARCHAR(100) NOT NULL, status INT NOT NULL DEFAULT 0)")
            .execute(&pool)
            .await
            .unwrap();

        let ins = Insert::<Postgres, PItem>::new()
            .set(PItem::ID, 1_i64)
            .set(PItem::NAME, "pg_test".to_string())
            .set(PItem::STATUS, 0_i32);
        ins.execute(&pool).await.unwrap();

        let upd = Update::<Postgres, PItem>::new().set(PItem::STATUS, 77_i32);
        upd.execute(&pool, |qb| {
            qb.push_where().field_eq("id", 1_i64);
        })
        .await
        .unwrap();

        let row = sqlx::query("SELECT status FROM test_db_pg_item WHERE id = 1")
            .fetch_one(&pool)
            .await
            .unwrap();
        let status: i32 = row.get("status");
        assert_eq!(status, 77);

        sqlx::query("DROP TABLE IF EXISTS test_db_pg_item")
            .execute(&pool)
            .await
            .unwrap();
    }
}
