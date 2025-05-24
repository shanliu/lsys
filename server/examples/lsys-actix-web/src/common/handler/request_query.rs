use std::ops::Deref;

use actix_utils::future::{err, ok, Ready};
use actix_web::{dev::Payload, web::Data, FromRequest, HttpRequest};

use actix_http::header;
use lsys_web::lsys_core::IntoFluentMessage;
use lsys_web::lsys_core::RequestEnv;
use lsys_web::{
    common::{JsonData, JsonResponse, RequestDao},
    dao::WebDao,
};

use super::ResponseJson;

//正常用户登陆，如cookie登陆

pub struct ReqQuery {
    pub inner: RequestDao,
    // pub req: HttpRequest,
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
                    inner: RequestDao::new(app_dao.clone().into_inner(), env),
                    // req: req.to_owned(),
                })
            }
            None => err(JsonResponse::data(JsonData::error())
                .set_message("not find webdao")
                .into()),
        }
    }
}
