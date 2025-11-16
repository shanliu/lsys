use crate::common::JsonData;
use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonError, JsonResponse, JsonResult, PageParam, UserAuthQueryDao},
    dao::access::api::system::user::CheckUserAppView,
};
use lsys_access::dao::AccessSession;
use lsys_app::dao::UserParentAppDataParam;
use lsys_app::model::AppRequestType;
use lsys_app::{
    dao::{AppAttrParam, AppRequestData, UserAppDataParam},
    model::{AppRequestStatus, AppStatus},
};
use lsys_core::fluent_message;
use serde::{Deserialize, Serialize};
use serde_json::json;
use serde_json::Map;
use serde_json::Value;

#[derive(Serialize)]
pub struct ShowAppRecord {
    pub id: u64,
    pub name: String,
    pub client_id: String,
    pub status: i8,
    pub user_id: u64,
    pub change_time: u64,
    pub change_user_id: u64,
    pub parent_app: Option<serde_json::Value>,
    pub exter_login: bool,
    pub oauth_client: bool,                           //是否启用OAUTH登录
    pub oauth_client_data: Option<serde_json::Value>, //OAUTH登录信息
    pub sup_app: bool,                                //是否开启子应用,开启后可查看子应用信息
    pub oauth_server: bool,                           //是否启用OAUTH服务
    pub oauth_server_scope_data: Option<Vec<serde_json::Value>>, //OAUTH服务SCOPE设置
    pub exter_feature: Option<Vec<String>>,           //外部功能及启用状态
    pub sub_app_count: Option<serde_json::Value>,     //子APP数量
    pub sub_req_pending_count: i64,                   //子APP请求数量
}

#[derive(Deserialize)]
pub struct UserAppListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub app_id: Option<u64>, //过滤指定APP
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub parent_app_id: Option<u64>, //获取指定父APP的子APP
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    pub client_id: Option<String>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_inner_feature: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_exter_feature: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_sub_app_count: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_oauth_client_data: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_oauth_server_data: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub attr_parent_app: Option<bool>,
}

pub async fn app_list(
    param: &UserAppListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let status = if let Some(e) = param.status {
        Some(match AppStatus::try_from(e) {
            Ok(ts) => ts,
            Err(err) => return Err(err.into()),
        })
    } else {
        None
    };

    if let Some(papp_id) = param.parent_app_id {
        let papp = req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .find_by_id(papp_id)
            .await?;
        req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .inner_feature_sub_app_check(&papp)
            .await?;
    }

    let app_param = UserAppDataParam {
        parent_app_id: param.parent_app_id,
        status,
        like_client_id: param.client_id.as_deref(),
        client_id: None,
        app_id: param.app_id,
    };
    let app_attr = AppAttrParam {
        inner_feature: param.attr_inner_feature.unwrap_or(true),
        exter_feature: param.attr_exter_feature.unwrap_or(true),
        sub_app_count: param.attr_sub_app_count.unwrap_or(true),
        oauth_client_data: param.attr_oauth_client_data.unwrap_or(true),
        oauth_server_data: param.attr_oauth_server_data.unwrap_or(true),
        parent_app: param.attr_parent_app.unwrap_or(true),
        req_pending_count: false,
        sub_req_pending_count: true,
    };

    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppView {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;
    let appdata = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .user_app_data(
            auth_data.user_id(),
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
                .user_app_count(auth_data.user_id(), &app_param)
                .await?,
        )
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": bind_vec_user_info_from_req!(req_dao,out,user_id),
        "total":count
    }))))
}

#[derive(Serialize)]
pub struct ShowParentAppRecord {
    pub id: u64,
    pub name: String,
    pub client_id: String,
    pub status: i8,
    pub user_id: u64,
    pub change_time: u64,
}
#[derive(Deserialize)]
pub struct UserParentAppListParam {
    pub key_word: Option<String>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn parent_app_list(
    param: &UserParentAppListParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let app_param = UserParentAppDataParam {
        key_word: param.key_word.as_deref(),
    };

    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppView {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let appdata = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .user_parent_app_data(&app_param, param.page.as_ref().map(|e| e.into()).as_ref())
        .await?;
    let out = appdata
        .into_iter()
        .map(|e| ShowParentAppRecord {
            id: e.id,
            name: e.name,
            client_id: e.client_id,
            status: e.status,
            user_id: e.user_id,
            change_time: e.change_time,
        })
        .collect::<Vec<_>>();
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_app
                .app_dao
                .app
                .user_parent_app_count(&app_param)
                .await?,
        )
    } else {
        None
    };

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": bind_vec_user_info_from_req!(req_dao,out,user_id),
        "total":count
    }))))
}

#[derive(Deserialize)]
pub struct SecretViewSecretParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub app_secret: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub notify_secret: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub oauth_secret: bool,
}

pub async fn secret_view(
    param: &SecretViewSecretParam,
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
            &CheckUserAppView {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;
    let mut out_data = Map::new();
    if param.app_secret {
       let app_secret_data=  req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .view_app_secret(&app, auth_data.user_id(), Some(&req_dao.req_env))
            .await?;
        out_data.insert("app_secret".to_string(), json!(app_secret_data));
    }
    if param.notify_secret {
        let notify_data=  req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .view_notify_secret(&app, auth_data.user_id(), Some(&req_dao.req_env))
            .await?;
         out_data.insert(
            "notify_secret".to_string(),
            json!({
                "secret":notify_data.secret_data,
                "timeout":notify_data.time_out
            }),
        );
    }
    if param.oauth_secret {
        req_dao
            .web_dao
            .web_app
            .app_dao
            .oauth_client
            .oauth_check(&app)
            .await?;
        let secret_data = req_dao
            .web_dao
            .web_app
            .app_dao
            .oauth_client
            .oauth_view_secret(&app, auth_data.user_id(), Some(&req_dao.req_env))
            .await?;
        out_data.insert("oauth_secret".to_string(), json!(secret_data));
    }
    Ok(JsonResponse::data(JsonData::body(Value::Object(out_data))))
}

pub async fn sub_app_secret_view(
    param: &SecretViewSecretParam,
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
    if app.parent_app_id == 0 {
        return Err(JsonError::JsonResponse(
            JsonData::default().set_code(403),
            fluent_message!("system-error", "can't see system app"),
        ));
    }
    let parent_app = req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .find_by_id(app.parent_app_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserAppView {
                res_user_id: parent_app.user_id,
            },
        )
        .await?;
    let mut out_data = Map::new();
    if param.app_secret {
        let app_secret_data= req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .view_app_secret(&app, auth_data.user_id(), Some(&req_dao.req_env))
            .await?;
        out_data.insert("app_secret".to_string(), json!(app_secret_data));
    }
    if param.notify_secret {
        let notify_data = req_dao
            .web_dao
            .web_app
            .app_dao
            .app
            .view_notify_secret(&app, auth_data.user_id(), Some(&req_dao.req_env))
            .await?;
        out_data.insert(
            "notify_secret".to_string(),
            json!({
                "secret":notify_data.secret_data,
                "timeout":notify_data.time_out
            }),
        );
    }
    if param.oauth_secret {
        let secret_data = req_dao
            .web_dao
            .web_app
            .app_dao
            .oauth_client
            .oauth_view_secret(&app, auth_data.user_id(), Some(&req_dao.req_env))
            .await?;
        out_data.insert("oauth_secret".to_string(), json!(secret_data));
    }
    Ok(JsonResponse::data(JsonData::body(Value::Object(out_data))))
}

#[derive(Deserialize)]
pub struct RequestListParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub request_type: Option<i8>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

#[derive(Serialize)]
pub struct ShowRequestRecord {
    pub id: u64,
    pub app_id: u64,
    pub status: i8,
    pub parent_app_id: u64,
    pub parent_app_name: String,
    pub parent_app_client_id: String,
    pub parent_app_user_id: u64,
    pub app_name: String,
    pub app_client: String,
    pub app_status: i8,
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

//指定APP的请求功能列表
pub async fn request_list(
    param: &RequestListParam,
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
            &CheckUserAppView {
                res_user_id: app.user_id,
            },
        )
        .await?;

    let status = if let Some(e) = param.status {
        Some(match AppRequestStatus::try_from(e) {
            Ok(ts) => ts,
            Err(err) => return Err(err.into()),
        })
    } else {
        None
    };
    let req_type = if let Some(e) = param.request_type {
        Some(match AppRequestType::try_from(e) {
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
            Some(auth_data.user_id()),
            Some(app.id),
            Some(app.parent_app_id),
            status,
            req_type,
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let out = appdata
            .into_iter()
            .map(|e| {
                let mut feature_data = None;
                let mut oauth_client_data = None;
                let mut change_data = None;
                match e.2 {
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
                    parent_app_id: e.1.parent_app_id,
                    parent_app_name: e.1.parent_app_name,
                    parent_app_client_id: e.1.parent_app_client_id,
                    parent_app_user_id: e.1.parent_app_user_id,
                    app_name: e.1.name,
                    app_client: e.1.client_id,
                    app_status: e.1.status,
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
                .app_request_count(  Some(auth_data.user_id()),Some(app.id), Some(app.parent_app_id), status, req_type)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": bind_vec_user_info_from_req!(
            req_dao,
            out,
            [confirm_user_id:"confirm_user_data"],
            true
        ),
        "total":count
    }))))
}

#[derive(Deserialize)]
pub struct SubRequestListParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_option_u64")]
    pub sub_app_id: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub request_type: Option<i8>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

#[derive(Serialize)]
pub struct ShowSubRequestRecord {
    pub id: u64,
    pub app_id: u64,
    pub status: i8,
    pub app_name: String,
    pub app_client: String,
    pub app_status: i8,
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

//指定APP的被请求功能列表
pub async fn sub_request_list(
    param: &SubRequestListParam,
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
            &CheckUserAppView {
                res_user_id: app.user_id,
            },
        )
        .await?;
    req_dao
        .web_dao
        .web_app
        .app_dao
        .app
        .inner_feature_sub_app_check(&app)
        .await?;
    let status = if let Some(e) = param.status {
        Some(match AppRequestStatus::try_from(e) {
            Ok(ts) => ts,
            Err(err) => return Err(err.into()),
        })
    } else {
        None
    };
    let req_type = if let Some(e) = param.request_type {
        Some(match AppRequestType::try_from(e) {
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
            None,
            param.sub_app_id,
            Some(app.id),
            status,
            req_type,
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let out = appdata
            .into_iter()
            .map(|e| {
                let mut feature_data = None;
                let mut oauth_client_data = None;
                let mut change_data = None;
                match e.2 {
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

                ShowSubRequestRecord {
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
                    app_name: e.1.name,
                    app_client: e.1.client_id,
                    app_status: e.1.status,
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
                .app_request_count( None,param.sub_app_id, Some(app.id), status, req_type)
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": bind_vec_user_info_from_req!(
            req_dao,
            out,
            request_user_id
        ),
        "total":count
    }))))
}
