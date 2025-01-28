use std::sync::Arc;

use async_trait::async_trait;
use lsys_access::dao::{
    AccessError, AccessResult, AccessSession, AccessSessionData, AccessSessionToken, SessionBody,
};
use lsys_core::{fluent_message, IntoFluentMessage};

use crate::model::AppModel;

use super::AppDao;

//OAUTH 登录后产生标识
#[derive(Clone, Debug, Default)]
pub struct RestAuthToken {
    pub client_id: String,
    pub token: String,
}
impl AccessSessionToken for RestAuthToken {}

impl From<&RestAuthData> for RestAuthToken {
    fn from(value: &RestAuthData) -> Self {
        RestAuthToken {
            client_id: value.app.client_id.to_owned(),
            token: value.session.token_data().to_owned(),
        }
    }
}

//oauth 登录后数据
pub struct RestAuthData {
    app: AppModel,
    session: SessionBody,
}
impl std::ops::Deref for RestAuthData {
    type Target = SessionBody;
    fn deref(&self) -> &SessionBody {
        &self.session
    }
}
impl AccessSessionData for RestAuthData {
    fn session_body(&self) -> &SessionBody {
        &self.session
    }
}

impl RestAuthData {
    pub fn new(app: AppModel, session: SessionBody) -> Self {
        Self { app, session }
    }
}

//oauth 登陆 session
pub struct RestAuthSession {
    app: Arc<AppDao>,
    user_token: RestAuthToken,
}
impl RestAuthSession {
    pub fn new(app: Arc<AppDao>, user_token: RestAuthToken) -> Self {
        Self { app, user_token }
    }
}

// 实现 AccessSession 调用方保持跟其他方式登录一致
// 不同处理在此 AccessSession 实现
#[async_trait]
impl AccessSession<RestAuthToken, RestAuthData> for RestAuthSession {
    fn get_session_token(&self) -> &RestAuthToken {
        &self.user_token
    }
    fn set_session_token(&mut self, token: RestAuthToken) {
        self.user_token = token
    }
    async fn get_session_data(&self) -> AccessResult<RestAuthData> {
        let app = self.app.rest_session_app(self).await?;
        let data = self
            .app
            .oauth_client
            .get_session_data(&app, &self.user_token.token)
            .await
            .map_err(|e| {
                AccessError::System(fluent_message!("app-session-get-error",{
                    "client_id":&self.user_token.client_id,
                    "msg":e.to_fluent_message(),
                }))
            })?;
        Ok(data)
    }
    async fn refresh_session(&mut self, reset_token: bool) -> AccessResult<RestAuthToken> {
        let app = self.app.rest_session_app(self).await?;
        let data = self
            .app
            .oauth_client
            .refresh_session(&app, &self.user_token.token, reset_token)
            .await
            .map_err(|e| {
                AccessError::System(fluent_message!("app-session-refresh-error",{
                    "client_id":&self.user_token.client_id,
                    "msg":e.to_fluent_message()
                }))
            })?;
        if reset_token {
            let token = RestAuthToken::from(&data);
            self.set_session_token(token.clone());
            return Ok(token);
        }
        Ok(self.user_token.clone())
    }
    async fn clear_session(&mut self) -> AccessResult<()> {
        let app = self.app.rest_session_app(self).await?;
        self.app
            .oauth_client
            .clear_session(&app, &self.user_token.token)
            .await
            .map_err(|e| {
                AccessError::System(fluent_message!(
                    "app-session-clear-error",
                    e.to_fluent_message()
                ))
            })?;
        Ok(())
    }
}
