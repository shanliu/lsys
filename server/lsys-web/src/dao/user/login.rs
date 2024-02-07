use std::net::IpAddr;

use lsys_core::{IntoFluentMessage, RequestEnv};
use lsys_user::dao::auth::{
    LoginEnv, LoginParam, UserAuthError, UserAuthResult, UserAuthSession, UserAuthStore,
    UserAuthTokenData, UserSession,
};

use tokio::sync::RwLock;

use crate::CaptchaParam;

use super::{ShowUserAuthData, WebUser};

impl WebUser {
    pub async fn user_login<'t, TO: LoginParam, T: UserAuthStore + Send + Sync>(
        &self,
        user_session: &RwLock<UserAuthSession<T>>,
        req_env: &RequestEnv,
        param: TO,
        code: Option<CaptchaParam>,
    ) -> UserAuthResult<(UserAuthTokenData, ShowUserAuthData)> {
        let lenv = LoginEnv {
            login_ip: req_env
                .request_ip
                .as_ref()
                .map(|e| e.parse::<IpAddr>().ok())
                .unwrap_or_default(),
        };
        let res = self.user_dao.user_auth.check(&param, &lenv).await;
        if let Err(UserAuthError::CheckCaptchaNeed(_)) = &res {
            if let Some(ref captcha_code) = code {
                let tres = self
                    .captcha
                    .valid_code(&crate::dao::CaptchaKey::Login)
                    .check_code(&captcha_code.key, &captcha_code.code)
                    .await;
                if let Err(captcha_err) = self
                    .captcha
                    .valid_code(&crate::dao::CaptchaKey::Login)
                    .clear_code(&captcha_code.key, &mut self.captcha.valid_code_builder())
                    .await
                {
                    tracing::warn!(
                        "clear login captcha fail:{} in [{}]",
                        captcha_err.to_fluent_message().default_format(),
                        &param.show_name()
                    );
                }
                tres?;
            } else {
                res?
            }
        } else {
            res?
        }
        let token = self.user_dao.user_auth.login(param, lenv).await?;
        user_session
            .write()
            .await
            .set_session_token(token.clone().into());
        let auth_data = user_session.read().await.get_session_data().await?;
        Ok((token, ShowUserAuthData::from(auth_data)))
    }
}
