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
    #[cfg(feature = "db")] //only mysql,so cfg db
    pub fn page_query(&self) -> OffsetPageQuery<'_> {
        OffsetPageQuery::new(self)
    }
}
#[cfg(feature = "db")]
pub struct OffsetPageQuery<'a> {
    pub param: &'a OffsetPageParam,
}
#[cfg(feature = "db")]
impl<'a> OffsetPageQuery<'a> {
    pub fn new(param: &'a OffsetPageParam) -> Self {
        Self { param }
    }
    pub fn limit_sql(&self) -> Option<String> {
        self.param
            .value
            .as_ref()
            .map(|val| format!(" limit {} offset {}", val.limit, val.offset))
    }
}
