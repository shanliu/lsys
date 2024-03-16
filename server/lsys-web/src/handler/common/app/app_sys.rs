use lsys_user::dao::auth::{SessionData, SessionTokenData, UserSession};
use serde::Deserialize;

use crate::dao::RequestAuthDao;

use crate::handler::access::AccessUserAppStatus;
use crate::{JsonData, JsonResult};

#[derive(Debug, Deserialize)]
pub struct AppStatusParam {
    app_id: u64,
    status: bool,
}

pub async fn app_status<T: SessionTokenData, D: SessionData, S: UserSession<T, D>>(
    param: AppStatusParam,
    req_dao: &RequestAuthDao<T, D, S>,
) -> JsonResult<JsonData> {
    let req_auth = req_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    let app = req_dao
        .web_dao
        .app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .user
        .rbac_dao
        .rbac
        .check(
            &AccessUserAppStatus {
                user_id: req_auth.user_data().user_id,
            },
            None,
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    req_dao
        .web_dao
        .app
        .app_dao
        .app
        .app_status(
            &app,
            param.status,
            &req_auth.user_data().user_id,
            Some(&req_dao.req_env),
        )
        .await
        .map_err(|e| req_dao.fluent_json_data(e))?;
    Ok(JsonData::default())
}
