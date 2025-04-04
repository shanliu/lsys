use super::{FieldItem, ModelTableField, ModelTableName};
// use sqlx::database::HasArguments;
use sqlx::query::Query;
use sqlx::Error;
use sqlx::{Arguments, Executor, IntoArguments};

pub trait UpdateData<'t> {
    fn diff_columns(&self) -> Vec<FieldItem>;
    fn sqlx_bind<'q>(
        &'q self,
        res: Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    ) -> Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>;
    fn sqlx_string(&self, field: &FieldItem) -> Option<String>;
}
//得到需要更改的设置数据集
pub trait ModelUpdateData<'t, CT>: ModelTableField + ModelTableName
where
    CT: UpdateData<'t>,
{
    fn diff(&'t self, source: &Option<&Self>) -> CT;
}

pub enum WhereOption {
    None,            //无WHERE条件,且无排序等
    Where(String),   //有WHERE条件
    NoWhere(String), //无WHERE条件,但有排序等后续SQL
}

/// 更新操作
pub struct Update<'t, T, CT>
where
    T: ModelUpdateData<'t, CT>,
    CT: UpdateData<'t>,
{
    pub change: CT,
    _marker: (
        std::marker::PhantomData<&'t CT>,
        std::marker::PhantomData<&'t T>,
    ),
}
impl<'t, T, CT> Update<'t, T, CT>
where
    T: ModelUpdateData<'t, CT>,
    CT: UpdateData<'t>,
{
    pub fn new(val: CT) -> Update<'t, T, CT> {
        Update {
            change: val,
            _marker: Default::default(),
        }
    }
    pub fn model<'q: 't>(val: &'q T, source: &Option<&T>) -> Update<'t, T, CT> {
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
        res: Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    ) -> Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments> {
        self.change.sqlx_bind(res)
    }
    pub async fn execute_by_where<'c, E>(
        &self,
        where_sql: &WhereOption,
        executor: E,
    ) -> Result<sqlx::mysql::MySqlQueryResult, Error>
    where
        for<'n> <sqlx::MySql as sqlx::Database>::Arguments<'n>:
            Arguments<'n> + IntoArguments<'n, sqlx::MySql>,
        E: Executor<'c, Database = sqlx::MySql>,
    {
        if self.empty_change() {
            return Ok(sqlx::mysql::MySqlQueryResult::default());
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
    pub async fn execute_by_pk<'c, E>(
        &self,
        source: &T,
        executor: E,
    ) -> Result<sqlx::mysql::MySqlQueryResult, Error>
    where
        for<'n> <sqlx::MySql as sqlx::Database>::Arguments<'n>:
            Arguments<'n> + IntoArguments<'n, sqlx::MySql>,
        E: Executor<'c, Database = sqlx::MySql>,
    {
        if self.empty_change() {
            return Ok(sqlx::mysql::MySqlQueryResult::default());
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
