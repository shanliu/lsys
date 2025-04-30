use crate::common::JsonData;
use crate::common::JsonResult;
use crate::common::{JsonResponse, UserAuthQueryDao};
use crate::dao::access::api::user::CheckUserAppEdit;
use lsys_access::dao::AccessSession;
use lsys_app::dao::AppDataParam;
use serde::Deserialize;
use serde_json::json;
#[derive(Deserialize)]
pub struct ChangeParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub name: String,
    pub client_id: String,
}

pub async fn change(param: &ChangeParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
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
    Ok(JsonResponse::default())
}

#[derive(Deserialize)]
pub struct AddAppSecretParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub secret: Option<String>,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub secret_timeout: u64,
}

pub async fn app_secret_add(
    param: &AddAppSecretParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
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
        .app_secret_add(
            &app,
            param.secret.as_deref(),
            param.secret_timeout,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(
        json!({"data":secret_data}),
    )))
}

#[derive(Deserialize)]
pub struct ChangeAppSecretParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub old_secret: String,
    pub secret: Option<String>,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub secret_timeout: u64,
}

pub async fn app_secret_change(
    param: &ChangeAppSecretParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
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
        .app_secret_change(
            &app,
            &param.old_secret,
            param.secret.as_deref(),
            param.secret_timeout,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(
        json!({"data":secret_data}),
    )))
}

#[derive(Deserialize)]
pub struct DelAppSecretParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub old_secret: String,
}

pub async fn app_secret_del(
    param: &DelAppSecretParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
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
        .app_secret_del(
            &app,
            &param.old_secret,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::default())
}

#[derive(Deserialize)]
pub struct ChangeNotifySecretParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub secret: Option<String>,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub secret_timeout: u64,
}

pub async fn notify_secret_change(
    param: &ChangeNotifySecretParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
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
        .notify_secret_change(
            &app,
            param.secret.as_deref(),
            param.secret_timeout,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(
        json!({"data":secret_data}),
    )))
}
