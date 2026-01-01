#![allow(dead_code)]
use actix_http::header::{self, HeaderValue};
use actix_utils::future::{ready, Ready};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use std::{future::Future, pin::Pin};

use futures_util::ready;
use std::{
    marker::PhantomData,
    task::{Context, Poll},
};

pub struct AllowOrigin(pub Vec<String>);

impl<S, B> Transform<S, ServiceRequest> for AllowOrigin
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AllowOriginMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AllowOriginMiddleware {
            service,
            allow_origin_list: self.0.clone(),
        }))
    }
}

pub struct AllowOriginMiddleware<S> {
    service: S,
    allow_origin_list: Vec<String>,
}
impl<S, B> Service<ServiceRequest> for AllowOriginMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = AllowOriginFuture<S, B>;
    actix_service::forward_ready!(service);
    fn call(&self, req: ServiceRequest) -> Self::Future {
        let mut push_origin = None;
        if !self.allow_origin_list.is_empty() {
            if let Some(head_id) = req.headers().get(header::ORIGIN) {
                if let Ok(val) = head_id.to_str() {
                    if self
                        .allow_origin_list
                        .iter()
                        .any(|o| o.as_str() == "*" || o.as_str() == val)
                    {
                        push_origin = Some(val.to_string());
                    }
                }
            }
        }
        let fut = self.service.call(req);
        AllowOriginFuture {
            fut,
            push_origin,
            _body: PhantomData,
        }
    }
}

#[pin_project::pin_project]
pub struct AllowOriginFuture<S: Service<ServiceRequest>, B> {
    #[pin]
    fut: S::Future,
    push_origin: Option<String>,
    _body: PhantomData<B>,
}

impl<S, B> Future for AllowOriginFuture<S, B>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = <S::Future as Future>::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut res: ServiceResponse<B> = ready!(this.fut.poll(cx))?;
        if let Some(origin) = this.push_origin {
            if let Ok(hval) = HeaderValue::from_str(origin.as_str()) {
                res.response_mut()
                    .headers_mut()
                    .append(header::ACCESS_CONTROL_ALLOW_ORIGIN, hval)
            }
        }
        Poll::Ready(Ok(res))
    }
}
