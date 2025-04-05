use async_trait::async_trait;
use lsys_core::{fluent_message, now_time};

use crate::model::{SessionModel, SessionStatus, UserModel};

use super::{AccessError, AccessResult};
//统一共有登录信息
pub struct SessionBody {
    session: SessionModel, //基本登陆信息
    user: UserModel,       //基本用户信息
}

impl SessionBody {
    pub fn new(user: UserModel, session: SessionModel) -> Self {
        Self { session, user }
    }
    pub fn session(&self) -> &SessionModel {
        &self.session
    }
    pub fn user(&self) -> &UserModel {
        &self.user
    }
    // 获取内部账号ID
    pub fn account_id(&self) -> AccessResult<u64> {
        if self.user.app_id != 0 {
            return Err(AccessError::BadAccount(fluent_message!(
                "access-not-account",{
                    "appid": self.user.app_id
                }
            )));
        }
        match self.user.user_data.parse::<u64>() {
            Ok(e) => Ok(e),
            Err(err) => Err(AccessError::BadAccount(fluent_message!(
                "access-parse-error",
                err
            ))),
        }
    }
    //返回登陆标识
    pub fn token_data(&self) -> &str {
        &self.session.token_data
    }
    //返回用户id
    pub fn user_id(&self) -> u64 {
        self.user.id
    }
    //当前session是否有效
    pub fn is_valid(&self) -> bool {
        SessionStatus::Enable.eq(self.session.status)
            && self.session.expire_time > now_time().unwrap_or_default()
    }
    //当前sesson是否有效，无效返回错误
    pub fn valid(&self) -> AccessResult<()> {
        if !self.is_valid() {
            return Err(AccessError::IsLogout);
        }
        Ok(())
    }
}

//登录标识数据，如cookie,sessionid等
//需根据使用登陆自行实现该特征
pub trait AccessSessionToken {}

//SESSION数据
pub trait AccessSessionData {
    fn session_body(&self) -> &SessionBody;
}

//统一的登录信息特征
#[async_trait]
pub trait AccessSession<T: AccessSessionToken, D: AccessSessionData> {
    //获取设置的登陆Token
    fn get_session_token(&self) -> &T;
    //设置登陆token
    fn set_session_token(&mut self, token: T);
    //获取登陆信息
    async fn get_session_data(&self) -> AccessResult<D>;
    //刷新登陆信息
    async fn refresh_session(&mut self, reset_token: bool) -> AccessResult<T>;
    //清除登陆信息
    async fn clear_session(&mut self) -> AccessResult<()>;
}
