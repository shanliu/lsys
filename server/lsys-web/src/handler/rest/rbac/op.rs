use crate::common::{JsonResponse, JsonResult, PageParam, RequestDao};

use crate::common::JsonData;
use lsys_app::model::AppModel;
use lsys_rbac::dao::{OpDataParam as DaoOpDataParam, RbacOpAddData, RbacOpData};
use serde::Deserialize;
use serde_json::json;

use super::{inner_app_rbac_check, inner_app_self_check, inner_user_data_to_user_id};

#[derive(Debug, Deserialize)]
pub struct OpAddParam {
    pub user_param: Option<String>,
    pub op_key: String,
    pub op_name: String,
    pub data: String,
}
pub async fn op_add(
    param: &OpAddParam,
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
        .op
        .add_op(
            &RbacOpAddData {
                user_id: target_user_id,
                app_id: Some(app.id),
                op_info: RbacOpData {
                    op_key: &param.op_key,
                    op_name: if param.op_name.is_empty() {
                        None
                    } else {
                        Some(&param.op_name)
                    },
                },
            },
            app.user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({"id": id}))))
}

#[derive(Debug, Deserialize)]
pub struct OpEditParam {
    pub op_id: u64,
    pub op_key: String,
    pub op_name: String,
    pub data: String,
}

pub async fn op_edit(
    param: &OpEditParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let op = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_id(&param.op_id)
        .await?;
    inner_app_self_check(app, op.app_id)?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .edit_op(
            &op,
            &RbacOpData {
                op_key: &param.op_key,
                op_name: if param.op_name.is_empty() {
                    None
                } else {
                    Some(&param.op_name)
                },
            },
            app.user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct OpDelParam {
    pub op_id: u64,
}

pub async fn op_del(
    param: &OpDelParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;
    let op = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_id(&param.op_id)
        .await?;
    inner_app_self_check(app, op.app_id)?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .del_op(&op, app.user_id, None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}
#[derive(Debug, Deserialize)]

pub struct OpDataParam {
    pub user_param: Option<String>,
    pub op_name: Option<String>,
    pub op_key: Option<String>,
    pub ids: Option<Vec<u64>>,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn op_data(
    param: &OpDataParam,
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
        .op
        .op_data(
            &DaoOpDataParam {
                user_id: target_user_id,
                app_id: Some(app.id),
                op_name: param.op_name.as_deref(),
                op_key: param.op_key.as_deref(),
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
                .op
                .op_count(&DaoOpDataParam {
                    user_id: app.user_id,
                    app_id: Some(app.id),
                    op_name: param.op_name.as_deref(),
                    op_key: param.op_key.as_deref(),
                    ids: param.ids.as_deref(),
                })
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": res,
        "count": count,
    }))))
}
