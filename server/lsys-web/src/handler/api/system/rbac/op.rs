use crate::common::{JsonData, JsonResponse, JsonResult, PageParam, UserAuthQueryDao};
use crate::dao::access::api::system::admin::{CheckAdminRbacEdit, CheckAdminRbacView};
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_rbac::dao::{OpDataParam as DaoOpDataParam, RbacOpAddData, RbacOpData};
use serde::Deserialize;
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
        .op_data(
            &DaoOpDataParam {
                user_id: 0,
                app_id: None,
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
        json!({ "data": res,"total":count}),
    )))
}
