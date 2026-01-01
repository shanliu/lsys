use super::{app_check_get, inner_user_data_to_user_id};
use crate::common::JsonData;
use crate::common::{JsonResponse, JsonResult, PageParam, UserAuthQueryDao};
use lsys_access::dao::AccessSession;
use lsys_rbac::dao::{OpDataAttrParam, OpDataParam, RbacOpAddData, RbacOpData};
use serde::{Deserialize, Serialize};
use serde_json::json;

//用户后台对APP的RBAC操作管理
#[derive(Debug, Deserialize)]
pub struct AppOpAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub op_key: String,
    pub op_name: Option<String>,
}
pub async fn app_op_add(
    param: &AppOpAddParam,
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
        .op
        .add_op(
            &RbacOpAddData {
                user_id,
                app_id: Some(app.id),
                op_info: RbacOpData {
                    op_key: &param.op_key,
                    op_name: param.op_name.as_deref().and_then(|e| {
                        if !e.is_empty() {
                            Some(e)
                        } else {
                            None
                        }
                    }),
                },
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({"id": id}))))
}
#[derive(Debug, Deserialize)]
pub struct AppOpEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub op_id: u64,
    pub op_key: String,
    pub op_name: Option<String>,
}

pub async fn app_op_edit(
    param: &AppOpEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let op = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_id(&param.op_id)
        .await?;
    app_check_get(op.app_id, true, &auth_data, req_dao).await?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .edit_op(
            &op,
            &RbacOpData {
                op_key: &param.op_key,
                op_name: param.op_name.as_deref().and_then(|e| {
                    if !e.is_empty() {
                        Some(e)
                    } else {
                        None
                    }
                }),
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct AppOpDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub op_id: u64,
}

pub async fn app_op_del(
    param: &AppOpDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let op = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_id(&param.op_id)
        .await?;
    app_check_get(op.app_id, true, &auth_data, req_dao).await?;

    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .del_op(&op, auth_data.user_id(), None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}
#[derive(Debug, Deserialize)]
pub struct AppOpDataParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub use_app_user: bool,
    pub user_param: Option<String>, //use_app_user为假时必填,用户标识
    pub op_name: Option<String>,
    pub op_key: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::common::deserialize_option_vec_u64"
    )]
    pub ids: Option<Vec<u64>>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub res_type_count: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub check_role_use: Option<bool>,
}

#[derive(Debug, Serialize)]
pub struct RbacOpRecord {
    pub id: u64,
    pub app_id: u64,
    pub op_key: String,
    pub op_name: String,
    pub change_user_id: u64,
    pub change_time: u64,
    pub res_type_count: i64,
    pub is_role_use: bool,
}
pub async fn app_op_data(
    param: &AppOpDataParam,
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

    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .op_info(
            &OpDataParam {
                user_id,
                app_id: Some(app.id),
                op_name: param.op_name.as_deref(),
                op_key: param.op_key.as_deref(),
                ids: param.ids.as_deref(),
            },
            &OpDataAttrParam {
                res_type_count: param.res_type_count.unwrap_or_default(),
                check_role_use: param.check_role_use.unwrap_or_default(),
            },
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?
        .into_iter()
        .map(|(e, i)| RbacOpRecord {
            id: e.id,
            app_id: e.app_id,
            op_key: e.op_key,
            op_name: e.op_name,
            change_user_id: e.change_user_id,
            change_time: e.change_time,
            res_type_count: i.res_type_count,
            is_role_use: i.is_role_use,
        })
        .collect::<Vec<RbacOpRecord>>();
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .op
                .op_count(&OpDataParam {
                    user_id,
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
    Ok(JsonResponse::data(JsonData::body(
        json!({"data":res, "count": count}),
    )))
}
