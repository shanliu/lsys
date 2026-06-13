//用户文件字典映射接口

use crate::common::JsonData;
use crate::common::JsonResponse;
use crate::common::JsonResult;
use crate::common::RequestDao;
use crate::dao::WebDao;
use lsys_file::model::FileChunkStatus;
use lsys_file::model::FileLineageRelType;
use lsys_file::model::FileModel;
use lsys_file::model::FileSourceType;
use lsys_file::model::FileStatus;
use lsys_file::model::FileUserStatus;
use serde_json::json;

pub async fn mapping_data(req_dao: &RequestDao, web_dao: &WebDao) -> JsonResult<JsonResponse> {
    let max_upload_size = web_dao.web_file.file_dao.runtime_setting().get_upload_max_file_size().await.unwrap_or(0);
    let upload_chunk_max = web_dao.web_file.file_dao.config().upload_chunk_max;

    let registry = web_dao.web_file.file_dao.oss_config().registry();
    let oss_types = registry.available_types();

    // storage_type: 本地(type="local") + 所有已注册 OSS 厂商(type="oss")
    let mut storage_type: Vec<serde_json::Value> =
        vec![var_json_format!(req_dao, FileModel::STORAGE_TYPE_LOCAL_PUBLIC, { "type": "local" })];
    for t in oss_types {
        storage_type.push(var_json_format!(req_dao, t, { "type": "oss" }));
    }

    Ok(JsonResponse::data(JsonData::body(json!({
        "max_upload_size": max_upload_size,
        "upload_chunk_max": upload_chunk_max,
        "storage_type": storage_type,
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
        "file_ref_status": vec![
            status_json_format!(req_dao, FileUserStatus::Normal),
            status_json_format!(req_dao, FileUserStatus::Deleted),
        ],
        "lineage_rel_type": vec![
            status_json_format!(req_dao, FileLineageRelType::Copy),
            status_json_format!(req_dao, FileLineageRelType::Convert),
            status_json_format!(req_dao, FileLineageRelType::OssSync),
        ],
    }))))
}
