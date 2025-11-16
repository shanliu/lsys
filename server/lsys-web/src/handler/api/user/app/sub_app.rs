use crate::common::{JsonData, JsonError, JsonResult};

use crate::common::{JsonResponse, PageParam, UserAuthQueryDao};
use crate::dao::access::api::system::user::{
    CheckUserAppEdit, CheckUserAppSenderSmsConfig, CheckUserAppView,
};
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_app::dao::{AppAttrParam, UserAppDataParam, UserSubAppParam, SUB_APP_SECRET_NOTIFY_TYPE};
use lsys_app::model::AppStatus;
use lsys_core::fluent_message;
use serde::Deserialize;
use serde::Serialize;
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

#[derive(Debug, Deserialize)]
pub struct SubAppNotifyGetConfigParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
}

pub async fn sub_app_notify_get_config( 
    param: &SubAppNotifyGetConfigParam,
    req_dao: &UserAuthQueryDao
) -> JsonResult<JsonResponse> {
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
        like_client_id: None,
        app_id: Some(param.app_id),
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
        .first()
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
        });
    Ok(JsonResponse::data(JsonData::body(json!({
        "data":data,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct SubAppNotifySetConfigParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub url: String,
}

pub async fn sub_app_notify_set_config(
    param: &SubAppNotifySetConfigParam,
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

#[derive(Deserialize)]
pub struct SubAppListParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

#[derive(Serialize)]
pub struct ShowSubAppRecord {
    pub id: u64,
    pub name: String,
    pub client_id: String,
    pub status: i8,
    pub user_id: u64,
    pub change_time: u64,
    pub change_user_id: u64,
    pub oauth_client: bool,                           //是否启用OAUTH登录
    pub oauth_client_data: Option<serde_json::Value>, //OAUTH登录信息
    pub exter_feature: Option<Vec<String>>,           //外部功能及启用状态
    pub sub_req_pending_count: i64,                   //子应用的请求数量
}

//用户层的子应用列表
pub async fn sub_app_list(
    param: &SubAppListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    // 获取父应用信息
    let app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(param.app_id)
        .await?;

    // 权限验证
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppView {
                res_user_id: app.user_id,
            },
        )
        .await?;

    // 检查父应用id必须为0（非子应用）
    if app.parent_app_id != 0 {
        return Err(JsonError::Message(fluent_message!(
            "app-only-parent-can-list-sub"
        )));
    }

    let status = if let Some(e) = param.status {
        Some(match AppStatus::try_from(e) {
            Ok(ts) => ts,
            Err(err) => return Err(err.into()),
        })
    } else {
        None
    };

    let app_param = UserSubAppParam {
        status,
        app_id: param.app_id,
    };

    let app_attr = AppAttrParam {
        inner_feature: true,
        exter_feature: true,
        sub_app_count: false,
        oauth_client_data: true,
        oauth_server_data: false,
        parent_app: false,
        req_pending_count: false,
        sub_req_pending_count: true,
    };

    let appdata = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .user_sub_app_data(
            &app_param,
            Some(&app_attr),
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;

    let out = appdata
        .into_iter()
        .map(|e| ShowSubAppRecord {
            id: e.0.id,
            name: e.0.name,
            client_id: e.0.client_id,
            status: e.0.status,
            user_id: e.0.user_id,
            change_time: e.0.change_time,
            change_user_id: e.0.change_user_id,
            oauth_client: e.1.oauth_client.unwrap_or(false),
            oauth_client_data: e.1.oauth_client_data.map(|t| {
                json!({
                    "callback_domain":t.callback_domain,
                    "scope_data":t.scope_data,
                })
            }),
            exter_feature: e.1.exter_feature,
            sub_req_pending_count: e.1.sub_req_pending_count.unwrap_or(0),
        })
        .collect::<Vec<_>>();

    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_app
                .app_dao
                .app
                .user_sub_app_count(&app_param)
                .await?,
        )
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": bind_vec_user_info_from_req!(
            req_dao,
            out,
            user_id,
            false
        ),
        "total":count
    }))))
}
