#![allow(dead_code)]
use actix_http::header::{HeaderName, HeaderValue};
use actix_utils::future::{Ready, ready};
use actix_web::{
    Error,
    dev::{Service, ServiceRequest, ServiceResponse, Transform},
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
    type Future = RequestIDFuture<S, B>;
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
        // 创建请求级别的 span，将 request_id 作为 span 字段注入。
        // 该请求处理链路中的所有日志都会自动携带 request_id，实现日志连贯性。
        // 后台任务等非 HTTP 场景可创建不带 request_id 的 span，request_id 天然可选。
        let span = tracing::info_span!("request", request_id = %req_id);

        let fut = self.service.call(req);
        RequestIDFuture {
            fut,
            name: self.name,
            req_id,
            span,
            _body: PhantomData,
        }
    }
}

#[pin_project::pin_project]
pub struct RequestIDFuture<S: Service<ServiceRequest>, B> {
    #[pin]
    fut: S::Future,
    name: &'static str,
    req_id: String,
    span: tracing::Span,
    _body: PhantomData<B>,
}

impl<S, B> Future for RequestIDFuture<S, B>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
{
    type Output = <S::Future as Future>::Output;
    fn poll(self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        let this = self.project();
        let _enter = this.span.enter();
        let mut res: ServiceResponse<B> = ready!(this.fut.poll(cx))?;
        if let Ok(hval) = HeaderValue::from_str(this.req_id.as_str()) {
            res.response_mut()
                .headers_mut()
                .append(HeaderName::from_static(this.name), hval)
        }
        Poll::Ready(Ok(res))
    }
}
