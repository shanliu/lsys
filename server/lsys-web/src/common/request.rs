// 定义外部请求封装

use std::{ops::Deref, sync::Arc};

use async_trait::async_trait;
use lsys_access::dao::{AccessSession, AccessSessionData, AccessSessionToken};
use lsys_app::dao::{RestAuthData, RestAuthSession, RestAuthToken};
use lsys_core::{FluentBundle, RequestEnv};

use lsys_user::dao::{UserAuthData, UserAuthSession, UserAuthToken};

use crate::{
    common::{JsonError, JsonResponse, JsonResult},
    dao::WebDao,
};
use tokio::sync::RwLock;

pub struct RequestDao {
    pub web_dao: Arc<WebDao>,
    pub req_env: RequestEnv,
    pub fluent: Arc<FluentBundle>,
}

impl RequestDao {
    pub fn new(web_dao: Arc<WebDao>, req_env: RequestEnv) -> Self {
        Self {
            fluent: web_dao.fluent.locale(req_env.request_lang.as_deref()),
            web_dao,
            req_env,
        }
    }
    pub fn fluent_error_json_response(&self, data: &JsonError) -> JsonResponse {
        data.to_json_response(&self.fluent)
    }
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

pub struct RequestAuthDao<T: AccessSessionToken, D: AccessSessionData, S: AccessSession<T, D>> {
    req_dao: RequestDao,
    pub user_session: RwLock<S>,
    marker_t: std::marker::PhantomData<T>,
    marker_d: std::marker::PhantomData<D>,
}

impl<T: AccessSessionToken, D: AccessSessionData, S: AccessSession<T, D>> Deref
    for RequestAuthDao<T, D, S>
{
    type Target = RequestDao;
    fn deref(&self) -> &Self::Target {
        &self.req_dao
    }
}

//解析TOKEN单独抽离出来,异步,避免一些框架的REQ无法SYNC

#[async_trait]
pub trait RequestSessionTokenPaser<T: AccessSessionToken> {
    //任意TOKEN数据
    type TD;
    //解析 TD 为 T
    async fn parse_user_token(&self, token_data: Self::TD) -> JsonResult<T>;
}

//执行顺序: get_token_data -> get_paser -> get_paser.parse_user_token -> finish_user_token
pub trait RequestSessionToken<T: AccessSessionToken> {
    type L: RequestSessionTokenPaser<T>;
    fn get_paser(&self) -> Self::L;
    fn get_token_data(&self) -> Option<<Self::L as RequestSessionTokenPaser<T>>::TD>;
    fn finish_user_token(&self, user_token: &T);
}

impl<T: AccessSessionToken, D: AccessSessionData, S: AccessSession<T, D>> RequestAuthDao<T, D, S> {
    pub fn new(web_dao: Arc<WebDao>, req_env: RequestEnv, user_session: S) -> Self {
        Self {
            req_dao: RequestDao::new(web_dao, req_env),
            user_session: RwLock::new(user_session),
            marker_t: std::marker::PhantomData,
            marker_d: std::marker::PhantomData,
        }
    }
    pub async fn set_request_token(&self, token: &impl RequestSessionToken<T>) -> JsonResult<()> {
        if let Some(token_data) = token.get_token_data() {
            let user_token = token.get_paser().parse_user_token(token_data).await?;
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
