//用户文件从URL创建接口

use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use crate::dao::access::api::system::user::CheckUserFileUpload;
use crate::dao::access::RbacAccessCheckEnv;
use lsys_access::dao::AccessSession;
use lsys_files::dao::ChunkInfo;
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct FileFromUrlParam {
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub app_id: u64,
    pub source_url: String,
    #[serde(
        default = "FileFromUrlParam::default_max_concurrency",
        deserialize_with = "crate::common::deserialize_u32"
    )]
    pub max_concurrency: u32,
}

impl FileFromUrlParam {
    fn default_max_concurrency() -> u32 {
        10
    }
}

/// 从URL创建文件
pub async fn file_from_url(
    param: &FileFromUrlParam,
    req_dao: &UserAuthQueryDao,
) -> JsonResult<JsonResponse> {
    let auth_data = req_dao.user_session.read().await.get_session_data().await?;
    let user_id = auth_data.user_id();
    let app = super::app_check_get(param.app_id, true, &auth_data, req_dao).await?;

    req_dao
        .web_dao
        .web_rbac
        .check(
            &RbacAccessCheckEnv::session_body(&auth_data, &req_dao.req_env),
            &CheckUserFileUpload {
                res_user_id: user_id,
            },
        )
        .await?;

    // 探测 URL 信息
    let url_info = req_dao
        .web_dao
        .web_files
        .file_dao
        .helper()
        .get_url_file_info(&param.source_url, param.max_concurrency as usize)
        .await?;

    // 文件大小校验
    let upload_config = &req_dao.web_dao.web_files.upload_config;
    if let Some(file_size) = url_info.file_size {
        if file_size > upload_config.max_upload_size {
            return Err(crate::common::JsonError::Message(
                lsys_core::fluent_message!("file-size-too-large",
                    {"size": file_size, "max": upload_config.max_upload_size}
                ),
            ));
        }
    }

    // 根据探测信息创建分片
    let chunks = if let Some(file_size) = url_info.file_size {
        req_dao
            .web_dao
            .web_files
            .file_dao
            .helper()
            .create_concurrent_chunks(file_size, url_info.max_concurrency as usize)?
    } else {
        // 未知文件大小，使用单分片
        vec![ChunkInfo {
            offset: 0,
            len: 0,
            md5: None,
        }]
    };

    let file_user_id = req_dao
        .web_dao
        .web_files
        .file_dao
        .create_from_url(
            &param.source_url,
            user_id,
            app.id,
            &chunks,
            url_info.content_type.as_deref(),
            Some(&req_dao.req_env),
        )
        .await?;

    Ok(JsonResponse::data(JsonData::body(json!({
        "file_user_id": file_user_id,
    }))))
}
