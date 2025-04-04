use crate::common::JsonResult;
use crate::common::{JsonData, UserAuthQueryDao};
use crate::dao::access::api::user::CheckUserAppEdit;

use lsys_access::dao::AccessSession;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct DeleteParam {
    pub app_id: u64,
}

pub async fn delete(param: &DeleteParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppEdit {
                res_user_id: app.user_id,
            },
        )
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .app_delete(&app, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}
