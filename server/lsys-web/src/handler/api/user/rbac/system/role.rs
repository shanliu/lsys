use crate::common::{JsonData, UserAuthQueryDao};
use crate::common::{JsonResult, PageParam};
use crate::dao::access::api::user::{CheckUserRbacEdit, CheckUserRbacView};
use lsys_access::dao::AccessSession;
use lsys_rbac::dao::{RbacRoleAddData, RbacRoleUserRangeData};
use lsys_rbac::{
    dao::RoleDataParam,
    model::{RbacRoleResRange, RbacRoleUserModel, RbacRoleUserRange},
};
use serde::{Deserialize, Serialize};
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct SystemRoleAddParam {
    pub role_key: String,
    pub role_name: String,
    pub user_range: i8,
    pub res_range: i8,
}

pub async fn system_role_add(
    param: &SystemRoleAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserRbacEdit {
                res_user_id: auth_data.user_id(),
            },
        )
        .await?;

    let role_info = match RbacRoleUserRange::try_from(param.user_range)? {
        RbacRoleUserRange::Custom => RbacRoleUserRangeData::Custom {
            role_name: &param.role_name,
        },
        RbacRoleUserRange::Session => RbacRoleUserRangeData::Session {
            role_key: &param.role_key,
            role_name: if param.role_name.is_empty() {
                None
            } else {
                Some(&param.role_name)
            },
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
    Ok(JsonData::data(json!({ "id": id.id })))
}

#[derive(Debug, Deserialize)]
pub struct SystemRoleEditParam {
    pub role_id: u64,
    pub role_key: String,
    pub role_name: String,
}

pub async fn system_role_edit(
    param: &SystemRoleEditParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
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
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserRbacEdit {
                res_user_id: role.user_id,
            },
        )
        .await?;

    let role_data = match RbacRoleUserRange::try_from(role.user_range)? {
        RbacRoleUserRange::Custom => RbacRoleUserRangeData::Custom {
            role_name: &param.role_name,
        },
        RbacRoleUserRange::Session => RbacRoleUserRangeData::Session {
            role_key: &param.role_key,
            role_name: if param.role_name.is_empty() {
                None
            } else {
                Some(&param.role_name)
            },
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
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct SystemRoleDelParam {
    pub role_id: u64,
}

pub async fn system_role_del(
    param: &SystemRoleDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
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
            &req_dao.req_env,
            Some(&auth_data),
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
    Ok(JsonData::default())
}

#[derive(Debug, Deserialize)]
pub struct SystemRoleDataParam {
    pub role_key: Option<String>,
    pub role_name: Option<String>,
    pub ids: Option<Vec<u64>>,
    pub user_range: Option<i8>,
    pub user_count: Option<bool>,
    pub user_data: Option<u64>,
    pub res_range: Option<i8>,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
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
}

pub async fn system_role_data(
    param: &SystemRoleDataParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(
            &req_dao.req_env,
            Some(&auth_data),
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

    let mut role_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .role_data(
            &RoleDataParam {
                user_id: auth_data.user_id(),
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
        .map(|e| SystemRoleDataRecord {
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
    Ok(JsonData::data(json!({ "data": role_data,"total":count})))
}
