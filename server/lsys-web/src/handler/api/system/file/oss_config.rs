use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::JsonPageData;
use lsys_core::db::{OffsetPageParam, OffsetPageValue};
use serde::Deserialize;
use serde_json::json;

// ==================== 列表 ====================

#[derive(Debug, Deserialize)]
pub struct AdminOssConfigListParam {
    pub page: Option<u64>,
    pub limit: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

/// 管理员查询 OSS 配置列表
pub async fn admin_oss_config_list(
    param: &AdminOssConfigListParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let page_num = param.page.unwrap_or(1).max(1);
    let page_size = param.limit.unwrap_or(20).min(100);
    let offset = (page_num - 1) * page_size;

    let page = OffsetPageParam::new(Some(OffsetPageValue::new(offset, page_size)));

    let oss_config = &web_dao.web_file.file_dao.oss_config();
    let data = oss_config.list_config(&page).await?;

    let items: Vec<serde_json::Value> = data
        .iter()
        .map(|item| {
            let m = item.model();
            json!({
                "id": m.id,
                "name": m.name,
                "config_key": item.config_key,
                "provider_type": item.provider_type,
                "provider_config": item.provider_config,
                "is_private": item.is_private,
                "change_user_id": m.change_user_id,
                "change_time": m.change_time,
            })
        })
        .collect();

    let total = oss_config.list_count().await?;

    Ok(JsonResponse::data(JsonData::body(JsonPageData::total(
        items, total,
    ))))
}

// ==================== 详情 ====================

#[derive(Debug, Deserialize)]
pub struct AdminOssConfigDetailParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
}

/// 管理员查看 OSS 配置详情
pub async fn admin_oss_config_detail(
    param: &AdminOssConfigDetailParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let oss_config = &web_dao.web_file.file_dao.oss_config();
    let item = oss_config.load_config(param.id).await?;
    let m = item.model();

    Ok(JsonResponse::data(JsonData::body(json!({
        "id": m.id,
        "name": m.name,
        "config_key": item.config_key,
        "provider_type": item.provider_type,
        "provider_config": item.provider_config,
        "is_private": item.is_private,
        "change_user_id": m.change_user_id,
        "change_time": m.change_time,
    }))))
}

// ==================== 新增 ====================

#[derive(Debug, Deserialize)]
pub struct AdminOssConfigAddParam {
    pub name: String,
    pub config_key: String,
    pub provider_type: String,
    pub provider_config: serde_json::Value,
    #[serde(default)]
    pub is_private: bool,
}

/// 管理员新增 OSS 配置
pub async fn admin_oss_config_add(
    param: &AdminOssConfigAddParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let oss_config = &web_dao.web_file.file_dao.oss_config();
    let id = oss_config
        .add_config(
            &param.name,
            &param.config_key,
            &param.provider_type,
            param.provider_config.clone(),
            param.is_private,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "id": id,
    }))))
}

// ==================== 修改 ====================

#[derive(Debug, Deserialize)]
pub struct AdminOssConfigEditParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
    pub name: String,
    pub provider_config: serde_json::Value,
    #[serde(default)]
    pub is_private: bool,
}

/// 管理员修改 OSS 配置
pub async fn admin_oss_config_edit(
    param: &AdminOssConfigEditParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let oss_config = &web_dao.web_file.file_dao.oss_config();
    oss_config
        .edit_config(
            param.id,
            &param.name,
            param.provider_config.clone(),
            param.is_private,
            auth_data.user_id(),
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::default())
}

// ==================== 删除 ====================

#[derive(Debug, Deserialize)]
pub struct AdminOssConfigDeleteParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
}

/// 管理员删除 OSS 配置
pub async fn admin_oss_config_delete(
    param: &AdminOssConfigDeleteParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    let auth_data = auth_dao
        .user_session
        .read()
        .await
        .get_session_data()
        .await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckAdminFileManage {},
        )
        .await?;

    let oss_config = &web_dao.web_file.file_dao.oss_config();
    oss_config
        .del_config(param.id, auth_data.user_id(), Some(&req_dao.req_env))
        .await?;

    Ok(JsonResponse::default())
}
