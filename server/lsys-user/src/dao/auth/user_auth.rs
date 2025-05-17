use async_trait::async_trait;
use base64::Engine;

use lsys_access::dao::{AccessDao, AccessSessionData, AccessSessionToken, SessionBody};
use lsys_core::fluent_message;
use lsys_core::now_time;

use std::str::FromStr;
use std::sync::Arc;

use base64::{
    alphabet,
    engine::{self, general_purpose},
};

use super::{UserAuthError, UserAuthResult};

const CUSTOM_ENGINE: engine::GeneralPurpose =
    engine::GeneralPurpose::new(&alphabet::URL_SAFE, general_purpose::NO_PAD);

//登录产生标识
#[derive(Clone, Debug, Default)]
pub struct UserAuthToken {
    pub app_id: u64,
    pub token: String,
    pub user_id: u64,
    pub time_out: u64,
}

impl UserAuthToken {
    pub fn new(app_id: u64, token: &str, user_id: u64, time_out: u64) -> Self {
        Self {
            app_id,
            token: token.to_string(),
            user_id,
            time_out,
        }
    }
    pub fn is_timeout(&self) -> bool {
        self.time_out <= now_time().unwrap_or_default()
    }
}

impl std::fmt::Display for UserAuthToken {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let str = format!(
            "{}-{}-{}-{}",
            self.app_id, self.user_id, self.token, self.time_out
        );
        write!(f, "{}", CUSTOM_ENGINE.encode(str.as_bytes()))
    }
}

impl FromStr for UserAuthToken {
    type Err = UserAuthError;
    /// 从TOKEN字符串还原
    fn from_str(token_str: &str) -> Result<Self, Self::Err> {
        let token_str = token_str.to_owned();
        let de64 = &CUSTOM_ENGINE
            .decode(token_str.as_bytes())
            .map_err(|e| UserAuthError::TokenParse(fluent_message!("user-auth-parse-error", e)))?;
        let token_str = String::from_utf8(de64.to_owned())
            .map_err(|e| UserAuthError::TokenParse(fluent_message!("user-auth-parse-error", e)))?;
        let mut token_split = token_str.split('-');
        let app_id = token_split
            .next()
            .ok_or_else(|| UserAuthError::TokenParse(fluent_message!("app-auth-parse-bad")))?;
        let app_id = app_id
            .parse::<u64>()
            .map_err(|e| UserAuthError::TokenParse(fluent_message!("app-auth-parse-error", e)))?;
        let user_id = token_split
            .next()
            .ok_or_else(|| UserAuthError::TokenParse(fluent_message!("user-auth-parse-bad")))?;
        let user_id = user_id
            .parse::<u64>()
            .map_err(|e| UserAuthError::TokenParse(fluent_message!("user-auth-parse-error", e)))?;
        let token = token_split
            .next()
            .ok_or_else(|| UserAuthError::TokenParse(fluent_message!("user-auth-parse-bad")))?
            .to_string();
        let time_out = token_split
            .next()
            .ok_or_else(|| UserAuthError::TokenParse(fluent_message!("user-auth-parse-bad")))?
            .parse::<u64>()
            .map_err(|e| UserAuthError::TokenParse(fluent_message!("user-auth-parse-error", e)))?;
        Ok(Self::new(app_id, &token, user_id, time_out))
    }
}
impl AccessSessionToken for UserAuthToken {}

impl From<&UserAuthData> for UserAuthToken {
    fn from(value: &UserAuthData) -> Self {
        UserAuthToken::new(
            value.session().user_app_id,
            value.token_data(),
            value.user_id(),
            value.session().expire_time,
        )
    }
}

const ACCESS_LOGIN_DATA: &str = "login-data";

//登录后数据
//从存储的登陆信息还原状态时
// oauth_app_id=0 必须为0，非oauth登陆
//  app_id=0 系统用户 user.user_data 必须为数字（字符串）
//  app_id>0 非系统用户 user.user_data 必须为字符串
pub struct UserAuthData {
    session_data: SessionBody,
    login_data: String,
}
impl UserAuthData {
    pub fn new(session_data: SessionBody, login_data: &str) -> UserAuthResult<Self> {
        if session_data.session().oauth_app_id != 0 {
            return Err(UserAuthError::System(fluent_message!("user-bad-session")));
        }
        Ok(Self {
            session_data,
            login_data: login_data.to_string(),
        })
    }
    pub fn login_data(&self) -> &str {
        &self.login_data
    }
}
impl std::ops::Deref for UserAuthData {
    type Target = SessionBody;
    fn deref(&self) -> &SessionBody {
        &self.session_data
    }
}
impl AccessSessionData for UserAuthData {
    fn session_body(&self) -> &SessionBody {
        &self.session_data
    }
}

#[async_trait]
pub trait UserLoginReload: Send + Sync {
    async fn reload(
        &self,
        session: &SessionBody,
        data: &str,
    ) -> Option<UserAuthResult<(String, u64)>>;
}

//验证登录相关接口
//不包含登录状态
pub struct UserAuthDao {
    access: Arc<AccessDao>,
    reload_type: Vec<Box<dyn UserLoginReload>>,
}
impl UserAuthDao {
    /// 对外对象创建
    pub fn new(access: Arc<AccessDao>, reload_type: Vec<Box<dyn UserLoginReload>>) -> Self {
        UserAuthDao {
            access,
            reload_type,
        }
    }
    //得到当前登陆用户
    //@todo 多加个一个参数，用于是否决定是否实时从数据库拿记录
    pub async fn get_session_data(
        &self,
        user_token: &UserAuthToken,
    ) -> UserAuthResult<UserAuthData> {
        let session = self
            .access
            .auth
            .cache()
            .login_data(user_token.app_id, 0, &user_token.token)
            .await?;
        let login_data = self
            .access
            .auth
            .cache()
            .session_get_data(&session, ACCESS_LOGIN_DATA)
            .await?;
        UserAuthData::new(session, &login_data.unwrap_or_default())
    }
    //重新加载当前用户
    //user_token 当前登陆的 UserAuthToken
    //reset_token 是否重新生成 UserAuthToken
    //返回UserAuthToken 但reset_token为true时为新生成的 UserAuthToken
    pub async fn reload(
        &self,
        user_token: &UserAuthToken,
        reset_token: bool,
    ) -> UserAuthResult<UserAuthToken> {
        let user_data = self.get_session_data(user_token).await?;
        let mut new_user_data = None;
        for tmp in self.reload_type.iter() {
            if let Some(res) = tmp
                .reload(&user_data.session_data, &user_data.login_data)
                .await
            {
                new_user_data = Some(res?);
                break;
            }
        }
        match new_user_data {
            Some((data, timeout)) => {
                let session = if reset_token {
                    self.access
                        .auth
                        .refresh_login(&user_data, Some(timeout), None)
                        .await?
                } else {
                    self.access.auth.extend_login(&user_data, timeout).await?
                };
                if !data.is_empty() {
                    self.access
                        .auth
                        .session_set_data(&user_data, ACCESS_LOGIN_DATA, data.as_str())
                        .await?;
                }
                Ok(UserAuthToken::new(
                    session.session().user_app_id,
                    session.token_data(),
                    session.user_id(),
                    session.session().expire_time,
                ))
            }
            None => {
                Err(UserAuthError::System(
                    fluent_message!("auth-not-support-reload"), //"{$user} is disable",
                ))
            }
        }
    }
    //退出登录
    pub async fn logout(&self, user_token: &UserAuthToken) -> UserAuthResult<()> {
        match self
            .access
            .auth
            .cache()
            .login_data(user_token.app_id, 0, &user_token.token)
            .await
        {
            Ok(e) => {
                self.access.auth.do_logout(&e).await?;
                Ok(())
            }
            Err(err) => match err {
                lsys_access::dao::AccessError::IsLogout
                | lsys_access::dao::AccessError::NotLogin => Ok(()),
                _ => Err(err.into()),
            },
        }
    }
    //获取当前登录状态
    pub async fn is_login(&self, user_token: &UserAuthToken) -> UserAuthResult<bool> {
        let session = self
            .access
            .auth
            .cache()
            .login_data(user_token.app_id, 0, &user_token.token)
            .await?;
        Ok(session.is_valid())
    }
}
