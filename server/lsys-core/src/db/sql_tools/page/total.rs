// 总记录数阈值常量
pub const DEFAULT_TOTAL_COUNT_THRESHOLD: u64 = 10000;

/// 总记录数查询结果
#[derive(Debug, Clone)]
pub enum TotalRow {
    /// 精确数字（< 阈值）
    Exact(u64),
    /// 超过阈值（>= 阈值），u64 为阈值
    Over(u64),
}

impl TotalRow {
    /// 是否为精确数值
    pub fn is_exact(&self) -> bool {
        matches!(self, TotalRow::Exact(_))
    }
}

/// 总记录数查询参数（枚举）
#[derive(Debug, Clone)]
pub enum TotalParam {
    /// 阈值模式：查询时使用 LIMIT threshold+1 判断是否超过阈值
    Threshold(u64),
    /// 全量模式：不限制，获取精确的全部记录数
    Full,
}

impl Default for TotalParam {
    fn default() -> Self {
        Self::Threshold(DEFAULT_TOTAL_COUNT_THRESHOLD)
    }
}

impl TotalParam {
    pub fn total_count_query(&self) -> TotalRowQuery<'_> {
        TotalRowQuery::new(self)
    }
}

pub struct TotalRowQuery<'a> {
    param: &'a TotalParam,
}

impl<'a> TotalRowQuery<'a> {
    pub fn new(param: &'a TotalParam) -> Self {
        Self { param }
    }

    /// 判断是否为阈值模式
    pub fn is_threshold_mode(&self) -> bool {
        matches!(self.param, TotalParam::Threshold(_))
    }

    /// 获取阈值 LIMIT 的数值（threshold + 1），用于 push_bind
    pub fn threshold_limit(&self) -> Option<u64> {
        match self.param {
            TotalParam::Threshold(threshold) => Some(threshold + 1),
            TotalParam::Full => None,
        }
    }

    /// 根据实际查询结果行数判断总数
    pub fn finalize(&self, actual_rows: u64) -> TotalRow {
        match self.param {
            TotalParam::Threshold(threshold) => {
                if actual_rows > *threshold {
                    TotalRow::Over(*threshold)
                } else {
                    TotalRow::Exact(actual_rows)
                }
            }
            TotalParam::Full => TotalRow::Exact(actual_rows),
        }
    }

    #[cfg(feature = "db")]
    pub fn push_limit<DB>(&self, qb: &mut sqlx::QueryBuilder<'_, DB>)
    where
        DB: sqlx::Database,
    {
        if let Some(threshold) = self.threshold_limit() {
            qb.push(format!(" LIMIT {}", threshold));
        }
    }
}
