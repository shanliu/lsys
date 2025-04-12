use crate::common::JsonData;
use crate::common::{JsonResponse, LimitParam, UserAuthQueryDao};
use crate::common::{JsonResult, PageParam};
use crate::dao::access::api::system::CheckAdminRbacEdit;
use crate::dao::access::api::user::{CheckUserRbacEdit, CheckUserRbacView};
use lsys_access::dao::{AccessSession, UserDataParam, UserInfo};
use lsys_rbac::dao::RoleAddUser;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct RbacRoleUserItemData {
    pub user_id: u64,
    pub timeout: u64,
}
#[derive(Debug, Deserialize)]
pub struct SystemRoleUserAddParam {
    pub role_id: u64,
    pub user_data: Vec<RbacRoleUserItemData>,
}

pub async fn system_role_user_add(
    param: &SystemRoleUserAddParam,
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
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct SystemRoleUserDelParam {
    pub role_id: u64,
    pub user_data: Vec<u64>,
}

pub async fn system_role_user_del(
    param: &SystemRoleUserDelParam,
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
        .del_user(
            &role,
            &param.user_data,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct SystemRoleUserDataParam {
    pub role_id: u64,
    pub all: bool,
    pub page: Option<PageParam>,
    pub count_num: Option<bool>,
}

pub async fn system_role_user_data(
    param: &SystemRoleUserDataParam,
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
            &req_dao.req_env,
            Some(&auth_data),
            &CheckUserRbacView {
                res_user_id: role.user_id,
            },
        )
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
    Ok(JsonResponse::data(JsonData::body(json!({
        "data":bind_vec_user_info_from_req!(
            req_dao,
            res,
            user_id
        ),
        "total":count,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct SystemRoleUserAvailableParam {
    pub user_data: String,
    pub limit: Option<LimitParam>,
    pub count_num: Option<bool>,
}

pub async fn system_role_user_available(
    param: &SystemRoleUserAvailableParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacEdit {})
        .await?;
    let user_param = UserDataParam {
        app_id: Some(0),
        user_data: None,
        user_account: None,
        user_any: Some(param.user_data.as_str()),
    };
    let (res, next) = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .user_data(&user_param, param.limit.as_ref().map(|e| e.into()).as_ref())
        .await?;
    let count = if param.count_num.unwrap_or(false) {
        Some(
            req_dao
                .web_dao
                .web_access
                .access_dao
                .user
                .user_count(&user_param)
                .await?,
        )
    } else {
        None
    };
    let out_res = res
        .into_iter()
        .map(|e| UserInfo::from(e).to_public())
        .collect::<Vec<_>>();
    Ok(JsonResponse::data(JsonData::body(json!({
        "data":out_res,
        "next":next,
        "total":count
    }))))
}
