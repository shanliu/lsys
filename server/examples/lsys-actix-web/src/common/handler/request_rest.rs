use actix_web::web::{Data, JsonBody};
use actix_web::{Error, FromRequest, HttpRequest, dev::Payload};
use async_trait::async_trait;
use futures_util::{FutureExt, ready};
use std::ops::Deref;
use std::sync::Arc;
use std::{
    future::Future,
    task::{Context, Poll},
};
use std::{pin::Pin, rc::Rc};

use lsys_web::lsys_app::dao::RestAuthToken;
use lsys_web::lsys_core::api_utils::{RestSignData, compute_rest_sign};
use lsys_web::lsys_core::fluents::{FluentBundle, FluentMessage, IntoFluentMessage};

use lsys_web::common::{
    JsonData, JsonError, JsonResponse, JsonResult, RequestSessionToken, RequestSessionTokenParser,
};
use lsys_web::dao::WebDao;

use serde::{Deserialize, de::DeserializeOwned};
use serde_json::Value;
use tracing::{debug, info};

use super::ResponseJson;

/// REST 解析错误
#[derive(Debug, Clone)]
pub enum RestError {
    /// 缺少 WebDao 配置
    ConfigNotFound,
    /// 查询参数解析错误
    QueryParseError(String),
    /// Payload 解析错误（URL 编码）
    PayloadParseError(String),
    /// Payload 解析错误（JSON body）
    BodyParseError(String),
    /// 签名密钥获取失败
    SignKeyError(String),
    /// 签名验证失败
    SignMismatch { computed: String, received: String },
    /// 参数解析错误
    ParamParseError(String),
    /// 参数为空或已被取走
    ParamEmpty,
}

impl IntoFluentMessage for RestError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            RestError::ConfigNotFound => {
                lsys_web::lsys_core::fluent_message!("rest-config-not-found")
            }
            RestError::QueryParseError(e) => {
                lsys_web::lsys_core::fluent_message!("rest-query-parse-error", e)
            }
            RestError::PayloadParseError(e) => {
                lsys_web::lsys_core::fluent_message!("rest-payload-parse-error", e)
            }
            RestError::BodyParseError(e) => {
                lsys_web::lsys_core::fluent_message!("rest-body-parse-error", e)
            }
            RestError::SignKeyError(e) => {
                lsys_web::lsys_core::fluent_message!("rest-sign-key-error", e)
            }
            RestError::SignMismatch { computed, received } => {
                lsys_web::lsys_core::fluent_message!("rest-sign-mismatch", {
                    "computed": computed,
                    "received": received
                })
            }
            RestError::ParamParseError(e) => {
                lsys_web::lsys_core::fluent_message!("rest-param-parse-error", e)
            }
            RestError::ParamEmpty => {
                lsys_web::lsys_core::fluent_message!("rest-param-empty")
            }
        }
    }
}

impl RestError {
    /// 获取错误的 sub_code
    pub fn sub_code(&self) -> &'static str {
        match self {
            RestError::ConfigNotFound => "rest_config",
            RestError::QueryParseError(_) => "rest_parse",
            RestError::PayloadParseError(_) => "rest_payload",
            RestError::BodyParseError(_) => "rest_payload",
            RestError::SignKeyError(_) => "rest_sign_key",
            RestError::SignMismatch { .. } => "rest_sign",
            RestError::ParamParseError(_) => "rest_param_wrong",
            RestError::ParamEmpty => "rest_param_empty",
        }
    }

    /// 获取 HTTP 状态码
    pub fn status_code(&self) -> u16 {
        200 // REST API 通常在 JSON 中返回错误，HTTP 状态码为 200
    }

    /// 转换为 JsonError
    pub fn into_json_error(self) -> JsonError {
        let code = self.status_code();
        let mut data = JsonData::error().set_sub_code(self.sub_code());
        if code != 200 {
            data = data.set_code(code);
        }
        JsonError::JsonResponse(data, self.to_fluent_message())
    }

    /// 转换为 JsonResponse（使用 fluent 进行国际化）
    #[allow(dead_code)]
    pub fn into_json_response(self, fluent: &FluentBundle) -> JsonResponse {
        self.into_json_error().to_json_response(fluent)
    }

    /// 转换为 JsonResponse（使用默认格式）
    pub fn into_json_response_default(self) -> JsonResponse {
        let msg = self.to_fluent_message();
        let code = self.status_code();
        let mut data = JsonData::error().set_sub_code(self.sub_code());
        if code != 200 {
            data = data.set_code(code);
        }
        JsonResponse::data(data).set_message(msg.default_format())
    }
}

#[derive(Deserialize)]
pub struct RestGet {
    //对外定义
    pub client_id: String,
    pub version: String,
    pub timestamp: String,
    pub sign: String,
    pub payload: Option<String>,
    pub request_ip: Option<String>,
    pub method: Option<String>,
    pub token: Option<String>,
}

#[allow(unused)]
pub struct RestRfc {
    //内部使用
    pub client_id: String,
    pub version: String,
    pub timestamp: String,
    pub sign: String,
    pub request_lang: Option<String>,
    pub payload: Option<Value>,
    pub request_ip: Option<String>,
    pub request_id: Option<String>,
    pub method: Option<String>,
    pub token: Option<String>,
}

type RestKeyGet =
    Box<dyn Fn(String, Data<WebDao>) -> Pin<Box<dyn Future<Output = Result<Vec<String>, String>>>>>;

type RestKeyGetOption = Option<Rc<RestKeyGet>>;

/// REST 解析器（核心工具层）
/// 
/// 封装了 REST 请求的解析、签名验证逻辑。
/// 
/// # 使用方式
/// 
/// 1. **作为 handler 参数（通过 RestQuery 中间件）**：
///    ```rust
///    pub async fn handler(rest: RestQuery) -> HttpResponse { ... }
///    ```
/// 
/// 2. **手动解析（异步）**：
///    ```rust
///    let (parser, web_dao, env) = parse_rest(&req, &mut payload).await?;
///    let param = parser.param::<MyParam>()?;
///    ```
pub struct RestParser {
    pub rfc: RestRfc,
    web_dao: Arc<WebDao>,
}

impl RestParser {
    /// 创建新的 RestParser
    pub(crate) fn new(rfc: RestRfc, web_dao: Arc<WebDao>) -> Self {
        Self { rfc, web_dao }
    }
    
    /// 验证签名
    async fn check_sign(
        rfc: &RestRfc,
        key_fn: &RestKeyGetOption,
        app_data: Data<WebDao>,
    ) -> Result<(), RestError> {
        match key_fn {
            Some(kfn) => {
                let key_res = kfn.as_ref()(rfc.client_id.clone(), app_data.clone())
                    .as_mut()
                    .await;
                match key_res {
                    Ok(app_keys) => {
                        let sign_data = RestSignData {
                            client_id: &rfc.client_id,
                            version: &rfc.version,
                            timestamp: &rfc.timestamp,
                            request_ip: rfc.request_ip.as_deref(),
                            method: rfc.method.as_deref(),
                            token: rfc.token.as_deref(),
                            payload: rfc.payload.as_ref(),
                        };

                        // 尝试所有密钥验证签名
                        let mut matched = false;
                        let mut last_result = None;
                        for key in &app_keys {
                            let result = compute_rest_sign(&sign_data, key);
                            if result.signature == rfc.sign {
                                matched = true;
                                break;
                            }
                            last_result = Some(result);
                        }

                        if matched {
                            Ok(())
                        } else {
                            let computed = last_result.map(|r| r.signature).unwrap_or_default();
                            debug!("target:{},request:{}", computed, &rfc.sign);
                            Err(RestError::SignMismatch {
                                computed,
                                received: rfc.sign.clone(),
                            })
                        }
                    }
                    Err(err) => Err(RestError::SignKeyError(err)),
                }
            }
            None => Ok(()),
        }
    }

    /// 获取自定义参数
    pub fn param<T: DeserializeOwned>(&self) -> Result<T, RestError> {
        match self.rfc.payload {
            Some(ref body) => serde_json::from_value::<T>(body.to_owned())
                .map_err(|e| RestError::ParamParseError(e.to_string())),
            None => Err(RestError::ParamEmpty),
        }
    }
    
    /// 获取应用信息
    pub async fn get_app(&self) -> Result<lsys_web::lsys_app::model::AppModel, RestError> {
        self.web_dao
            .web_app
            .app_dao
            .app
            .cache()
            .find_by_client_id(&self.rfc.client_id)
            .await
            .map_err(|e| RestError::SignKeyError(format!("{:?}", e)))
    }
}

pub struct RestParserTokenParser {}

#[async_trait]
impl RequestSessionTokenParser<RestAuthToken> for RestParserTokenParser {
    type TD = (String, String);
    async fn parse_user_token(&self, (client_id, token): Self::TD) -> JsonResult<RestAuthToken> {
        Ok(RestAuthToken { client_id, token })
    }
}

#[async_trait]
impl RequestSessionToken<RestAuthToken> for RestParser {
    type L = RestParserTokenParser;
    fn get_parser(&self) -> Self::L {
        RestParserTokenParser {}
    }
    fn get_token_data(&self) -> Option<(String, String)> {
        self.rfc.token.as_ref().and_then(|e| {
            if e.is_empty() {
                None
            } else {
                Some((self.rfc.client_id.clone(), e.to_owned()))
            }
        })
    }
    fn finish_user_token(&self, _: &RestAuthToken) {}
}

/// 从 HttpRequest 和 Payload 中解析 REST 请求
/// 
/// 这是一个公开的异步解析函数，返回 `RestFuture`。
/// 
/// # 参数
/// - `req`: HTTP 请求
/// - `payload`: 请求 payload（用于读取 body）
/// 
/// # 返回
/// - `RestFuture`: 实现了 `Future<Output = Result<RestParser, RestError>>`
/// 
/// # 使用示例
/// ```rust
/// let parser = parse_rest(&req, &mut payload).await?;
/// let param = parser.param::<MyParam>()?;
/// ```
pub fn parse_rest(req: &HttpRequest, payload: &mut Payload) -> RestFuture {
    // 获取配置
    let config = req
        .app_data::<RestQueryConfig>()
        .or_else(|| req.app_data::<RestQueryConfig>())
        .unwrap_or(&RestQueryConfig::default())
        .to_owned();
    
    // 判断是否是 JSON 请求
    let json_req = req
        .headers()
        .get("Content-type")
        .map(|e| e.to_str().unwrap_or_default())
        .unwrap_or_default()
        .contains("application/json");
    
    // 提取请求头信息
    let request_id = req
        .headers()
        .get("X-Request-ID")
        .map(|e| e.to_str().unwrap_or_default().to_string());
    
    let request_lang = req
        .headers()
        .get(actix_web::http::header::ACCEPT_LANGUAGE)
        .and_then(|e| e.to_str().ok())
        .and_then(|s| s.split(',').next())
        .map(|s| s.split(';').next().unwrap_or(s).trim())
        .map(|s| {
            if let Some((lang, region)) = s.split_once('-') {
                format!("{}_{}", lang.to_lowercase(), region.to_uppercase())
            } else {
                s.to_lowercase()
            }
        });

    // 获取 WebDao
    let app_dao = match req.app_data::<Data<WebDao>>() {
        Some(dao) => Some(dao.clone()),
        None => {
            return RestFuture {
                app_dao: None,
                key_fn: None,
                rfc: None,
                body: None,
                future: None,
                error: Some(RestError::ConfigNotFound),
            };
        }
    };

    // 解析查询参数
    let rfc = match serde_urlencoded::from_str::<RestGet>(req.query_string()) {
        Ok(get_param) => {
            let mut rfc = RestRfc {
                request_id,
                request_lang,
                client_id: get_param.client_id,
                version: get_param.version,
                timestamp: get_param.timestamp,
                sign: get_param.sign,
                payload: None,
                request_ip: get_param.request_ip,
                method: get_param.method,
                token: get_param.token,
            };
            
            // 如果不是 JSON 请求，尝试从 URL 参数解析 payload
            if !json_req
                && let Some(pl) = get_param.payload
                && !pl.is_empty()
            {
                match serde_json::from_str::<Value>(pl.as_str()) {
                    Ok(val) => {
                        rfc.payload = Some(val);
                    }
                    Err(err) => {
                        return RestFuture {
                            app_dao: None,
                            key_fn: None,
                            rfc: None,
                            body: None,
                            future: None,
                            error: Some(RestError::PayloadParseError(err.to_string())),
                        };
                    }
                }
            }
            Some(rfc)
        }
        Err(err) => {
            return RestFuture {
                app_dao: None,
                key_fn: None,
                rfc: None,
                body: None,
                future: None,
                error: Some(RestError::QueryParseError(err.to_string())),
            };
        }
    };

    // 如果是 JSON 请求，创建 body reader
    let body = if json_req {
        let path = req.path().to_string();
        let data = JsonBody::new(req, payload, None, false)
            .limit(config.limit)
            .map(|res: Result<Value, _>| match res {
                Ok(data) => Ok(data),
                Err(e) => Err(Error::from(e)),
            })
            .map(move |res| match res {
                Ok(data) => Ok(data),
                Err(e) => {
                    info!(
                        "Failed to deserialize Json from payload. Request path: {}",
                        path
                    );
                    Err(RestError::BodyParseError(e.to_string()))
                }
            })
            .boxed_local();
        Some(data)
    } else {
        None
    };

    RestFuture {
        app_dao,
        key_fn: config.app_key,
        rfc,
        body,
        future: None,
        error: None,
    }
}

// REST 解析 Future
// 
// 这个 Future 处理异步的 REST 请求解析，包括：
// 1. 读取 body（如果是 JSON 请求）
// 2. 验证签名
// 3. 创建 RequestEnv
// 4. 返回 (RestParser, Arc<WebDao>, RequestEnv)

// Body 解析 Future 类型
type BodyFuture = Pin<Box<dyn Future<Output = Result<Value, RestError>>>>;

// RestFuture 的输出类型
type RestFutureOutput = Result<RestParser, RestError>;

// 主处理 Future 类型
type MainFuture = Pin<Box<dyn Future<Output = RestFutureOutput>>>;

pub struct RestFuture {
    app_dao: Option<Data<WebDao>>,
    key_fn: RestKeyGetOption,
    rfc: Option<RestRfc>,
    body: Option<BodyFuture>,
    future: Option<MainFuture>,
    error: Option<RestError>,
}

impl Future for RestFuture {
    type Output = RestFutureOutput;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 如果已经有 future，继续 poll（处理异步重入 - 必须放在第一行）
        if let Some(ref mut future) = self.future {
            return future.as_mut().poll(cx);
        }

        // 如果有错误，直接返回（不 take，允许多次重入返回相同错误）
        if let Some(ref err) = self.error {
            return Poll::Ready(Err(err.clone()));
        }

        // 获取 app_dao 和 key_fn（如果没有说明初始化时就出错了）
        let app_dao = match self.app_dao.as_ref() {
            Some(dao) => dao.clone(),
            None => return Poll::Ready(Err(RestError::ConfigNotFound)),
        };
        let key_fn = self.key_fn.clone();

        match &mut self.body {
            // 有 body 需要读取（JSON 请求）
            Some(body_data) => {
                // 等待 body 读取完成
                let body_res = ready!(body_data.as_mut().poll(cx));
                let rfc_data = self.rfc.take();
                match rfc_data {
                    Some(mut rfc) => {
                        // 创建异步 future 进行签名验证和环境创建
                        let mut future = Box::pin(async move {
                            match body_res {
                                Ok(body) => {
                                    rfc.payload = Some(body);
                                    RestParser::check_sign(&rfc, &key_fn, app_dao.clone()).await?;
                                    
                                    Ok(RestParser::new(rfc, app_dao.into_inner()))
                                }
                                Err(err) => Err(err),
                            }
                        });
                        // 先 poll 一次，如果 Pending 才保存 future（避免不必要的状态保存）
                        match future.as_mut().poll(cx) {
                            Poll::Ready(item) => Poll::Ready(item),
                            Poll::Pending => {
                                self.get_mut().future = Some(future);
                                Poll::Pending
                            }
                        }
                    }
                    None => {
                        // rfc 已经被 take 了，理论上不应该到这里（除非多次重入）
                        Poll::Ready(Err(RestError::ConfigNotFound))
                    }
                }
            }
            // 没有 body（GET 请求或 URL 编码）
            None => {
                let rfc_data = self.rfc.take();
                match rfc_data {
                    Some(rfc) => {
                        // 创建异步 future 进行签名验证和环境创建
                        let mut future = Box::pin(async move {
                            RestParser::check_sign(&rfc, &key_fn, app_dao.clone()).await?;
                            
                            Ok(RestParser::new(rfc, app_dao.into_inner()))
                        });
                        // 先 poll 一次，如果 Pending 才保存 future（避免不必要的状态保存）
                        match future.as_mut().poll(cx) {
                            Poll::Ready(item) => Poll::Ready(item),
                            Poll::Pending => {
                                self.get_mut().future = Some(future);
                                Poll::Pending
                            }
                        }
                    }
                    None => {
                        // rfc 已经被 take 了，理论上不应该到这里（除非多次重入）
                        Poll::Ready(Err(RestError::ConfigNotFound))
                    }
                }
            }
        }
    }
}

#[derive(Clone)]
pub struct RestQueryConfig {
    limit: usize,
    app_key: RestKeyGetOption,
}

impl RestQueryConfig {
    /// Set maximum accepted payload size. By default this limit is 16kB.
    #[allow(dead_code)]
    pub fn limit(mut self, limit: usize) -> Self {
        self.limit = limit;
        self
    }
    #[allow(dead_code)]
    pub fn app_key_fn(mut self, f: RestKeyGet) -> Self {
        self.app_key = Some(Rc::new(f));
        self
    }
}

/// Allow shared refs used as default.
const DEFAULT_CONFIG: RestQueryConfig = RestQueryConfig {
    limit: 16_384, // 2^14 bytes (~16kB)
    app_key: None,
};

impl Default for RestQueryConfig {
    fn default() -> Self {
        DEFAULT_CONFIG
    }
}

/// RestQuery - REST 中间件封装（用于 handler 参数）
/// 
/// 这是 `RestParser` 的中间件版本，实现了 `FromRequest` trait，
/// 可以作为 Actix Web handler 的参数使用。
/// 
/// # 使用方式
/// 
/// 1. **强制认证**：
///    ```rust
///    pub async fn handler(rest: RestQuery) -> HttpResponse { ... }
///    ```
/// 
/// 2. **可选认证**：
///    ```rust
///    pub async fn handler(rest: Option<RestQuery>) -> HttpResponse { ... }
///    ```
pub struct RestQuery(RestParser);

impl Deref for RestQuery {
    type Target = RestParser;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[async_trait]
impl RequestSessionToken<RestAuthToken> for RestQuery {
    type L = RestParserTokenParser;
    fn get_parser(&self) -> Self::L {
        self.0.get_parser()
    }
    fn get_token_data(&self) -> Option<(String, String)> {
        self.0.get_token_data()
    }
    fn finish_user_token(&self, token: &RestAuthToken) {
        self.0.finish_user_token(token)
    }
}

/// RestQuery 的 Future 实现
/// 
/// 这个 Future 只负责类型转换：
/// 1. Poll RestFuture 得到 (RestParser, Arc<WebDao>, RequestEnv)
/// 2. 构造 RestQuery
/// 3. 将 RestError 转换为 ResponseJson
pub struct RestQueryFuture {
    rest_fut: RestFuture,
}

impl Future for RestQueryFuture {
    type Output = Result<RestQuery, ResponseJson>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // Poll RestFuture 得到 (RestParser, Arc<WebDao>, RequestEnv)
        match ready!(Pin::new(&mut self.rest_fut).poll(cx)) {
            Ok(parser) => {
                // 直接构造 RestQuery（同步操作）
                Poll::Ready(Ok(RestQuery(parser)))
            }
            Err(err) => {
                // 类型转换：RestError → ResponseJson
                Poll::Ready(Err(err.into_json_response_default().into()))
            }
        }
    }
}

impl FromRequest for RestQuery {
    type Error = ResponseJson;
    type Future = RestQueryFuture;
    
    fn from_request(req: &HttpRequest, payload: &mut Payload) -> Self::Future {
        RestQueryFuture {
            rest_fut: parse_rest(req, payload),
        }
    }
}
