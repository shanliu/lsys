use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct MfaSubject {
    pub app_id: u64,
    pub user_data: String,
}

impl MfaSubject {
    pub fn new(app_id: u64, user_data: impl Into<String>) -> Self {
        Self {
            app_id,
            user_data: user_data.into(),
        }
    }
}
