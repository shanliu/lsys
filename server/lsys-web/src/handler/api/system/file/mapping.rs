use crate::common::{JsonData, JsonResponse, JsonResult, UserAuthQueryDao};
use lsys_file::model::{
    FileChunkStatus, FileModel, FileSourceType, FileStatus, FileTagStatus, FileUserStatus,
};
use serde_json::json;

/// POST /api/system/file/mapping — 文件管理字典映射
///
/// 返回文件相关的所有枚举类型映射，含多语言文本，以及存储类型（本地+已注册 OSS）。
pub async fn admin_file_mapping(req_dao: &UserAuthQueryDao) -> JsonResult<JsonResponse> {
    let registry = req_dao.web_dao.web_files.file_dao.oss_config().registry();
    let oss_types = registry.available_types();

    // storage_type: 本地(type="local") + 所有已注册 OSS 厂商(type="oss")
    let mut storage_type: Vec<serde_json::Value> =
        vec![var_json_format!(req_dao, FileModel::STORAGE_TYPE_LOCAL_PUBLIC, { "type": "local" })];
    for t in oss_types {
        storage_type.push(var_json_format!(req_dao, t, { "type": "oss" }));
    }

    Ok(JsonResponse::data(JsonData::body(json!({
        "storage_type": storage_type,
        "file_source_type": vec![
            status_json_format!(req_dao, FileSourceType::Upload),
            status_json_format!(req_dao, FileSourceType::Url),
            status_json_format!(req_dao, FileSourceType::LocalPath),
            status_json_format!(req_dao, FileSourceType::OssSync),
        ],
        "file_status": vec![
            status_json_format!(req_dao, FileStatus::Normal),
            status_json_format!(req_dao, FileStatus::Deleted),
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
        "file_tag_status": vec![
            status_json_format!(req_dao, FileTagStatus::Normal),
            status_json_format!(req_dao, FileTagStatus::Deleted),
        ],
    }))))
}
