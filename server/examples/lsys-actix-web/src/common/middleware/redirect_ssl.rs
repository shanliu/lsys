

use actix_utils::future::{ready, Ready};
use actix_web::{
    body::BoxBody, dev::{Service, ServiceRequest, ServiceResponse, Transform}, Error
};
use actix_web::HttpResponse;
use std::{
    future::Future,
    pin::Pin,
};

pub struct RedirectSsl {
    is_use:bool
}
impl RedirectSsl{
    pub fn new(is_use:bool)->Self{
        Self{is_use}
    }
}


impl<S> Transform<S, ServiceRequest> for RedirectSsl
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Transform = RedirectSslMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RedirectSslMiddleware {
            service,
            is_use:self.is_use
        }))
    }
}

pub struct RedirectSslMiddleware<S> {
    service: S,
    is_use:bool
}
impl<S> Service<ServiceRequest> for RedirectSslMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<BoxBody>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<BoxBody>;
    type Error = Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>>>>;
    actix_service::forward_ready!(service);
    fn call(&self, req: ServiceRequest) -> Self::Future {
        if self.is_use &&req.request().connection_info().scheme() =="http" {
            let host = req.request().connection_info().host().to_string();
            let url = format!("https://{}{}", host, req.uri());
            let response =  HttpResponse::MovedPermanently()
            .insert_header((actix_http::header::LOCATION, url))
            .finish();
            return  Box::pin(async move {
                Ok(req.into_response(response)) 
            })
        }
        Box::pin(self.service.call(req))
    }
}


