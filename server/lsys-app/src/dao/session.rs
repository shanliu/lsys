use std::sync::Arc;

use async_trait::async_trait;
use lsys_core::get_message;
use lsys_user::dao::auth::{
    SessionData, SessionToken, SessionTokenData, SessionUserData, UserAuthError, UserAuthResult,
    UserSession,
};
use serde::{Deserialize, Serialize};

use crate::model::AppsTokenModel;

use super::{AppDao, AppsError, AppsResult};

//OAUTH 登录后产生标识
#[derive(Clone, Debug)]
pub struct RestAuthTokenData {
    pub client_id: String,
    pub token: String,
}
impl SessionTokenData for RestAuthTokenData {}

//oauth 登录后数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestAuthData {
    session_data: SessionUserData,
    pub token: AppsTokenModel,
}

impl SessionData for RestAuthData {
    fn user_data(&self) -> &SessionUserData {
        &self.session_data
    }
}
impl RestAuthData {
    pub fn new(session_data: SessionUserData, token: AppsTokenModel) -> Self {
        Self {
            session_data,
            token,
        }
    }
    pub fn check_scope(&self, scope: &str) -> AppsResult<()> {
        let split = scope.split(',');
        for sp in split {
            if !self.token.scope.contains(sp) {
                return Err(AppsError::ScopeNotFind(format!("Unauthorized:{}", sp)));
            }
        }
        Ok(())
    }
}

//oauth 登陆 session
pub struct RestAuthSession {
    app: Arc<AppDao>,
    user_token: SessionToken<RestAuthTokenData>,
}
impl RestAuthSession {
    pub fn new(app: Arc<AppDao>, user_token: SessionToken<RestAuthTokenData>) -> Self {
        Self { app, user_token }
    }
    fn token_result<'t>(
        &self,
        user_token: &'t SessionToken<RestAuthTokenData>,
    ) -> UserAuthResult<&'t RestAuthTokenData> {
        user_token.data().ok_or_else(|| {
            UserAuthError::System(get_message!(
                &self.app.fluent,
                "auth-not-login",
                "user not login"
            ))
        })
    }
}

// 实现 UserSession 调用方保持跟其他方式登录一致
// 不同处理在此 UserSession 实现
#[async_trait]
impl UserSession<RestAuthTokenData, RestAuthData> for RestAuthSession {
    fn get_session_token(&self) -> &SessionToken<RestAuthTokenData> {
        &self.user_token
    }
    fn set_session_token(&mut self, token: SessionToken<RestAuthTokenData>) {
        self.user_token = token
    }
    async fn get_session_data(&self) -> UserAuthResult<RestAuthData> {
        let token_data = self.token_result(&self.user_token)?;
        let data = self
            .app
            .app_oauth
            .get_session_data(token_data)
            .await
            .map_err(|e| UserAuthError::System(e.to_string()))?;
        Ok(data)
    }
    async fn refresh_session(&mut self, reset_token: bool) -> UserAuthResult<RestAuthTokenData> {
        let token_data = self.token_result(&self.user_token)?;
        let app = self
            .app
            .app
            .cache()
            .find_by_client_id(&token_data.client_id)
            .await
            .map_err(|e| UserAuthError::System(e.to_string()))?;
        let data = self
            .app
            .app_oauth
            .refresh_session(&app, token_data, reset_token)
            .await
            .map_err(|e| UserAuthError::System(e.to_string()))?;
        self.set_session_token(SessionToken::from(data.clone()));
        Ok(data)
    }
    async fn clear_session(&mut self) -> UserAuthResult<()> {
        self.app
            .app_oauth
            .clear_session(&self.user_token)
            .await
            .map_err(|e| UserAuthError::System(e.to_string()))?;
        Ok(())
    }
}
