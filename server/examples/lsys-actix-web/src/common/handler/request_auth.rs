use std::{ops::Deref, str::FromStr};

use actix_utils::future::{err, ok, Ready};
use actix_web::{dev::Payload, web::Data, FromRequest, HttpMessage, HttpRequest};

use lsys_web::lsys_app::dao::{RestAuthSession, RestAuthToken};
use lsys_web::lsys_core::{now_time, IntoFluentMessage, RequestEnv};

use lsys_web::lsys_user::dao::{UserAuthSession, UserAuthToken};
use lsys_web::{
    common::{
        JsonData, JsonResponse, RequestAuthDao as Request, RequestSessionToken, RestAuthQueryDao,
        UserAuthQueryDao,
    },
    dao::WebDao,
};

use actix_http::header;

use super::{ResponseJson, AUTH_COOKIE_NAME};

//正常用户登陆，如cookie登陆

pub struct UserAuthQuery {
    pub inner: UserAuthQueryDao,
    pub req: HttpRequest,
}

impl Deref for UserAuthQuery {
    type Target = UserAuthQueryDao;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl FromRequest for UserAuthQuery {
    type Error = ResponseJson;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let user_dao_opt = req.app_data::<Data<WebDao>>();
        match user_dao_opt {
            Some(app_dao) => {
                let user_lang = req
                    .headers()
                    .get(header::ACCEPT_LANGUAGE)
                    .and_then(|t| t.to_str().map(|s| s.split(',').next().unwrap_or(s)).ok())
                    .unwrap_or("zh_CN")
                    .replace('-', "_");
                let user_agent = req
                    .headers()
                    .get("User-Agent")
                    .and_then(|e| e.to_str().ok());
                let request_id = req
                    .headers()
                    .get("X-Request-ID")
                    .and_then(|e| e.to_str().ok());
                let device_id = req
                    .headers()
                    .get("X-Device-ID")
                    .and_then(|e| e.to_str().ok());
                let ip = req.connection_info();
                let env = match RequestEnv::new(
                    Some(&user_lang),
                    ip.realip_remote_addr(),
                    request_id,
                    user_agent,
                    device_id,
                ) {
                    Ok(tmp) => tmp,
                    Err(verr) => {
                        return err(JsonResponse::data(
                            JsonData::default()
                                .set_sub_code("env_valid_err")
                                .set_code(400),
                        )
                        .set_message(verr.to_fluent_message().default_format())
                        .into())
                    }
                };
                ok(Self {
                    inner: Request::new(
                        app_dao.clone().into_inner(),
                        env,
                        UserAuthSession::new(
                            app_dao.web_user.user_dao.auth_dao.clone(),
                            UserAuthToken::default(),
                        ),
                    ),
                    req: req.to_owned(),
                })
            }
            None => err(JsonResponse::data(JsonData::error())
                .set_message("not find webdao")
                .into()),
        }
    }
}

//COOKIE登陆实现[默认实现]
pub struct CookieToken<'t> {
    request_dao: &'t UserAuthQuery,
}
impl<'t> From<&'t UserAuthQuery> for CookieToken<'t> {
    fn from(request_dao: &'t UserAuthQuery) -> Self {
        Self { request_dao }
    }
}
impl CookieToken<'_> {
    pub fn set_token(&self, token: UserAuthToken) {
        self.request_dao
            .req
            .extensions_mut()
            .insert::<UserAuthToken>(token);
    }
}

impl RequestSessionToken<UserAuthToken> for CookieToken<'_> {
    fn get_user_token(&self) -> UserAuthToken {
        if let Some(cookie) = self.request_dao.req.cookie(AUTH_COOKIE_NAME) {
            UserAuthToken::from_str(cookie.value()).unwrap_or_default()
        } else {
            UserAuthToken::default()
        }
    }
    fn is_refresh(&self, token: &UserAuthToken) -> bool {
        if !token.token.is_empty() {
            return now_time().unwrap_or_default() - 30 > token.time_out;
        }
        false
    }
    fn refresh_user_token(&self, token: &UserAuthToken) {
        self.set_token(token.to_owned());
    }
}

//oauth 登陆实现，跟普通登陆实现方式不相同
pub struct OauthAuthQuery {
    pub inner: RestAuthQueryDao,
    // pub req: HttpRequest,
}

impl Deref for OauthAuthQuery {
    type Target = RestAuthQueryDao;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl FromRequest for OauthAuthQuery {
    type Error = ResponseJson;
    type Future = Ready<Result<Self, Self::Error>>;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let user_dao_opt = req.app_data::<Data<WebDao>>();
        match user_dao_opt {
            Some(app_dao) => {
                let user_lang = req
                    .headers()
                    .get(header::ACCEPT_LANGUAGE)
                    .and_then(|t| t.to_str().map(|s| s.split(',').next().unwrap_or(s)).ok())
                    .unwrap_or("zh_CN")
                    .replace('-', "_");
                let user_agent = req
                    .headers()
                    .get("User-Agent")
                    .and_then(|e| e.to_str().ok());
                let request_id = req
                    .headers()
                    .get("X-Request-ID")
                    .and_then(|e| e.to_str().ok());
                let device_id = req
                    .headers()
                    .get("X-Device-ID")
                    .and_then(|e| e.to_str().ok());
                let ip = req.connection_info();
                let env = match RequestEnv::new(
                    Some(&user_lang),
                    ip.realip_remote_addr(),
                    request_id,
                    user_agent,
                    device_id,
                ) {
                    Ok(tmp) => tmp,
                    Err(verr) => {
                        return err(JsonResponse::data(
                            JsonData::default()
                                .set_sub_code("env_valid_err")
                                .set_code(400),
                        )
                        .set_message(verr.to_fluent_message().default_format())
                        .into())
                    }
                };

                ok(Self {
                    inner: Request::new(
                        app_dao.clone().into_inner(),
                        env,
                        RestAuthSession::new(
                            app_dao.web_app.app_dao.clone(),
                            RestAuthToken::default(),
                        ),
                    ),
                })
            }
            None => err(JsonResponse::data(JsonData::error())
                .set_message("not find webdao")
                .into()),
        }
    }
}
