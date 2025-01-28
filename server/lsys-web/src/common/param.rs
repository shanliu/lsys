use serde::Deserialize;

// 定义一些公共参数

#[derive(Debug, Deserialize)]
pub struct PageParam {
    page: u64,
    limit: u64,
}
impl Default for PageParam {
    fn default() -> Self {
        Self { page: 1, limit: 10 }
    }
}
impl From<&PageParam> for lsys_core::PageParam {
    fn from(p: &PageParam) -> Self {
        let limit = if p.limit > 100 { 100 } else { p.limit };
        lsys_core::PageParam::page(p.page, limit)
    }
}

#[derive(Debug, Deserialize)]
pub struct LimitParam {
    pos: Option<String>,
    eq_pos: Option<bool>,
    limit: u64,
    next: bool,         //true > pos 的limit 条记录 false <pos 的 limit 条记录
    more: Option<bool>, //是否检测有下一页数据 null或false 不检测
}
impl Default for LimitParam {
    fn default() -> Self {
        Self {
            pos: None,
            eq_pos: None,
            limit: 10,
            next: false,
            more: Some(false),
        }
    }
}
impl From<&LimitParam> for lsys_core::LimitParam {
    fn from(p: &LimitParam) -> Self {
        let limit = if p.limit > 100 { 100 } else { p.limit };
        let pos = p
            .pos
            .as_ref()
            .map(|e| e.parse::<u64>().ok())
            .unwrap_or_default();
        lsys_core::LimitParam::new(
            pos,
            p.eq_pos.unwrap_or(false),
            limit,
            p.next,
            p.more.unwrap_or(false),
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
