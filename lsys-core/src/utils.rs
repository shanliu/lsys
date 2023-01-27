use std::time::{SystemTime, SystemTimeError};

pub fn now_time() -> Result<u64, SystemTimeError> {
    Ok(SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)?
        .as_secs())
}

pub struct PageParam {
    pub offset: u64,
    pub limit: u64,
}
impl PageParam {
    pub fn new(offset: u64, limit: u64) -> Self {
        Self { offset, limit }
    }
    pub fn page(page: u64, limit: u64) -> Self {
        let offset = if page > 0 { (page - 1) * limit } else { 0 };
        Self::new(offset, limit)
    }
}
