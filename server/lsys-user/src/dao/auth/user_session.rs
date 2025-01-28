use async_trait::async_trait;

use super::{UserAuthDao, UserAuthData, UserAuthError, UserAuthToken};
use lsys_access::dao::{AccessError, AccessResult, AccessSession};
use lsys_core::IntoFluentMessage;
// use std::str::FromStr;
use std::sync::Arc;

pub struct UserAuthSession {
    pub(crate) auth: Arc<UserAuthDao>,
    pub(crate) user_token: UserAuthToken,
}

#[async_trait]
impl AccessSession<UserAuthToken, UserAuthData> for UserAuthSession {
    fn get_session_token(&self) -> &UserAuthToken {
        &self.user_token
    }
    fn set_session_token(&mut self, token: UserAuthToken) {
        self.user_token = token
    }
    async fn get_session_data(&self) -> AccessResult<UserAuthData> {
        Ok(self
            .auth
            .get_session_data(&self.user_token)
            .await
            .map_err(|e| match e {
                UserAuthError::AccessError(err) => err,
                err => AccessError::System(err.to_fluent_message()),
            })?)
    }
    async fn refresh_session(&mut self, reset_token: bool) -> AccessResult<UserAuthToken> {
        let token = self
            .auth
            .reload(&self.user_token, reset_token)
            .await
            .map_err(|e| match e {
                UserAuthError::AccessError(err) => err,
                err => AccessError::System(err.to_fluent_message()),
            })?;
        self.user_token = token.clone();
        Ok(token)
    }
    async fn clear_session(&mut self) -> AccessResult<()> {
        self.auth
            .logout(&self.user_token)
            .await
            .map_err(|e| match e {
                UserAuthError::AccessError(err) => err,
                err => AccessError::System(err.to_fluent_message()),
            })?;
        Ok(())
    }
}

impl UserAuthSession {
    pub fn new(auth: Arc<UserAuthDao>, user_token: UserAuthToken) -> UserAuthSession {
        Self { auth, user_token }
    }
}
