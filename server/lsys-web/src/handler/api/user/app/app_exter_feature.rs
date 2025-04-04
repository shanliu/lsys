use crate::common::{JsonData, UserAuthQueryDao};
use crate::common::{JsonError, JsonResult};
use crate::dao::access::api::user::CheckUserAppEdit;

use lsys_access::dao::AccessSession;
use lsys_app::model::AppRequestStatus;
use lsys_core::fluent_message;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct RequestExterFeatureParam {
    pub app_id: u64,
    pub featuer_data: Vec<String>,
}

pub async fn request_exter_feature(
    param: &RequestExterFeatureParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(&param.app_id)
        .await?;
    //添加外部功能时检测是否开通必要的的依赖关系
    let mut featch_key = Vec::with_capacity(param.featuer_data.len());
    for fk in param.featuer_data.iter() {
        match fk.as_str() {
            crate::handler::APP_FEATURE_RBAC => {
                req_dao
                    .web_dao
                    .web_app
                    .app_dao
                    .app
                    .inner_feature_sub_app_check(&app)
                    .await?;
                featch_key.push(fk.as_str());
            }
            tmp => {
                featch_key.push(tmp);
            }
        }
    }
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

    if app.parent_app_id > 0 {
        let parent_app = req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .find_by_id(&app.parent_app_id)
            .await?;
        //父应用必须已开通对应外部权限
        req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .exter_feature_check(&parent_app, &featch_key)
            .await?;
    }

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .exter_feature_request(
            &app,
            &featch_key,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}

#[derive(Deserialize)]
pub struct ConfirmExterFeatureParam {
    pub app_id: u64,
    pub app_req_id: u64,
    pub confirm_status: i8,
    pub confirm_note: String,
}

pub async fn confirm_exter_feature(
    param: &ConfirmExterFeatureParam,
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
    //在申请入口判断父应用是否有对应外部权限,已申请正常审核
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
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserAppEdit {
                res_user_id: parent_app.user_id,
            },
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
