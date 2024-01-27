use std::ops::Deref;

use actix_utils::future::{err, ok, Ready};
use actix_web::{dev::Payload, web::Data, FromRequest, HttpRequest};

use lsys_core::RequestEnv;
use lsys_web::{
    dao::{RequestDao, WebDao},
    JsonData,
};

use reqwest::header::{self, HeaderValue};

use super::ResponseJson;

//正常用户登陆，如cookie登陆

pub struct ReqQuery {
    pub inner: RequestDao,
    pub req: HttpRequest,
}

impl Deref for ReqQuery {
    type Target = RequestDao;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl FromRequest for ReqQuery {
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
                    .replace('-', "_")
                    .to_owned();
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
                    inner: RequestDao::new(
                        app_dao.clone().into_inner(),
                        RequestEnv::new(
                            Some(user_lang),
                            Some(ip),
                            Some(request_id),
                            Some(user_agent),
                        ),
                    ),
                    req: req.to_owned(),
                })
            }
            None => err(JsonData::message_error("not find webdao").into()),
        }
    }
}
