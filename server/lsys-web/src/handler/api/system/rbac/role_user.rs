use crate::{
    common::{JsonData, JsonResponse, JsonResult, LimitParam, UserAuthQueryDao},
    dao::access::api::system::{CheckAdminRbacEdit, CheckAdminRbacView},
};
use lsys_access::dao::{AccessSession, UserDataParam, UserInfo};
use serde_json::json;

use crate::common::{JsonError, PageParam};
use lsys_core::fluent_message;
use lsys_rbac::dao::RoleAddUser;
use serde::Deserialize;

#[derive(Debug, Deserialize)]
pub struct RoleUserItemParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub user_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub timeout: u64,
}

#[derive(Debug, Deserialize)]
pub struct RoleUserAddParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    pub user_data: Vec<RoleUserItemParam>,
}

pub async fn role_user_add(
    param: &RoleUserAddParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacEdit {})
        .await?;

    let role = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .find_by_id(&param.role_id)
        .await?;

    let user_data = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_users_by_ids(
            &param
                .user_data
                .iter()
                .map(|e| e.user_id)
                .collect::<Vec<_>>(),
        )
        .await?;
    let mut add_user_data = vec![];
    for user_item in &param.user_data {
        match user_data.get(user_item.user_id) {
            Some(user_model) => {
                if user_model.app_id != 0 {
                    return Err(JsonError::Message(fluent_message!(
                        "role-user-not-system-user",
                        {
                            "user_name": user_model.user_nickname,
                            "user_id": user_model.id,
                            "app_id": user_model.app_id
                        }
                    )));
                }
                add_user_data.push(RoleAddUser {
                    user_id: user_item.user_id,
                    timeout: user_item.timeout,
                });
            }
            None => {
                return Err(JsonError::Message(fluent_message!(
                    "role-user-not-found",
                    {
                        "user_id": user_item.user_id
                    }
                )));
            }
        }
    }
    req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .role
        .add_user(
            &role,
            &add_user_data,
            auth_data.user_id(),
            None,
            Some(&req_dao.req_env),
        )
        .await?;
    Ok(JsonResponse::default())
}

#[derive(Debug, Deserialize)]
pub struct RoleUserDelParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_vec_u64")]
    pub user_data: Vec<u64>,
}

pub async fn role_user_del(
    param: &RoleUserDelParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacEdit {})
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
pub struct RoleUserDataParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub all: bool,
    pub page: Option<PageParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn role_user_data(
    param: &RoleUserDataParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;

    req_dao
        .web_dao
        .web_rbac
        .check(&req_dao.req_env, Some(&auth_data), &CheckAdminRbacView {})
        .await?;

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
    Ok(JsonResponse::data(JsonData::body(json!({
            "data":bind_vec_user_info_from_req!(
                req_dao,
                res,
                user_id
            ),
            "total":count
    }))))
}

#[derive(Debug, Deserialize)]
pub struct RoleUserAvailableParam {
    pub user_data: Option<String>,
    pub limit: Option<LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

pub async fn role_user_available(
    param: &RoleUserAvailableParam,
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
        user_any: param.user_data.as_deref(),
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
    let out_res = res.into_iter().map(UserInfo::from).collect::<Vec<_>>();
    Ok(JsonResponse::data(JsonData::body(json!({
        "data":out_res,
        "next":next,
        "total":count
    }))))
}
