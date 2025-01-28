use crate::common::JsonData;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::system::CheckAdminApp;
use lsys_access::dao::AccessSession;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AppDeleteParam {
    pub app_id: u64,
}
//APP 删除
pub async fn app_delete(
    param: &AppDeleteParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.access_env().await?, &CheckAdminApp {}, None)
        .await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .app_delete(&app, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}
