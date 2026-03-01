//用户文件字典映射接口

use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::UserAuthQueryDao;
use lsys_files::model::FileChunkStatus;
use lsys_files::model::FileSourceType;
use lsys_files::model::FileStatus;
use lsys_files::model::FileUserStatus;
use serde_json::json;

pub async fn mapping_data(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let min_chunk_size = req_dao
        .web_dao
        .web_files
        .file_dao
        .config()
        .min_chunk_size;

    let upload_config = &req_dao.web_dao.web_files.upload_config;

    Ok(JsonResponse::data(JsonData::body(json!({
        "min_chunk_size": min_chunk_size,
        "max_upload_size": upload_config.max_upload_size,
        "chunk_threshold": upload_config.chunk_threshold,
        "default_chunk_size": upload_config.default_chunk_size,
        "file_source_type": vec![
            status_json_format!(req_dao, FileSourceType::Upload),
            status_json_format!(req_dao, FileSourceType::Url),
            status_json_format!(req_dao, FileSourceType::LocalPath),
            status_json_format!(req_dao, FileSourceType::OssSync),
        ],
        "file_status": vec![
            status_json_format!(req_dao, FileStatus::Normal),
            status_json_format!(req_dao, FileStatus::Unfinished),
            status_json_format!(req_dao, FileStatus::Failed),
        ],
        "file_chunk_status": vec![
            status_json_format!(req_dao, FileChunkStatus::Normal),
            status_json_format!(req_dao, FileChunkStatus::Deleted),
            status_json_format!(req_dao, FileChunkStatus::Unfinished),
            status_json_format!(req_dao, FileChunkStatus::Failed),
            status_json_format!(req_dao, FileChunkStatus::Merged),
            status_json_format!(req_dao, FileChunkStatus::Cleaned),
        ],
        "file_user_status": vec![
            status_json_format!(req_dao, FileUserStatus::Normal),
            status_json_format!(req_dao, FileUserStatus::Deleted),
        ],
    }))))
}
