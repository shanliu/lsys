use super::{app_check_get, inner_user_data_to_user_id};
use crate::common::JsonData;
use crate::common::{JsonResponse, JsonResult, PageParam, UserAuthQueryDao};
use lsys_access::dao::AccessSession;
use lsys_rbac::dao::{RbacResAddData, RbacResData, ResDataParam};
use serde::Deserialize;
use serde_json::json;

//用户后台对APP的RBAC资源管理
#[derive(Debug, Deserialize)]
pub struct AppResAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub res_name: Option<String>,
    pub res_type: String,
    pub res_data: String,
}

pub async fn app_res_add(
    param: &AppResAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = app_check_get(param.app_id, true, &auth_data, req_dao).await?;
    let user_id = inner_user_data_to_user_id(
        &app,
        param.use_app_user,
        param.user_param.as_deref(),
        req_dao,
    )
    .await?;
    let id = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .add_res(
            &RbacResAddData {
                user_id,
                app_id: Some(app.id),
                res_info: RbacResData {
                    res_name: param.res_name.as_deref().and_then(|e| {
                        if !e.is_empty() {
                            Some(e)
                        } else {
                            None
                        }
                    }),
                    res_type: &param.res_type,
                    res_data: &param.res_data,
                },
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": id }))))
}

#[derive(Debug, Deserialize)]
pub struct AppResEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub res_id: u64,
    pub res_name: Option<String>,
    pub res_type: String,
    pub res_data: String,
}

pub async fn app_res_edit(
    param: &AppResEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_id(&param.res_id)
        .await?;
    app_check_get(res.app_id, true, &auth_data, req_dao).await?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .edit_res(
            &res,
            &RbacResData {
                res_name: param.res_name.as_deref().and_then(|e| {
                    if !e.is_empty() {
                        Some(e)
                    } else {
                        None
                    }
                }),
                res_type: &param.res_type,
                res_data: &param.res_data,
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct AppResDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub res_id: u64,
}

pub async fn app_res_del(
    param: &AppResDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_id(&param.res_id)
        .await?;
    app_check_get(res.app_id, true, &auth_data, req_dao).await?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .del_res(&res, auth_data.user_id(), None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}
#[derive(Debug, Deserialize)]
pub struct AppResParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub res_type: Option<String>,
    pub res_data: Option<String>,
    pub res_name: Option<String>,
    pub ids: Option<Vec<u64>>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn app_res_data(
    param: &AppResParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;
    let user_id = inner_user_data_to_user_id(
        &app,
        param.use_app_user,
        param.user_param.as_deref(),
        req_dao,
    )
    .await?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_data(
            &ResDataParam {
                user_id: Some(user_id),
                app_id: Some(app.id),
                res_data: param.res_data.as_deref(),
                res_type: param.res_type.as_deref(),
                res_name: param.res_name.as_deref(),
                ids: param.ids.as_deref(),
            },
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .res
                .res_count(&ResDataParam {
                    user_id: Some(user_id),
                    app_id: Some(app.id),
                    res_data: param.res_data.as_deref(),
                    res_type: param.res_type.as_deref(),
                    res_name: param.res_name.as_deref(),
                    ids: param.ids.as_deref(),
                })
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": res, "count": count }),
    )))
}
