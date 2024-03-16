use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};

use crate::handler::common::rbac::{
    rbac_access_check, rbac_menu_check, RbacAccessParam, RbacMenuParam,
};
use crate::{dao::RequestAuthDao, JsonData, JsonResult};

pub async fn user_access_check<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RbacAccessParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_access_check(
        req_auth.user_data().user_id,
        param,
        &req_dao.web_dao.user.rbac_dao,
        req_dao,
    )
    .await
}

pub async fn user_menu_check<'t, T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: RbacMenuParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    rbac_menu_check(
        req_auth.user_data().user_id,
        param,
        &req_dao.web_dao.user.rbac_dao,
    )
    .await
}
