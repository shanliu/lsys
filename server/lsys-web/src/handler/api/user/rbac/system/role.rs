use crate::common::JsonData;
use crate::common::{JsonResponse, UserAuthQueryDao};
use crate::common::{JsonResult, PageParam};
use crate::dao::access::api::system::user::{CheckUserRbacEdit, CheckUserRbacView};
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_rbac::dao::{RbacRoleAddData, RbacRoleUserRangeData, RoleDataAttrParam};
use lsys_rbac::{
    dao::RoleDataParam,
    model::{RbacRoleResRange, RbacRoleUserModel, RbacRoleUserRange},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct SystemRoleAddParam {
    pub role_key: String,
    pub role_name: Option<String>,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub user_range: i8,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub res_range: i8,
}

pub async fn system_role_add(
    param: &SystemRoleAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserRbacEdit {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let role_info = match RbacRoleUserRange::try_from(param.user_range)? {
        RbacRoleUserRange::Custom => RbacRoleUserRangeData::Custom {
            role_name: param.role_name.as_deref().unwrap_or_default(),
        },
        RbacRoleUserRange::Session => RbacRoleUserRangeData::Session {
            role_key: &param.role_key,
            role_name: param.role_name.as_deref().and_then(|e| {
                if e.is_empty() {
                    None
                } else {
                    Some(e)
                }
            }),
        },
    };
    let res_range = RbacRoleResRange::try_from(param.res_range)?;
    let id = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .add_role(
            &RbacRoleAddData {
                user_id: auth_data.user_id(),
                app_id: Some(0),
                role_info,
                res_range,
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({ "id": id.id }))))
}

#[derive(Debug, Deserialize)]
pub struct SystemRoleEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    pub role_key: String,
    pub role_name: Option<String>,
}

pub async fn system_role_edit(
    param: &SystemRoleEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserRbacEdit {
                res_user_id: role.user_id,
            },
        )
        .await?;

    let role_data = match RbacRoleUserRange::try_from(role.user_range)? {
        RbacRoleUserRange::Custom => RbacRoleUserRangeData::Custom {
            role_name: param.role_name.as_deref().unwrap_or_default(),
        },
        RbacRoleUserRange::Session => RbacRoleUserRangeData::Session {
            role_key: &param.role_key,
            role_name: param.role_name.as_deref().and_then(|e| {
                if !e.is_empty() {
                    Some(e)
                } else {
                    None
                }
            }),
        },
    };

    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .edit_role(
            &role,
            &role_data,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct SystemRoleDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
}

pub async fn system_role_del(
    param: &SystemRoleDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserRbacEdit {
                res_user_id: role.user_id,
            },
        )
        .await?;

    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .del_role(&role, auth_data.user_id(), None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct SystemRoleDataParam {
    pub role_key: Option<String>,
    pub role_name: Option<String>,
    #[serde(
        default,
        deserialize_with = "crate::common::deserialize_option_vec_u64"
    )]
    pub ids: Option<Vec<u64>>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub user_range: Option<i8>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub user_count: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub user_data: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub res_range: Option<i8>,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub res_count: Option<bool>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub res_op_count: Option<bool>,
}

#[derive(Serialize)]
pub struct SystemRoleDataRecord {
    pub id: u64,
    pub user_id: u64,
    pub role_key: String,
    pub user_range: i8,
    pub res_range: i8,
    pub role_name: String,
    pub status: i8,
    pub change_user_id: u64,
    pub change_time: u64,
    pub user_count: Option<i64>,
    pub user_data: Option<Vec<RbacRoleUserModel>>,
    pub res_count: Option<i64>,
    pub res_op_count: Option<i64>,
}

pub async fn system_role_data(
    param: &SystemRoleDataParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserRbacView {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let user_range = if let Some(e) = param.user_range {
        Some(RbacRoleUserRange::try_from(e)?)
    } else {
        None
    };
    let res_range = if let Some(e) = param.res_range {
        Some(RbacRoleResRange::try_from(e)?)
    } else {
        None
    };

    let role_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .role_info(
            &RoleDataParam {
                user_id: auth_data.user_id(),
                app_id: Some(0),
                ids: param.ids.as_deref(),
                user_range,
                res_range,
                role_key: param.role_key.as_deref(),
                role_name: param.role_name.as_deref(),
            },
            &RoleDataAttrParam {
                user_count: param.user_count,
                user_data: param.user_data,
                res_count: param.res_count,
                res_op_count: param.res_op_count,
            },
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?
        .into_iter()
        .map(|(e, info)| SystemRoleDataRecord {
            id: e.id,
            user_id: e.user_id,
            role_key: e.role_key,
            user_range: e.user_range,
            res_range: e.res_range,
            role_name: e.role_name,
            status: e.status,
            change_user_id: e.change_user_id,
            change_time: e.change_time,
            user_count: info.user_count,
            user_data: info.user_data,
            res_count: info.res_count,
            res_op_count: info.res_op_count,
        })
        .collect::<Vec<_>>();

    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .role
                .role_count(&RoleDataParam {
                    user_id: auth_data.user_id(),
                    app_id: Some(0),
                    ids: param.ids.as_deref(),
                    user_range,
                    res_range,
                    role_key: param.role_key.as_deref(),
                    role_name: param.role_name.as_deref(),
                })
                .await?,
        )
    } else {
        None
    };
    Ok(JsonResponse::data(JsonData::body(
        json!({ "data": role_data,"total":count}),
    )))
}
