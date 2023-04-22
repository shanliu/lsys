use std::{ops::Deref, str::FromStr};

use actix_utils::future::{err, ok, Ready};
use actix_web::{dev::Payload, web::Data, FromRequest, HttpMessage, HttpRequest};

use lsys_app::dao::session::RestAuthSession;
use lsys_core::now_time;
use lsys_user::dao::auth::{SessionToken, UserAuthSession, UserAuthTokenData};
use lsys_web::{
    dao::{RequestDao as Request, RequestEnv, RequestToken, RestAuthQueryDao},
    dao::{UserAuthQueryDao, WebDao},
    JsonData,
};

use reqwest::header::HeaderValue;

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
                let user_agent = req
                    .headers()
                    .get("User-Agent")
                    .unwrap_or(&HeaderValue::from_static(""))
                    .to_str()
                    .unwrap_or_default()
                    .to_owned();
                let request_id = req
                    .headers()
                    .get("X-Request-ID")
                    .unwrap_or(&HeaderValue::from_static(""))
                    .to_str()
                    .unwrap_or_default()
                    .to_owned();
                let ip = req
                    .connection_info()
                    .realip_remote_addr()
                    .unwrap_or_default()
                    .to_owned();
                ok(Self {
                    inner: Request::new(
                        app_dao.clone().into_inner(),
                        RequestEnv {
                            request_id,
                            ip,
                            user_agent,
                        },
                        UserAuthSession::new(
                            app_dao.user.user_dao.user_auth.clone(),
                            SessionToken::default(),
                        ),
                    ),
                    req: req.to_owned(),
                })
            }
            None => err(JsonData::message_error("not find webdao").into()),
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
impl<'t> CookieToken<'t> {
    pub fn set_token(&self, token: SessionToken<UserAuthTokenData>) {
        self.request_dao
            .req
            .extensions_mut()
            .insert::<SessionToken<UserAuthTokenData>>(token);
    }
}

impl<'t> RequestToken<UserAuthTokenData> for CookieToken<'t> {
    fn get_user_token(&self) -> SessionToken<UserAuthTokenData> {
        if let Some(cookie) = self.request_dao.req.cookie(AUTH_COOKIE_NAME) {
            SessionToken::<UserAuthTokenData>::from_str(cookie.value()).unwrap_or_default()
        } else {
            SessionToken::<UserAuthTokenData>::default()
        }
    }
    fn is_refresh(&self, token: &SessionToken<UserAuthTokenData>) -> bool {
        if token.is_ok() {
            if let Some(data) = token.data() {
                return now_time().unwrap_or_default() - 10 > data.time_out;
            }
        }
        false
    }
    fn refresh_user_token(&self, token: &SessionToken<UserAuthTokenData>) {
        self.set_token(token.to_owned());
    }
}

//oauth 登陆实现，跟普通登陆实现方式不相同
pub struct OauthAuthQuery {
    pub inner: RestAuthQueryDao,
    pub req: HttpRequest,
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
                let user_agent = req
                    .headers()
                    .get("User-Agent")
                    .unwrap_or(&HeaderValue::from_static(""))
                    .to_str()
                    .unwrap_or_default()
                    .to_owned();
                let request_id = req
                    .headers()
                    .get("X-Request-ID")
                    .unwrap_or(&HeaderValue::from_static(""))
                    .to_str()
                    .unwrap_or_default()
                    .to_owned();
                let ip = req
                    .connection_info()
                    .realip_remote_addr()
                    .unwrap_or_default()
                    .to_owned();
                ok(Self {
                    inner: Request::new(
                        app_dao.clone().into_inner(),
                        RequestEnv {
                            request_id,
                            ip,
                            user_agent,
                        },
                        RestAuthSession::new(app_dao.app.app_dao.clone(), SessionToken::default()),
                    ),
                    req: req.to_owned(),
                })
            }
            None => err(JsonData::message_error("not find webdao").into()),
        }
    }
}
