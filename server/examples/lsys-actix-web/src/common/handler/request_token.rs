use std::future::Future;
use std::pin::Pin;
use std::str::FromStr;
use std::task::{Context, Poll};

use actix_web::web::Data;
use actix_web::{FromRequest, HttpRequest, dev::Payload};
use async_trait::async_trait;
use futures_util::ready;
use lsys_web::common::{
    JsonData, JsonError, JsonResponse, JsonResult, RequestSessionToken, RequestSessionTokenParser,
};
use lsys_web::dao::WebDao;
use lsys_web::lsys_core::fluent_message;
use lsys_web::lsys_core::fluents::{FluentMessage, IntoFluentMessage};
use lsys_web::lsys_user::dao::UserAuthToken;

use super::ResponseJson;

/// token 信封前缀标记，用于快速识别并拒绝随机伪造 token。
const TOKEN_PREFIX: &str = "lsys";

/// 校验和长度（hex 字符数），48bit。
const CHECKSUM_LEN: usize = 12;


/// 登录 token 校验配置（对应 [`request_rest`](super::request_rest) 的 `RestQueryConfig`）。
///
/// 在 `create_server` 启动时由 [`from_config`](Self::from_config) 读取一次配置构建，
/// 经 `app_data` 注入；解析阶段由 [`from_request`](Self::from_request) 取出，避免每个
/// 请求重复读配置。`sign_key` 非空启用「前缀 + 校验和」防伪闸门，空则关闭（兼容旧 token）。
#[derive(Clone, Default)]
pub struct TokenSignConfig {
    sign_key: String,
}

impl TokenSignConfig {
    /// 从密钥管理器或应用配置读取校验密钥 共享登录密钥
    pub fn from_config(web_dao: &WebDao) -> Self {
        let sign_key = web_dao
            .secret_manager
            .get("login_key")
            .and_then(|b| std::str::from_utf8(b).ok())
            .map(|s| s.to_string())
            .unwrap_or_default();
        Self { sign_key }
    }

    /// 从请求注入数据取出（未注入时回退到空闸门，对应 `RestQueryConfig::default`）。
    pub fn from_request(req: &HttpRequest) -> Self {
        req.app_data::<TokenSignConfig>()
            .cloned()
            .unwrap_or_default()
    }

    /// 校验密钥；空字符串表示关闭闸门。
    pub fn sign_key(&self) -> &str {
        &self.sign_key
    }
}

/// 计算 token 校验和：md5(sign_key + inner) 取前 [`CHECKSUM_LEN`] 位 hex。
fn token_checksum(sign_key: &str, inner: &str) -> String {
    let digest = md5::compute(format!("{}{}", sign_key, inner));
    let hex = format!("{:x}", digest);
    hex[..CHECKSUM_LEN].to_string()
}

/// 给不透明 token 加上「前缀 + 校验和」信封（登录下发时使用）。
///
/// - `sign_key` 为空时不加信封，直接返回原始 token（兼容模式）；
/// - 非空时返回 `lsys.<checksum>.<inner>`。
pub fn wrap_token(inner: &str, sign_key: &str) -> String {
    if sign_key.is_empty() {
        return inner.to_string();
    }
    format!(
        "{}.{}.{}",
        TOKEN_PREFIX,
        token_checksum(sign_key, inner),
        inner
    )
}

/// 校验并剥离「前缀 + 校验和」信封，返回内部不透明 token。
///
/// 这是一道廉价的「DoS 闸门」：在查 cache/DB 之前，先用密钥校验校验和。
/// 随机伪造的 token 因校验和不匹配会被直接拒绝，无需任何存储查询。
///
/// - `sign_key` 为空（未配置）：关闭闸门。带前缀则剥离，否则原样返回。
/// - `sign_key` 非空（启用）：必须带合法前缀且校验和匹配，否则报错。
pub(crate) fn verify_token(wrapped: &str, sign_key: &str) -> JsonResult<String> {
    let segments: Vec<&str> = wrapped.splitn(3, '.').collect();
    let has_prefix = segments.len() == 3 && segments[0] == TOKEN_PREFIX;

    if sign_key.is_empty() {
        // 闸门关闭：兼容原始 token
        if has_prefix {
            return Ok(segments[2].to_string());
        }
        return Ok(wrapped.to_string());
    }

    // 闸门开启：严格校验
    if !has_prefix {
        return Err(JsonError::Message(fluent_message!("user-auth-token-format")));
    }
    let (checksum, inner) = (segments[1], segments[2]);
    let expect = token_checksum(sign_key, inner);
    if !constant_time_eq(checksum.as_bytes(), expect.as_bytes()) {
        return Err(JsonError::Message(fluent_message!("user-auth-token-sign")));
    }
    Ok(inner.to_string())
}

/// 定长比较，避免校验和比较的计时侧信道。
fn constant_time_eq(a: &[u8], b: &[u8]) -> bool {
    if a.len() != b.len() {
        return false;
    }
    let mut diff = 0u8;
    for (x, y) in a.iter().zip(b.iter()) {
        diff |= x ^ y;
    }
    diff == 0
}

/// token 解析器（核心工具层），被 [`TokenParser`]（含 [`TokenStr`] 别名）与
/// [`BearerQuery`] 共用。
///
/// 解析顺序：先 [`verify_token`] 过校验和闸门，再 [`UserAuthToken::from_str`]，
/// 二者都在查 cache/DB 之前完成，因此可廉价拦截随机伪造 token。
///
/// 请注意：Bearer 解析是**无状态**的，不做任何服务端自动续期。token 的过期/
/// 轮换完全由客户端显式调用 `/api/auth/token_refresh` 驱动（详见该端点）。
pub struct TokenStrParser {
    sign_key: String,
}

#[async_trait]
impl RequestSessionTokenParser<UserAuthToken> for TokenStrParser {
    type TD = String;
    async fn parse_user_token(&self, token_str: String) -> JsonResult<UserAuthToken> {
        let inner = verify_token(&token_str, &self.sign_key)?;
        let token = UserAuthToken::from_str(&inner)?;
        Ok(token)
    }
}

/// token 解析结果（核心数据层），由 [`parse_token`] 异步构造。
///
/// 持有「待解析 token」与「校验密钥」，实现 [`RequestSessionToken`]，可直接交给
/// `set_request_token`。与 [`request_rest`](super::request_rest) 的 `RestParser`
/// 对应，保持整体风格统一。
///
/// 另对外暴露别名 [`TokenStr`]，用于「手动直传 token」的下载场景。
pub struct TokenParser {
    token: String,
    sign_key: String,
}

impl TokenParser {
    /// 用已知校验密钥构造（内部使用，由 [`parse_token`] / [`TokenFuture`] 调用）。
    pub(crate) fn with_sign_key(token: String, sign_key: String) -> Self {
        Self { token, sign_key }
    }
}

impl RequestSessionToken<UserAuthToken> for TokenParser {
    type L = TokenStrParser;
    fn get_parser(&self) -> Self::L {
        TokenStrParser {
            sign_key: self.sign_key.clone(),
        }
    }
    fn get_token_data(&self) -> Option<String> {
        if self.token.is_empty() {
            None
        } else {
            Some(self.token.clone())
        }
    }
    fn finish_user_token(&self, _user_token: &UserAuthToken) {}
}

/// token 解析阶段错误（仅覆盖「取 token / 取密钥」阶段；真正的信封校验与不透明
/// token 解析见 [`verify_token`]）。对应 `RestError` 的角色。
#[derive(Debug, Clone)]
pub enum TokenError {
    /// 缺少 WebDao 配置
    ConfigNotFound,
    /// 缺少 Authorization 头
    MissingHeader,
    /// Authorization 头解析失败
    HeaderParse(String),
    /// 不是 Bearer token
    NotBearer,
    /// Bearer token 为空
    EmptyToken,
}

impl IntoFluentMessage for TokenError {
    fn to_fluent_message(&self) -> FluentMessage {
        match self {
            TokenError::ConfigNotFound => fluent_message!("auth-config-not-found"),
            TokenError::MissingHeader => fluent_message!("auth-missing-header"),
            TokenError::HeaderParse(e) => fluent_message!("auth-header-parse", e),
            TokenError::NotBearer => fluent_message!("auth-not-bearer"),
            TokenError::EmptyToken => fluent_message!("auth-empty-token"),
        }
    }
}

impl TokenError {
    /// 错误 sub_code
    pub fn sub_code(&self) -> &'static str {
        match self {
            TokenError::ConfigNotFound => "auth_config",
            TokenError::MissingHeader => "auth_miss_header",
            TokenError::HeaderParse(_) => "auth_parse_header",
            TokenError::NotBearer => "auth_not_bearer",
            TokenError::EmptyToken => "auth_empty_token",
        }
    }

    /// 转换为 JsonError
    pub fn into_json_error(self) -> JsonError {
        JsonError::JsonResponse(
            JsonData::error().set_sub_code(self.sub_code()),
            self.to_fluent_message(),
        )
    }

    /// 转为 [`ResponseJson`]（默认格式）。
    pub fn into_json_response_default(self) -> ResponseJson {
        let msg = self.to_fluent_message();
        JsonResponse::data(JsonData::error().set_sub_code(self.sub_code()))
            .set_message(msg.default_format())
            .into()
    }
}

/// 解析已取得的 token 字符串，返回 [`TokenFuture`]。
///
/// 对应 [`request_rest`](super::request_rest) 的 `parse_rest`，但以 **token 为入参**：
/// header 的提取由 [`BearerQuery`] 完成，这里只负责「取签名密钥 + 组装解析结果」。
///
/// 当前取签名密钥是同步读配置；若将来需要异步获取（如远程 / DB 下发密钥），
/// 可直接在 [`TokenFuture`] 内部的 future 中改为 `.await`，对调用方透明。
///
/// # 使用示例
/// ```rust
/// let parser = parse_token(&req, token).await?;
/// auth_dao.set_request_token(&parser).await?;
/// ```
pub fn parse_token(req: &HttpRequest, token: String) -> TokenFuture {
    match req.app_data::<Data<WebDao>>() {
        Some(_) => TokenFuture {
            token: Some(token),
            req: Some(req.clone()),
            future: None,
            error: None,
        },
        None => TokenFuture::failed(TokenError::ConfigNotFound),
    }
}

// TokenFuture 输出类型
type TokenFutureOutput = Result<TokenParser, TokenError>;
// 主处理 future 类型（异步获取签名密钥并组装 TokenParser）
type TokenMainFuture = Pin<Box<dyn Future<Output = TokenFutureOutput>>>;

/// [`parse_token`] 返回的 Future，解析为 [`TokenParser`]。
///
/// 结构对齐 [`request_rest`](super::request_rest) 的 `RestFuture`：内部用一个 boxed
/// future 承载（当前同步的）签名密钥获取，预留异步扩展位。
pub struct TokenFuture {
    token: Option<String>,
    req: Option<HttpRequest>,
    future: Option<TokenMainFuture>,
    error: Option<TokenError>,
}

impl TokenFuture {
    /// 构造一个直接返回错误的 Future（用于 header 提取阶段就失败的情况）。
    pub(crate) fn failed(err: TokenError) -> Self {
        Self {
            token: None,
            req: None,
            future: None,
            error: Some(err),
        }
    }
}

impl Future for TokenFuture {
    type Output = TokenFutureOutput;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        // 已有进行中的 future，继续 poll（异步重入，必须放在第一行）
        if let Some(ref mut future) = self.future {
            return future.as_mut().poll(cx);
        }
        // 初始化即出错，直接返回（允许多次重入返回相同错误）
        if let Some(ref err) = self.error {
            return Poll::Ready(Err(err.clone()));
        }
        let token = self.token.take().unwrap_or_default();
        let req = self.req.take();
        // 包成 future：当前从注入的 `TokenSignConfig`（app_data）取密钥，将来可改为异步获取
        let mut future: TokenMainFuture = Box::pin(async move {
            let sign_key = req
                .as_ref()
                .map(TokenSignConfig::from_request)
                .unwrap_or_default()
                .sign_key()
                .to_string();
            Ok(TokenParser::with_sign_key(token, sign_key))
        });
        // 先 poll 一次，Pending 才保存（避免不必要的状态保存）
        match future.as_mut().poll(cx) {
            Poll::Ready(item) => Poll::Ready(item),
            Poll::Pending => {
                self.get_mut().future = Some(future);
                Poll::Pending
            }
        }
    }
}

/// `Authorization: Bearer <token>` 提取器（从 header 解析用户登录态）。
///
/// 不透明 Reference Token 的 header 入口：从 `Authorization` 头取出 Bearer token，
/// 交给 [`parse_token`] 组装为 [`TokenParser`]，再由 `set_request_token` 走校验和
/// 闸门 + 服务端 cache/DB 验证。
///
/// 结构对齐 [`request_rest`](super::request_rest) 的 `RestQuery`：薄封装 + `Deref`。
///
/// # 使用方式
/// ```rust
/// pub async fn handler(bearer: BearerQuery, auth_dao: UserAuthQuery) -> HttpResponse {
///     auth_dao.set_request_token(&bearer).await?;
///     // ...
/// }
/// ```
pub struct BearerQuery(TokenParser);

impl std::ops::Deref for BearerQuery {
    type Target = TokenParser;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl RequestSessionToken<UserAuthToken> for BearerQuery {
    type L = TokenStrParser;
    fn get_parser(&self) -> Self::L {
        self.0.get_parser()
    }
    fn get_token_data(&self) -> Option<String> {
        self.0.get_token_data()
    }
    fn finish_user_token(&self, token: &UserAuthToken) {
        self.0.finish_user_token(token)
    }
}

/// 从 `Authorization` 头提取 Bearer token 字符串。
fn extract_bearer(req: &HttpRequest) -> Result<String, TokenError> {
    let head = req
        .headers()
        .get("Authorization")
        .ok_or(TokenError::MissingHeader)?;
    let raw = head
        .to_str()
        .map_err(|e| TokenError::HeaderParse(e.to_string()))?;
    let token = raw.trim();
    if !token.starts_with("Bearer ") {
        return Err(TokenError::NotBearer);
    }
    let token = token[7..].trim().to_string();
    if token.is_empty() {
        return Err(TokenError::EmptyToken);
    }
    Ok(token)
}

/// [`BearerQuery`] 的 Future：poll [`TokenFuture`] 并做类型 / 错误转换。
///
/// 对应 [`request_rest`](super::request_rest) 的 `RestQueryFuture`。
pub struct BearerQueryFuture {
    fut: TokenFuture,
}

impl Future for BearerQueryFuture {
    type Output = Result<BearerQuery, ResponseJson>;

    fn poll(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<Self::Output> {
        match ready!(Pin::new(&mut self.fut).poll(cx)) {
            Ok(parser) => Poll::Ready(Ok(BearerQuery(parser))),
            Err(err) => Poll::Ready(Err(err.into_json_response_default())),
        }
    }
}

impl FromRequest for BearerQuery {
    type Error = ResponseJson;
    type Future = BearerQueryFuture;
    fn from_request(req: &HttpRequest, _: &mut Payload) -> Self::Future {
        let fut = match extract_bearer(req) {
            Ok(token) => parse_token(req, token),
            Err(err) => TokenFuture::failed(err),
        };
        BearerQueryFuture { fut }
    }
}
