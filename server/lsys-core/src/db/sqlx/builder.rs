use sqlx::{Database, Encode, QueryBuilder, Type};

/// QueryBuilder 扩展 trait - 提供便捷的 SQL 构建方法
///
/// # 示例
/// ```ignore
/// use lsys_core::db::QueryBuilderExt;
///
/// let mut qb = QueryBuilder::<MySql>::new("SELECT * FROM users");
/// qb.push_where()
///   .field_eq("status", 1)
///   .push_and()
///   .field_in("role", [1, 2, 3]);
/// ```
pub trait QueryBuilderExt<'args, DB: Database> {
    /// 推入 " WHERE "
    fn push_where(&mut self) -> &mut Self;

    /// 推入 " AND "
    fn push_and(&mut self) -> &mut Self;

    /// 推入 " OR "
    fn push_or(&mut self) -> &mut Self;

    /// 推入 `field=?` 并绑定值
    fn field_eq<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field!=?` 并绑定值
    fn field_ne<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field>?` 并绑定值
    fn field_gt<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field>=?` 并绑定值
    fn field_gte<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field<?` 并绑定值
    fn field_lt<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field<=?` 并绑定值
    fn field_lte<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field LIKE ?` 并绑定值
    fn field_like<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field IN (?, ?, ...)`
    fn field_in<I>(&mut self, field: &str, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field IN (?, ?, ...)` - 自动 copy 切片元素（用于数值类型）
    fn field_in_copied<T>(&mut self, field: &str, slice: &[T]) -> &mut Self
    where
        T: Copy + 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field IN (?, ?, ...)` - 处理字符串切片（&[&str], &[String] 等），自动转换为 Vec<String>
    fn field_in_string<T>(&mut self, field: &str, slice: &[T]) -> &mut Self
    where
        T: ToString,
        String: 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field NOT IN (?, ?, ...)`
    fn field_not_in<I>(&mut self, field: &str, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field NOT IN (?, ?, ...)` - 自动 copy 切片元素（用于数值类型）
    fn field_not_in_copied<T>(&mut self, field: &str, slice: &[T]) -> &mut Self
    where
        T: Copy + 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field NOT IN (?, ?, ...)` - 处理字符串切片（&[&str], &[String] 等），自动转换为 Vec<String>
    fn field_not_in_string<T>(&mut self, field: &str, slice: &[T]) -> &mut Self
    where
        T: ToString,
        String: 'args + Encode<'args, DB> + Type<DB>;

    /// 推入 `field IS NULL`
    fn field_is_null(&mut self, field: &str) -> &mut Self;

    /// 推入 `field IS NOT NULL`
    fn field_not_null(&mut self, field: &str) -> &mut Self;

    /// 推入值列表 `(?, ?, ...)` (不含字段名)
    fn push_list<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: 'args + Encode<'args, DB> + Type<DB>;
}

impl<'args, DB: Database> QueryBuilderExt<'args, DB> for QueryBuilder<'args, DB> {
    fn push_where(&mut self) -> &mut Self {
        self.push(" WHERE ");
        self
    }

    fn push_and(&mut self) -> &mut Self {
        self.push(" AND ");
        self
    }

    fn push_or(&mut self) -> &mut Self {
        self.push(" OR ");
        self
    }

    fn field_eq<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>,
    {
        self.push(field);
        self.push("=");
        self.push_bind(value);
        self
    }

    fn field_ne<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>,
    {
        self.push(field);
        self.push("!=");
        self.push_bind(value);
        self
    }

    fn field_gt<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>,
    {
        self.push(field);
        self.push(">");
        self.push_bind(value);
        self
    }

    fn field_gte<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>,
    {
        self.push(field);
        self.push(">=");
        self.push_bind(value);
        self
    }

    fn field_lt<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>,
    {
        self.push(field);
        self.push("<");
        self.push_bind(value);
        self
    }

    fn field_lte<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>,
    {
        self.push(field);
        self.push("<=");
        self.push_bind(value);
        self
    }

    fn field_like<T>(&mut self, field: &str, value: T) -> &mut Self
    where
        T: 'args + Encode<'args, DB> + Type<DB>,
    {
        self.push(field);
        self.push(" LIKE ");
        self.push_bind(value);
        self
    }

    fn field_in<I>(&mut self, field: &str, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: 'args + Encode<'args, DB> + Type<DB>,
    {
        self.push(field);
        self.push(" IN ");
        self.push_list(iter);
        self
    }

    fn field_in_copied<T>(&mut self, field: &str, slice: &[T]) -> &mut Self
    where
        T: Copy + 'args + Encode<'args, DB> + Type<DB>,
    {
        self.push(field);
        self.push(" IN ");
        self.push_list(slice.iter().copied());
        self
    }

    fn field_in_string<T>(&mut self, field: &str, slice: &[T]) -> &mut Self
    where
        T: ToString,
        String: 'args + Encode<'args, DB> + Type<DB>,
    {
        let strings: Vec<String> = slice.iter().map(|s| s.to_string()).collect();
        self.field_in(field, strings)
    }

    fn field_not_in<I>(&mut self, field: &str, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: 'args + Encode<'args, DB> + Type<DB>,
    {
        self.push(field);
        self.push(" NOT IN ");
        self.push_list(iter);
        self
    }

    fn field_not_in_copied<T>(&mut self, field: &str, slice: &[T]) -> &mut Self
    where
        T: Copy + 'args + Encode<'args, DB> + Type<DB>,
    {
        self.push(field);
        self.push(" NOT IN ");
        self.push_list(slice.iter().copied());
        self
    }

    fn field_not_in_string<T>(&mut self, field: &str, slice: &[T]) -> &mut Self
    where
        T: ToString,
        String: 'args + Encode<'args, DB> + Type<DB>,
    {
        let strings: Vec<String> = slice.iter().map(|s| s.to_string()).collect();
        self.field_not_in(field, strings)
    }

    fn field_is_null(&mut self, field: &str) -> &mut Self {
        self.push(field);
        self.push(" IS NULL");
        self
    }

    fn field_not_null(&mut self, field: &str) -> &mut Self {
        self.push(field);
        self.push(" IS NOT NULL");
        self
    }

    fn push_list<I>(&mut self, iter: I) -> &mut Self
    where
        I: IntoIterator,
        I::Item: 'args + Encode<'args, DB> + Type<DB>,
    {
        self.push("(");
        let mut sep = self.separated(", ");
        for item in iter {
            sep.push_bind(item);
        }
        sep.push_unseparated(")");
        self
    }
}

/// WHERE 条件构建器 —— 自动管理 WHERE / AND 前缀
///
/// # 示例
/// ```ignore
/// use lsys_core::db::WhereClause;
/// use lsys_core::db::QueryBuilderExt;
///
/// let mut qb = QueryBuilder::<MySql>::new("SELECT * FROM users");
/// let mut wc = WhereClause::new(&mut qb);
/// wc.and().field_eq("status", 1);
/// ```
pub struct WhereClause<'a, 'args, DB: Database> {
    pub builder: &'a mut QueryBuilder<'args, DB>,
    has_condition: bool,
}

impl<'a, 'args, DB: Database> WhereClause<'a, 'args, DB> {
    /// 创建新的 WHERE 构建器
    ///
    /// # 参数
    /// - `builder`: QueryBuilder 的可变引用
    ///
    /// 默认 has_condition 为 false，如需其他值，调用 set_condition 设置
    pub fn new(builder: &'a mut QueryBuilder<'args, DB>) -> Self {
        Self {
            builder,
            has_condition: false,
        }
    }

    /// 设置是否已有条件
    pub fn set_condition(&mut self, has_condition: bool) -> &mut Self {
        self.has_condition = has_condition;
        self
    }

    /// 是否已添加过条件
    pub fn has_condition(&self) -> bool {
        self.has_condition
    }

    /// 添加条件分隔符：首次调用时添加 WHERE，后续调用添加指定的分隔符（如 AND/OR）
    pub fn split(&mut self, split: &str) -> &mut QueryBuilder<'args, DB> {
        if self.has_condition {
            self.builder.push(split);
        } else {
            self.builder.push(" WHERE ");
            self.has_condition = true;
        }
        self.builder
    }

    /// 获取内部 QueryBuilder 的可变引用，并标记已添加条件
    pub fn and(&mut self) -> &mut QueryBuilder<'args, DB> {
        self.split(" AND ");
        self.builder
    }

    /// 获取内部 QueryBuilder 的可变引用，并标记已添加条件（OR 连接）
    pub fn or(&mut self) -> &mut QueryBuilder<'args, DB> {
        self.split(" OR ");
        self.builder
    }

    /// 获取内部 QueryBuilder 的可变引用
    pub fn builder(&mut self) -> &mut QueryBuilder<'args, DB> {
        self.builder
    }
}
