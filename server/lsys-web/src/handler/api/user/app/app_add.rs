use crate::common::JsonResult;
use crate::common::{JsonData, UserAuthQueryDao};
use crate::dao::access::api::user::CheckUserAppEdit;
use lsys_access::dao::AccessSession;
use lsys_app::dao::AppDataParam;
use serde_json::json;

pub struct AddParam {
    pub name: String,
    pub client_id: String,
    pub parent_app_id: Option<u64>,
}

pub async fn add(param: &AddParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppEdit {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let user_app_id = auth_data.session().user_app_id;
    let parent_app = if user_app_id > 0 {
        let tmp_app = req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .find_by_id(&user_app_id)
            .await?;
        Some(tmp_app)
    } else if let Some(parent_id) = param.parent_app_id {
        let tmp_app = req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .find_by_id(&parent_id)
            .await?;
        Some(tmp_app)
    } else {
        None
    };

    if let Some(parent_app) = &parent_app {
        req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .inner_feature_sub_app_check(parent_app)
            .await?;
    }

    let app_id = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .app_new_request(
            auth_data.user_id(),
            parent_app.as_ref(),
            user_app_id,
            &AppDataParam {
                name: &param.name,
                client_id: &param.client_id,
            },
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::data(json!({"id":app_id})))
}
