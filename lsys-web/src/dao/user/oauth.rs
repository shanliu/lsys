use lsys_core::get_message;
use lsys_user::{
    dao::{
        account::{UserAccountError, UserAccountResult},
        auth::{UserAuthError, UserAuthResult},
    },
    model::{UserExternalModel, UserExternalStatus, UserInfoModelRef},
};
use serde::Serialize;
use sqlx_model::model_option_set;

use crate::{
    dao::RequestEnv,
    module::oauth::{OauthCallbackParam, OauthLogin, OauthLoginData, OauthLoginParam},
};

use super::{UserRegData, WebUser};

impl WebUser {
    //得到外部登录实现
    pub async fn user_external_oauth<
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
    //得到外部登录URL
    pub async fn user_external_login_url<
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
    //得到外部数据转为内部UserExternalModel
    pub async fn user_external_login<
        't,
        T: OauthLogin<L, P, D>,
        L: OauthLoginParam + Send + Sync,
        P: OauthCallbackParam + Send + Sync,
        D: Serialize + Send + Sync,
    >(
        &self,
        oauth: &T,
        req_env: &RequestEnv,
        param: &P,
    ) -> UserAuthResult<(UserExternalModel, OauthLoginData, D)> {
        let (data, ext_data) = oauth
            .login_callback(self, param)
            .await
            .map_err(UserAuthError::System)?;
        let user_ext_rs = self
            .user_dao
            .user_account
            .user_external
            .find_by_external(&data.config_name, &data.external_type, &data.external_id)
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
                        nikename: data.external_nikename.clone(),
                        passwrod: None,
                        name: None,
                        email: None,
                        mobile: None,
                        external: Some((
                            data.config_name.to_owned(),
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
                        data.config_name.to_owned(),
                        data.external_type.clone(),
                        data.external_id.clone(),
                    )
                    .await?;
                self.user_dao
                    .user_account
                    .user_external
                    .token_update(
                        &user_ext,
                        data.external_name.clone(),
                        data.token_data.clone(),
                        data.token_timeout,
                        Some(data.external_nikename.clone()),
                        data.external_gender.clone(),
                        data.external_link.clone(),
                        data.external_pic.clone(),
                    )
                    .await?;
                user_ext
            }
            Some(user_ext) => user_ext,
        };
        Ok((user_ext, data, ext_data))
    }
    /// 已登陆后绑定外部账号
    pub async fn user_external_bind<
        't,
        T: OauthLogin<L, P, D>,
        L: OauthLoginParam + Send + Sync,
        P: OauthCallbackParam + Send + Sync,
        D: Serialize + Send + Sync,
    >(
        &self,
        oauth: &T,
        param: &P,
        user_id: u64,
    ) -> UserAccountResult<(UserExternalModel, OauthLoginData, D)> {
        let (param, ext_data) = oauth
            .login_callback(self, param)
            .await
            .map_err(UserAccountError::System)?;
        let user_external = &self.user_dao.user_account.user_external;
        let extdata = user_external
            .find_by_external(&param.config_name, &param.external_type, &param.external_id)
            .await;
        let ext_op = match extdata {
            Ok(ext) => {
                if UserExternalStatus::Enable.eq(ext.status) {
                    if ext.user_id != user_id {
                        return Err(UserAccountError::System(get_message!(&self.fluent,
                            "user-external-other-bind","this account {$name} bind on other account[{$id}]",
                            ["name"=>ext.external_name,"id"=>ext.user_id ]
                        )));
                    }
                    Some(ext)
                } else {
                    None
                }
            }
            Err(err) => {
                if !err.is_not_found() {
                    return Err(err);
                } else {
                    None
                }
            }
        };
        let user = &self.user_dao.user_account.user.find_by_id(&user_id).await?;
        let ext = match ext_op {
            Some(ext) => ext,
            None => {
                let ext_id = user_external
                    .add_external(
                        user,
                        param.config_name.clone(),
                        param.external_type.to_owned(),
                        param.external_id.clone(),
                        param.external_name.clone(),
                        None,
                    )
                    .await?;
                user_external.find_by_id(&ext_id).await?
            }
        };
        user_external
            .token_update(
                &ext,
                param.external_name.to_owned(),
                param.token_data.to_owned(),
                param.token_timeout,
                Some(param.external_nikename.clone()),
                param.external_gender.to_owned(),
                param.external_link.to_owned(),
                param.external_pic.to_owned(),
            )
            .await?;
        Ok((ext, param, ext_data))
    }
}
