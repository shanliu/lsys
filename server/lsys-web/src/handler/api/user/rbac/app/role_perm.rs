use super::app_check_get;
use crate::common::JsonData;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use crate::common::{JsonError, PageParam};
use lsys_access::dao::AccessSession;

use lsys_core::fluent_message;
use lsys_rbac::dao::RolePerm;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct AppRolePermItemParam {
    pub op_id: u64,
    pub res_id: u64,
}

#[derive(Debug, Deserialize)]
pub struct AppRolePermAddParam {
    pub role_id: u64,
    pub perm_data: Vec<AppRolePermItemParam>,
}

pub async fn app_role_perm_add(
    param: &AppRolePermAddParam,
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
    let app = app_check_get(role.app_id, true, &auth_data, req_dao).await?;
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
        let op = if let Some(op) =
            op_data
                .get(&pr.op_id)
                .and_then(|e| if e.app_id != app.id { Some(e) } else { None })
        {
            op
        } else {
            return Err(JsonError::Message(fluent_message!(
                "role-prem-bad-op",{
                    "op_id":pr.op_id
                }
            )));
        };
        let res = if let Some(res) =
            res_data
                .get(&pr.res_id)
                .and_then(|e| if e.app_id != app.id { Some(e) } else { None })
        {
            res
        } else {
            return Err(JsonError::Message(fluent_message!(
                "role-prem-bad-res",{
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
        .add_perm(&role, &param_data, app.id, None, Some(&req_dao.req_env))
        .await?;
    Ok(JsonData::default())
}
#[derive(Debug, Deserialize)]
pub struct AppRolePermDelParam {
    pub role_id: u64,
    pub perm_data: Vec<AppRolePermItemParam>,
}

pub async fn app_role_perm_del(
    param: &AppRolePermDelParam,
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
    let app = app_check_get(role.app_id, true, &auth_data, req_dao).await?;
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
        let op = if let Some(op) =
            op_data
                .get(&pr.op_id)
                .and_then(|e| if e.app_id != app.id { Some(e) } else { None })
        {
            op
        } else {
            return Err(JsonError::Message(fluent_message!(
                "role-prem-bad-op",{
                    "op_id":pr.op_id
                }
            )));
        };
        let res = if let Some(op) = res_data.get(&pr.res_id) {
            op
        } else {
            return Err(JsonError::Message(fluent_message!(
                "role-prem-bad-res",{
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
    Ok(JsonData::default())
}
#[derive(Debug, Deserialize)]
pub struct AppRolePermDataParam {
    pub role_id: u64,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn app_role_perm_data(
    param: &AppRolePermDataParam,
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
    app_check_get(role.app_id, false, &auth_data, req_dao).await?;
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
    Ok(JsonData::data(json!({
        "data": res,
        "count": count
    })))
}
