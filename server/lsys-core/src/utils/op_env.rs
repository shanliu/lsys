use crate::now_time;

pub struct RequestEnv {
    pub request_time: u64,
    pub request_ip: Option<String>,
    pub request_id: Option<String>,
    pub request_user_agent: Option<String>,
}

impl RequestEnv {
    pub fn new(
        request_ip: Option<String>,
        request_id: Option<String>,
        request_user_agent: Option<String>,
    ) -> Self {
        Self {
            request_time: now_time().unwrap_or_default(),
            request_ip,
            request_id,
            request_user_agent,
        }
    }
}
