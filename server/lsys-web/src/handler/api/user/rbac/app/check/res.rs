use crate::{
    common::{JsonData, JsonResult, PageParam, UserAuthQueryDao},
    handler::api::user::rbac::app::{app_check_get, parent_app_check},
};
use lsys_rbac::{dao::AccessSessionRole, model::RbacRoleResRange};
use serde::Deserialize;
use serde_json::json;

//获取指定用户可访问的资源数据
#[derive(Debug, Deserialize)]
pub struct AppResUserFromUserParam {
    pub app_id: u64,
    pub access_user_param: String,
    pub page: Option<PageParam>,
}
//1 得到用户列表
pub async fn app_res_user_from_user(
    param: &AppResUserFromUserParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = parent_app_check(req_dao).await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    let user_info = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, &param.access_user_param, None, None)
        .await?;

    let mut user_ids = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_user_list_from_user(user_info.id, param.page.as_ref().map(|e| e.into()).as_ref())
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
        .find_res_user_count_from_user(user_info.id)
        .await?;
    Ok(JsonData::data(json!({
        "user_data": user_data,
        "is_system": is_system,
        "count": count,
    })))
}
#[derive(Debug, Deserialize)]
pub struct AppResInfoFromUserParam {
    pub app_id: u64,
    pub access_user_param: String,
}

//2 根据用户查找最近授权详细
pub async fn app_res_info_from_user(
    param: &AppResInfoFromUserParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = parent_app_check(req_dao).await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    let user_info = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, &param.access_user_param, None, None)
        .await?;

    let res_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_data_from_custom_user(auth_data.user_id(), user_info.id)
        .await?;
    Ok(JsonData::data(json!(res_data)))
}
#[derive(Debug, Deserialize)]
pub struct AppResListFromUserParam {
    pub app_id: u64,
    pub access_user_param: String,
    pub role_user_id: u64,
    pub res_range: i8,
    pub page: Option<PageParam>,
}

//3 如果配置关系,查询具体的配置授权
pub async fn app_res_list_from_user(
    param: &AppResListFromUserParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = parent_app_check(req_dao).await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    let user_info = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, &param.access_user_param, None, None)
        .await?;

    let res_range = RbacRoleResRange::try_from(param.res_range)?;
    let prem_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_list_from_custom_user(
            user_info.id,
            param.role_user_id,
            Some(app.id),
            res_range,
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let count = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_res_count_from_custom_user(user_info.id, param.role_user_id, Some(app.id), res_range)
        .await?;
    Ok(JsonData::data(json!({
        "prem_data": prem_data,
        "count": count,
    })))
}

#[derive(Debug, Deserialize)]
pub struct AppResListFromSessionParam {
    pub app_id: u64,
    pub role_key: String,
    pub access_user_param: String,
    pub page: Option<PageParam>,
}
//3 如果是会话角色,根据会话角色查询该会话角色的授权资源
pub async fn app_res_info_from_session(
    param: &AppResListFromSessionParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonData> {
    let auth_data = parent_app_check(req_dao).await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    let user_info = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, &param.access_user_param, None, None)
        .await?;

    let rs = req_dao
        .web_dao
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
    let mut prem_data = vec![];
    let mut count = 0;
    match rs {
        ref d @ (RbacRoleResRange::Include | RbacRoleResRange::Exclude) => {
            prem_data = req_dao
                .web_dao
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
    Ok(JsonData::data(json!({
        "allow_all_res": all_res,
        "prem_data": prem_data,
        "count": count,
    })))
}
