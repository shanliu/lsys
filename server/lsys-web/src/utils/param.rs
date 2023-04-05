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
impl From<PageParam> for lsys_core::PageParam {
    fn from(p: PageParam) -> Self {
        lsys_core::PageParam::page(p.page, p.limit)
    }
}

#[derive(Debug, Deserialize)]
pub struct CaptchaParam {
    pub key: String,
    pub code: String,
}
