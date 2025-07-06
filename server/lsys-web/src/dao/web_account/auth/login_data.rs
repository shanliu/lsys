//统一登陆过程
use super::WebUserAuth;

use crate::common::JsonResult;

use crate::dao::ShowUserAuthData;

use lsys_user::dao::UserAuthData;

pub struct UserAuthDataOptionData {
    pub auth: Option<bool>,
    pub password_timeout: Option<bool>,
}
impl WebUserAuth {
    pub async fn login_data_from_user_auth(
        &self,
        auth_data: &UserAuthData,
        param: &UserAuthDataOptionData,
    ) -> JsonResult<(Option<ShowUserAuthData>, bool)> {
        let out_auth_data = if param.auth.unwrap_or(false) {
            Some(self.create_show_account_auth_data(auth_data).await?)
        } else {
            None
        };
        let account = self.user_dao.account_dao.session_account(auth_data).await?;

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
        Ok((out_auth_data, passwrod_timeout))
    }
}
