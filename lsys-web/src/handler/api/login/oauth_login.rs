use crate::{
    dao::{UserAuthQueryDao, WebDao},
    module::oauth::{OauthCallbackParam, OauthLogin, OauthLoginParam},
    {JsonData, JsonResult},
};

pub async fn user_oauth<
    T: OauthLogin<L, P, D>,
    L: OauthLoginParam + Send + Sync,
    P: OauthCallbackParam + Send + Sync,
    D: Serialize + Send + Sync,
>(
    config_key: &str,
    app_dao: &WebDao,
) -> JsonResult<T> {
    Ok(app_dao.user.user_oauth::<T, L, P, D>(config_key).await?)
}

use serde::Serialize;
use serde_json::json;
pub async fn user_oauth_login<
    T: OauthLogin<L, P, D>,
    L: OauthLoginParam + Send + Sync,
    P: OauthCallbackParam + Send + Sync,
    D: Serialize + Send + Sync,
>(
    config_key: &str,
    app_dao: &WebDao,
    param: &L,
) -> JsonResult<JsonData> {
    let oauth = &user_oauth::<T, L, P, D>(config_key, app_dao).await?;
    app_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(0, &[], &res_data!(SystemLogin))
        .await?;
    let url = app_dao
        .user
        .user_oauth_login::<T, L, P, D>(oauth, param)
        .await?;
    Ok(JsonData::data(json!({ "url": url })))
}
pub async fn user_oauth_callback<
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
    let oauth = &user_oauth::<O, L, P, Q>(config_key, &req_dao.web_dao).await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(0, &[], &res_data!(SystemLogin))
        .await?;
    let (_, authlock, data) = req_dao
        .web_dao
        .user
        .user_oauth_callback::<O, L, P, Q, _>(
            oauth,
            &req_dao.user_session,
            config_key,
            &req_dao.req_env,
            param,
        )
        .await?;
    Ok(JsonData::data(json!({
        "auth_data":data,
        "token":authlock.to_string()
    })))
}
