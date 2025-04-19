//统一登陆过程
use super::WebUserAuth;
use crate::common::{CaptchaParam, JsonError, JsonResult};
use crate::dao::{OauthCallbackParam, OauthLogin, OauthLoginParam};
use lsys_access::dao::AccessSession;
use lsys_core::{fluent_message, now_time, IntoFluentMessage, RequestEnv};
use lsys_user::dao::login::ExternalLogin;
use lsys_user::dao::{
    login::{AccountLoginEnv, AccountLoginMeta, AccountLoginParam},
    AuthCode, UserAuthData, UserAuthError, UserAuthSession, UserAuthToken,
};
use lsys_user::dao::{
    login::{
        EmailCodeLoginData, EmailCodeLoginMeta, EmailLoginData, EmailLoginMeta, ExternalLoginData,
        ExternalLoginMeta, MobileCodeLoginData, MobileCodeLoginMeta, MobileLoginData,
        MobileLoginMeta, NameLoginData, NameLoginMeta,
    },
    CODE_LOGIN_TYPE,
};
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use std::net::IpAddr;
use tokio::sync::RwLock;
#[derive(Debug, Clone, Serialize)]
pub struct ShowUserAuthData {
    pub login_type: String,
    pub login_data: Value,
    pub user_id: u64,
    pub user_nickname: String,
    pub empty_password: bool,
    pub account_id: u64,
    pub time_out: u64,
    pub login_time: u64,
}
impl WebUserAuth {
    //成功登录，统一返回用户登录信息
    pub async fn create_show_account_auth_data(
        &self,
        auth_data: &UserAuthData,
    ) -> JsonResult<ShowUserAuthData> {
        let show_login_data = if auth_data.session().login_type == NameLoginMeta::login_type() {
            json!(NameLoginData::from(&self.user_dao.account_dao, auth_data)
                .await?
                .to_json())
        } else if auth_data.session().login_type == EmailCodeLoginMeta::login_type() {
            json!(
                EmailCodeLoginData::from(&self.user_dao.account_dao, auth_data)
                    .await?
                    .to_json()
            )
        } else if auth_data.session().login_type == EmailLoginMeta::login_type() {
            json!(EmailLoginData::from(&self.user_dao.account_dao, auth_data)
                .await?
                .to_json())
        } else if auth_data.session().login_type == MobileCodeLoginMeta::login_type() {
            json!(
                MobileCodeLoginData::from(&self.user_dao.account_dao, auth_data)
                    .await?
                    .to_json()
            )
        } else if auth_data.session().login_type == MobileLoginMeta::login_type() {
            json!(MobileLoginData::from(&self.user_dao.account_dao, auth_data)
                .await?
                .to_json())
        } else if auth_data.session().login_type == ExternalLoginMeta::login_type() {
            json!(
                ExternalLoginData::from(&self.user_dao.account_dao, auth_data)
                    .await?
                    .to_json()
            )
        } else if auth_data.session().login_type == CODE_LOGIN_TYPE {
            json!({})
        } else {
            return Err(JsonError::Message(fluent_message!("bad-session-data")));
        };

        let stime = now_time().unwrap_or_default();
        let time_out = auth_data.session().expire_time;
        let login_time = time_out.saturating_sub(stime);
        Ok(ShowUserAuthData {
            login_type: auth_data.session().login_type.to_owned(),
            login_data: show_login_data,
            user_id: auth_data.user_id(),
            user_nickname: auth_data.user().user_name.clone(),
            account_id: auth_data.account_id().unwrap_or_default(),
            empty_password: self
                .user_dao
                .account_dao
                .session_account(auth_data)
                .await
                .map(|e| e.password_id == 0)
                .unwrap_or(false),
            time_out,
            login_time,
        })
    }
}

impl WebUserAuth {
    //内部账号登录
    pub async fn user_login<TO: AccountLoginParam>(
        &self,
        param: &TO,
        code: Option<&CaptchaParam>,
        user_session: &RwLock<UserAuthSession>,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
        let lenv = AccountLoginEnv {
            login_ip: env_data
                .map(|e| e.request_ip.as_ref().map(|e| e.parse::<IpAddr>().ok()))
                .unwrap_or_default()
                .unwrap_or_default(),
        };
        let res = self.user_dao.auth_account_dao.check(param, &lenv).await;
        if let Err(UserAuthError::CheckCaptchaNeed(_)) = &res {
            if let Some(captcha_code) = code {
                let tres = self
                    .captcha
                    .valid_code(&crate::dao::CaptchaKey::Login)
                    .check_code(&captcha_code.into())
                    .await;
                if let Err(captcha_err) = self
                    .captcha
                    .valid_code(&crate::dao::CaptchaKey::Login)
                    .destroy_code(&captcha_code.key, &mut self.captcha.valid_code_builder())
                    .await
                {
                    tracing::warn!(
                        "clear login captcha fail:{} in [{}]",
                        captcha_err.to_fluent_message().default_format(),
                        param.account_name()
                    );
                }
                tres?;
            } else {
                res?
            }
        } else {
            res?
        }
        let token = self.user_dao.auth_account_dao.login(param, lenv).await?;
        user_session.write().await.set_session_token(token.clone());
        let auth_data = user_session.read().await.get_session_data().await?;
        Ok((token, self.create_show_account_auth_data(&auth_data).await?))
    }
    //通过APP code登录
    // 由  self.user_dao.auth_code_dao.code_login 产生 login_code
    pub async fn app_code_login(
        &self,
        app_id: u64,
        token_data: &str,
        code: Option<&CaptchaParam>,
        user_session: &RwLock<UserAuthSession>,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
        let login_ip = env_data
            .map(|e| e.request_ip.to_owned())
            .unwrap_or_default()
            .unwrap_or_default();
        let ua = env_data
            .map(|e| e.request_user_agent.to_owned())
            .unwrap_or_default()
            .unwrap_or_default();

        let session_body = self
            .user_dao
            .auth_code_dao
            .login_data(app_id, token_data)
            .await?;
        if session_body.session().device_name.is_empty()
            || session_body.session().device_name != ua
            || session_body.session().login_ip.is_empty()
            || session_body.session().login_ip != login_ip
        {
            if let Some(captcha_code) = code {
                let tres = self
                    .captcha
                    .valid_code(&crate::dao::CaptchaKey::Login)
                    .check_code(&captcha_code.into())
                    .await;
                if let Err(captcha_err) = self
                    .captcha
                    .valid_code(&crate::dao::CaptchaKey::Login)
                    .destroy_code(&captcha_code.key, &mut self.captcha.valid_code_builder())
                    .await
                {
                    tracing::warn!(
                        "clear login captcha fail:{} in code:{}",
                        captcha_err.to_fluent_message().default_format(),
                        token_data
                    );
                }
                tres?;
            } else {
                return Err(
                    UserAuthError::CheckCaptchaNeed(fluent_message!("auth-need-captcha")).into(),
                );
            }
        }
        let token = AuthCode::to_token(&session_body);
        user_session.write().await.set_session_token(token.clone());
        let auth_data = user_session.read().await.get_session_data().await?;
        Ok((token, self.create_show_account_auth_data(&auth_data).await?))
    }
    //外部账号登录
    pub async fn external_login<
        T: OauthLogin<L, P, D>,
        L: OauthLoginParam + Send + Sync,
        P: OauthCallbackParam + Send + Sync,
        D: Serialize + Send + Sync,
    >(
        &self,
        oauth: &T,
        param: &P,
        op_user_id: u64,
        user_session: &RwLock<UserAuthSession>,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<(UserAuthToken, ShowUserAuthData)> {
        let res = self
            .oauth_user_login(oauth, param, op_user_id, env_data)
            .await?;
        let (ext_model, _, ext_data) = res;
        let login_env = AccountLoginEnv {
            login_ip: env_data
                .map(|e| e.request_ip.as_ref().map(|e| e.parse::<IpAddr>().ok()))
                .unwrap_or_default()
                .unwrap_or_default(),
        };
        let token = self
            .user_dao
            .auth_account_dao
            .login(
                &ExternalLogin::new(
                    self.user_dao.account_dao.clone(),
                    ext_model.clone(),
                    ext_data,
                ),
                login_env,
            )
            .await?;
        user_session.write().await.set_session_token(token.clone());
        let auth_data = user_session.read().await.get_session_data().await?;
        Ok((token, self.create_show_account_auth_data(&auth_data).await?))
    }

    /// logout
    pub async fn user_logout(&self, user_session: &RwLock<UserAuthSession>) -> JsonResult<()> {
        user_session.write().await.clear_session().await?;
        Ok(())
    }
}
