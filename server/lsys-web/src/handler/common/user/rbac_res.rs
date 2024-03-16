use crate::handler::common::rbac::{
    rbac_res_add, rbac_res_delete, rbac_res_edit, rbac_res_list_data, rbac_res_tags, ResAddParam,
    ResDeleteParam, ResEditParam, ResListDataParam, ResTagsParam,
};
use crate::{
    dao::RequestAuthDao,
    {JsonData, JsonResult},
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
pub async fn user_res_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: ResAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_res_add(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_res_edit<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: ResEditParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_res_edit(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_res_delete<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: ResDeleteParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_res_delete(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_res_list_data<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: ResListDataParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_res_list_data(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_res_tags<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: ResTagsParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_res_tags(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}
