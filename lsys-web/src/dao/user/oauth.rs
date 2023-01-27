use std::net::IpAddr;

use lsys_user::{
    dao::auth::{
        ExternalLogin, LoginEnv, UserAuthError, UserAuthResult, UserAuthSession, UserAuthStore,
        UserAuthTokenData, UserSession,
    },
    model::{UserExternalModel, UserExternalStatus, UserInfoModelRef},
};
use serde::Serialize;
use sqlx_model::model_option_set;
use tokio::sync::RwLock;

use crate::{
    dao::RequestEnv,
    module::oauth::{OauthCallbackParam, OauthLogin, OauthLoginParam},
};

use super::{ShowUserAuthData, UserRegData, WebUser};

impl WebUser {
    pub async fn user_oauth<
        T: OauthLogin<L, P, D>,
        L: OauthLoginParam + Send + Sync,
        P: OauthCallbackParam + Send + Sync,
        D: Serialize + Send + Sync,
    >(
        &self,
        config_key: &str,
    ) -> UserAuthResult<T> {
        let oauth: T = T::load_config(self, config_key)
            .await
            .map_err(UserAuthError::System)?;
        Ok(oauth)
    }

    pub async fn user_oauth_login<
        T: OauthLogin<L, P, D>,
        L: OauthLoginParam + Send + Sync,
        P: OauthCallbackParam + Send + Sync,
        D: Serialize + Send + Sync,
    >(
        &self,
        oauth: &T,
        param: &L,
    ) -> UserAuthResult<String> {
        let url = oauth
            .login_url(self, param)
            .await
            .map_err(UserAuthError::System)?;
        Ok(url)
    }
    pub async fn user_oauth_callback<
        't,
        T: OauthLogin<L, P, D>,
        L: OauthLoginParam + Send + Sync,
        P: OauthCallbackParam + Send + Sync,
        D: Serialize + Send + Sync,
        A: UserAuthStore + Send + Sync,
    >(
        &self,
        oauth: &T,
        user_session: &RwLock<UserAuthSession<A>>,
        config_key: &str,
        req_env: &RequestEnv,
        param: &P,
    ) -> UserAuthResult<(UserExternalModel, UserAuthTokenData, ShowUserAuthData)> {
        let login_env = LoginEnv {
            login_ip: req_env.ip.parse::<IpAddr>().ok(),
        };
        let (data, ext_data) = oauth
            .login_callback(self, param)
            .await
            .map_err(UserAuthError::System)?;
        let user_ext_rs = self
            .user_dao
            .user_account
            .user_external
            .find_by_external(
                &config_key.to_owned(),
                &data.external_type,
                &data.external_id,
            )
            .await;

        let ext_op = match user_ext_rs {
            Ok(ext) => {
                if UserExternalStatus::Enable.eq(ext.status) {
                    Some(ext)
                } else {
                    None
                }
            }
            Err(err) => {
                if !err.is_not_found() {
                    return Err(err.into());
                } else {
                    None
                }
            }
        };
        let user_ext = match ext_op {
            None => {
                let reg_from = format!("oauth-{}", data.external_type);
                let reg_op = req_env.ip.clone();
                let mut info = model_option_set!(UserInfoModelRef,{
                    reg_ip:reg_op,
                    reg_from:reg_from,
                });
                if let Some(ref img) = data.external_pic {
                    info.headimg = Some(img);
                }
                let user = self
                    .reg_user(UserRegData {
                        nikename: data.external_nikename,
                        passwrod: None,
                        name: None,
                        email: None,
                        mobile: None,
                        external: Some((
                            config_key.to_owned(),
                            data.external_type.clone(),
                            data.external_id.clone(),
                            data.external_name.clone(),
                        )),
                        info: Some(info),
                    })
                    .await?;
                let user_ext = self
                    .user_dao
                    .user_account
                    .user_external
                    .find_by_user_external(
                        &user,
                        config_key.to_owned(),
                        data.external_type,
                        data.external_id,
                    )
                    .await?;
                self.user_dao
                    .user_account
                    .user_external
                    .token_update(
                        &user_ext,
                        data.external_name,
                        data.token_data,
                        data.token_timeout,
                        data.external_gender,
                        data.external_link,
                        data.external_pic,
                    )
                    .await?;
                user_ext
            }
            Some(user_ext) => user_ext,
        };
        let token = self
            .user_dao
            .user_auth
            .login(
                ExternalLogin {
                    external: user_ext.clone(),
                    ext_data,
                },
                login_env,
            )
            .await?;
        user_session
            .write()
            .await
            .set_session_token(token.clone().into());
        let user_data = user_session.read().await.get_session_data().await?;
        Ok((user_ext, token, ShowUserAuthData::from(user_data)))
    }
}
