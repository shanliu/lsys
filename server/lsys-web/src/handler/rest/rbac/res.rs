use super::{inner_app_rbac_check, inner_app_self_check, inner_user_data_to_user_id};
use crate::common::JsonData;
use crate::common::{JsonResponse, JsonResult, PageParam, RequestDao};
use lsys_app::model::AppModel;
use lsys_rbac::dao::{RbacResAddData, RbacResData, ResDataParam};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct ResAddParam {
    pub user_param: Option<String>,
    pub res_name: String,
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
    let target_user_id =
        inner_user_data_to_user_id(app, param.user_param.as_deref(), req_dao).await?;

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
                    res_name: if param.res_name.is_empty() {
                        None
                    } else {
                        Some(&param.res_name)
                    },
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
    pub res_id: u64,
    pub res_name: String,
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
                res_name: if param.res_name.is_empty() {
                    None
                } else {
                    Some(&param.res_name)
                },
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
    pub user_param: Option<String>,
    pub res_type: Option<String>,
    pub res_data: Option<String>,
    pub res_name: Option<String>,
    pub ids: Option<Vec<u64>>,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}
//资源列表
pub async fn res_data(
    param: &ResParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let target_user_id =
        inner_user_data_to_user_id(app, param.user_param.as_deref(), req_dao).await?;

    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .res_data(
            &ResDataParam {
                user_id: Some(target_user_id),
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
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": res,"total":count}),
    )))
}
