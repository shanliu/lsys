mod insert;
mod update;

use sqlx::query::{Query, QueryAs};
use sqlx::FromRow;
use std::fmt::Display;

pub use insert::*;
pub use update::*;

use super::SqlQuote;

// 统一表前缀
lazy_static::lazy_static! {
    static ref TABLE_PREFIX: parking_lot::RwLock<String> = parking_lot::RwLock::new("".to_string());
}

/// 表名
pub struct TableName {
    db: String,
    name: String,
}
impl Display for TableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}{}", self.db, TABLE_PREFIX.read(), self.name)
    }
}
impl SqlQuote<String> for TableName {
    fn sql_quote(&self) -> String {
        format!("{}{}{}", self.db, TABLE_PREFIX.read(), self.name)
    }
}
impl TableName {
    /// 设置表前缀
    pub fn set_prefix(str: String) {
        *TABLE_PREFIX.write() = str;
    }
    /// 新建表名
    pub fn new(name: &str) -> Self {
        let (db, name) = match name.rfind('.') {
            Some(index) => name.split_at(index + 1),
            None => ("", name),
        };
        Self {
            db: db.to_string(),
            name: name.to_owned(),
        }
    }
    /// 得到完整表名
    pub fn full_name(&self) -> String {
        format!("{}{}{}", self.db, TABLE_PREFIX.read(), self.name)
    }
}

/// model实现得到表名trait
pub trait ModelTableName {
    fn table_name() -> TableName;
}
/// model实现得到表字段和字段值绑定 trait
pub trait ModelTableField {
    fn table_pk() -> TableFields;
    fn table_column() -> TableFields;
    fn query_sqlx_bind<'t>(
        &'t self,
        table_field_val: &FieldItem,
        res: Query<'t, sqlx::MySql, sqlx::mysql::MySqlArguments>,
    ) -> Query<'t, sqlx::MySql, sqlx::mysql::MySqlArguments>;
    fn query_as_sqlx_bind<'t, M>(
        &'t self,
        table_field_val: &FieldItem,
        res: QueryAs<'t, sqlx::MySql, M, sqlx::mysql::MySqlArguments>,
    ) -> QueryAs<'t, sqlx::MySql, M, sqlx::mysql::MySqlArguments>
    where
        for<'r> M: FromRow<'r, sqlx::mysql::MySqlRow> + Send + Unpin;
}

/// 表字段
#[derive(Clone, PartialEq, Eq)]
pub struct FieldItem {
    pub name: String,
    pub column_name: String,
}
impl Display for FieldItem {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.name)
    }
}
impl FieldItem {
    pub fn new(name: &str, column_name: &str) -> Self {
        FieldItem {
            name: name.to_string(),
            column_name: column_name.to_string(),
        }
    }
}

/// 表字段容器
pub struct TableFields(Vec<FieldItem>);
impl Display for TableFields {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let fileds = self
            .0
            .iter()
            .map(|e| format!("{e}"))
            .collect::<Vec<String>>()
            .join(",");
        write!(f, "{fileds}")
    }
}
impl TableFields {
    pub fn new(fields: Vec<FieldItem>) -> Self {
        TableFields(fields)
    }
    /// 合并一批外部的表字段列表,去除重复
    pub fn marge(&mut self, field: &[FieldItem]) {
        for val in field.iter() {
            if !self.0.iter().any(|e| e.name == val.name) {
                self.0.push(val.to_owned())
            }
        }
    }
    /// 跟指定表字段列表取并集
    pub fn intersect(&mut self, field: &[FieldItem]) {
        self.0 = self
            .0
            .iter()
            .filter_map(|e| {
                if field.contains(e) {
                    Some(e.to_owned())
                } else {
                    None
                }
            })
            .collect();
    }
    /// 删除表指定字段
    pub fn del(&mut self, name: &str) {
        self.0 = self
            .0
            .iter()
            .filter_map(|e| {
                if name == e.name {
                    None
                } else {
                    Some(e.to_owned())
                }
            })
            .collect();
    }
    /// 得到字段列表
    pub fn to_vec(&self) -> Vec<String> {
        let field = self.0.iter();
        field
            .map(|e| e.column_name.clone())
            .collect::<Vec<String>>()
    }
}
