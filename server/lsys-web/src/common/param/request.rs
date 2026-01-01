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
impl From<&PageParam> for lsys_core::PageParam {
    fn from(p: &PageParam) -> Self {
        let limit = if p.limit > 100 { 100 } else { p.limit };
        lsys_core::PageParam::page(p.page, limit)
    }
}

#[derive(Debug, Deserialize)]
pub struct LimitParam {
    #[serde(default, deserialize_with = "super::deserialize_option_string")]
    pos: Option<String>, //起点位置，默认起点0，可传字符串或数字
    #[serde(default, deserialize_with = "super::deserialize_option_bool")]
    eq_pos: Option<bool>, //是否等于起点位置，传真或假
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
            eq_pos: None,
            limit: 10,
            forward: false,
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
            p.forward,
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
