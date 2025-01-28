use super::{FieldItem, ModelTableField, ModelTableName};
// use sqlx::database::HasArguments;
use sqlx::query::Query;
use sqlx::{Arguments, Executor, IntoArguments};
use sqlx::{Database, Error};

pub trait UpdateData<'t, DB>
where
    DB: Database,
{
    fn diff_columns(&self) -> Vec<FieldItem>;
    fn sqlx_bind<'q>(
        &'q self,
        res: Query<'q, DB, DB::Arguments<'q>>,
    ) -> Query<'q, DB, DB::Arguments<'q>>;
    fn sqlx_string(&self, field: &FieldItem) -> Option<String>;
}
//得到需要更改的设置数据集
pub trait ModelUpdateData<'t, DB, CT>: ModelTableField<DB> + ModelTableName
where
    CT: UpdateData<'t, DB>,
    DB: Database,
{
    fn diff(&'t self, source: &Option<&Self>) -> CT;
}

pub enum WhereOption {
    None,            //无WHERE条件,且无排序等
    Where(String),   //有WHERE条件
    NoWhere(String), //无WHERE条件,但有排序等后续SQL
}

/// 更新操作
pub struct Update<'t, DB, T, CT>
where
    T: ModelUpdateData<'t, DB, CT>,
    CT: UpdateData<'t, DB>,
    DB: Database,
{
    pub change: CT,
    _marker: (
        std::marker::PhantomData<&'t CT>,
        std::marker::PhantomData<&'t T>,
        std::marker::PhantomData<DB>,
    ),
}
impl<'t, DB, T, CT> Update<'t, DB, T, CT>
where
    T: ModelUpdateData<'t, DB, CT>,
    CT: UpdateData<'t, DB>,
    DB: Database,
{
    pub fn new(val: CT) -> Update<'t, DB, T, CT> {
        Update {
            change: val,
            _marker: Default::default(),
        }
    }
    pub fn model<'q: 't>(val: &'q T, source: &Option<&T>) -> Update<'t, DB, T, CT> {
        Update {
            change: val.diff(source),
            _marker: Default::default(),
        }
    }
    pub fn empty_change(&self) -> bool {
        self.change.diff_columns().is_empty()
    }
    pub fn sql_sets(&self) -> String {
        let diff = self.change.diff_columns();
        let mut values = Vec::<String>::with_capacity(diff.len());
        for val in diff.iter() {
            values.push(format!("{}=?", val.name));
        }
        values.join(",")
    }
    pub fn sql_values_sets(&self) -> String {
        let diff = self.change.diff_columns();
        let mut values = Vec::<String>::with_capacity(diff.len());
        for val in diff.iter() {
            if let Some(setval) = self.change.sqlx_string(val) {
                values.push(format!("{}={}", val.name, setval));
            }
        }
        values.join(",")
    }
    pub fn bind_values<'q>(
        &'q self,
        res: Query<'q, DB, DB::Arguments<'q>>,
    ) -> Query<'q, DB, DB::Arguments<'q>> {
        self.change.sqlx_bind(res)
    }
    pub async fn execute_by_where<'c, E>(
        &self,
        where_sql: &WhereOption,
        executor: E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'n> DB::Arguments<'n>: Arguments<'n> + IntoArguments<'n, DB>,
        E: Executor<'c, Database = DB>,
    {
        if self.empty_change() {
            return Ok(<DB as Database>::QueryResult::default());
        }
        let table = T::table_name();
        let values = self.sql_sets();
        let sql = match where_sql {
            WhereOption::Where(wsql) => {
                format!("UPDATE {} SET {} WHERE {}", table.full_name(), values, wsql)
            }
            WhereOption::None => {
                format!("UPDATE {} SET {}", table.full_name(), values)
            }
            WhereOption::NoWhere(other) => {
                format!("UPDATE {} SET {} {}", table.full_name(), values, other)
            }
        };
        let mut res = sqlx::query(sql.as_str());
        res = self.bind_values(res);
        executor.execute(res).await
    }
    // execute_by_sql!(Update<DB,T,CT>);
    pub async fn execute_by_pk<'c, E>(
        &self,
        source: &T,
        executor: E,
    ) -> Result<<DB as Database>::QueryResult, Error>
    where
        for<'n> DB::Arguments<'n>: Arguments<'n> + IntoArguments<'n, DB>,
        E: Executor<'c, Database = DB>,
    {
        if self.empty_change() {
            return Ok(<DB as Database>::QueryResult::default());
        }
        let table = T::table_name();
        let pkf = T::table_pk();
        let mut where_sql = vec![];
        let values = self.sql_sets();
        for val in pkf.0.iter() {
            where_sql.push(format!("{}=?", val.name));
        }
        let sql = format!(
            "UPDATE {} SET {} WHERE {}",
            table.full_name(),
            values,
            where_sql.join(" and ")
        );
        let mut res = sqlx::query(sql.as_str());
        res = self.bind_values(res);
        for val in pkf.0.iter() {
            res = source.query_sqlx_bind(val, res);
        }
        executor.execute(res).await
    }
}
