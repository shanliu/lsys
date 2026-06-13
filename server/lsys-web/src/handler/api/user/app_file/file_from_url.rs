//用户文件从URL创建接口

use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::user::CheckUserFileUpload;
use lsys_access::dao::AccessSession;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct FileFromUrlParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub source_url: String,
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
    /// 存储类型: local_public / local_private / local_crypto，默认 local_public
    #[serde(default = "FileFromUrlParam::default_storage_type")]
    pub storage_type: String,
}

impl FileFromUrlParam {
    fn default_storage_type() -> String {
        lsys_file::model::FileModel::STORAGE_TYPE_LOCAL_PUBLIC.to_string()
    }
}

/// 从URL创建文件
pub async fn file_from_url(
    param: &FileFromUrlParam,
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
    let user_id = auth_data.user_id();
    let app = super::app_check_get(param.app_id, true, &auth_data, req_dao, web_dao).await?;

    web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileUpload {
                res_user_id: user_id,
            },
        )
        .await?;

    // 用运行时配置限制用户请求的并发数
    let tag_refs: Vec<&str> = param
        .tag_names
        .as_deref()
        .unwrap_or(&[])
        .iter()
        .map(String::as_str)
        .collect();
    let file_ref_id = web_dao
        .web_file.file_dao
        .create_from_url_auto(
            &param.source_url,
            user_id,
            user_id,
            app.id,
            &param.storage_type,
            &tag_refs,
            None, // expire_time
            None, // wait_timeout
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "id": file_ref_id,
    }))))
}
