use lsys_core::db::OffsetPageParam;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct PageParam {
    pub page: u64,
    pub limit: u64,
}

pub trait ToOffsetPageParam {
    fn to_offset_page_param(&self) -> OffsetPageParam;
}
impl ToOffsetPageParam for Option<PageParam> {
    fn to_offset_page_param(&self) -> OffsetPageParam {
        //这里控制分页默认值
        self.as_ref()
            .map(|e| e.to_offset_page_param())
            .unwrap_or_else(|| {
                ToOffsetPageParam::to_offset_page_param(&PageParam { page: 1, limit: 10 })
            })
    }
}
impl ToOffsetPageParam for PageParam {
    fn to_offset_page_param(&self) -> OffsetPageParam {
        let limit = if self.limit > 100 { 100 } else { self.limit };
        let offset = (self.page.saturating_sub(1)) * limit;
        OffsetPageParam::new(Some(lsys_core::db::OffsetPageValue::new(offset, limit)))
    }
}
