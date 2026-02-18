use std::borrow::Cow;
use std::marker::PhantomData;
use std::ops::Deref;

/// 字段元信息（无泛型，可放入数组）
/// 使用 Cow 支持静态和动态字符串
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FieldMeta {
    /// Rust 结构体字段名
    pub name: Cow<'static, str>,
    /// 数据库列名
    pub column: Cow<'static, str>,
}

impl FieldMeta {
    /// 创建字段元信息（const 版本，用于编译时常量）
    pub const fn new(name: &'static str, column: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            column: Cow::Borrowed(column),
        }
    }

    /// 字段名与列名相同（const 版本）
    pub const fn same(name: &'static str) -> Self {
        Self {
            name: Cow::Borrowed(name),
            column: Cow::Borrowed(name),
        }
    }
}

/// 类型安全的字段标识符
///
/// 内部包含 FieldMeta，通过 Deref 可直接访问 name/column
#[derive(Debug)]
pub struct Field<T> {
    meta: FieldMeta,
    _marker: PhantomData<T>,
}

impl<T> Field<T> {
    /// 创建字段（字段名与列名相同）- const 版本
    pub const fn new(name: &'static str) -> Self {
        Self {
            meta: FieldMeta::same(name),
            _marker: PhantomData,
        }
    }

    /// 创建字段（指定不同的列名）- const 版本
    pub const fn with_column(name: &'static str, column: &'static str) -> Self {
        Self {
            meta: FieldMeta::new(name, column),
            _marker: PhantomData,
        }
    }

    /// 从 FieldMeta 创建（用于动态场景）
    pub fn from_meta(meta: FieldMeta) -> Self {
        Self {
            meta,
            _marker: PhantomData,
        }
    }

    /// 获取字段元信息的引用
    pub fn meta(&self) -> &FieldMeta {
        &self.meta
    }
}

// 通过 Deref 直接访问 field.name / field.column

impl<T> Deref for Field<T> {
    type Target = FieldMeta;

    fn deref(&self) -> &Self::Target {
        &self.meta
    }
}

impl<T> Clone for Field<T> {
    fn clone(&self) -> Self {
        Self {
            meta: self.meta.clone(),
            _marker: PhantomData,
        }
    }
}

impl<T> std::fmt::Display for Field<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.meta.column.as_ref())
    }
}
