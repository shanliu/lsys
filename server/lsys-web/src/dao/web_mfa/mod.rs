use lsys_mfa::dao::MfaTotpDao;
use std::sync::Arc;

pub struct WebMfa {
    pub totp_dao: Arc<MfaTotpDao>,
}

impl WebMfa {
    pub fn new(totp_dao: Arc<MfaTotpDao>) -> Self {
        Self { totp_dao }
    }
}
