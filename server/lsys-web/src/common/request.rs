use std::{ops::Deref, sync::Arc};

use lsys_access::dao::{AccessSession, AccessSessionData, AccessSessionToken};
use lsys_app::dao::{RestAuthData, RestAuthSession, RestAuthToken};
use lsys_core::{FluentBundle, IntoFluentMessage, RequestEnv};

use lsys_user::dao::{UserAuthData, UserAuthSession, UserAuthToken};

use crate::{
    common::{JsonData, JsonError},
    dao::WebDao,
};
use tokio::sync::RwLock;
use tracing::warn;

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
    pub fn fluent_error_json_data(&self, data: &JsonError) -> JsonData {
        data.to_json_data(&self.fluent)
    }
    pub fn fluent_error_string(&self, data: &JsonError) -> String {
        match data {
            JsonError::Error(fluent_error_json_data) => {
                fluent_error_json_data.fluent_format(&self.fluent)
            }
            JsonError::Message(fluent_message) => self.fluent.format_message(fluent_message),
            JsonError::JsonData(_, fluent_message) => self.fluent.format_message(fluent_message),
        }
    }
}

pub struct RequestAuthDao<T: AccessSessionToken, D: AccessSessionData, S: AccessSession<T, D>> {
    req_dao: RequestDao,
    // pub web_dao: Arc<WebDao>,
    // pub req_env: RequestEnv,
    pub user_session: RwLock<S>,
    // fluent: Arc<FluentBundle>,
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

//登陆信息特征
pub trait RequestSessionToken<T: AccessSessionToken> {
    //获取登陆信息 SessionToken
    fn get_user_token(&self) -> T;
    //是否可以支持刷新，如cookie等需要定时刷新登陆信息
    fn is_refresh(&self, token: &T) -> bool;
    //但支持刷新时，实现刷新具体操作
    fn refresh_user_token(&self, token: &T);
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
    pub async fn set_request_token(&self, token: &impl RequestSessionToken<T>) {
        let mut set = self.user_session.write().await;
        let user_token = token.get_user_token();
        if token.is_refresh(&user_token) {
            set.set_session_token(user_token);
            match set.refresh_session(true).await {
                Ok(rut) => {
                    token.refresh_user_token(&rut);
                }
                Err(e) => {
                    warn!(
                        "check user auth error:{}",
                        e.to_fluent_message().default_format()
                    );
                }
            }
        } else {
            set.set_session_token(user_token);
        };
    }
}

pub type UserAuthQueryDao = RequestAuthDao<UserAuthToken, UserAuthData, UserAuthSession>;

pub type RestAuthQueryDao = RequestAuthDao<RestAuthToken, RestAuthData, RestAuthSession>;
