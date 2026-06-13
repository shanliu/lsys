use std::ops::Deref;
use std::pin::Pin;
use std::str::FromStr;
use std::sync::Arc;
use std::future::Future;
use std::task::{Context, Poll};

use actix_web::cookie::Cookie;
use actix_web::{FromRequest, HttpMessage, HttpRequest, dev::Payload, web::Data};

use lsys_web::common::{JsonData, JsonResponse, JsonResult, RequestAuthDao, RequestSessionToken, RequestSessionTokenParser, UserAuthQueryDao};
use lsys_web::lsys_core::fluents::IntoFluentMessage;
use lsys_web::lsys_core::utils::now_time;
use lsys_web::lsys_user::dao::{UserAuthSession, UserAuthToken};
use lsys_web::dao::WebDao;
use async_trait::async_trait;
use crate::common::handler::AUTH_COOKIE_NAME;
/// 临近过期续期阈值（秒）：服务端 session 剩余有效期 ≤ 此值时触发滑动续期。
pub(crate) const REFRESH_BEFORE_SECS: u64 = 300;

use super::ResponseJson;

#[allow(unused)]
pub struct CookieTokenPaser {
    web_dao: Arc<WebDao>,
}

#[async_trait]
impl RequestSessionTokenParser<UserAuthToken> for CookieTokenPaser {
    type TD = Cookie<'static>;
    async fn parse_user_token(&self, cookie: Cookie<'static>) -> JsonResult<UserAuthToken> {
        // token 自带过期时间（服务端内部编码携带 time_out）：直接用 token 判断，
        // 无需在解析阶段加载服务端会话数据，省去一次 cache/DB 查询。
        let token = UserAuthToken::from_str(cookie.value())?;

        // best-effort 服务端滑动续期：仅当 token 未过期且临近过期时，才查库延长。
        // 续期失败不影响鉴权，下游 `get_session_data` 仍是权威校验。
        if !token.is_timeout() {
            let now_t = now_time().unwrap_or_default();
            if token.time_out.saturating_sub(now_t) <= REFRESH_BEFORE_SECS {
                let auth_dao = &self.web_dao.web_user.user_dao.auth_dao;
                if let Ok(new_token) = auth_dao.reload(&token, false).await {
                    return Ok(new_token);
                }
            }
        }
        Ok(token)
    }
}
/// COOKIE 登陆实现（默认实现）
///
/// 持有 HTTP 请求和 WebDao，实现 `RequestSessionToken` trait，
/// 可配合 `RequestAuthDao::set_request_token` 完成 Cookie token 注入。
#[allow(unused)]
pub struct CookieToken {
    req: HttpRequest,
    web_dao: Arc<WebDao>,
}

#[allow(unused)]
impl CookieToken {
    pub(crate) fn new(req: HttpRequest, web_dao: Arc<WebDao>) -> Self {
        Self { req, web_dao }
    }
}

impl RequestSessionToken<UserAuthToken> for CookieToken {
    type L = CookieTokenPaser;
    fn get_parser(&self) -> Self::L {
        CookieTokenPaser {
            web_dao: self.web_dao.clone(),
        }
    }
    fn get_token_data(&self) -> Option<Cookie<'static>> {
        self.req
            .cookie(AUTH_COOKIE_NAME)
            .and_then(|e| if e.value().is_empty() { None } else { Some(e) })
    }
    fn finish_user_token(&self, user_token: &UserAuthToken) {
        self.req
            .extensions_mut()
            .insert::<UserAuthToken>(user_token.to_owned());
    }
}

/// Cookie 解析错误类型
#[allow(unused)]
#[derive(Debug, Clone)]
pub enum CookieError {
    /// 请求中缺少认证 Cookie
    MissingCookie,
    /// Cookie 值解析失败
    ParseError(String),
    /// 系统级错误（如 DAO 调用失败）
    SystemError(String),
}

impl CookieError {
    #[allow(dead_code)]
    fn into_json_response_default(self) -> JsonResponse {
        let (sub_code, msg) = match &self {
            CookieError::MissingCookie => ("cookie_miss", "missing auth cookie".to_string()),
            CookieError::ParseError(e) => ("cookie_parse", format!("cookie parse error: {e}")),
            CookieError::SystemError(e) => ("cookie_system", format!("system error: {e}")),
        };
        JsonResponse::data(JsonData::error().set_sub_code(sub_code)).set_message(msg)
    }
}

/// 从请求中解析 Cookie 并返回 [`CookieFuture`]。
///
/// # 参数
/// - `req`: HTTP 请求引用
///
/// # 返回
/// - `CookieFuture`: 解析为 `Result<UserAuthToken, CookieError>`
///
/// # 使用示例
/// ```rust
/// let token = parse_cookie(&req).await?;
/// ```
#[allow(unused)]
pub fn parse_cookie(req: &HttpRequest) -> CookieFuture {
    let result = (|| {
        // 读取 Cookie，值为空视为不存在
        let cookie = req
            .cookie(AUTH_COOKIE_NAME)
            .and_then(|c| if c.value().is_empty() { None } else { Some(c) })
            .ok_or(CookieError::MissingCookie)?;

        // 从 Cookie 字符串解析 UserAuthToken
        let token = UserAuthToken::from_str(cookie.value())
            .map_err(|e| CookieError::ParseError(e.to_fluent_message().default_format()))?;

        // token 不再携带过期时间：直接返回，过期与否由服务端 session 校验（expire_time）决定。
        Ok(token)
    })();
    CookieFuture { result: Some(result) }
}

/// [`parse_cookie`] 返回的 Future，解析为 `Result<UserAuthToken, CookieError>`。
///
/// Cookie 解析全程同步，包装成 Future 是为了对齐
/// [`parse_token`](super::request_token::parse_token) / [`parse_rest`](super::request_rest::parse_rest)
/// 的调用风格，同时为后续异步扩展预留位。
#[allow(unused)]
pub struct CookieFuture {
    result: Option<Result<UserAuthToken, CookieError>>,
}

impl Future for CookieFuture {
    type Output = Result<UserAuthToken, CookieError>;

    fn poll(mut self: Pin<&mut Self>, _cx: &mut Context<'_>) -> Poll<Self::Output> {
        Poll::Ready(self.result.take().unwrap_or(Err(CookieError::MissingCookie)))
    }
}

/// CookieQuery —— Cookie 认证中间件封装（用于 handler 参数）
///
/// 实现了 `FromRequest`，可作为 Actix Web handler 的参数直接使用。
/// 内部封装了已完成 Cookie 认证的 `UserAuthQueryDao`。
///
/// # 使用方式
///
/// ```rust
/// pub async fn handler(cookie_query: CookieQuery) -> HttpResponse { ... }
/// ```
#[allow(unused)]
pub struct CookieQuery {
    inner: UserAuthQueryDao,
}

impl Deref for CookieQuery {
    type Target = UserAuthQueryDao;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

/// CookieQuery 的异步 Future
///
/// 1. 从 `app_data` 获取 `WebDao`
/// 2. 异步调用 `parse_cookie` 解析并验证用户 token
/// 3. 构造包含该 token 的 `UserAuthQueryDao` → `CookieQuery`
#[allow(dead_code)]
pub struct CookieQueryFut {
    fut: Pin<Box<dyn Future<Output = Result<CookieQuery, ResponseJson>>>>,
}

impl Future for CookieQueryFut {
    type Output = Result<CookieQuery, ResponseJson>;
    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        self.fut.as_mut().poll(cx)
    }
}

impl FromRequest for CookieQuery {
    type Error = ResponseJson;
    type Future = CookieQueryFut;

    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let web_dao_opt = req.app_data::<Data<WebDao>>().cloned();
        let req = req.clone();

        CookieQueryFut {
            fut: Box::pin(async move {
                // 获取 WebDao
                let web_dao = web_dao_opt.ok_or_else(|| {
                    ResponseJson::from(
                        JsonResponse::data(JsonData::error()).set_message("not find webdao"),
                    )
                })?;

                // 异步解析 Cookie token
                let token = parse_cookie(&req)
                    .await
                    .map_err(|e| ResponseJson::from(e.into_json_response_default()))?;

                // 将解析得到的 token 写入请求扩展，方便后续中间件读取
                req.extensions_mut().insert::<UserAuthToken>(token.clone());

                // 构造带 token 的 UserAuthQueryDao
                Ok(CookieQuery {
                    inner: RequestAuthDao::new(UserAuthSession::new(
                        web_dao.web_user.user_dao.auth_dao.clone(),
                        token,
                    )),
                })
            }),
        }
    }
}
