// 定义外部请求封装

use std::sync::Arc;

use async_trait::async_trait;
use lsys_access::dao::{AccessSession, AccessSessionData, AccessSessionToken};
use lsys_app::dao::{RestAuthData, RestAuthSession, RestAuthToken};
use lsys_core::fluents::{FluentBundle, FluentMgr};
use lsys_core::utils::RequestEnv;

use lsys_user::dao::{UserAuthData, UserAuthSession, UserAuthToken};

use crate::common::{JsonError, JsonResponse, JsonResult};
use tokio::sync::RwLock;

/// 请求环境上下文
///
/// 包含从 HTTP 请求中提取的环境信息和国际化工具。
/// 不包含业务依赖（WebDao），业务依赖应通过参数传入。
pub struct RequestDao {
    pub req_env: RequestEnv,
    pub fluent: Arc<FluentBundle>,
}

impl RequestDao {
    pub fn new(fluent_mgr: &FluentMgr, req_env: RequestEnv) -> Self {
        Self {
            fluent: fluent_mgr.locale(req_env.request_lang.as_deref()),
            req_env,
        }
    }

    // 将 JsonError 转为 JsonResponse
    pub fn fluent_error_json_response(&self, data: &JsonError) -> JsonResponse {
        data.to_json_response(&self.fluent)
    }

    // 将 JsonError 转为本地化字符串
    pub fn fluent_error_string(&self, data: &JsonError) -> String {
        match data {
            JsonError::Error(fluent_error_json_data) => {
                fluent_error_json_data.fluent_format(&self.fluent)
            }
            JsonError::Message(fluent_message) => self.fluent.format_message(fluent_message),
            JsonError::JsonResponse(_, fluent_message) => {
                self.fluent.format_message(fluent_message)
            }
        }
    }
}

/// 认证会话管理器
///
/// 只负责管理认证会话状态，不包含任何业务依赖。
/// 如果需要访问业务逻辑，应通过参数传入 WebDao。
pub struct RequestAuthDao<T: AccessSessionToken, D: AccessSessionData, S: AccessSession<T, D>> {
    pub user_session: RwLock<S>,
    marker_t: std::marker::PhantomData<T>,
    marker_d: std::marker::PhantomData<D>,
}

//解析TOKEN单独抽离出来,异步,避免一些框架的REQ无法SYNC

#[async_trait]
pub trait RequestSessionTokenParser<T: AccessSessionToken> {
    //任意TOKEN数据
    type TD;
    //解析 TD 为 T
    async fn parse_user_token(&self, token_data: Self::TD) -> JsonResult<T>;
}

//执行顺序: get_token_data -> get_parser -> get_parser.parse_user_token -> finish_user_token
pub trait RequestSessionToken<T: AccessSessionToken> {
    type L: RequestSessionTokenParser<T>;
    fn get_parser(&self) -> Self::L;
    fn get_token_data(&self) -> Option<<Self::L as RequestSessionTokenParser<T>>::TD>;
    fn finish_user_token(&self, user_token: &T);
}

impl<T: AccessSessionToken, D: AccessSessionData, S: AccessSession<T, D>> RequestAuthDao<T, D, S> {
    pub fn new(user_session: S) -> Self {
        Self {
            user_session: RwLock::new(user_session),
            marker_t: std::marker::PhantomData,
            marker_d: std::marker::PhantomData,
        }
    }
    pub async fn set_request_token(&self, token: &impl RequestSessionToken<T>) -> JsonResult<()> {
        if let Some(token_data) = token.get_token_data() {
            let user_token = token.get_parser().parse_user_token(token_data).await?;
            token.finish_user_token(&user_token);
            self.user_session
                .write()
                .await
                .set_session_token(user_token);
        }
        Ok(())
    }
}

pub type UserAuthQueryDao = RequestAuthDao<UserAuthToken, UserAuthData, UserAuthSession>;

pub type RestAuthQueryDao = RequestAuthDao<RestAuthToken, RestAuthData, RestAuthSession>;
