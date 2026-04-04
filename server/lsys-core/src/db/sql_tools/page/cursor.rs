use std::fmt::Display;

#[cfg(feature = "db")]
use sqlx::QueryBuilder;
#[cfg(feature = "db-mysql")]
use sqlx::MySql;
#[cfg(feature = "db-postgres")]
use sqlx::Postgres;
#[cfg(feature = "db-sqlite")]
use sqlx::Sqlite;

pub struct CursorConfig {
    primary_sort: CursorPageSort,
    cursor_extra: Vec<(String, CursorPageSort)>,
}
impl CursorConfig {
    pub fn new(primary_sort: CursorPageSort, cursor_extra: Vec<(String, CursorPageSort)>) -> Self {
        Self {
            primary_sort,
            cursor_extra,
        }
    }
    pub fn primary(primary_sort: CursorPageSort) -> Self {
        Self {
            primary_sort,
            cursor_extra: Vec::new(),
        }
    }
    pub fn primary_sort(&self) -> CursorPageSort {
        self.primary_sort
    }
    pub fn cursor_extra(&self) -> &Vec<(String, CursorPageSort)> {
        &self.cursor_extra
    }
}
// 游标值数据
pub trait CursorValue {
    /// SQL-formatted primary key value, returns Box<dyn Display> for type erasure.
    /// 注意：建议实现 `CursorBind` trait 代替此方法，以使用 bind 参数。
    fn key_value(&self) -> Box<dyn Display>;
    /// Optional additional ORDER BY columns for deterministic ordering.
    /// Returns (column_name, sql_value) pairs.
    /// 注意：建议实现 `CursorBind::push_extra_bind()` 代替此方法。
    fn extra_values(&self) -> Vec<(String, Box<dyn Display>)> {
        Vec::new()
    }
}

pub enum CursorLimit {
    None,
    Limit { limit: u64, more: bool },
}

//不实现DEFAULT ,默认值由外部去控制,不在公共实现中控制

pub struct CursorPageParam<C: CursorValue> {
    pub config: CursorConfig,
    pub dir: CursorPageDir,
    pub cursor: Option<C>,
    pub limit: CursorLimit,
}

impl<C: CursorValue> CursorPageParam<C> {
    pub fn new(
        dir: CursorPageDir,
        config: CursorConfig,
        cursor: Option<C>,
        limit: CursorLimit,
    ) -> Self {
        Self {
            dir,
            config,
            cursor,
            limit,
        }
    }
    #[cfg(feature = "db")]
    pub fn page_query<'a, 'b>(&'a self, primary_key: &'b str) -> CursorPageQuery<'a, 'b, C> {
        CursorPageQuery::new(primary_key, self)
    }
}

//表主排序唯一数据排序方法
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CursorPageSort {
    Asc,
    Desc,
}

impl CursorPageSort {
    pub fn as_sql(&self) -> &'static str {
        match self {
            CursorPageSort::Asc => "asc",
            CursorPageSort::Desc => "desc",
        }
    }
}
//数据获取方向
#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum CursorPageDir {
    Next,
    Prev,
}

#[derive(Debug, Default)]
pub struct CursorPageData<C: CursorValue> {
    pub next_cursor: Option<C>,
    pub prev_cursor: Option<C>,
}
#[cfg(feature = "db")]
pub struct CursorPageQuery<'a, 'b, C: CursorValue> {
    primary_key: &'b str,
    param: &'a CursorPageParam<C>,
}
#[cfg(feature = "db")]
impl<'a, 'b, C: CursorValue> CursorPageQuery<'a, 'b, C> {
    pub fn new(primary_key: &'b str, param: &'a CursorPageParam<C>) -> Self {
        Self { primary_key, param }
    }
    pub fn query_limit(&self) -> Option<u64> {
        match &self.param.limit {
            CursorLimit::None => None,
            CursorLimit::Limit { limit, more } => {
                Some(
                    limit
                        + if self.param.cursor.is_some() {
                            if *more {
                                2 //多取一个等于当前记录 还有下一个记录
                            } else {
                                1 //多取一个等于当前记录
                            }
                        } else if *more {
                            1 //多取一个下一个记录
                        } else {
                            0 //不多取
                        },
                )
            }
        }
    }
    /// V1：整理结果，并基于 head/tail 生成 cursor。
    /// `get_cursor`：从返回记录中提取 cursor（例如 `|row| row.id`）。
    pub fn finalize<T, F1, F2>(
        &self,
        rows: &mut Vec<T>,
        filter_first: F1,
        get_cursor: F2,
    ) -> CursorPageData<C>
    where
        F1: Fn(&T, &C) -> bool,
        F2: Fn(&T) -> C,
    {
        let mut has_first = false;
        if let Some(cursor) = &self.param.cursor
            && !rows.is_empty() && filter_first(&rows[0], cursor)
        {
            rows.remove(0);
            has_first = true;
        }
        match self.param.limit {
            CursorLimit::None => CursorPageData {
                next_cursor: None,
                prev_cursor: None,
            },
            CursorLimit::Limit { limit, more: _ } => {
                let has_more = rows.len() as u64 > limit;
                if has_more {
                    rows.truncate(limit as usize);
                }
                let mut next_cursor = None;
                let mut prev_cursor = None;

                if !rows.is_empty() {
                    match self.param.dir {
                        CursorPageDir::Next => {
                            //99-90 // 99-90 上一页 99 下一页 90
                            if has_more {
                                next_cursor = Some(get_cursor(&rows[rows.len() - 1]));
                            }
                            if has_first {
                                prev_cursor = Some(get_cursor(&rows[0]));
                            }
                        }
                        CursorPageDir::Prev => {
                            //90-99 // 99-90 上一页 99 下一页 90
                            if has_more {
                                prev_cursor = Some(get_cursor(&rows[rows.len() - 1]));
                            }
                            if has_first {
                                next_cursor = Some(get_cursor(&rows[0]));
                            }
                        }
                    };
                }
                if self.param.dir == CursorPageDir::Prev {
                    rows.reverse();
                };
                CursorPageData {
                    next_cursor,
                    prev_cursor,
                }
            }
        }
    }
    pub fn order_by_sql(&self) -> String {
        let sort_key_dir = |sort: &CursorPageSort, dir: &CursorPageDir| -> &'static str {
            match (sort, dir) {
                (CursorPageSort::Desc, CursorPageDir::Next) => "desc",
                (CursorPageSort::Desc, CursorPageDir::Prev) => "asc",
                (CursorPageSort::Asc, CursorPageDir::Next) => "asc",
                (CursorPageSort::Asc, CursorPageDir::Prev) => "desc",
            }
        };

        let mut parts: Vec<String> = Vec::new();
        // 1. Extra Sorts
        for (col, sort) in self.param.config.cursor_extra() {
            parts.push(format!("{} {}", col, sort_key_dir(sort, &self.param.dir)));
        }
        // 2. Primary Key Sort
        parts.push(format!(
            "{} {}",
            self.primary_key,
            sort_key_dir(&self.param.config.primary_sort(), &self.param.dir)
        ));

        format!("order by {}", parts.join(", "))
    }

    pub fn push_order_by<DB>(&self, qb: &mut QueryBuilder<'_, DB>)
    where
        DB: sqlx::Database,
    {
        qb.push(" ");
        qb.push(self.order_by_sql());
    }

    pub fn push_limit<DB>(&self, qb: &mut QueryBuilder<'_, DB>)
    where
        DB: sqlx::Database,
    {
        if let Some(limit) = self.query_limit() {
            // 使用 SQL 字面量而不是 bind，避免类型问题
            qb.push(format!(" limit {}", limit));
        }
    }
}

// ----    CursorValue implementations  ----

impl CursorValue for u64 {
    fn key_value(&self) -> Box<dyn Display> {
        Box::new(*self)
    }
}

// ----    CursorBind: bind-aware cursor trait (multi-database)  ----

/// Bind-aware cursor trait —— 为游标分页提供参数化绑定支持。
///
/// 相比 `CursorValue`（返回 `Box<dyn Display>` 后字符串内联），
/// `CursorBind` 通过 `push_bind` 将值安全地绑定到 `QueryBuilder`，
/// 避免 SQL 注入和跨数据库转义问题。
///
/// 泛型参数 `DB` 指定目标数据库后端（MySql / Postgres / Sqlite）。
///
/// # 示例 (MySQL)
/// ```ignore
/// impl CursorBind<MySql> for MyCursor {
///     fn push_key_bind(&self, qb: &mut QueryBuilder<'_, MySql>) {
///         qb.push_bind(self.id);
///     }
///     fn push_extra_bind(&self, col_index: usize, qb: &mut QueryBuilder<'_, MySql>) {
///         if col_index == 0 {
///             qb.push_bind(self.name.clone());
///         }
///     }
/// }
/// ```
#[cfg(feature = "db")]
pub trait CursorBind<DB: sqlx::Database>: CursorValue {
    /// 将主键值推入 QueryBuilder 作为绑定参数
    fn push_key_bind(&self, qb: &mut QueryBuilder<'_, DB>);
    /// 将第 col_index 个附加排序列的值推入 QueryBuilder 作为绑定参数
    fn push_extra_bind(&self, _col_index: usize, _qb: &mut QueryBuilder<'_, DB>) {}
}

#[cfg(feature = "db-mysql")]
impl CursorBind<MySql> for u64 {
    fn push_key_bind(&self, qb: &mut QueryBuilder<'_, MySql>) {
        qb.push_bind(*self);
    }
}

#[cfg(feature = "db-postgres")]
impl CursorBind<Postgres> for u64 {
    fn push_key_bind(&self, qb: &mut QueryBuilder<'_, Postgres>) {
        qb.push_bind(*self as i64);
    }
}

#[cfg(feature = "db-sqlite")]
impl CursorBind<Sqlite> for u64 {
    fn push_key_bind(&self, qb: &mut QueryBuilder<'_, Sqlite>) {
        qb.push_bind(*self as i64);
    }
}

/// CursorPageQuery 的 bind 版本方法 —— 需要 `C: CursorBind<DB>`
#[cfg(feature = "db")]
impl<'a, 'b, C: CursorValue> CursorPageQuery<'a, 'b, C> {
    /// 将游标 WHERE 条件推入 QueryBuilder（使用 bind 参数）。
    ///
    /// 如果无游标（cursor 为 None），不推入任何内容。
    /// 调用方应检查 `has_cursor()` 来决定是否需要 AND 连接。
    pub fn push_where<DB: sqlx::Database>(&self, qb: &mut QueryBuilder<'_, DB>)
    where
        C: CursorBind<DB>,
    {
        use crate::db::QueryBuilderExt;
        let Some(c) = self.param.cursor.as_ref() else {
            return;
        };

        let op =
            |sort: CursorPageSort, dir: CursorPageDir, eq: bool| -> &'static str {
                match (sort, dir, eq) {
                    (CursorPageSort::Desc, CursorPageDir::Next, false) => "<",
                    (CursorPageSort::Desc, CursorPageDir::Next, true) => "<=",
                    (CursorPageSort::Desc, CursorPageDir::Prev, false) => ">",
                    (CursorPageSort::Desc, CursorPageDir::Prev, true) => ">=",
                    (CursorPageSort::Asc, CursorPageDir::Next, false) => ">",
                    (CursorPageSort::Asc, CursorPageDir::Next, true) => ">=",
                    (CursorPageSort::Asc, CursorPageDir::Prev, false) => "<",
                    (CursorPageSort::Asc, CursorPageDir::Prev, true) => "<=",
                }
            };

        let extra_sorts = self.param.config.cursor_extra();
        let pk_eq = match self.param.limit {
            CursorLimit::Limit { more, .. } => more,
            CursorLimit::None => false,
        };

        if extra_sorts.is_empty() {
            // 简单情况：仅主键比较
            qb.push(format!(
                "{} {} ",
                self.primary_key,
                op(self.param.config.primary_sort(), self.param.dir, pk_eq)
            ));
            c.push_key_bind(qb);
        } else {
            // 复杂情况：多列排序的 OR 链
            qb.push("(");
            for i in 0..extra_sorts.len() {
                if i > 0 {
                    qb.push(" OR ");
                }
                qb.push("(");
                // 前面列的等值条件
                for (j, (col_j, _)) in extra_sorts.iter().enumerate().take(i) {
                    qb.push(format!("{} = ", col_j));
                    c.push_extra_bind(j, qb);
                    qb.push_and();
                }
                // 第 i 列的比较条件
                let (ref col_i, sort_i) = extra_sorts[i];
                qb.push(format!(
                    "{} {} ",
                    col_i,
                    op(sort_i, self.param.dir, false)
                ));
                c.push_extra_bind(i, qb);
                qb.push(")");
            }
            // 最后一个 OR 分支：所有附加列等值 + 主键比较
            qb.push(" OR (");
            for (j, (col_j, _)) in extra_sorts.iter().enumerate() {
                qb.push(format!("{} = ", col_j));
                c.push_extra_bind(j, qb);
                qb.push_and();
            }
            qb.push(format!(
                "{} {} ",
                self.primary_key,
                op(self.param.config.primary_sort(), self.param.dir, pk_eq)
            ));
            c.push_key_bind(qb);
            qb.push("))");
        }
    }

    /// 是否存在游标值
    pub fn has_cursor(&self) -> bool {
        self.param.cursor.is_some()
    }
}
