use std::sync::Arc;

use lsys_app::dao::session::{RestAuthData, RestAuthSession, RestAuthTokenData};
use lsys_user::dao::auth::{
    SessionData, SessionToken, SessionTokenData, UserAuthData, UserAuthRedisStore, UserAuthSession,
    UserAuthTokenData, UserSession,
};

use tokio::sync::RwLock;
use tracing::warn;

use crate::{dao::WebDao, RelationParam};

pub struct RequestEnv {
    pub request_id: String,
    pub ip: String,
    pub user_agent: String,
}

pub struct RequestDao<T: SessionTokenData, D: SessionData, S: UserSession<T, D>> {
    pub web_dao: Arc<WebDao>,
    pub req_env: RequestEnv,
    pub user_session: RwLock<S>,
    marker_t: std::marker::PhantomData<T>,
    marker_d: std::marker::PhantomData<D>,
}
//登陆信息特征
pub trait RequestToken<T: SessionTokenData> {
    //获取登陆信息 SessionToken
    fn get_user_token(&self) -> SessionToken<T>;
    //是否可以支持刷新，如cookie等需要定时刷新登陆信息
    fn is_refresh(&self, token: &SessionToken<T>) -> bool;
    //但支持刷新时，实现刷新具体操作
    fn refresh_user_token(&self, token: &SessionToken<T>);
}

impl<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>> RequestDao<T, D, S> {
    pub fn new(web_dao: Arc<WebDao>, req_env: RequestEnv, user_session: S) -> Self {
        Self {
            user_session: RwLock::new(user_session),
            web_dao,
            req_env,
            marker_t: std::marker::PhantomData::default(),
            marker_d: std::marker::PhantomData::default(),
        }
    }
    pub async fn set_request_token(&self, token: &impl RequestToken<T>) {
        let mut set = self.user_session.write().await;
        let user_token = token.get_user_token();
        if token.is_refresh(&user_token) {
            set.set_session_token(user_token);
            match set.refresh_session(true).await {
                Ok(rut) => {
                    token.refresh_user_token(&rut.into());
                }
                Err(e) => {
                    warn!("check user auth error:{}", e);
                }
            }
        } else {
            set.set_session_token(user_token);
        };
    }
    /// 当前登录用户跟指定资源的用户的关系
    /// res_user_id 资源用户id，非用户资源传入0
    pub async fn get_user_relation_role(&self, res_user_id: &[u64]) -> Vec<RelationParam> {
        let auth = self.user_session.read().await;
        if let Ok(ut) = auth.get_session_data().await {
            return self
                .web_dao
                .user
                .user_relation_key(ut.user_data(), res_user_id)
                .await;
        }
        vec![]
    }
}

pub type UserAuthQueryDao =
    RequestDao<UserAuthTokenData, UserAuthData, UserAuthSession<UserAuthRedisStore>>;

pub type RestAuthQueryDao = RequestDao<RestAuthTokenData, RestAuthData, RestAuthSession>;
