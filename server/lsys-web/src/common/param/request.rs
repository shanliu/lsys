use lsys_core::db::{
    CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort, OffsetPageParam,
    OffsetPageValue,
};
use serde::Deserialize;
#[derive(Debug, Deserialize)]
pub struct PageParam {
    #[serde(deserialize_with = "super::deserialize_u64")]
    page: u64,
    #[serde(deserialize_with = "super::deserialize_u64")]
    limit: u64,
}
impl Default for PageParam {
    fn default() -> Self {
        Self { page: 1, limit: 10 }
    }
}

pub trait ToOffsetPageParam {
    fn to_offset_page_param(&self) -> OffsetPageParam;
}
impl ToOffsetPageParam for Option<PageParam> {
    fn to_offset_page_param(&self) -> OffsetPageParam {
        //这里控制分页默认值
        self.as_ref()
            .map(|e| e.to_offset_page_param())
            .unwrap_or_else(|| ToOffsetPageParam::to_offset_page_param(&PageParam::default()))
    }
}
impl ToOffsetPageParam for PageParam {
    fn to_offset_page_param(&self) -> OffsetPageParam {
        let limit = if self.limit > 100 { 100 } else { self.limit };
        let offset = (self.page.saturating_sub(1)) * limit;
        OffsetPageParam::new(Some(OffsetPageValue::new(offset, limit)))
    }
}

#[derive(Debug, Deserialize)]
pub struct LimitParam {
    #[serde(default, deserialize_with = "super::deserialize_option_string")]
    pos: Option<String>, //起点位置，默认起点0，可传字符串或数字
    #[serde(deserialize_with = "super::deserialize_u64")]
    limit: u64, //显示数量
    #[serde(deserialize_with = "super::deserialize_bool")]
    forward: bool, //获取上一页还是下一页
    #[serde(default, deserialize_with = "super::deserialize_option_bool")]
    more: Option<bool>, //是否检测有下一页数据 null或false 不检测
}
impl Default for LimitParam {
    fn default() -> Self {
        Self {
            pos: None,
            limit: 10,
            forward: false,
            more: Some(false),
        }
    }
}

pub trait ToCursorPageParam {
    fn to_u64_cursor_page_param(&self, primary_sort: CursorPageSort) -> CursorPageParam<u64>;
}
impl ToCursorPageParam for Option<LimitParam> {
    fn to_u64_cursor_page_param(&self, primary_sort: CursorPageSort) -> CursorPageParam<u64> {
        self.as_ref()
            .map(|e| e.to_u64_cursor_page_param(primary_sort))
            .unwrap_or_else(|| LimitParam::default().to_u64_cursor_page_param(primary_sort))
    }
}
impl ToCursorPageParam for LimitParam {
    fn to_u64_cursor_page_param(&self, primary_sort: CursorPageSort) -> CursorPageParam<u64> {
        let limit = if self.limit > 100 { 100 } else { self.limit };
        let offset = self.pos.as_ref().and_then(|e| e.parse::<u64>().ok());
        CursorPageParam::new(
            if self.forward {
                CursorPageDir::Next
            } else {
                CursorPageDir::Prev
            },
            CursorConfig::primary(primary_sort),
            offset,
            CursorLimit::Limit {
                limit,
                more: self.more.unwrap_or(false),
            },
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct CaptchaParam {
    pub key: String,
    pub code: String,
}

impl<'t> From<&'t CaptchaParam> for lsys_core::CheckCodeData<'t> {
    fn from(p: &'t CaptchaParam) -> lsys_core::CheckCodeData<'t> {
        lsys_core::CheckCodeData::new(&p.key, &p.code)
    }
}
