use std::borrow::Cow;
use std::fmt::Display;

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

    pub fn full_name(&self) -> String {
        let mut out = String::with_capacity(self.db.len() + self.name.len());
        out.push_str(&self.db);
        out.push_str(&self.name);
        out
    }

    /// 获取原始表名（不带前缀）
    pub fn raw_name(&self) -> &str {
        &self.name
    }

    /// 获取数据库部分（如 "other_db."）
    pub fn db_part(&self) -> &str {
        &self.db
    }
}

impl Display for TableName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}{}", self.db, self.name)
    }
}

impl TableName {
    /// SQL 标识符引用：简单标识符直接返回，否则加双引号并转义
    pub fn quoted(&self) -> String {
        let full = self.full_name();
        if Self::is_simple_identifier(&full) {
            full
        } else {
            let escaped = full.replace('"', "\"\"");
            format!("\"{}\"", escaped)
        }
    }

    /// 判断是否为简单标识符
    fn is_simple_identifier(s: &str) -> bool {
        let Some(first) = s.chars().next() else {
            return false;
        };
        if !first.is_ascii_alphabetic() && first != '_' {
            return false;
        }
        s.chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '.')
    }
}

/// 表元信息 trait
pub trait TableMeta {
    fn table_name() -> TableName;
}
