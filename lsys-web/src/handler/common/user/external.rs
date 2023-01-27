use crate::{
    dao::RequestDao,
    {JsonData, JsonResult},
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use lsys_user::model::UserExternalStatus;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ExternalBindParam {
    pub config_name: String,
    pub external_type: String,
    pub external_id: String,
    pub external_name: String,
    pub token_data: String,
    pub token_timeout: u64,
    pub external_gender: Option<String>,
    pub external_link: Option<String>,
    pub external_pic: Option<String>,
}

/// 已登陆后绑定外部账号
pub async fn user_external_bind<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: ExternalBindParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .access
        .check(
            req_auth.user_data().user_id,
            &[],
            &res_data!(UserExternalEdit(req_auth.user_data().user_id)),
        )
        .await?;
    let user_external = &req_dao.web_dao.user.user_dao.user_account.user_external;
    let extdata = user_external
        .find_by_external(&param.config_name, &param.external_type, &param.external_id)
        .await;
    let ext_op = match extdata {
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
    let user = req_dao
        .web_dao
        .user
        .user_dao
        .user_account
        .user
        .find_by_id(&req_auth.user_data().user_id)
        .await?;
    let ext = match ext_op {
        Some(ext) => ext,
        None => {
            let ext_id = user_external
                .add_external(
                    &user,
                    param.config_name.clone(),
                    param.external_type,
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
            param.external_name,
            param.token_data,
            param.token_timeout,
            param.external_gender,
            param.external_link,
            param.external_pic,
        )
        .await?;
    Ok(JsonData::data(json!({ "id": ext.id })))
}

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
                    .access
                    .check(
                        req_auth.user_data().user_id,
                        &[],
                        &res_data!(UserExternalEdit(req_auth.user_data().user_id)),
                    )
                    .await?;
                req_dao
                    .web_dao
                    .user
                    .user_dao
                    .user_account
                    .user_external
                    .del_external(&ext, None)
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
