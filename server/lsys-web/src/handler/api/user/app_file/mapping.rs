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

    // storage_type: 本地(type="local") + 所有已配置的 OSS 实例(type="oss")
    // 本地类型完整下发，支持 local_public/local_private/local_crypto 互转。
    let mut storage_type: Vec<serde_json::Value> = vec![
        var_json_format!(req_dao, FileModel::STORAGE_TYPE_LOCAL_PUBLIC, { "type": "local" }),
        var_json_format!(req_dao, FileModel::STORAGE_TYPE_LOCAL_PRIVATE, { "type": "local" }),
        var_json_format!(req_dao, FileModel::STORAGE_TYPE_LOCAL_CRYPTO, { "type": "local" }),
    ];

    // 获取所有已配置的 OSS 实例（config_key），而非厂商类型（provider_type）
    // 因为 storage_type 字段存储的是 config_key，文件拷贝时使用 find_by_config_key 查找配置
    let oss_config_dao = web_dao.web_file.file_dao.oss_config();
    let page = lsys_core::db::OffsetPageParam::new(None); // 获取所有配置
    if let Ok(oss_configs) = oss_config_dao.list_config(&page).await {
        for config in oss_configs {
            storage_type.push(serde_json::json!({
                "key": &config.config_key,
                "val": &config.model().name,
                "type": "oss",
                "provider_type": &config.provider_type,
                "is_private": config.is_private
            }));
        }
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
