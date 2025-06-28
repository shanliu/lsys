use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao},
    dao::access::api::system::admin::{CheckAdminRbacEdit, CheckAdminRbacView},
};
use lsys_access::dao::AccessSession;
use serde_json::json;

use crate::common::PageParam;

use lsys_rbac::{
    dao::{RbacRoleAddData, RbacRoleUserRangeData, RoleDataParam as DaoRoleDataParam},
    model::{RbacRoleResRange, RbacRoleUserModel, RbacRoleUserRange},
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
pub struct RoleAddParam {
    pub role_key: String,
    pub role_name: Option<String>,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub user_range: i8,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub res_range: i8,
}

pub async fn role_add(
    param: &RoleAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminRbacEdit {},
        )
        .await?;
    let role_info = match RbacRoleUserRange::try_from(param.user_range)? {
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
    let res_range = RbacRoleResRange::try_from(param.res_range)?;
    let id = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .add_role(
            &RbacRoleAddData {
                user_id: 0,
                app_id: Some(0),
                role_info,
                res_range,
            },
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({"id": id.id}))))
}

#[derive(Debug, Deserialize)]
pub struct RoleEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    pub role_key: String,
    pub role_name: Option<String>,
}

pub async fn role_edit(
    param: &RoleEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminRbacEdit {},
        )
        .await?;
    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
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
pub struct RoleDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
}

pub async fn role_del(
    param: &RoleDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminRbacEdit {},
        )
        .await?;
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
        .rbac_dao
        .role
        .del_role(&role, auth_data.user_id(), None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]

pub struct RoleDataParam {
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
}

#[derive(Serialize)]
struct RbacRoleDataRecord {
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
}

pub async fn role_data(
    param: &RoleDataParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminRbacView {},
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

    let mut role_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .role_data(
            &DaoRoleDataParam {
                user_id: 0,
                app_id: Some(0),
                ids: param.ids.as_deref(),
                user_range,
                res_range,
                role_key: param.role_key.as_deref(),
                role_name: param.role_name.as_deref(),
            },
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?
        .into_iter()
        .map(|e| RbacRoleDataRecord {
            id: e.id,
            user_id: e.user_id,
            role_key: e.role_key,
            user_range: e.user_range,
            res_range: e.res_range,
            role_name: e.role_name,
            status: e.status,
            change_user_id: e.change_user_id,
            change_time: e.change_time,
            user_count: None,
            user_data: None,
        })
        .collect::<Vec<_>>();

    if param.user_count.unwrap_or(false) {
        let role_ids = role_data.iter().map(|e| e.id).collect::<Vec<_>>();
        let user_data = req_dao
            .web_dao
            .web_rbac
            .rbac_dao
            .role
            .role_user_group_count(&role_ids, false)
            .await?;
        for tmp in role_data.iter_mut() {
            tmp.user_count = user_data.get(&tmp.id).copied();
        }
    }
    let user_data_limit = param.user_data.unwrap_or(0);
    if user_data_limit > 0 {
        let role_ids = role_data.iter().map(|e| e.id).collect::<Vec<_>>();
        let user_data = req_dao
            .web_dao
            .web_rbac
            .rbac_dao
            .role
            .role_user_group_data(
                &role_ids,
                None,
                false,
                Some(&lsys_core::PageParam::new(0, user_data_limit)),
            )
            .await?;

        for tmp in role_data.iter_mut() {
            tmp.user_data = user_data.get(&tmp.id).map(|e| e.to_owned());
        }
    }

    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .role
                .role_count(&DaoRoleDataParam {
                    user_id: 0,
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
