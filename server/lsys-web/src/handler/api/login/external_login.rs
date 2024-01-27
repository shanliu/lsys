use std::net::IpAddr;

use crate::{
    dao::{user::ShowUserAuthData, UserAuthQueryDao},
    handler::access::AccessSystemLogin,
    module::oauth::{OauthCallbackParam, OauthLogin, OauthLoginParam},
    JsonResult,
};

use lsys_user::dao::auth::{ExternalLogin, LoginEnv, UserAuthTokenData, UserSession};
use serde::Serialize;

//检查权限并完成回调
pub async fn user_external_login_callback<
    O: OauthLogin<L, P, Q>,
    L: OauthLoginParam + Send + Sync,
    P: OauthCallbackParam + Send + Sync,
    Q: Serialize + Send + Sync,
>(
    config_key: &str,
    req_dao: &UserAuthQueryDao,
    param: &P,
) -> JsonResult<(UserAuthTokenData, ShowUserAuthData)> {
    let oauth = &req_dao
        .web_dao
        .user
        .user_external_oauth::<O, L, P, Q>(config_key)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(&AccessSystemLogin {}, None)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let res = req_dao
        .web_dao
        .user
        .user_external_login::<O, L, P, Q>(oauth, &req_dao.req_env, param, Some(&req_dao.req_env))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let (ext_model, _, ext_data) = res;
    let login_env = LoginEnv {
        login_ip: req_dao
            .req_env
            .request_ip
            .as_ref()
            .map(|e| e.parse::<IpAddr>().ok())
            .unwrap_or_default(),
    };
    let token = req_dao
        .web_dao
        .user
        .user_dao
        .user_auth
        .login(
            ExternalLogin {
                external: ext_model.clone(),
                ext_data,
            },
            login_env,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .user_session
        .write()
        .await
        .set_session_token(token.clone().into());
    let user_data = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = ShowUserAuthData::from(user_data);
    Ok((token, data))
}
