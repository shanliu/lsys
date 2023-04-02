use std::net::IpAddr;

use crate::{
    dao::{UserAuthQueryDao, WebDao},
    module::oauth::{OauthCallbackParam, OauthLogin, OauthLoginParam},
    {JsonData, JsonResult},
};

use lsys_user::dao::auth::{ExternalLogin, LoginEnv, UserSession};
use serde::Serialize;
use serde_json::json;

//检查权限并完成回调
pub async fn user_external_callback<
    't,
    O: OauthLogin<L, P, Q>,
    L: OauthLoginParam + Send + Sync,
    P: OauthCallbackParam + Send + Sync,
    Q: Serialize + Send + Sync,
>(
    config_key: &str,
    req_dao: &UserAuthQueryDao,
    param: &P,
) -> JsonResult<JsonData> {
    let oauth = &req_dao
        .web_dao
        .user
        .user_external_oauth::<O, L, P, Q>(config_key)
        .await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(&AccessSystemLogin {})
        .await?;
    let (ext_model, _, ext_data) = req_dao
        .web_dao
        .user
        .user_external_callback::<O, L, P, Q, _>(oauth, config_key, &req_dao.req_env, param)
        .await?;
    let login_env = LoginEnv {
        login_ip: req_dao.req_env.ip.parse::<IpAddr>().ok(),
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
        .await?;
    req_dao
        .user_session
        .write()
        .await
        .set_session_token(token.into());
    let user_data = req_dao.user_session.read().await.get_session_data().await?;
    Ok(JsonData::data(json!({
        "auth_data":data,
        "token":authlock.to_string()
    })))
}
