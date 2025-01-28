use crate::common::{JsonData, UserAuthQueryDao};
use crate::common::{JsonError, JsonResult};
use crate::dao::access::api::user::CheckUserAppEdit;
use lsys_access::dao::AccessSession;
use lsys_app::model::AppRequestStatus;
use lsys_core::fluent_message;

pub struct AppConfirmParam {
    pub app_req_id: u64,
    pub confirm_status: i8,
    pub confirm_note: String,
}

pub async fn app_confirm(
    param: &AppConfirmParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let req_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .request_find_by_id(&param.app_req_id)
        .await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&req_app.app_id)
        .await?;
    if app.user_app_id == 0 {
        return Err(JsonError::Message(fluent_message!("not-user-app-confirm")));
    }

    let parent_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&req_app.parent_app_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.access_env().await?,
            &CheckUserAppEdit {
                res_user_id: parent_app.user_id,
            },
            None,
        )
        .await?;

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .inner_feature_sub_app_check(&parent_app)
        .await?;

    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let confirm_status = AppRequestStatus::try_from(param.confirm_status)?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .app_confirm_request(
            &app,
            &req_app,
            confirm_status,
            &param.confirm_note,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}
