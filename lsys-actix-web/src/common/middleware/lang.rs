use actix_utils::future::{ready, Ready};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error,
};
use lsys_web::dao::WebDao;

use std::option::Option::Some;
use std::sync::Arc;

pub struct LangSet {
    web_dao: Arc<WebDao>,
}

impl LangSet {
    pub fn new(web_dao: Arc<WebDao>) -> Self {
        LangSet { web_dao }
    }
}

impl<S, B> Transform<S, ServiceRequest> for LangSet
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = LangSetMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(LangSetMiddleware {
            service,
            web_dao: self.web_dao.clone(),
        }))
    }
}

pub struct LangSetMiddleware<S> {
    service: S,
    web_dao: Arc<WebDao>,
}

impl<S, B> Service<ServiceRequest> for LangSetMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = S::Future;
    actix_service::forward_ready!(service);
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let head = req.head().headers.get(header::ACCEPT_LANGUAGE);
        if let Some(value) = head {
            if let Ok(lang) = value.to_str() {
                if let Some(set_lang) = lang.to_owned().split(',').next() {
                    self.web_dao.user.user_dao.fluent.set_lang(set_lang);
                }
            }
        }
        self.service.call(req)
    }
}
