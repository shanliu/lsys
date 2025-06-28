use crate::common::{JsonData, JsonError, JsonResult};

use crate::common::{JsonResponse, UserAuthQueryDao};
use crate::dao::access::api::system::user::{CheckUserAppEdit, CheckUserAppSenderSmsConfig};
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_app::dao::{UserAppDataParam, SUB_APP_SECRET_NOTIFY_TYPE};
use lsys_app::model::AppStatus;
use lsys_core::fluent_message;
use serde::Deserialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct RequestExterSubAppParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
}

pub async fn sub_app_request(
    param: &RequestExterSubAppParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(param.app_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
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
        .inner_feature_sub_app_request(&app, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

pub async fn sub_app_notify_get_config(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppSenderSmsConfig {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;
    let app_param = UserAppDataParam {
        status: Some(AppStatus::Enable),
        parent_app_id: Some(auth_data.session().user_app_id),
        client_id: None,
        app_id: None,
    };
    let apps = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .user_app_data(auth_data.user_id(), &app_param, None, None)
        .await?;
    let notify = req_dao
        .web_dao
        .web_app
        .app_dao
        .app_notify
        .record
        .find_config_by_apps(
            &apps.iter().map(|e| e.0.id).collect::<Vec<_>>(),
            SUB_APP_SECRET_NOTIFY_TYPE,
        )
        .await?;
    let data = apps
        .into_iter()
        .map(|e| {
            let n = notify.iter().find(|t| t.app_id == e.0.id);
            let url = n.map(|t| &t.call_url);
            let change_time = n.map(|t| {
                if t.change_time > 0 {
                    t.change_time
                } else {
                    t.create_time
                }
            });
            let change_user_id = n.map(|t| t.change_user_id);
            json!({
                "app_id":e.0.id,
                "app_name":e.0.name,
                 "call_url":url,
                 "change_time":change_time,
                 "change_user_id":change_user_id,
            })
        })
        .collect::<Vec<_>>();
    Ok(JsonResponse::data(JsonData::body(json!({
        "data":data,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct SubAppNotifyConfigParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub url: String,
}

pub async fn sub_app_notify_set_config(
    param: &SubAppNotifyConfigParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(param.app_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppEdit {
                res_user_id: app.user_id,
            },
        )
        .await?;
    app.app_status_check()?;
    if app.parent_app_id != 0 {
        return Err(JsonError::Message(fluent_message!(
            "app-notify-only-parent"
        )));
    }
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app_notify
        .record
        .set_app_config(
            &app,
            SUB_APP_SECRET_NOTIFY_TYPE,
            &param.url,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}
