use sqlx::{Database, Encode, QueryBuilder, Type};
use std::borrow::Cow;

/// Trait 用于将值绑定到 QueryBuilder
///
/// 使用 HRTB 让实现可以在任意生命周期下工作
pub(crate) trait BindValue<DB: Database>: Send + Sync {
    /// 将值绑定到 QueryBuilder
    ///
    /// 'q 是 QueryBuilder 的生命周期，在调用时确定
    fn bind_to<'q>(&self, builder: &mut QueryBuilder<'q, DB>);
}

/// 为所有实现了 Encode + Type 的类型实现 BindValue
///
/// 使用 Clone 来支持多次绑定（如果需要）
impl<T, DB> BindValue<DB> for T
where
    DB: Database,
    T: Send + Sync + Clone + 'static,
    for<'q> T: Encode<'q, DB> + Type<DB>,
{
    fn bind_to<'q>(&self, builder: &mut QueryBuilder<'q, DB>) {
        builder.push_bind(self.clone());
    }
}

/// 存储的值，用于延迟绑定
///
/// 动态回调类型别名
pub(crate) type DynamicCallback<DB> = Box<dyn for<'q> Fn(&mut QueryBuilder<'q, DB>) + Send + Sync>;

/// 不使用生命周期参数，所有值都以所有权形式存储
pub(crate) enum StoredValue<DB: Database> {
    /// 绑定值
    Bind(Box<dyn BindValue<DB>>),
    /// SQL 表达式
    Expr(Cow<'static, str>),
    /// 动态回调
    Dynamic(DynamicCallback<DB>),
}

/// 字段值，用于设置 INSERT/UPDATE 的字段
///
/// 泛型 T 与 Field<T> 强关联
pub enum FieldValue<DB: Database, T> {
    /// 具体的值
    Value(T),
    /// SQL 表达式（如 "NOW()", "DEFAULT"）
    Expr(Cow<'static, str>),
    /// 跳过此字段
    Skip,
    /// 动态回调
    Dynamic(DynamicCallback<DB>),
}

impl<DB: Database, T> std::fmt::Debug for FieldValue<DB, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FieldValue::Value(_) => write!(f, "FieldValue::Value(<value>)"),
            FieldValue::Expr(s) => f.debug_tuple("FieldValue::Expr").field(s).finish(),
            FieldValue::Dynamic(_) => write!(f, "FieldValue::Dynamic(<closure>)"),
            FieldValue::Skip => write!(f, "FieldValue::Skip"),
        }
    }
}

impl<DB: Database, T> FieldValue<DB, T> {
    /// 检查是否为 Skip
    pub fn is_skip(&self) -> bool {
        matches!(self, Self::Skip)
    }

    /// 创建 SQL 表达式值
    pub fn expr(sql: impl Into<Cow<'static, str>>) -> Self {
        Self::Expr(sql.into())
    }

    /// 创建动态回调
    pub fn dynamic<F>(f: F) -> Self
    where
        F: for<'q> Fn(&mut QueryBuilder<'q, DB>) + Send + Sync + 'static,
    {
        Self::Dynamic(Box::new(f))
    }
}


/// Trait 用于将值转换为 FieldValue
///
/// 泛型 T 与 Field<T> 强关联
pub trait IntoFieldValue<DB: Database, T> {
    /// 转换为 FieldValue
    fn into_field_value(self) -> FieldValue<DB, T>;
}

/// FieldValue 本身的转换
impl<DB: Database, T> IntoFieldValue<DB, T> for FieldValue<DB, T> {
    fn into_field_value(self) -> FieldValue<DB, T> {
        self
    }
}

/// 宏：为基本类型实现 IntoFieldValue（所有权）
macro_rules! impl_into_field_value_owned {
    ($($t:ty),*) => {
        $(
            impl<DB: Database> IntoFieldValue<DB, $t> for $t
            where
                for<'q> $t: Encode<'q, DB> + Type<DB> + Send + Sync + Clone,
            {
                fn into_field_value(self) -> FieldValue<DB, $t> {
                    FieldValue::Value(self)
                }
            }
        )*
    };
}

// 为基本类型实现（所有权）
impl_into_field_value_owned!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool);

/// 宏：为基本类型的引用实现 IntoFieldValue（Copy）
macro_rules! impl_into_field_value_ref {
    ($($t:ty),*) => {
        $(
            impl<DB: Database> IntoFieldValue<DB, $t> for &$t
            where
                for<'q> $t: Encode<'q, DB> + Type<DB> + Send + Sync + Clone + Copy,
            {
                fn into_field_value(self) -> FieldValue<DB, $t> {
                    FieldValue::Value(*self)
                }
            }
        )*
    };
}

// 为基本类型的引用实现（Copy）
impl_into_field_value_ref!(i8, i16, i32, i64, u8, u16, u32, u64, f32, f64, bool);

// String 的实现（所有权）
impl<DB: Database> IntoFieldValue<DB, String> for String
where
    for<'q> String: Encode<'q, DB> + Type<DB> + Send + Sync + Clone,
{
    fn into_field_value(self) -> FieldValue<DB, String> {
        FieldValue::Value(self)
    }
}

// &str 的实现（必要的 clone）
impl<DB: Database> IntoFieldValue<DB, String> for &str
where
    for<'q> String: Encode<'q, DB> + Type<DB> + Send + Sync + Clone,
{
    fn into_field_value(self) -> FieldValue<DB, String> {
        FieldValue::Value(self.to_string())
    }
}

// &String 的实现（必要的 clone）
impl<DB: Database> IntoFieldValue<DB, String> for &String
where
    for<'q> String: Encode<'q, DB> + Type<DB> + Send + Sync + Clone,
{
    fn into_field_value(self) -> FieldValue<DB, String> {
        FieldValue::Value(self.clone())
    }
}

// Option<T> 的实现（所有权）
impl<DB: Database, T> IntoFieldValue<DB, Option<T>> for Option<T>
where
    T: Send + Sync + Clone,
    for<'q> Option<T>: Encode<'q, DB> + Type<DB>,
{
    fn into_field_value(self) -> FieldValue<DB, Option<T>> {
        FieldValue::Value(self)
    }
}

// &Option<T> 的实现（clone）
impl<DB: Database, T> IntoFieldValue<DB, Option<T>> for &Option<T>
where
    T: Clone + Send + Sync,
    for<'q> Option<T>: Encode<'q, DB> + Type<DB>,
{
    fn into_field_value(self) -> FieldValue<DB, Option<T>> {
        FieldValue::Value(self.clone())
    }
}

// Option<String> 由泛型实现覆盖，无需特化

// Option<&str> 的实现（必要的 clone）
impl<DB: Database> IntoFieldValue<DB, Option<String>> for Option<&str>
where
    for<'q> Option<String>: Encode<'q, DB> + Type<DB> + Send + Sync + Clone,
{
    fn into_field_value(self) -> FieldValue<DB, Option<String>> {
        FieldValue::Value(self.map(|s| s.to_string()))
    }
}

// Option<&String> 的实现（必要的 clone）
impl<DB: Database> IntoFieldValue<DB, Option<String>> for Option<&String>
where
    for<'q> Option<String>: Encode<'q, DB> + Type<DB> + Send + Sync + Clone,
{
    fn into_field_value(self) -> FieldValue<DB, Option<String>> {
        FieldValue::Value(self.cloned())
    }
}