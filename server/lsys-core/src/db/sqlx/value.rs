use sqlx::{Database, Encode, Type};

use crate::db::SqlExpr;

pub trait BindableValue<DB: Database>: Send + Sync {
    fn add_to_args_dyn<'q>(&'q self, args: &mut <DB as Database>::Arguments<'q>)
    where
        for<'a_> <DB as Database>::Arguments<'a_>: sqlx::Arguments<'a_>;
}

impl<T, DB> BindableValue<DB> for T
where
    DB: Database,
    T: Send + Sync + Clone,
    for<'q> T: Encode<'q, DB> + Type<DB>,
{
    fn add_to_args_dyn<'q>(&'q self, args: &mut <DB as Database>::Arguments<'q>)
    where
        for<'a_> <DB as Database>::Arguments<'a_>: sqlx::Arguments<'a_>,
    {
        let _ = sqlx::Arguments::add(args, self.clone());
    }
}

pub(crate) enum StoredValue<'a, DB: Database> {
    Bind(Box<dyn BindableValue<DB> + 'a>),
    Expr(String),
}

pub enum FieldValue<'a, DB: Database> {
    Bind(Box<dyn BindableValue<DB> + 'a>),
    Expr(String),
    Skip,
}

impl<'a, DB: Database> std::fmt::Debug for FieldValue<'a, DB> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::Bind(_) => write!(f, "FieldValue::Bind(<trait object>)"),
            FieldValue::Expr(s) => f.debug_tuple("FieldValue::Expr").field(s).finish(),
            FieldValue::Skip => write!(f, "FieldValue::Skip"),
        }
    }
}

impl<'a, DB: Database> FieldValue<'a, DB> {
    pub fn is_skip(&self) -> bool {
        matches!(self, Self::Skip)
    }
}

pub struct Skip;

pub trait IntoFieldValue<'a, DB: Database, T> {
    fn into_field_value(self) -> FieldValue<'a, DB>;
}

impl<'a, DB: Database, T, E: std::fmt::Display> IntoFieldValue<'a, DB, T> for SqlExpr<E> {
    fn into_field_value(self) -> FieldValue<'a, DB> {
        FieldValue::Expr(self.0.to_string())
    }
}

impl<'a, DB: Database, T> IntoFieldValue<'a, DB, T> for Skip {
    fn into_field_value(self) -> FieldValue<'a, DB> {
        FieldValue::Skip
    }
}

macro_rules! impl_into_field_value {
    ($($t:ty),*) => {
        $(
            impl<'a, DB: Database> IntoFieldValue<'a, DB, $t> for $t
            where
                for<'q> $t: Encode<'q, DB> + Type<DB> + Send + Sync,
            {
                fn into_field_value(self) -> FieldValue<'a, DB> {
                    FieldValue::Bind(Box::new(self))
                }
            }
        )*
    };
}

impl_into_field_value!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, String);

// For strings, convert &str to String because &str doesn't fulfill `for<'q> &'q str: Encode<'q, DB>`
impl<'a, DB: Database> IntoFieldValue<'a, DB, &'a str> for &'a str
where
    for<'q> String: Encode<'q, DB> + Type<DB> + Send + Sync,
{
    fn into_field_value(self) -> FieldValue<'a, DB> {
        FieldValue::Bind(Box::new(self.to_string()))
    }
}

impl<'a, DB: Database> IntoFieldValue<'a, DB, String> for &'a str
where
    for<'q> String: Encode<'q, DB> + Type<DB> + Send + Sync,
{
    fn into_field_value(self) -> FieldValue<'a, DB> {
        FieldValue::Bind(Box::new(self.to_string()))
    }
}

impl<'a, DB: Database> IntoFieldValue<'a, DB, String> for &'a String
where
    for<'q> String: Encode<'q, DB> + Type<DB> + Send + Sync,
{
    fn into_field_value(self) -> FieldValue<'a, DB> {
        FieldValue::Bind(Box::new(self.to_string()))
    }
}

#[derive(Debug, Clone, Default)]
pub enum SqlSuffix<'a> {
    #[default]
    None,
    Where(&'a str),
    Suffix(&'a str),
}

impl<'a> SqlSuffix<'a> {
    pub fn to_sql(&self) -> String {
        match self {
            SqlSuffix::None => String::new(),
            SqlSuffix::Where(s) => {
                let s = s.trim();
                if s.is_empty() {
                    String::new()
                } else {
                    format!(" WHERE {}", s)
                }
            }
            SqlSuffix::Suffix(s) => {
                let s = s.trim();
                if s.is_empty() {
                    String::new()
                } else {
                    format!(" {}", s)
                }
            }
        }
    }
}
