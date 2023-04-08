use crate::handler::common::rbac::{
    rbac_role_add, rbac_role_add_user, rbac_role_delete, rbac_role_delete_user, rbac_role_edit,
    rbac_role_list_data, rbac_role_list_user, rbac_role_tags, rbac_user_relation_data,
    rbac_user_role_options, RoleAddParam, RoleAddUserParam, RoleDeleteParam, RoleDeleteUserParam,
    RoleEditParam, RoleListDataParam, RoleListUserParam, RoleOptionsParam, RoleRelationDataParam,
    RoleTagsParam,
};
use crate::{
    dao::RequestDao,
    {JsonData, JsonResult},
};
use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
pub async fn user_role_add<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleAddParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    rbac_role_add(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
    )
    .await
}

pub async fn user_role_edit<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleEditParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    rbac_role_edit(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
    )
    .await
}

pub async fn user_role_list_user<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleListUserParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    rbac_role_list_user(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
    )
    .await
}

pub async fn user_role_add_user<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleAddUserParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    rbac_role_add_user(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
    )
    .await
}

pub async fn user_role_delete_user<
    't,
    T: SessionTokenData,
    D: SessionData,
    S: UserSession<T, D>,
>(
    param: RoleDeleteUserParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    rbac_role_delete_user(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
    )
    .await
}

pub async fn user_role_delete<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleDeleteParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    rbac_role_delete(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
    )
    .await
}

pub async fn user_role_list_data<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleListDataParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    rbac_role_list_data(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
    )
    .await
}

pub async fn user_role_tags<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleTagsParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    rbac_role_tags(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
    )
    .await
}

pub async fn user_role_options<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleOptionsParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    rbac_user_role_options(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
    )
    .await
}

pub async fn user_relation_data<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RoleRelationDataParam,
    req_dao: &RequestDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao.user_session.read().await.get_session_data().await?;
    rbac_user_relation_data(
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_auth.user_data().user_id,
    )
    .await
}
