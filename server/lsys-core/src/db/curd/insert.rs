use super::{
    FieldItem, ModelTableField, ModelTableName, ModelUpdateData, TableFields, Update, UpdateData,
};
use sqlx::query::Query;
use sqlx::Error;
use sqlx::{Arguments, Executor, IntoArguments};
use std::vec;

/// 插入操作
pub trait InsertData<'t> {
    fn columns(&self) -> Vec<FieldItem>;
    fn sqlx_bind<'q>(
        &'q self,
        field: &FieldItem,
        res: Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    ) -> Query<'q, sqlx::MySql, sqlx::mysql::MySqlArguments>;
    fn sqlx_string(&self, field: &FieldItem) -> Option<String>;
}
pub trait ModelInsertData<'t, DT>: ModelTableField + ModelTableName
where
    DT: InsertData<'t>,
{
    fn insert_data(&'t self) -> DT;
}

pub struct Insert<'q, T, DT>
where
    T: ModelTableName,
    DT: InsertData<'q>,
{
    pub val: Vec<DT>,
    pub fields: TableFields,
    _marker: (
        std::marker::PhantomData<T>,
        std::marker::PhantomData<&'q DT>,
    ),
}
impl<'q, T, DT> Insert<'q, T, DT>
where
    T: ModelTableName,
    DT: InsertData<'q>,
{
    pub fn new(val: DT) -> Self {
        let column = val.columns();
        Self {
            val: vec![val],
            fields: TableFields(column),
            _marker: Default::default(),
        }
    }
    pub fn new_vec(val: Vec<DT>) -> Self {
        let mut fields = TableFields::new(vec![]);
        for tmp in val.iter() {
            fields.marge(&tmp.columns());
        }
        Self {
            val,
            fields,
            _marker: Default::default(),
        }
    }
    pub fn model<'t: 'q, MI>(val: &'t MI) -> Self
    where
        MI: ModelInsertData<'q, DT>,
    {
        let ival = val.insert_data();
        let column = ival.columns();
        Self {
            val: vec![ival],
            fields: TableFields(column),
            _marker: Default::default(),
        }
    }
    pub fn model_vec<'t: 'q, MI>(val: &'t Vec<MI>) -> Self
    where
        MI: ModelInsertData<'q, DT>,
    {
        let mut vals = vec![];
        let mut fields = TableFields::new(vec![]);
        for tmp in val {
            let ival = tmp.insert_data();
            fields.marge(&ival.columns());
            vals.push(ival);
        }
        Self {
            val: vals,
            fields,
            _marker: Default::default(),
        }
    }
    pub fn sql_param(&self) -> Vec<String> {
        let mut values = Vec::<String>::with_capacity(self.val.len());
        for _ in self.val.iter() {
            let len = self.fields.0.len();
            // let mut value = Vec::with_capacity(len);
            // for _ in 0..len {
            //     value.push("?");
            // }
            let val: String = vec!["?"; len].join(",");
            let val = "(".to_string() + val.as_str() + ")";
            values.push(val);
        }
        values
    }
    pub fn sql_values(&self) -> Vec<String> {
        let mut values = Vec::<String>::with_capacity(self.val.len());
        for val in self.val.iter() {
            let mut value = Vec::with_capacity(self.fields.0.len());
            for field in &self.fields.0 {
                if let Some(ival) = val.sqlx_string(field) {
                    value.push(ival);
                }
            }
            let val: String = value.join(",");
            let val = "(".to_string() + val.as_str() + ")";
            values.push(val);
        }
        values
    }
    pub fn bind_values<'t>(
        &'t self,
        mut res: Query<'t, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    ) -> Query<'t, sqlx::MySql, sqlx::mysql::MySqlArguments> {
        for val in self.val.iter() {
            for field in &self.fields.0 {
                res = val.sqlx_bind(field, res);
            }
        }
        res
    }
    pub async fn execute<'c, E>(self, executor: E) -> Result<sqlx::mysql::MySqlQueryResult, Error>
    where
        for<'n> <sqlx::MySql as sqlx::Database>::Arguments<'n>:
            Arguments<'n> + IntoArguments<'n, sqlx::MySql>,
        E: Executor<'c, Database = sqlx::MySql>,
    {
        let table = T::table_name();
        let vals = self.sql_param();
        let sql = format!(
            "INSERT INTO {} ({})VALUES {}",
            table.full_name(),
            self.fields.to_vec().join(","),
            vals.join(",")
        );
        let mut res = sqlx::query(sql.as_str());
        res = self.bind_values(res);
        executor.execute(res).await
    }
    pub async fn execute_update<'c, 't, CT, IT, E>(
        self,
        update: &Update<'t, IT, CT>,
        executor: E,
    ) -> Result<sqlx::mysql::MySqlQueryResult, Error>
    where
        IT: ModelUpdateData<'t, CT>,
        CT: UpdateData<'t>,
        for<'n> <sqlx::MySql as sqlx::Database>::Arguments<'n>:
            Arguments<'n> + IntoArguments<'n, sqlx::MySql>,
        E: Executor<'c, Database = sqlx::MySql>,
    {
        let table = T::table_name();
        let vals = self.sql_param();
        let sql = format!(
            "INSERT INTO {} ({})VALUES {} ON DUPLICATE KEY UPDATE {}",
            table.full_name(),
            self.fields.to_vec().join(","),
            vals.join(","),
            update.sql_sets()
        );
        let mut res = sqlx::query(sql.as_str());
        res = self.bind_values(res);
        res = update.bind_values(res);
        executor.execute(res).await
    }
}
