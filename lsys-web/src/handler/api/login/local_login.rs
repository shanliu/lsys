use crate::{
    dao::RequestDao,
    {CaptchaParam, JsonData, JsonResult},
};

use lsys_user::dao::auth::UserAuthRedisStore;
use lsys_user::dao::auth::{
    EmailCodeLogin, EmailLogin, MobileCodeLogin, MobileLogin, NameLogin, UserAuthData,
    UserAuthSession, UserAuthTokenData,
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::json;

macro_rules! login_method {
    ($fn:ident,{$($name:ident:$name_type:ty),+$(,)*},{$($login_param:expr),+$(,)*}) => {
        pub async fn $fn<'t>(
            $($name:$name_type),+,
            req_dao: &RequestDao<UserAuthTokenData,UserAuthData,UserAuthSession<UserAuthRedisStore>>,
        ) -> JsonResult<JsonData> {
            req_dao
                .web_dao
                .user
                .rbac_dao
                .rbac
                .access
                .check(0, &[],&res_data!(SystemLogin) )
                .await?;
            let (token, data) = req_dao
                .web_dao
                .user
                .user_login(
                    &req_dao.user_session,
                    &req_dao.req_env,
                    $($login_param),+
                )
                .await?;
            Ok(JsonData::data(json!({
                "auth_data":data,
                "token":token.to_string(),
            })))
        }
    };
}
#[derive(Deserialize)]
pub struct NameLoginParam {
    name: String,
    password: String,
    captcha: Option<CaptchaParam>,
}
login_method!(
    user_login_from_name,
    { param: NameLoginParam },
    {
        NameLogin {
            name: param.name.clone(),
            password: param.password.clone(),
        },
        param.captcha
    }
);

#[derive(Deserialize)]
pub struct EmailLoginParam {
    email: String,
    password: String,
    captcha: Option<CaptchaParam>,
}
login_method!(
    user_login_from_email,
    { param: EmailLoginParam },
    {
        EmailLogin {
            password:param.password.clone(),
            email:param.email.clone(),
        },
        param.captcha
    }
);

#[derive(Deserialize)]
pub struct EmailCodeLoginParam {
    email: String,
    code: String,
    captcha: Option<CaptchaParam>,
}
login_method!(
    user_login_from_email_code,
    { param: EmailCodeLoginParam },
    {
        EmailCodeLogin {
            code:param.code.clone(),
            email:param.email.clone(),
        },
        param.captcha
    }
);

#[derive(Deserialize)]
pub struct MobileLoginParam {
    area_code: String,
    mobile: String,
    password: String,
    captcha: Option<CaptchaParam>,
}
login_method!(
    user_login_from_mobile,
    { param: MobileLoginParam },
    {
        MobileLogin {
            area_code:param.area_code.clone(),
            mobile:param.mobile.clone(),
            password:param.password.clone(),
        },
        param.captcha
    }
);

#[derive(Deserialize)]
pub struct MobileCodeLoginParam {
    area_code: String,
    mobile: String,
    code: String,
    captcha: Option<CaptchaParam>,
}
login_method!(
    user_login_from_mobile_code,
    { param: MobileCodeLoginParam },
    {
        MobileCodeLogin {
            area_code:param.area_code.clone(),
            mobile:param.mobile.clone(),
            code:param.code.clone(),
        },
        param.captcha
    }
);

#[derive(Deserialize)]
pub struct MobileSendCodeLoginParam {
    area_code: String,
    mobile: String,
    captcha: CaptchaParam,
}

pub async fn user_login_mobile_send_code<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: MobileSendCodeLoginParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let valid_code = req_dao
        .web_dao
        .captcha
        .valid_code(&crate::dao::CaptchaKey::LoginSmsCode);
    valid_code
        .check_code(&param.captcha.key, &param.captcha.code)
        .await?;
    let data = MobileCodeLogin::valid_code_set(
        req_dao.web_dao.redis.clone(),
        &mut EmailCodeLogin::valid_code_builder(),
        &param.area_code,
        &param.mobile,
    )
    .await?;
    req_dao
        .web_dao
        .smser
        .send_valid_code(&param.area_code, &param.mobile, &data.0, &data.1)
        .await?;
    let _ = valid_code
        .clear_code(
            &param.captcha.key,
            &mut req_dao.web_dao.captcha.valid_code_builder(),
        )
        .await;
    Ok(JsonData::message("login mail is send").set_data(json!({ "ttl": data.1 })))
}

#[derive(Deserialize)]
pub struct EmailSendCodeLoginParam {
    email: String,
    captcha: CaptchaParam,
}

pub async fn user_login_email_send_code<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: EmailSendCodeLoginParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let valid_code = req_dao
        .web_dao
        .captcha
        .valid_code(&crate::dao::CaptchaKey::LoginEmailCode);
    valid_code
        .check_code(&param.captcha.key, &param.captcha.code)
        .await?;
    let data = EmailCodeLogin::valid_code_set(
        req_dao.web_dao.redis.clone(),
        &mut EmailCodeLogin::valid_code_builder(),
        &param.email,
    )
    .await?;
    req_dao
        .web_dao
        .mailer
        .send_valid_code(&param.email, &data.0, &data.1)
        .await?;
    let _ = valid_code
        .clear_code(
            &param.captcha.key,
            &mut req_dao.web_dao.captcha.valid_code_builder(),
        )
        .await;
    Ok(JsonData::message("login mail is send").set_data(json!({ "ttl": data.1 })))
}
