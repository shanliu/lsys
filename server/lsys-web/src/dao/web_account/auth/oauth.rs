//统一ouath登陆及已登陆账号的oauth绑定
use lsys_core::model_option_set;
use lsys_core::{fluent_message, RequestEnv};
use lsys_user::model::{AccountExternalModel, AccountExternalStatus, AccountInfoModelRef};
use serde::Serialize;

use crate::common::{
    JsonError, JsonResult, OauthCallbackParam, OauthLogin, OauthLoginData, OauthLoginParam,
};

use super::{AccountRegData, WebUserAuth};

impl WebUserAuth {
    //得到外部登录URL
    pub async fn oauth_user_login_url<
        T: OauthLogin<L, P, D>,
        L: OauthLoginParam + Send + Sync,
        P: OauthCallbackParam + Send + Sync,
        D: Serialize + Send + Sync,
    >(
        &self,
        oauth: &T,
        param: &L,
    ) -> JsonResult<String> {
        let url = oauth
            .login_url(param)
            .await
            .map_err(|e| JsonError::Message(fluent_message!("user-external-login-url", e)))?;
        Ok(url)
    }
    //得到外部数据转为内部AccountExternalModel
    pub async fn oauth_user_login<
        T: OauthLogin<L, P, D>,
        L: OauthLoginParam + Send + Sync,
        P: OauthCallbackParam + Send + Sync,
        D: Serialize + Send + Sync,
    >(
        &self,
        oauth: &T,
        param: &P,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<(AccountExternalModel, OauthLoginData, D)> {
        let (data, ext_data) = oauth
            .login_callback(param)
            .await
            .map_err(|e| JsonError::Message(fluent_message!("user-external-call", e)))?;
        let user_ext_rs = self
            .user_dao
            .account_dao
            .account_external
            .find_by_external(&data.config_name, &data.external_type, &data.external_id)
            .await;

        let ext_op = match user_ext_rs {
            Ok(ext) => {
                if AccountExternalStatus::Enable.eq(ext.status) {
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
                let reg_op = env_data
                    .map(|e| e.request_id.as_ref().map(|e| e.to_string()))
                    .unwrap_or_default()
                    .unwrap_or_default();
                let mut info = model_option_set!(AccountInfoModelRef,{
                    reg_ip:reg_op,
                    reg_from:reg_from,
                });
                if let Some(ref img) = data.external_pic {
                    info.headimg = Some(img);
                }
                let user = self
                    .reg_user(
                        &AccountRegData {
                            status_enable: true,
                            nikename: data.external_nikename.as_str(),
                            passwrod: None,
                            name: None,
                            email: None,
                            mobile: None,
                            external: Some((
                                data.config_name.as_str(),
                                data.external_type.as_str(),
                                data.external_id.as_str(),
                                data.external_name.as_str(),
                            )),
                            info: Some(info),
                        },
                        op_user_id,
                        env_data,
                    )
                    .await?;
                let user_ext = self
                    .user_dao
                    .account_dao
                    .account_external
                    .find_by_account_external(
                        &user,
                        &data.config_name,
                        &data.external_type,
                        &data.external_id,
                    )
                    .await?;
                self.user_dao
                    .account_dao
                    .account_external
                    .token_update(
                        &user_ext,
                        &data.external_name,
                        &data.token_data,
                        data.token_timeout,
                        Some(&data.external_nikename),
                        data.external_gender.as_deref(),
                        data.external_link.as_deref(),
                        data.external_pic.as_deref(),
                        op_user_id,
                        env_data,
                    )
                    .await?;
                user_ext
            }
            Some(user_ext) => user_ext,
        };
        Ok((user_ext, data, ext_data))
    }
    /// 已登陆后绑定外部账号
    pub async fn oauth_user_bind<
        T: OauthLogin<L, P, D>,
        L: OauthLoginParam + Send + Sync,
        P: OauthCallbackParam + Send + Sync,
        D: Serialize + Send + Sync,
    >(
        &self,
        oauth: &T,
        param: &P,
        account_id: u64,
        op_user_id: u64,
        env_data: Option<&RequestEnv>,
    ) -> JsonResult<(AccountExternalModel, OauthLoginData, D)> {
        let (param, ext_data) = oauth
            .login_callback(param)
            .await
            .map_err(|e| JsonError::Message(fluent_message!("user-external-call", e)))?;
        let user_external = &self.user_dao.account_dao.account_external;
        let extdata = user_external
            .find_by_external(&param.config_name, &param.external_type, &param.external_id)
            .await;
        let ext_op = match extdata {
            Ok(ext) => {
                if AccountExternalStatus::Enable.eq(ext.status) {
                    if ext.account_id != account_id {
                        return Err(JsonError::Message(
                            fluent_message!("user-external-other-bind",
                                {"name":ext.external_name,"id":ext.account_id }
                            ),
                        )); //"this account {$name} bind on other account[{$id}]",
                    }
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
        let user = &self
            .user_dao
            .account_dao
            .account
            .find_by_id(&account_id)
            .await?;
        let ext = match ext_op {
            Some(ext) => ext,
            None => {
                let ext_id = user_external
                    .add_external(
                        user,
                        &param.config_name,
                        &param.external_type,
                        &param.external_id,
                        &param.external_name,
                        op_user_id,
                        None,
                        env_data,
                    )
                    .await?;
                user_external.find_by_id(&ext_id).await?
            }
        };
        user_external
            .token_update(
                &ext,
                &param.external_name,
                &param.token_data,
                param.token_timeout,
                Some(&param.external_nikename),
                param.external_gender.as_deref(),
                param.external_link.as_deref(),
                param.external_pic.as_deref(),
                op_user_id,
                env_data,
            )
            .await?;
        Ok((ext, param, ext_data))
    }
}
