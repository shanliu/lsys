use super::{inner_app_rbac_check, inner_app_self_check, inner_user_data_to_user_id};
use crate::common::JsonData;
use crate::common::{JsonResponse, JsonResult, PageParam, RequestDao};
use lsys_app::model::AppModel;
use lsys_rbac::dao::{RbacResAddData, RbacResData, ResDataAttrParam, ResDataParam};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ResAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub res_name: Option<String>,
    pub res_type: String,
    pub res_data: String,
}
//资源添加
pub async fn res_add(
    param: &ResAddParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let target_user_id = inner_user_data_to_user_id(
        app,
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
                user_id: target_user_id,
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
            app.user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({ "id": id }))))
}

#[derive(Debug, Deserialize)]
pub struct ResEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub res_id: u64,
    pub res_name: Option<String>,
    pub res_type: String,
    pub res_data: String,
}
//资源编辑
pub async fn res_edit(
    param: &ResEditParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;

    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_id(&param.res_id)
        .await?;
    inner_app_self_check(app, res.app_id)?;
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
            app.user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ResDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub res_id: u64,
}
//资源删除
pub async fn res_del(
    param: &ResDelParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;

    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_id(&param.res_id)
        .await?;
    inner_app_self_check(app, res.app_id)?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .del_res(&res, app.user_id, None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct ResParam {
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub res_type: Option<String>,
    pub res_data: Option<String>,
    pub res_name: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::common::deserialize_option_vec_u64"
    )]
    pub ids: Option<Vec<u64>>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub op_count: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub perm_count: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct RbacResRecord {
    pub id: u64,
    pub user_id: u64,
    pub res_type: String,
    pub res_data: String,
    pub res_name: String,
    pub change_time: u64,
    pub op_count: i64,
    pub perm_count: i64,
}

//资源列表
pub async fn res_data(
    param: &ResParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let target_user_id = inner_user_data_to_user_id(
        app,
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
        .res_info(
            &ResDataParam {
                user_id: Some(target_user_id),
                app_id: Some(app.id),
                res_data: param.res_data.as_deref(),
                res_type: param.res_type.as_deref(),
                res_name: param.res_name.as_deref(),
                ids: param.ids.as_deref(),
            },
            &ResDataAttrParam {
                op_count: param.op_count.unwrap_or(true),
                perm_count: param.perm_count.unwrap_or(true),
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
                    user_id: Some(target_user_id),
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
    let res = res
        .into_iter()
        .map(|(e, info)| RbacResRecord {
            id: e.id,
            user_id: e.user_id,
            res_type: e.res_type,
            res_data: e.res_data,
            res_name: e.res_name,
            change_time: e.change_time,
            op_count: info.op_count,
            perm_count: info.perm_count,
        })
        .collect::<Vec<_>>();
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": bind_vec_user_info_from_req!(req_dao, res, user_id),
        "total":count
    }))))
}
