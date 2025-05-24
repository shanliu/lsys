#![allow(dead_code)]
use actix_http::header::{HeaderName, HeaderValue};
use actix_utils::future::{ready, Ready};
use actix_web::{
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
    Error,
};
use futures_util::ready;
use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    task::{Context, Poll},
};
pub struct RequestID {
    name: &'static str,
}

impl RequestID {
    pub fn new(name: Option<&'static str>) -> Self {
        RequestID {
            name: name.unwrap_or("x-request-id"),
        }
    }
}

impl<S, B> Transform<S, ServiceRequest> for RequestID
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = RequestIDMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;
    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestIDMiddleware {
            service,
            name: self.name,
        }))
    }
}

pub struct RequestIDMiddleware<S> {
    service: S,
    name: &'static str,
}
impl<S, B> Service<ServiceRequest> for RequestIDMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = DefaultHeaderFuture<S, B>;
    actix_service::forward_ready!(service);
    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let mut req_id = String::from("");
        if let Some(head_id) = req.headers().get(self.name) {
            req_id = head_id.to_str().unwrap_or_default().to_string();
        }
        if req_id.is_empty() {
            req_id = nanoid::nanoid!(
                16,
                &(b'0'..=b'9')
                    .chain(b'a'..=b'z')
                    .map(|c| c as char)
                    .collect::<Vec<char>>()
            );
            if let Ok(hval) = HeaderValue::from_str(req_id.as_str()) {
                let name = HeaderName::from_static(self.name);
                req.headers_mut().insert(name, hval);
            }
        }
        let fut = self.service.call(req);
        DefaultHeaderFuture {
            fut,
            name: self.name,
            req_id,
            _body: PhantomData,
        }
    }
}

#[pin_project::pin_project]
pub struct DefaultHeaderFuture<S: Service<ServiceRequest>, B> {
    #[pin]
    fut: S::Future,
    name: &'static str,
    req_id: String,
    _body: PhantomData<B>,
}

impl<S, B> Future for DefaultHeaderFuture<S, B>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = <S::Future as Future>::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let mut res: ServiceResponse<B> = ready!(this.fut.poll(cx))?;
        if let Ok(hval) = HeaderValue::from_str(this.req_id.as_str()) {
            res.response_mut()
                .headers_mut()
                .append(HeaderName::from_static(this.name), hval)
        }
        Poll::Ready(Ok(res))
    }
}
