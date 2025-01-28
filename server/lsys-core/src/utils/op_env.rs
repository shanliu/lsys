use crate::now_time;

#[derive(Clone)]
pub struct RequestEnv {
    pub request_time: u64,
    pub request_lang: Option<String>,
    pub request_ip: Option<String>,
    pub request_id: Option<String>,
    pub request_user_agent: Option<String>,
    pub device_id: Option<String>,
}

impl RequestEnv {
    pub fn new(
        request_lang: Option<&str>,
        request_ip: Option<&str>,
        request_id: Option<&str>,
        request_user_agent: Option<&str>,
        device_id: Option<&str>,
    ) -> Self {
        Self {
            request_lang: request_lang.map(|e| e.to_string()),
            request_time: now_time().unwrap_or_default(),
            request_ip: request_ip.map(|e| e.to_string()),
            request_id: request_id.map(|e| e.to_string()),
            device_id: device_id.map(|e| e.to_string()),
            request_user_agent: request_user_agent.map(|e| e.to_string()),
        }
    }
}
