use async_trait::async_trait;
use lsys_core::now_time;
use serde::{Deserialize, Serialize};

use super::UserAuthResult;

//登录标识数据，如cookie,sessionid等
//需根据使用登陆自行实现该特征
pub trait SessionTokenData: Send + Sync {}

//公共登录标识
#[derive(Clone, Debug)]
pub struct SessionToken<T: SessionTokenData> {
    data: Option<T>,
}

impl<T: SessionTokenData> Default for SessionToken<T> {
    fn default() -> Self {
        Self { data: None }
    }
}

impl<T: SessionTokenData> SessionToken<T> {
    pub fn from_data(data: Option<T>) -> Self {
        Self { data }
    }
    pub fn data(&self) -> Option<&T> {
        self.data.as_ref()
    }
    pub fn clear(&mut self) {
        self.data = None
    }
}
impl<T: SessionTokenData> From<T> for SessionToken<T> {
    fn from(s: T) -> SessionToken<T> {
        SessionToken::from_data(Some(s))
    }
}
//公共的登陆用户数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUserData {
    pub user_id: u64,
    pub user_nickname: String,
    pub user_password_id: u64,
    pub time_out: u64,
}
impl SessionUserData {
    //当前登陆数据是否超时
    pub fn is_timeout(&self) -> bool {
        self.time_out <= now_time().unwrap_or_default()
    }
}
//SESSION数据
pub trait SessionData {
    fn user_data(&self) -> &SessionUserData;
}

//统一的登录信息特征
#[async_trait]
pub trait UserSession<T: SessionTokenData, D: SessionData> {
    //获取设置的登陆Token
    fn get_session_token(&self) -> &SessionToken<T>;
    //设置登陆token
    fn set_session_token(&mut self, token: SessionToken<T>);
    //获取登陆信息
    async fn get_session_data(&self) -> UserAuthResult<D>;
    //刷新登陆信息
    async fn refresh_session(&mut self, reset_token: bool) -> UserAuthResult<T>;
    //清除登陆信息
    async fn clear_session(&mut self) -> UserAuthResult<()>;
}
