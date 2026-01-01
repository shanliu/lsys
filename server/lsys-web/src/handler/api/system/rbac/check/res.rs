use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonResponse, JsonResult, PageParam, UserAuthQueryDao},
    dao::access::api::system::admin::CheckAdminRbacView,
};
use lsys_access::dao::AccessSession;
use lsys_rbac::{dao::AccessSessionRole, model::RbacRoleResRange};
use serde::Deserialize;
use serde_json::json;
#[derive(Debug, Deserialize)]
pub struct ResUserFromUserParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub access_user_id: u64,
    pub page: Option<PageParam>,
}
use crate::common::JsonData;
//1 得到用户列表
pub async fn check_res_user_from_user(
    param: &ResUserFromUserParam,
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
    let mut user_ids = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_user_list_from_user(
            param.access_user_id,
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let is_system = user_ids.contains(&0);
    user_ids.retain(|x| *x != 0);
    let user_data = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_users_by_ids(&user_ids)
        .await?
        .into_array();
    let count = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_user_count_from_user(param.access_user_id)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": user_data,
        "is_system": is_system,
        "total": count,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct ResInfoFromUserParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub access_user_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_user_id: u64, //0 系统
}

//2 根据用户查找最近授权详细
pub async fn check_res_info_from_user(
    param: &ResInfoFromUserParam,
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
    let res_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_data_from_custom_user(param.role_user_id, param.access_user_id)
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": res_data
    }))))
}

#[derive(Debug, Deserialize)]
pub struct ResListFromUserParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub access_user_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_user_id: u64,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub role_app_id: Option<u64>,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub user_range: i8,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub res_range: i8,
    pub page: Option<PageParam>,
}

//3 如果配置关系,查询具体的配置授权
pub async fn check_res_list_from_user(
    param: &ResListFromUserParam,
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
    let res_range = RbacRoleResRange::try_from(param.res_range)?;
    let perm_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_list_from_custom_user(
            param.access_user_id,
            param.role_user_id,
            param.role_app_id,
            res_range,
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_count_from_custom_user(
            param.access_user_id,
            param.role_user_id,
            param.role_app_id,
            res_range,
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": perm_data,
        "total": count,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct ResListFromSessionParam {
    pub role_key: String,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub user_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub page: Option<PageParam>,
}

//3 如果是会话角色,根据会话角色查询该会话角色的授权资源
pub async fn check_res_info_from_session(
    param: &ResListFromSessionParam,
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
    let rs = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_range_from_session_role(&AccessSessionRole {
            role_key: &param.role_key,
            user_id: param.user_id,
            app_id: param.app_id,
        })
        .await?;
    let mut all_res = false;
    let mut perm_data = vec![];
    let mut count = 0;
    match rs {
        ref d @ (RbacRoleResRange::Include | RbacRoleResRange::Exclude) => {
            perm_data = req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .access
                .find_res_list_from_session_role(
                    &AccessSessionRole {
                        role_key: &param.role_key,
                        user_id: param.user_id,
                        app_id: param.app_id,
                    },
                    *d,
                    param.page.as_ref().map(|e| e.into()).as_ref(),
                )
                .await?;
            count = req_dao
                .web_dao
                .web_rbac
                .rbac_dao
                .access
                .find_res_count_from_session_role(
                    &AccessSessionRole {
                        role_key: &param.role_key,
                        user_id: param.user_id,
                        app_id: param.app_id,
                    },
                    *d,
                )
                .await?;
        }
        RbacRoleResRange::Any => {
            all_res = true;
        }
    }

    Ok(JsonResponse::data(JsonData::body(json!({
        "data": perm_data,
        "all_res": all_res,
        "total": count,
    }))))
}
