//统一登陆过程
use super::WebUserAuth;

use crate::common::JsonResult;

use crate::dao::ShowUserAuthData;
use lsys_access::dao::AccessSession;

use lsys_user::dao::{UserAuthData, UserAuthSession};

use tokio::sync::RwLock;

pub struct UserAuthDataOptionData {
    pub reload_auth: Option<bool>,
    pub auth: Option<bool>,
    pub password_timeout: Option<bool>,
}
impl WebUserAuth {
    pub async fn login_data_from_user_auth(
        &self,
        user_session: &RwLock<UserAuthSession>,
        param: &UserAuthDataOptionData,
    ) -> JsonResult<(UserAuthData, Option<ShowUserAuthData>, bool)> {
        let auth_data = user_session.read().await.get_session_data().await?;
        let out_auth_data = if param.reload_auth.unwrap_or(false) {
            let mut session = user_session.write().await;
            let _ = session.refresh_session(true).await;
            Some(self.create_show_account_auth_data(&auth_data).await?)
        } else if param.auth.unwrap_or(false) {
            Some(self.create_show_account_auth_data(&auth_data).await?)
        } else {
            None
        };
        let account = self
            .user_dao
            .account_dao
            .session_account(&auth_data)
            .await?;

        let passwrod_timeout = if param.password_timeout.unwrap_or(false) {
            self.user_dao
                .account_dao
                .account_password
                .password_timeout(account.password_id)
                .await
                .unwrap_or(false)
        } else {
            false
        };
        Ok((auth_data, out_auth_data, passwrod_timeout))
    }
}
