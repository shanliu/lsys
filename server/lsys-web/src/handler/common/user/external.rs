use crate::{
    dao::{RequestDao, WebDao},
    handler::access::{AccessSystemLogin, AccessUserExternalEdit},
    module::oauth::{OauthCallbackParam, OauthLogin, OauthLoginParam},
    {JsonData, JsonResult},
};

use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use lsys_user::model::UserExternalStatus;
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ExternalDeleteParam {
    pub ext_id: u64,
}
pub async fn user_external_delete<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: ExternalDeleteParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let res = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user_external
        .find_by_id(&param.ext_id)
        .await;

    match res {
        Ok(ext) => {
            if UserExternalStatus::Enable.eq(ext.status) {
                req_dao
                    .web_dao
                    .user
                    .rbac_dao
                    .rbac
                    .check(
                        &AccessUserExternalEdit {
                            user_id: req_auth.user_data().user_id,
                            res_user_id: req_auth.user_data().user_id,
                        },
                        None,
                    )
                    .await?;
                req_dao
                    .web_dao
                    .user
                    .user_dao
                    .user_account
                    .user_external
                    .del_external(&ext, None, Some(&req_dao.req_env))
                    .await?;
            }
        }
        Err(e) => {
            if !e.is_not_found() {
                return Err(e.into());
            }
        }
    }
    Ok(JsonData::message("del ext ok"))
}

#[derive(Debug, Deserialize)]
pub struct ExternalListDataParam {
    pub oauth_type: Option<Vec<String>>,
}

pub async fn user_external_list_data<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: ExternalListDataParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    let data = req_dao
        .web_dao
        .user
        .user_external(req_auth.user_data().user_id, param.oauth_type.as_deref())
        .await?;
    Ok(JsonData::data(json!({
        "data": data ,
        "total":data.len(),
    })))
}

//检查权限并获取登录URL
pub async fn user_external_login_url<
    T: OauthLogin<L, P, D>,
    L: OauthLoginParam + Send + Sync,
    P: OauthCallbackParam + Send + Sync,
    D: Serialize + Send + Sync,
>(
    config_key: &str,
    app_dao: &WebDao,
    param: &L,
) -> JsonResult<JsonData> {
    let oauth = &app_dao
        .user
        .user_external_oauth::<T, L, P, D>(config_key)
        .await?;
    app_dao
        .user
        .rbac_dao
        .rbac
        .check(&AccessSystemLogin {}, None)
        .await?;
    let url = app_dao
        .user
        .user_external_login_url::<T, L, P, D>(oauth, param)
        .await?;
    Ok(JsonData::data(json!({ "url": url })))
}
