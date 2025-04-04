use crate::common::JsonResult;
use crate::common::{JsonData, UserAuthQueryDao};
use crate::dao::access::api::user::CheckUserAppEdit;
use lsys_access::dao::AccessSession;
use lsys_app::dao::AppDataParam;
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
pub struct ChangeParam {
    pub app_id: u64,
    pub name: String,
    pub client_id: String,
}

pub async fn change(param: &ChangeParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonData> {
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

    if app.parent_app_id > 0 {
        let tmp_app = req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .find_by_id(&app.parent_app_id)
            .await?;
        req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .inner_feature_sub_app_check(&tmp_app)
            .await?;
    }

    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .app_change_request(
            &app,
            &AppDataParam {
                name: &param.name,
                client_id: &param.client_id,
            },
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonData::default())
}
#[derive(Deserialize)]
pub struct ResetSecretParam {
    pub app_id: u64,
    pub secret: Option<String>,
}

pub async fn secret_reset(
    param: &ResetSecretParam,
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

    let secret_data = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .app_reset_secret(
            &app,
            param.secret.as_deref(),
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonData::data(json!({"data":secret_data})))
}
