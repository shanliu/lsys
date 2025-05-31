use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::PageParam;
use crate::common::UserAuthQueryDao;
use crate::dao::access::api::system::CheckAdminApp;
use lsys_access::dao::AccessSession;
use lsys_app::dao::AppAttrParam;
use lsys_app::dao::AppRequestData;
use lsys_app::dao::SystemAppParam;
use lsys_app::model::AppRequestStatus;
use lsys_app::model::AppStatus;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;

#[derive(Deserialize)]
pub struct ListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub user_id: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub app_id: Option<u64>,
    pub app_name: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    pub client_id: Option<String>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

#[derive(Serialize)]
pub struct ShowAppRecord {
    pub id: u64,
    pub name: String,
    pub client_id: String,
    pub status: i8,
    pub user_id: u64,
    pub change_time: u64,
    pub change_user_id: u64,
    pub parent_app: Option<serde_json::Value>, //对应的父应用信息
    pub exter_login: bool,
    pub oauth_client: bool,                           //是否启用OAUTH登录
    pub oauth_client_data: Option<serde_json::Value>, //OAUTH登录信息
    pub sup_app: bool,                                //是否开启子应用,可查看子应用信息
    pub oauth_server: bool,                           //是否启用OAUTH服务
    pub oauth_server_scope_data: Option<Vec<serde_json::Value>>, //OAUTH服务SCOPE设置
    pub exter_feature: Option<Vec<String>>,           //外部功能及启用状态
    pub sub_app_count: Option<serde_json::Value>,     //子APP数量
}
//系统所有的APP列表
pub async fn app_list(param: &ListParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminApp {})
        .await?;
    let status = if let Some(e) = param.status {
        Some(match AppStatus::try_from(e) {
            Ok(ts) => ts,
            Err(err) => return Err(err.into()),
        })
    } else {
        None
    };
    let app_param = SystemAppParam {
        user_id: param.user_id,
        status,
        client_id: param.client_id.as_deref(),
        app_id: param.app_id,
        app_name: param.app_name.as_deref(),
    };
    let app_attr = AppAttrParam {
        inner_feature: true,
        exter_feature: true,
        sub_app_count: true,
        oauth_client_data: true,
        oauth_server_data: true,
        parent_app: true,
    };
    let appdata = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .system_app_data(
            &app_param,
            Some(&app_attr),
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let out = appdata
        .into_iter()
        .map(|e| ShowAppRecord {
            id: e.0.id,
            name: e.0.name,
            client_id: e.0.client_id,
            status: e.0.status,
            user_id: e.0.user_id,
            change_time: e.0.change_time,
            change_user_id: e.0.change_user_id,
            parent_app: e.1.parent_app.map(|t| {
                json!({
                    "id":t.id,
                    "name":t.name,
                    "user_id":t.user_id,
                })
            }),
            exter_login: e.1.exter_login.unwrap_or(false),
            oauth_client: e.1.oauth_client.unwrap_or(false),
            sup_app: e.1.sup_app.unwrap_or(false),
            oauth_server: e.1.oauth_server.unwrap_or(false),
            oauth_client_data: e.1.oauth_client_data.map(|t| {
                json!({
                    "callback_domain":t.callback_domain,
                    "scope_data":t.scope_data,
                })
            }),
            oauth_server_scope_data: e.1.oauth_server_scope_data.map(|t| {
                t.into_iter()
                    .map(|s| {
                        json!({
                            "scope_key":s.scope_key,
                            "scope_name":s.scope_name,
                        })
                    })
                    .collect::<Vec<_>>()
            }),
            exter_feature: e.1.exter_feature,
            sub_app_count: e.1.sub_app_count.map(|t| {
                let enable = t
                    .iter()
                    .find(|s| AppStatus::Enable.eq(s.0))
                    .map(|s| s.1)
                    .unwrap_or_default();
                let init = t
                    .iter()
                    .find(|s| AppStatus::Init.eq(s.0))
                    .map(|s| s.1)
                    .unwrap_or_default();
                let disable = t
                    .iter()
                    .find(|s| AppStatus::Disable.eq(s.0))
                    .map(|s| s.1)
                    .unwrap_or_default();
                json!({
                    "enable":enable,
                    "init":init,
                    "disable":disable
                })
            }),
        })
        .collect::<Vec<_>>();

    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_app
                .app_dao
                .app
                .system_app_count(&app_param)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": bind_vec_user_info_from_req!(
            req_dao,
            out,
            user_id,false
        ),
        "total":count
    }))))
}

#[derive(Deserialize)]
pub struct RequestListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub app_id: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

#[derive(Serialize)]
pub struct ShowRequestRecord {
    pub id: u64,
    pub app_id: u64,
    pub status: i8,
    pub request_type: i8,
    pub request_user_id: u64,
    pub request_time: u64,
    pub confirm_user_id: u64,
    pub confirm_time: u64,
    pub confirm_note: String,
    pub feature_data: Option<serde_json::Value>,
    pub oauth_client_data: Option<serde_json::Value>,
    pub change_data: Option<serde_json::Value>,
}

//请求系统审核的列表
pub async fn request_list(
    param: &RequestListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminApp {})
        .await?;
    let status = if let Some(e) = param.status {
        Some(match AppRequestStatus::try_from(e) {
            Ok(ts) => ts,
            Err(err) => return Err(err.into()),
        })
    } else {
        None
    };
    let appdata = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .app_request_data(
            param.app_id,
            Some(0),
            status,
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let out = appdata
            .into_iter()
            .map(|e| {
                let mut feature_data = None;
                let mut oauth_client_data = None;
                let mut change_data = None;
                match e.1 {
                    AppRequestData::Feature(d) => {
                        feature_data = Some(json!({
                            "feature":d.feature_data,
                        }))
                    }
                    AppRequestData::ChangeInfo(d) => {
                        change_data = Some(json!({
                            "name":d.name,
                            "client_id":d.client_id,
                        }))
                    }
                    AppRequestData::OAuthClient(d) => {
                        oauth_client_data = Some(json!({
                            "scope_data":d.scope_data.split(",").map(|e|e.to_string()).collect::<Vec<String>>()
                        }))
                    }
                    AppRequestData::None => {}
                };

                ShowRequestRecord {
                    id: e.0.id,
                    status: e.0.status,
                    app_id: e.0.app_id,
                    request_type: e.0.request_type,
                    request_user_id: e.0.request_user_id,
                    request_time: e.0.request_time,
                    confirm_user_id: e.0.confirm_user_id,
                    confirm_time: e.0.confirm_time,
                    confirm_note: e.0.confirm_note,
                    feature_data,
                    oauth_client_data,
                    change_data,
                }
            })
            .collect::<Vec<_>>();

    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_app
                .app_dao
                .app
                .app_request_count(param.app_id, Some(0), status)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": bind_vec_user_info_from_req!(
            req_dao,
            out,
            request_user_id,
            false
        ),
        "total":count
    }))))
}
