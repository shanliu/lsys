use crate::handler::common::rbac::{
    rbac_role_add, rbac_role_add_user, rbac_role_delete, rbac_role_delete_user, rbac_role_edit,
    rbac_role_list_data, rbac_role_list_user, rbac_role_tags, rbac_user_relation_data,
    rbac_user_role_options, RoleAddParam, RoleAddUserParam, RoleDeleteParam, RoleDeleteUserParam,
    RoleEditParam, RoleListDataParam, RoleListUserParam, RoleOptionsParam, RoleRelationDataParam,
    RoleTagsParam,
};
use crate::{
    dao::RequestAuthDao,
    {JsonData, JsonResult},
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
pub async fn user_role_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleAddParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_role_add(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_role_edit<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleEditParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_role_edit(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_role_list_user<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleListUserParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_role_list_user(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_role_add_user<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleAddUserParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_role_add_user(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_role_delete_user<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleDeleteUserParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_role_delete_user(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_role_delete<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleDeleteParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_role_delete(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_role_list_data<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleListDataParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_role_list_data(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_role_tags<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleTagsParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_role_tags(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_role_options<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleOptionsParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_user_role_options(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}

pub async fn user_relation_data<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleRelationDataParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_user_relation_data(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
        req_dao,
    )
    .await
}
