mod captcha_key;
pub use captcha_key::*;
use lsys_core::ValidCode;

use crate::common::ValidCodeDataCaptcha;

pub struct AppCaptcha {
    redis: deadpool_redis::Pool,
}

impl AppCaptcha {
    pub fn new(redis: deadpool_redis::Pool) -> Self {
        Self { redis }
    }
    pub fn valid_code(&self, captcha_key: &CaptchaKey) -> ValidCode {
        ValidCode::new(
            self.redis.clone(),
            &format!("captcha-{}", captcha_key),
            true,
        )
    }
    pub fn valid_code_builder(&self) -> ValidCodeDataCaptcha {
        ValidCodeDataCaptcha::default()
    }
    pub async fn clear_code(&self, valid_code: &ValidCode, tag: &str) {
        let _ = valid_code
            .clear_code(tag, &mut self.valid_code_builder())
            .await;
    }
}
