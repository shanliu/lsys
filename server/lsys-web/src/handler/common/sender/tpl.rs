use crate::{
    dao::RequestAuthDao,
    handler::access::{AccessAdminSenderTplEdit, AccessAdminSenderTplView},
    JsonData, JsonResult, PageParam,
};
use lsys_sender::model::SenderType;
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct TplListParam {
    pub user_id: Option<u64>,
    pub sender_type: Option<i8>,
    pub id: Option<u64>,
    pub tpl_id: Option<String>,
    pub count_num: Option<bool>,
    pub page: Option<PageParam>,
}
pub async fn tpl_body_list<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: TplListParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminSenderTplView {
                user_id: req_auth.user_data().user_id,
                res_user_id: param.user_id.unwrap_or(req_auth.user_data().user_id),
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let sender_type = match param.sender_type {
        Some(e) => Some(SenderType::try_from(e).map_err(|e| req_dao.fluent_json_data(e))?),
        None => None,
    };
    let data = req_dao
        .web_dao
        .sender_tpl
        .list_data(
            &param.user_id.unwrap_or(req_auth.user_data().user_id),
            &sender_type,
            &param.id,
            &param.tpl_id,
            &Some(param.page.unwrap_or_default().into()),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .sender_tpl
                .list_count(
                    &param.user_id.unwrap_or(req_auth.user_data().user_id),
                    &sender_type,
                    &param.id,
                    &param.tpl_id,
                )
                .await
                .map_err(|e| req_dao.fluent_json_data(e))?,
        )
    } else {
        None
    };
    Ok(JsonData::data(json!({ "data": data,"total":count })))
}

#[derive(Debug, Deserialize)]
pub struct TplAddParam {
    pub tpl_id: String,

    pub tpl_data: String,
    pub sender_type: i8,
    pub user_id: Option<u64>,
}
pub async fn tpl_body_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: TplAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminSenderTplEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: param.user_id.unwrap_or(req_auth.user_data().user_id),
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let sender_type =
        SenderType::try_from(param.sender_type).map_err(|e| req_dao.fluent_json_data(e))?;
    let id = req_dao
        .web_dao
        .sender_tpl
        .add(
            sender_type,
            &param.tpl_id,
            &param.tpl_data,
            &param.user_id.unwrap_or(req_auth.user_data().user_id),
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::data(json!({ "id": id })))
}

#[derive(Debug, Deserialize)]
pub struct TplEditParam {
    pub id: u64,
    pub tpl_data: String,
}
pub async fn tpl_body_edit<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: TplEditParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let tpl = req_dao
        .web_dao
        .sender_tpl
        .find_by_id(&param.id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminSenderTplEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: tpl.user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .sender_tpl
        .edit(
            &tpl,
            &param.tpl_data,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct TplDelParam {
    pub id: u64,
}
pub async fn tpl_body_del<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: TplDelParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let res = req_dao.web_dao.sender_tpl.find_by_id(&param.id).await;
    let data = match res {
        Ok(d) => d,
        // Err(SenderError::Sqlx(sqlx::Error::RowNotFound)) => {
        //     return Ok(JsonData::message("not find"))
        // }
        Err(e) => return Err(req_dao.fluent_json_data(e)),
    };
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessAdminSenderTplEdit {
                user_id: req_auth.user_data().user_id,
                res_user_id: data.user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .sender_tpl
        .del(&data, &req_auth.user_data().user_id, Some(&req_dao.req_env))
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}
