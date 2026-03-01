use std::fmt::Display;

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
    /// Use `.sql_quote()` to create the value, ensuring proper SQL escaping.
    fn key_value(&self) -> Box<dyn Display>;
    /// Optional additional ORDER BY columns for deterministic ordering.
    /// Returns (column_name, sql_value) pairs. Use `.sql_quote()` for values.
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
    #[cfg(feature = "db-mysql")] //only mysql,so cfg db
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
#[cfg(feature = "db-mysql")]
pub struct CursorPageQuery<'a, 'b, C: CursorValue> {
    primary_key: &'b str,
    param: &'a CursorPageParam<C>,
}
#[cfg(feature = "db-mysql")]
impl<'a, 'b, C: CursorValue> CursorPageQuery<'a, 'b, C> {
    pub fn new(primary_key: &'b str, param: &'a CursorPageParam<C>) -> Self {
        Self { primary_key, param }
    }
    fn query_limit(&self) -> Option<u64> {
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
        if let Some(cursor) = &self.param.cursor {
            if !rows.is_empty() && filter_first(&rows[0], cursor) {
                rows.remove(0);
                has_first = true;
            }
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
    pub fn where_sql(&self) -> String {
        let Some(c) = self.param.cursor.as_ref() else {
            return String::new();
        };

        let op = |sort: CursorPageSort, dir: CursorPageDir, eq: bool| -> String {
            format!(
                "{}{}",
                match (sort, dir) {
                    (CursorPageSort::Desc, CursorPageDir::Next) => "<",
                    (CursorPageSort::Desc, CursorPageDir::Prev) => ">",
                    (CursorPageSort::Asc, CursorPageDir::Next) => ">",
                    (CursorPageSort::Asc, CursorPageDir::Prev) => "<",
                },
                if eq { "=" } else { "" }
            )
        };
        // cursor values
        let extra_values = c.extra_values();

        // Config Sorts
        let extra_sorts = self.param.config.cursor_extra();

        let mut ors: Vec<String> = Vec::new();

        // Build OR-chain for lexicographic comparison over extra columns
        for i in 0..extra_sorts.len() {
            let mut parts: Vec<String> = Vec::new();
            // equality for previous extra columns
            for (col_j, _) in &extra_sorts[0..i] {
                // Match by column name
                if let Some((_, val_j)) = extra_values.iter().find(|(k, _)| k == col_j) {
                    parts.push(format!("{} = {}", col_j, val_j));
                }
            }
            // comparison on the i-th extra column
            let (ref col_i, sort_i) = extra_sorts[i];
            if let Some((_, val_i)) = extra_values.iter().find(|(k, _)| k == col_i) {
                parts.push(format!(
                    "{} {} {}",
                    col_i,
                    op(sort_i, self.param.dir, false),
                    val_i
                ));
            }

            ors.push(parts.join(" AND "));
        }

        // Final clause: all extras equal AND primary key comparison
        let mut last_parts: Vec<String> = Vec::new();
        for (ref col_j, _) in extra_sorts {
            if let Some((_, val_j)) = extra_values.iter().find(|(k, _)| k == col_j) {
                last_parts.push(format!("{} = {}", col_j, val_j));
            }
        }
        last_parts.push(format!(
            "{} {} {}",
            self.primary_key,
            op(
                self.param.config.primary_sort(),
                self.param.dir,
                match self.param.limit {
                    CursorLimit::Limit { limit: _, more } => more,
                    CursorLimit::None => false,
                }
            ),
            c.key_value()
        ));
        ors.push(last_parts.join(" AND "));

        if ors.len() == 1 {
            ors.pop().unwrap_or_default()
        } else {
            format!("({})", ors.join(" OR "))
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
    pub fn limit_sql(&self) -> Option<String> {
        self.query_limit().map(|limit| format!("limit {}", limit))
    }

    /// 生成完整的查询 SQL 片段（WHERE + ORDER BY + LIMIT）
    ///
    /// # Arguments
    /// * `extra_where_suffix` - 额外的业务 WHERE 条件
    ///
    /// # Returns
    /// 返回组合好的 SQL 片段
    pub fn build_query_sql(&self, extra_where_suffix: Option<&str>) -> String {
        let mut parts = Vec::new();
        let cursor_sql = self.where_sql();
        let extra_where_sql = extra_where_suffix.unwrap_or_default();
        let where_clause = if !cursor_sql.is_empty() && !extra_where_sql.is_empty() {
            format!("WHERE {} AND {}", cursor_sql, extra_where_sql)
        } else if !cursor_sql.is_empty() && extra_where_sql.is_empty() {
            format!("WHERE {}", cursor_sql)
        } else if cursor_sql.is_empty() && !extra_where_sql.is_empty() {
            format!("WHERE {}", extra_where_sql)
        } else {
            String::new()
        };
        if !where_clause.is_empty() {
            parts.push(where_clause);
        }
        parts.push(self.order_by_sql());
        if let Some(limit) = self.limit_sql() {
            parts.push(limit);
        }
        parts.join(" ")
    }
}

// ----    CursorValue implementations  ----

impl CursorValue for u64 {
    fn key_value(&self) -> Box<dyn Display> {
        Box::new(*self)
    }
}
