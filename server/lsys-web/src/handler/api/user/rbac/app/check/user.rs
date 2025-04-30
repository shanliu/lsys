use crate::common::JsonData;
use crate::{
    common::{JsonResponse, JsonResult, PageParam, UserAuthQueryDao},
    handler::api::user::rbac::app::{app_check_get, parent_app_check},
};
use lsys_rbac::dao::{CustomUserListResData, SessionUserListResData};
use serde::Deserialize;
use serde_json::json;

//根据资源得到用户授权详细

#[derive(Debug, Deserialize)]
pub struct AppUserFromResParam {
    pub user_param: String, //资源用户ID
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub res_type: String, //资源类型
    pub res_data: String, //资源数据
    pub op_key: String,   //授权操作结构列表
}

//1 得到指定资源的授权详细
pub async fn app_res_user_from_res(
    param: &AppUserFromResParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = parent_app_check(req_dao).await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    let user_info = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, &param.user_param, None, None)
        .await?;

    let user_set_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_user_data_from_res(
            user_info.id,
            app.id,
            &param.res_type,
            &param.res_data,
            &param.op_key,
        )
        .await?;
    let pub_set_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_user_data_from_public(user_info.id, param.app_id)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "user_data": user_set_data,
        "pub_data": pub_set_data,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct AppResRoleFromResParam {
    pub user_param: String, //资源用户ID
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub res_type: String, //资源类型
    pub res_data: String, //资源数据
    pub op_key: String,   //授权操作结构列表
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub res_range_exclude: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub res_range_any: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub res_range_include: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub is_system: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub is_self: bool,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_vec_i8")]
    pub user_range: Option<Vec<i8>>,
    pub page: Option<PageParam>,
}

//获取非特定用户授权的角色列表
pub async fn app_res_session_role_data_from_res(
    param: &AppResRoleFromResParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = parent_app_check(req_dao).await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    let user_info = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, &param.user_param, None, None)
        .await?;

    let res_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_session_role_list_from_res(
            &SessionUserListResData {
                user_id: user_info.id,
                app_id: param.app_id,
                res_type: &param.res_type,
                res_data: &param.res_data,
                op_key: &param.op_key,
                res_range_exclude: param.res_range_exclude,
                res_range_any: param.res_range_any,
                res_range_include: param.res_range_include,
                is_system: param.is_system,
                is_self: param.is_self,
            },
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let res_count = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_session_role_count_from_res(&SessionUserListResData {
            user_id: user_info.id,
            app_id: param.app_id,
            res_type: &param.res_type,
            res_data: &param.res_data,
            op_key: &param.op_key,
            res_range_exclude: param.res_range_exclude,
            res_range_any: param.res_range_any,
            res_range_include: param.res_range_include,
            is_system: param.is_system,
            is_self: param.is_self,
        })
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": res_data,
        "count": res_count,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct AppResUserDataFromResParam {
    pub user_param: String, //资源用户ID7
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64, //用户ID下的app
    pub res_type: String,   //资源类型
    pub res_data: String,   //资源数据
    pub op_key: String,     //授权操作结构列表
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub res_range_exclude: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub res_range_any: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub res_range_include: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub is_system: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    pub is_self: bool,
    pub page: Option<PageParam>,
}
//获取特定用户授权列表
pub async fn app_res_user_data_from_res(
    param: &AppResUserDataFromResParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = parent_app_check(req_dao).await?;
    let app = app_check_get(param.app_id, false, &auth_data, req_dao).await?;

    let user_info = req_dao
        .web_dao
        .web_access
        .access_dao
        .user
        .cache()
        .sync_user(app.id, &param.user_param, None, None)
        .await?;

    let res_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_custom_user_list_from_res(
            &CustomUserListResData {
                user_id: user_info.id,
                app_id: param.app_id,
                res_type: &param.res_type,
                res_data: &param.res_data,
                op_key: &param.op_key,
                res_range_exclude: param.res_range_exclude,
                res_range_any: param.res_range_any,
                res_range_include: param.res_range_include,
                is_system: param.is_system,
                is_self: param.is_self,
            },
            param.page.as_ref().map(|e| e.into()).as_ref(),
        )
        .await?;
    let res_count = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_custom_user_count_from_res(&CustomUserListResData {
            user_id: user_info.id,
            app_id: param.app_id,
            res_type: &param.res_type,
            res_data: &param.res_data,
            op_key: &param.op_key,
            res_range_exclude: param.res_range_exclude,
            res_range_any: param.res_range_any,
            res_range_include: param.res_range_include,
            is_system: param.is_system,
            is_self: param.is_self,
        })
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "data": res_data,
        "count": res_count,
    }))))
}
