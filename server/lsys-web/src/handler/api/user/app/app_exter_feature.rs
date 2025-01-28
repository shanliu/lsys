use crate::common::{JsonData, UserAuthQueryDao};
use crate::common::{JsonError, JsonResult};
use crate::dao::access::api::user::CheckUserAppEdit;

use lsys_access::dao::AccessSession;
use lsys_app::model::AppRequestStatus;
use lsys_core::fluent_message;

pub struct AppRequestExterFeatureParam {
    pub app_id: u64,
    pub featuer_data: Vec<String>,
}

pub async fn app_request_exter_feature(
    param: &AppRequestExterFeatureParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
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
            &req_dao.access_env().await?,
            &CheckUserAppEdit {
                res_user_id: app.user_id,
            },
            None,
        )
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .exter_feature_request(
            &app,
            &param
                .featuer_data
                .iter()
                .map(|e| e.as_str())
                .collect::<Vec<_>>(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

pub struct AppConfirmExterFeatureParam {
    pub app_id: u64,
    pub app_req_id: u64,
    pub confirm_status: i8,
    pub confirm_note: String,
}

pub async fn app_confirm_exter_feature(
    param: &AppConfirmExterFeatureParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let req_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .request_find_by_id(&param.app_req_id)
        .await?;
    let confirm_status = AppRequestStatus::try_from(param.confirm_status)?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;

    if app.user_app_id == 0 {
        return Err(JsonError::Message(fluent_message!("not-user-app-confirm")));
    }

    let parent_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&app.parent_app_id)
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
        .exter_feature_confirm(
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
