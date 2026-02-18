/// 字段值枚举 - 移除泛型T，直接存储可绑定的值
pub enum FieldValue<'a> {
    /// 设置值 (参数绑定) - 存储 trait object
    Bind(Box<dyn BindableValue + 'a>),
    /// SQL 表达式 (直接拼接)
    Expr(String),
    /// 跳过/撤销 (不生成 SQL)
    Skip,
}

// 手动实现 Debug，因为 dyn BindableValue 不自动实现 Debug
impl<'a> std::fmt::Debug for FieldValue<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::Bind(_) => write!(f, "FieldValue::Bind(<trait object>)"),
            FieldValue::Expr(s) => f.debug_tuple("FieldValue::Expr").field(s).finish(),
            FieldValue::Skip => write!(f, "FieldValue::Skip"),
        }
    }
}

impl<'a> FieldValue<'a> {
    pub fn is_skip(&self) -> bool {
        matches!(self, Self::Skip)
    }
}

/// 跳过标记
pub struct Skip;

// ============== 统一转换 Trait ==============

/// 将各种类型转换为 FieldValue - 添加生命周期参数
pub trait IntoFieldValue<'a, T> {
    fn into_field_value(self) -> FieldValue<'a>;
}

// SqlExpr<E> -> Expr(s) 任意字段类型
impl<'a, T, E: std::fmt::Display> IntoFieldValue<'a, T> for SqlExpr<E> {
    fn into_field_value(self) -> FieldValue<'a> {
        FieldValue::Expr(self.0.to_string())
    }
}

// Skip -> Skip 任意字段类型
impl<'a, T> IntoFieldValue<'a, T> for Skip {
    fn into_field_value(self) -> FieldValue<'a> {
        FieldValue::Skip
    }
}

// 为基本类型实现 T -> FieldValue - 存储为 Bind
// 注意：移除 i128/u128，因为 sqlx 不支持这些类型
macro_rules! impl_into_field_value {
    ($($t:ty),*) => {
        $(
            impl<'a> IntoFieldValue<'a, $t> for $t {
                fn into_field_value(self) -> FieldValue<'a> {
                    FieldValue::Bind(Box::new(self))
                }
            }
        )*
    };
}

impl_into_field_value!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool, String);

// &str 零拷贝支持
impl<'a> IntoFieldValue<'a, &'a str> for &'a str {
    fn into_field_value(self) -> FieldValue<'a> {
        FieldValue::Bind(Box::new(self))
    }
}

// &str -> String 字段的隐式转换 (零拷贝，因为 sqlx 支持 &str 绑定到 VARCHAR)
impl<'a> IntoFieldValue<'a, String> for &'a str {
    fn into_field_value(self) -> FieldValue<'a> {
        FieldValue::Bind(Box::new(self))
    }
}

// &String -> &str 零拷贝转换 (关键：避免隐式拷贝)
impl<'a> IntoFieldValue<'a, String> for &'a String {
    fn into_field_value(self) -> FieldValue<'a> {
        // 通过 as_str() 转为 &str，利用 sqlx 对 &str 的原生支持
        FieldValue::Bind(Box::new(self.as_str()))
    }
}

// ============== SQL 后缀枚举 ==============

/// SQL 后缀（WHERE/ORDER BY/LIMIT 等）
#[derive(Debug, Clone, Default)]
pub enum SqlSuffix<'a> {
    /// 无后缀
    #[default]
    None,

    /// 带 WHERE 前缀
    Where(&'a str),

    /// 不带 WHERE 前缀（直接拼接）
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

// ============== 存储值（类型擦除后） ==============

use sqlx::mysql::{MySql, MySqlArguments};
use sqlx::Encode;
use sqlx::Type;

use crate::db::SqlExpr;

/// 可绑定的值 trait
///
/// 使用动态分发时，我们直接操作 Arguments 而不是 Query
pub trait BindableValue: Send + Sync {
    /// 将值添加到 Arguments 中
    ///
    /// 这避免了 Query<'q> 生命周期不变性的问题
    fn add_to_args(&self, args: &mut MySqlArguments);
}

/// 为所有实现了 Encode + Type 的类型自动实现
impl<T> BindableValue for T
where
    T: for<'q> Encode<'q, MySql> + Type<MySql> + Send + Sync,
{
    fn add_to_args(&self, args: &mut MySqlArguments) {
        use sqlx::Arguments;
        // add 方法返回 Result，这里直接 unwrap（因为 MySQL 很少失败）
        let _ = args.add(self);
    }
}

/// 存储类型擦除后的值（内部使用）
pub(crate) enum StoredValue<'a> {
    Bind(Box<dyn BindableValue + 'a>),
    Expr(String),
}
