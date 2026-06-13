use crate::common::{JsonData, ToOffsetPageParam};
use crate::common::{JsonResponse, JsonResult, PageParam, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::handler::api::user::app_rbac::{app_check_get, parent_app_check};
use lsys_rbac::{dao::AccessSessionRole, model::RbacRoleResRange};
use serde::Deserialize;
use serde_json::json;
//获取指定用户可访问的资源数据
#[derive(Debug, Deserialize)]
pub struct AppResUserFromUserParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub access_user_param: String,
    pub page: Option<PageParam>,
}
//1 得到用户列表
pub async fn app_res_user_from_user(
    param: &AppResUserFromUserParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = parent_app_check(auth_dao).await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao, web_dao).await?;

    let user_info = web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, &param.access_user_param, None, None)
        .await?;

    let mut user_ids = web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_user_list_from_user(user_info.id, &param.page.to_offset_page_param())
        .await?;
    let is_system = user_ids.contains(&0);
    user_ids.retain(|x| *x != 0);
    let user_data = web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .find_users_by_ids(&user_ids)
        .await?
        .into_array();
    let count = web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_user_count_from_user(user_info.id)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "user_data": user_data,
        "is_system": is_system,
        "count": count,
    }))))
}
#[derive(Debug, Deserialize)]
pub struct AppResInfoFromUserParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub access_user_param: String,
}

//2 根据用户查找最近授权详细
pub async fn app_res_info_from_user(
    param: &AppResInfoFromUserParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = parent_app_check(auth_dao).await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao, web_dao).await?;

    let user_info = web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, &param.access_user_param, None, None)
        .await?;

    let res_data = web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_data_from_custom_user(auth_data.user_id(), user_info.id)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!(res_data))))
}
#[derive(Debug, Deserialize)]
pub struct AppResListFromUserParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub access_user_param: String,
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub role_user_id: u64,
    #[serde(deserialize_with = "crate::common::deserialize_i8")]
    pub res_range: i8,
    pub page: Option<PageParam>,
}

//3 如果配置关系,查询具体的配置授权
pub async fn app_res_list_from_user(
    param: &AppResListFromUserParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = parent_app_check(auth_dao).await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao, web_dao).await?;

    let user_info = web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, &param.access_user_param, None, None)
        .await?;

    let res_range = RbacRoleResRange::try_from(param.res_range)?;
    let perm_data = web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_list_from_custom_user(
            user_info.id,
            param.role_user_id,
            Some(app.id),
            res_range,
            &param.page.to_offset_page_param(),
        )
        .await?;
    let count = web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_count_from_custom_user(user_info.id, param.role_user_id, Some(app.id), res_range)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "perm_data": perm_data,
        "count": count,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct AppResListFromSessionParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub role_key: String,
    pub access_user_param: String,
    pub page: Option<PageParam>,
}
//3 如果是会话角色,根据会话角色查询该会话角色的授权资源
pub async fn app_res_info_from_session(
    param: &AppResListFromSessionParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = parent_app_check(auth_dao).await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao, web_dao).await?;

    let user_info = web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, &param.access_user_param, None, None)
        .await?;

    let rs = web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_range_from_session_role(&AccessSessionRole {
            role_key: &param.role_key,
            user_id: user_info.id,
            app_id: app.id,
        })
        .await?;
    let mut all_res = false;
    let mut perm_data = vec![];
    let mut count = 0;
    match rs {
        ref d @ (RbacRoleResRange::Include | RbacRoleResRange::Exclude) => {
            perm_data = web_dao
                .web_rbac
                .rbac_dao
                .access
                .find_res_list_from_session_role(
                    &AccessSessionRole {
                        role_key: &param.role_key,
                        user_id: user_info.id,
                        app_id: app.id,
                    },
                    *d,
                    &param.page.to_offset_page_param(),
                )
                .await?;
            count = web_dao
                .web_rbac
                .rbac_dao
                .access
                .find_res_count_from_session_role(
                    &AccessSessionRole {
                        role_key: &param.role_key,
                        user_id: user_info.id,
                        app_id: app.id,
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
        "allow_all_res": all_res,
        "perm_data": perm_data,
        "count": count,
    }))))
}
