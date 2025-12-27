use crate::common::{JsonData, JsonResponse, JsonResult, PageParam, UserAuthQueryDao};
use crate::dao::access::api::system::admin::{CheckAdminRbacEdit, CheckAdminRbacView};
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_rbac::dao::{OpDataAttrParam, OpDataParam as DaoOpDataParam, RbacOpAddData, RbacOpData};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct OpAddParam {
    pub op_key: String,
    pub op_name: Option<String>,
}

pub async fn op_add(param: &OpAddParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminRbacEdit {},
        )
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let id = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .add_op(
            &RbacOpAddData {
                user_id: 0,
                app_id: Some(0),
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
    Ok(JsonResponse::data(JsonData::body(json!({ "id": id }))))
}

#[derive(Debug, Deserialize)]
pub struct OpEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub op_id: u64,
    pub op_key: String,
    pub op_name: Option<String>,
}

pub async fn op_edit(param: &OpEditParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminRbacEdit {},
        )
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let op = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_id(&param.op_id)
        .await?;
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
pub struct OpDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub op_id: u64,
}

pub async fn op_del(param: &OpDelParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminRbacEdit {},
        )
        .await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let op = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_id(&param.op_id)
        .await?;
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
pub struct OpDataParam {
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

pub async fn op_data(param: &OpDataParam, req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminRbacView {},
        )
        .await?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .op_info(
            &DaoOpDataParam {
                user_id: 0,
                app_id: None,
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
                .op_count(&DaoOpDataParam {
                    user_id: 0,
                    app_id: None,
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
        json!({ "data":res,"total":count}),
    )))
}
