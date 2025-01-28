use std::{fmt::Display, str::FromStr};

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
