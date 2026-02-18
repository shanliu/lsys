use parking_lot::RwLock;
use std::borrow::Cow;
use std::fmt::Display;

use crate::db::{SqlExpr, SqlQuote};

lazy_static::lazy_static! {
    static ref TABLE_PREFIX: RwLock<String> = RwLock::new("".to_string());
}

/// 表名结构体
/// 使用 Cow 同时支持静态和动态表名
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TableName {
    db: Cow<'static, str>,
    name: Cow<'static, str>,
}

impl TableName {
    /// 创建表名（自动处理静态和动态字符串）
    pub fn new(full_name: impl Into<Cow<'static, str>>) -> Self {
        let full_name = full_name.into();
        match full_name.rfind('.') {
            Some(idx) if idx > 0 && idx < full_name.len() - 1 => {
                let db = full_name[..idx + 1].to_owned();
                let name = full_name[idx + 1..].to_owned();
                Self {
                    db: Cow::Owned(db),
                    name: Cow::Owned(name),
                }
            }
            _ => Self {
                db: Cow::Borrowed(""),
                name: full_name,
            },
        }
    }

    /// 分别指定数据库和表名（自动处理静态和动态）
    pub fn with_db(db: impl Into<Cow<'static, str>>, name: impl Into<Cow<'static, str>>) -> Self {
        Self {
            db: db.into(),
            name: name.into(),
        }
    }

    pub fn set_prefix(prefix: String) {
        *TABLE_PREFIX.write() = prefix;
    }

    fn with_prefix<R>(&self, f: impl FnOnce(&str) -> R) -> R {
        let prefix = TABLE_PREFIX.read();
        f(&prefix)
    }

    pub fn full_name(&self) -> String {
        self.with_prefix(|prefix| {
            let mut out = String::with_capacity(self.db.len() + prefix.len() + self.name.len());
            out.push_str(&self.db);
            out.push_str(prefix);
            out.push_str(&self.name);
            out
        })
    }

    /// 获取原始表名（不带前缀）
    pub fn raw_name(&self) -> &str {
        &self.name
    }

    /// 获取数据库部分（如 "other_db."）
    pub fn db_part(&self) -> &str {
        &self.db
    }

    /// 转换为 SqlExpr 用于 sql_format! 宏（不转义，带反引号）
    pub fn to_expr(&self) -> SqlExpr<String> {
        SqlExpr(format!("`{}`", self.full_name()))
    }
}

impl Display for TableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.with_prefix(|prefix| write!(f, "{}{}{}", self.db, prefix, self.name))
    }
}

impl SqlQuote<String> for TableName {
    fn sql_quote(&self) -> String {
        format!("`{}`", self.full_name())
    }
}

/// 表元信息 trait
pub trait TableMeta {
    fn table_name() -> TableName;
}
