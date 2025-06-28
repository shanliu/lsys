use crate::common::JsonData;
use crate::dao::access::RbacAccessCheckEnv;
use crate::{
    common::{JsonResponse, JsonResult, PageParam, UserAuthQueryDao},
    dao::access::api::system::admin::CheckAdminRbacView,
};
use lsys_access::dao::AccessSession;
use lsys_rbac::dao::{CustomUserListResData, SessionUserListResData};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct UserFromResParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    user_id: u64, //资源用户ID
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    app_id: u64, //用户ID下的app
    res_type: String, //资源类型
    res_data: String, //资源数据
    op_key: String,   //授权操作结构列表
}

//1 得到指定资源的授权详细
pub async fn check_res_user_from_res(
    param: &UserFromResParam,
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
    let user_set_data = req_dao
        .web_dao
        .web_rbac
        .rbac_dao
        .access
        .find_user_data_from_res(
            param.user_id,
            param.app_id,
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
        .find_user_data_from_public(param.user_id, param.app_id)
        .await?;
    Ok(JsonResponse::data(JsonData::body(json!({
        "user_data": user_set_data,
        "pub_data": pub_set_data,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct ResRoleFromResParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub user_id: u64, //资源用户ID
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64, //用户ID下的app
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
    pub page: Option<PageParam>,
}

//获取非特定用户授权的角色列表
pub async fn check_res_role_data_from_res(
    param: &ResRoleFromResParam,
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
        .find_session_role_list_from_res(
            &SessionUserListResData {
                user_id: param.user_id,
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
            user_id: param.user_id,
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
        "total": res_count,
    }))))
}

#[derive(Debug, Deserialize)]
pub struct ResUserDataFromResParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    user_id: u64, //资源用户ID
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    app_id: u64, //用户ID下的app
    res_type: String, //资源类型
    res_data: String, //资源数据
    op_key: String,   //授权操作结构列表
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    res_range_exclude: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    res_range_any: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    res_range_include: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    is_system: bool,
    #[serde(deserialize_with = "crate::common::deserialize_bool")]
    is_self: bool,
    pub page: Option<PageParam>,
}

//获取特定用户授权列表
pub async fn check_res_user_data_from_res(
    param: &ResUserDataFromResParam,
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
        .find_custom_user_list_from_res(
            &CustomUserListResData {
                user_id: param.user_id,
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
            user_id: param.user_id,
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
        "total": res_count,
    }))))
}
