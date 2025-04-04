use super::app_check_get;
use super::inner_user_data_to_user_id;
use crate::common::JsonData;
use crate::common::JsonResult;
use crate::common::PageParam;
use crate::common::UserAuthQueryDao;
use lsys_access::dao::AccessSession;
use lsys_access::dao::UserInfo;
use lsys_rbac::dao::RbacRoleAddData;
use lsys_rbac::dao::RbacRoleUserRangeData;
use lsys_rbac::{
    dao::RoleDataParam as DaoRoleDataParam,
    model::{RbacRoleResRange, RbacRoleUserRange},
};
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct AppRoleAddParam {
    pub app_id: u64,
    pub user_param: Option<String>,
    pub role_key: String,
    pub role_name: String,
    pub user_range: i8,
    pub res_range: i8,
}

pub async fn app_role_add(
    param: &AppRoleAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = app_check_get(param.app_id, true, &auth_data, req_dao).await?;
    let user_id = inner_user_data_to_user_id(&app, param.user_param.as_deref(), req_dao).await?;

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
                user_id,
                app_id: Some(app.id),
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
pub struct AppRoleEditParam {
    pub role_id: u64,
    pub role_key: String,
    pub role_name: String,
}

pub async fn app_role_edit(
    param: &AppRoleEditParam,
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
    app_check_get(role.app_id, true, &auth_data, req_dao).await?;

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
pub struct AppRoleDelParam {
    pub res_id: u64,
}

pub async fn app_role_del(
    param: &AppRoleDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.res_id)
        .await?;
    app_check_get(role.app_id, true, &auth_data, req_dao).await?;
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
pub struct AppRoleDataParam {
    pub app_id: u64,
    pub user_param: Option<String>,
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

#[derive(Debug, Serialize)]
pub struct AppRoleUserDataRecord {
    pub user_data: Option<UserInfo>,
    pub timeout: u64,
    pub change_time: u64,
}

#[derive(Debug, Serialize)]
pub struct AppRoleDataRecord {
    pub id: u64,
    pub user_id: u64,
    pub role_key: String,
    pub user_range: i8,
    pub res_range: i8,
    pub role_name: String,
    pub change_time: u64,
    pub user_count: Option<i64>,
    pub user_list: Option<Vec<AppRoleUserDataRecord>>,
}

pub async fn app_role_data(
    param: &AppRoleDataParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;
    let user_id = inner_user_data_to_user_id(&app, param.user_param.as_deref(), req_dao).await?;

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
                user_id,
                app_id: Some(app.id),
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
        .map(|e| AppRoleDataRecord {
            id: e.id,
            user_id: e.user_id,
            role_key: e.role_key,
            user_range: e.user_range,
            res_range: e.res_range,
            role_name: e.role_name,
            change_time: e.change_time,
            user_count: None,
            user_list: None,
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

        let user_data_ids = user_data
            .iter()
            .flat_map(|e| e.1.iter().map(|f| f.user_id).collect::<Vec<u64>>())
            .collect::<Vec<_>>();
        let user_info = req_dao
            .web_dao
            .web_access
            .access_dao
            .user
            .cache()
            .find_users_by_ids(&user_data_ids)
            .await?;
        for tmp in role_data.iter_mut() {
            tmp.user_list = user_data.get(&tmp.id).map(|e| {
                e.iter()
                    .map(|f| AppRoleUserDataRecord {
                        timeout: f.timeout,
                        change_time: f.change_time,
                        user_data: user_info.get(&f.user_id).to_owned(),
                    })
                    .collect::<Vec<_>>()
            });
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
                    user_id,
                    app_id: Some(app.id),
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

    Ok(JsonData::data(json!({
        "data": bind_vec_user_info_from_req!(req_dao, role_data, user_id),
        "count": count
    })))
}
