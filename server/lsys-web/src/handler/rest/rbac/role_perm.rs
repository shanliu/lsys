use super::inner_app_rbac_check;
use super::inner_app_self_check;
use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::RequestDao;
use crate::common::{JsonError, PageParam};
use lsys_app::model::AppModel;
use lsys_core::fluent_message;
use lsys_rbac::dao::RolePerm;
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct RolePermItemParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub op_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub res_id: u64,
}
#[derive(Debug, Deserialize)]
pub struct RolePermAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    pub perm_data: Vec<RolePermItemParam>,
}

pub async fn role_perm_add(
    param: &RolePermAddParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;
    inner_app_self_check(app, role.app_id)?;
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
            inner_app_self_check(app, op.app_id)?;
            op
        } else {
            return Err(JsonError::Message(fluent_message!(
                "role-prem-bad-op",{
                    "op_id":pr.op_id
                }
            )));
        };
        let res = if let Some(res) = res_data.get(&pr.res_id) {
            inner_app_self_check(app, res.app_id)?;
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
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct RolePermDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    pub perm_data: Vec<RolePermItemParam>,
}

pub async fn role_perm_del(
    param: &RolePermDelParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;
    inner_app_self_check(app, role.app_id)?;
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
            app.user_id,
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct RolePermParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn role_perm_data(
    param: &RolePermParam,
    app: &AppModel,
    req_dao: &RequestDao,
) -> JsonResult<JsonResponse> {
    inner_app_rbac_check(app, req_dao).await?;

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;
    inner_app_self_check(app, role.app_id)?;
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
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": res,
        "count": count
    }))))
}
