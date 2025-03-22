use super::inner_app_rbac_check;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::common::{JsonError, PageParam};
use lsys_access::dao::AccessSession;
use lsys_app::model::AppModel;
use lsys_core::fluent_message;
use lsys_rbac::dao::RbacRoleAddData;
use lsys_rbac::dao::RbacRoleUserRangeData;
use lsys_rbac::{
    dao::{RoleAddUser, RoleDataParam as DaoRoleDataParam, RolePerm, RolePermData},
    model::{RbacRoleModel, RbacRoleResRange, RbacRoleUserModel, RbacRoleUserRange},
};

pub struct RoleAddParam {
    pub role_key: String,
    pub role_name: String,
    pub user_range: i8,
    pub res_range: i8,
}

pub async fn role_add(
    param: &RoleAddParam,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<RbacRoleModel> {
    inner_app_rbac_check(app, req_dao).await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

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
            app.user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(id)
}

pub struct RoleEditParam {
    pub role_id: u64,
    pub role_key: String,
    pub role_name: String,
}

pub async fn role_edit(
    param: &RoleEditParam,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<()> {
    inner_app_rbac_check(app, req_dao).await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
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
    Ok(())
}

pub struct RoleDelData {
    pub res_id: u64,
}

pub async fn role_del(
    param: &RoleDelData,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<()> {
    inner_app_rbac_check(app, req_dao).await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.res_id)
        .await?;
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .del_role(&role, auth_data.user_id(), None, Some(&req_dao.req_env))
        .await?;
    Ok(())
}

pub struct RoleDataParam {
    pub user_id: u64,
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

pub struct RoleDataRecord {
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
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(Vec<RoleDataRecord>, Option<i64>)> {
    inner_app_rbac_check(app, req_dao).await?;
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
                user_id: param.user_id,
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
        .map(|e| RoleDataRecord {
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
                    user_id: param.user_id,
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
    Ok((role_data, count))
}

pub struct RolePermItemData {
    pub op_id: u64,
    pub res_id: u64,
}

pub struct RolePermAddParam {
    pub role_id: u64,
    pub perm_data: Vec<RolePermItemData>,
}

pub async fn role_perm_add(
    param: &RolePermAddParam,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<()> {
    inner_app_rbac_check(app, req_dao).await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;
    let op_id = param.perm_data.iter().map(|e| e.op_id).collect::<Vec<_>>();
    let res_id = param.perm_data.iter().map(|e| e.op_id).collect::<Vec<_>>();
    let op_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_ids(&op_id)
        .await?;
    let res_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_ids(&res_id)
        .await?;

    let mut param_data = Vec::with_capacity(param.perm_data.len());
    for pr in param.perm_data.iter() {
        let op = if let Some(op) = op_data.get(&pr.op_id) {
            op
        } else {
            return Err(JsonError::Message(fluent_message!(
                "role_prem_bad_op",{
                    "op_id":pr.op_id
                }
            )));
        };
        let res = if let Some(op) = res_data.get(&pr.res_id) {
            op
        } else {
            return Err(JsonError::Message(fluent_message!(
                "role_prem_bad_res",{
                    "res_id":pr.res_id
                }
            )));
        };
        param_data.push(RolePerm { op, res });
    }
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .add_perm(
            &role,
            &param_data,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(())
}

pub struct RolePermDelParam {
    pub role_id: u64,
    pub perm_data: Vec<RolePermItemData>,
}

pub async fn role_perm_del(
    param: &RolePermDelParam,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<()> {
    inner_app_rbac_check(app, req_dao).await?;
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;
    let op_id = param.perm_data.iter().map(|e| e.op_id).collect::<Vec<_>>();
    let res_id = param.perm_data.iter().map(|e| e.op_id).collect::<Vec<_>>();
    let op_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .op
        .find_by_ids(&op_id)
        .await?;
    let res_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .res
        .find_by_ids(&res_id)
        .await?;

    let mut param_data = Vec::with_capacity(param.perm_data.len());
    for pr in param.perm_data.iter() {
        let op = if let Some(op) = op_data.get(&pr.op_id) {
            op
        } else {
            return Err(JsonError::Message(fluent_message!(
                "role_prem_bad_op",{
                    "op_id":pr.op_id
                }
            )));
        };
        let res = if let Some(op) = res_data.get(&pr.res_id) {
            op
        } else {
            return Err(JsonError::Message(fluent_message!(
                "role_prem_bad_res",{
                    "res_id":pr.res_id
                }
            )));
        };
        param_data.push(RolePerm { op, res });
    }
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .del_perm(
            &role,
            &param_data,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(())
}

pub struct RolePermParam {
    pub role_id: u64,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn role_perm_data(
    param: &RolePermParam,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(Vec<RolePermData>, Option<i64>)> {
    inner_app_rbac_check(app, req_dao).await?;
    //let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .role_perm_data(&role, param.page.as_ref().map(|e| e.into()).as_ref())
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .role
                .role_perm_count(&role)
                .await?,
        )
    } else {
        None
    };
    Ok((res, count))
}

pub struct RoleUserItemData {
    pub user_id: u64,
    pub timeout: u64,
}
pub struct RoleUserAddParam {
    pub role_id: u64,
    pub user_data: Vec<RoleUserItemData>,
}

pub async fn role_user_add(
    param: &RoleUserAddParam,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<()> {
    inner_app_rbac_check(app, req_dao).await?;
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
        .rbac_dao
        .role
        .add_user(
            &role,
            &param
                .user_data
                .iter()
                .map(|e| RoleAddUser {
                    user_id: e.user_id,
                    timeout: e.timeout,
                })
                .collect::<Vec<_>>(),
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(())
}

pub struct RoleUserDelParam {
    pub role_id: u64,
    pub user_data: Vec<u64>,
}

pub async fn role_user_del(
    param: &RoleUserDelParam,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<()> {
    inner_app_rbac_check(app, req_dao).await?;
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
        .rbac_dao
        .role
        .del_user(
            &role,
            &param.user_data,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(())
}

pub struct RoleUserParam {
    pub role_id: u64,
    pub all: bool,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn role_user_data(
    param: &RoleUserParam,
    app: &AppModel,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<(Vec<RbacRoleUserModel>, Option<i64>)> {
    inner_app_rbac_check(app, req_dao).await?;
    //  let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;
    let res = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .role_user_data(
            &role,
            param.all,
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .role
                .role_user_count(&role, param.all)
                .await?,
        )
    } else {
        None
    };
    Ok((res, count))
}
