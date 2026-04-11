pub struct OffsetPageValue {
    pub offset: u64,
    pub limit: u64,
}
impl OffsetPageValue {
    pub fn new(offset: u64, limit: u64) -> Self {
        Self { offset, limit }
    }
    pub fn page(page: u64, limit: u64) -> Self {
        let offset = if page > 0 { (page - 1) * limit } else { 0 };
        Self::new(offset, limit)
    }
}

//不实现DEFAULT ,默认值由外部去控制,不在公共实现中控制
pub struct OffsetPageParam {
    #[allow(dead_code)]
    value: Option<OffsetPageValue>,
}

impl OffsetPageParam {
    pub fn new(value: Option<OffsetPageValue>) -> Self {
        Self { value }
    }
    pub fn page_value(&self) -> Option<&OffsetPageValue> {
        self.value.as_ref()
    }
}

#[cfg(feature = "db")]
impl OffsetPageParam {
    pub fn push_limit<DB>(&self, qb: &mut sqlx::QueryBuilder<'_, DB>)
    where
        DB: sqlx::Database,
    {
        if let Some(pv) = &self.value {
            qb.push(format!(" limit {} offset {}", pv.limit, pv.offset));
        }
    }
}
