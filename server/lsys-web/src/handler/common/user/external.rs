use crate::{
    dao::{RequestAuthDao, RequestDao},
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
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
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
                    .await
                    .map_err(|e| req_dao.fluent_json_data(e))?;
                req_dao
                    .web_dao
                    .user
                    .user_dao
                    .user_account
                    .user_external
                    .del_external(&ext, None, Some(&req_dao.req_env))
                    .await
                    .map_err(|e| req_dao.fluent_json_data(e))?;
            }
        }
        Err(e) => {
            if !e.is_not_found() {
                return Err(req_dao.fluent_json_data(e));
            }
        }
    }
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct ExternalListDataParam {
    pub oauth_type: Option<Vec<String>>,
}

pub async fn user_external_list_data<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: ExternalListDataParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let data = req_dao
        .web_dao
        .user
        .user_external(req_auth.user_data().user_id, param.oauth_type.as_deref())
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
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
    param: &L,
    req_dao: &RequestDao,
) -> JsonResult<JsonData> {
    let oauth = &req_dao
        .web_dao
        .user
        .user_external_oauth::<T, L, P, D>(config_key)
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
    let url = req_dao
        .web_dao
        .user
        .user_external_login_url::<T, L, P, D>(oauth, param)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "url": url })))
}
