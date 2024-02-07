use std::{ops::Deref, sync::Arc};

use lsys_app::dao::session::{RestAuthData, RestAuthSession, RestAuthTokenData};
use lsys_core::{FluentBundle, IntoFluentMessage, RequestEnv};
use lsys_user::dao::auth::{
    SessionData, SessionToken, SessionTokenData, UserAuthData, UserAuthRedisStore, UserAuthSession,
    UserAuthTokenData, UserSession,
};

use tokio::sync::RwLock;
use tracing::warn;

use crate::{dao::WebDao, FluentFormat, FluentJsonData, JsonData};

pub struct RequestDao {
    pub web_dao: Arc<WebDao>,
    pub req_env: RequestEnv,
    fluent: Arc<FluentBundle>,
}

impl RequestDao {
    pub fn new(web_dao: Arc<WebDao>, req_env: RequestEnv) -> Self {
        Self {
            fluent: web_dao.fluent.locale(req_env.request_lang.as_deref()),
            web_dao,
            req_env,
        }
    }
    pub fn fluent_json_data<F: FluentJsonData + FluentFormat>(&self, data: F) -> JsonData {
        JsonData::fluent_from(&self.fluent, data)
    }
    pub fn fluent_string<F: FluentFormat>(&self, data: F) -> String {
        data.fluent_format(&self.fluent)
    }
}

pub struct RequestAuthDao<T: SessionTokenData, D: SessionData, S: UserSession<T, D>> {
    req_dao: RequestDao,
    // pub web_dao: Arc<WebDao>,
    // pub req_env: RequestEnv,
    pub user_session: RwLock<S>,
    // fluent: Arc<FluentBundle>,
    marker_t: std::marker::PhantomData<T>,
    marker_d: std::marker::PhantomData<D>,
}

impl<T: SessionTokenData, D: SessionData, S: UserSession<T, D>> Deref for RequestAuthDao<T, D, S> {
    type Target = RequestDao;
    fn deref(&self) -> &Self::Target {
        &self.req_dao
    }
}

//登陆信息特征
pub trait RequestSessionToken<T: SessionTokenData> {
    //获取登陆信息 SessionToken
    fn get_user_token(&self) -> SessionToken<T>;
    //是否可以支持刷新，如cookie等需要定时刷新登陆信息
    fn is_refresh(&self, token: &SessionToken<T>) -> bool;
    //但支持刷新时，实现刷新具体操作
    fn refresh_user_token(&self, token: &SessionToken<T>);
}

impl<T: SessionTokenData, D: SessionData, S: UserSession<T, D>> RequestAuthDao<T, D, S> {
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
                    token.refresh_user_token(&rut.into());
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

pub type UserAuthQueryDao =
    RequestAuthDao<UserAuthTokenData, UserAuthData, UserAuthSession<UserAuthRedisStore>>;

pub type RestAuthQueryDao = RequestAuthDao<RestAuthTokenData, RestAuthData, RestAuthSession>;
