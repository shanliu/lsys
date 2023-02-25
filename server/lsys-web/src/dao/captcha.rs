use std::{fmt::Display, str::FromStr};

use lsys_core::ValidCode;

use crate::module::captcha::ValidCodeDataCaptcha;

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum CaptchaKey {
    ResetPasswordMail,
    ResetPasswordSms,
    RegSms,
    RegEmail,
    Login,
    LoginEmailCode,
    LoginSmsCode,
    AddEmailCode,
    AddSmsCode,
}
const CAPTCHA_MAP: &[(CaptchaKey, &str)] = &[
    (CaptchaKey::ResetPasswordMail, "reset-password-send-mail"),
    (CaptchaKey::ResetPasswordSms, "reset-password-send-sms"),
    (CaptchaKey::Login, "login"),
    (CaptchaKey::RegSms, "reg-sms"),
    (CaptchaKey::RegEmail, "reg-email"),
    (CaptchaKey::LoginEmailCode, "login-email"),
    (CaptchaKey::LoginSmsCode, "login-sms"),
    (CaptchaKey::AddEmailCode, "add-email"),
    (CaptchaKey::AddSmsCode, "add-sms"),
];
impl FromStr for CaptchaKey {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        for (key, str_key) in CAPTCHA_MAP {
            if *str_key == s {
                return Ok(*key);
            }
        }
        Err(format!("parse captcha key is fail,not find this key:{}", s))
    }
}
impl Display for CaptchaKey {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        for (key, str_key) in CAPTCHA_MAP {
            if key == self {
                return f.write_str(str_key);
            }
        }
        f.write_str("")
    }
}

pub struct WebAppCaptcha {
    redis: deadpool_redis::Pool,
}

impl WebAppCaptcha {
    pub fn new(redis: deadpool_redis::Pool) -> Self {
        Self { redis }
    }
    pub fn valid_code(&self, captcha_key: &CaptchaKey) -> ValidCode {
        ValidCode::new(self.redis.clone(), format!("captcha-{}", captcha_key), true)
    }
    pub fn valid_code_builder(&self) -> ValidCodeDataCaptcha {
        ValidCodeDataCaptcha::default()
    }
    pub async fn clear_code(&self, valid_code: &ValidCode, tag: &String) {
        let _ = valid_code
            .clear_code(tag, &mut self.valid_code_builder())
            .await;
    }
}
