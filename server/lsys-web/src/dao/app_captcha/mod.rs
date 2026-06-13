mod captcha_data;
mod captcha_key;
pub use captcha_data::*;
pub use captcha_key::*;
use lsys_core::{fluents::IntoFluentMessage, valid_code::ValidCode};
use tracing::warn;

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
            Some(6),
        )
    }
    pub fn valid_code_builder(&self) -> CaptchaValidCodeData {
        CaptchaValidCodeData::new(60)
    }
    pub async fn destroy_code(&self, valid_code: &ValidCode, tag: &str) {
        if let Err(e) = valid_code
            .destroy_code(tag, &mut self.valid_code_builder())
            .await
        {
            warn!(
                "destroy_code: failed for tag={}: {}",
                tag,
                e.to_fluent_message().default_format()
            );
        }
    }
}
