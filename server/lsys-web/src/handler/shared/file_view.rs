//! 文件相关的共享输出整形
//!
//! `file_list_response` 收纳 rest / service 两个通道完全一致的「文件列表查询 → JSON」逻辑。
//! 通道之间的差异（鉴权 gate、过滤主体 user_id/app_id 的来源）由各通道在构造 `filter`
//! 之前自行处理，本函数只接收已构造好的 `filter` 与分页/计数参数。

use crate::common::{JsonData, JsonPageData, JsonResponse, JsonResult};
use crate::dao::WebDao;
use lsys_core::api_utils::{PageCursorValue, PageTotalRowValue};
use lsys_core::db::TotalParam;
use lsys_file::dao::{FileDataListParam, FileListAttrParam};
use lsys_file::model::FileModel;
use serde_json::json;

/// 文件列表查询的统一输出整形。
///
/// 调用方负责：
/// - 完成本通道的鉴权 gate（如 rest 的 `check_rest_app`）；
/// - 构造 `filter`（其中已固化通道差异，如 service 取自参数、rest 固定为当前应用）。
///
/// 本函数负责：分页参数构造、`list_files`、批量取 URL、逐项 JSON 整形、可选计数与游标包装。
pub async fn file_list_response(
    filter: &FileDataListParam<'_>,
    cursor: Option<u64>,
    limit: Option<u64>,
    count_num: bool,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse> {
    use lsys_core::db::{
        CursorConfig, CursorLimit, CursorPageDir, CursorPageParam, CursorPageSort,
    };

    let limit_val = limit.unwrap_or(20).min(100);
    let page = CursorPageParam::new(
        CursorPageDir::Next,
        CursorConfig::primary(CursorPageSort::Desc),
        cursor,
        CursorLimit::Limit {
            limit: limit_val,
            more: false,
        },
    );

    let attr_param = FileListAttrParam {
        attr_local: Some(true),
        attr_oss: Some(true),
        ..Default::default()
    };

    let (data, page_data) = web_dao
        .web_file
        .file_dao
        .data_dao()
        .list_files(filter, &page, &attr_param)
        .await?;

    // 批量获取文件 URL
    let file_models: Vec<FileModel> = data
        .iter()
        .map(|item| FileModel {
            id: item.item.file_id,
            storage_type: item.item.storage_type.clone(),
            status: item.item.status,
            origin_name: item.item.file_name.clone(),
            file_md5: item.item.file_md5.clone(),
            file_size: item.item.file_size,
            content_type: item.item.content_type.clone(),
            ..Default::default()
        })
        .collect();
    let url_map = web_dao
        .web_file
        .file_dao
        .data_dao()
        .get_file_urls(&file_models)
        .await
        .unwrap_or_default();

    let mut items: Vec<serde_json::Value> = Vec::with_capacity(data.len());
    for item in &data {
        let url = url_map.get(&item.item.file_id).cloned().flatten();
        let mut obj = json!({
            "id": item.item.id,
            "file_id": item.item.file_id,
            "file_name": item.item.file_name,
            "file_md5": item.item.file_md5,
            "file_size": item.item.file_size,
            "storage_type": item.item.storage_type,
            "status": item.item.status,
            "content_type": item.item.content_type,
            "source_url": item.item.source_url,
            "is_downloading": item.attr_url_downloading,
            "file_url": url,
            "add_time": item.item.file_ref_add_time,
            "user_id": item.item.user_id,
        });

        if let Some(local) = &item.attr_local {
            obj["local_path"] = json!(local.local_path);
            obj["source_type"] = json!(local.source_type);
            obj["file_chunk_total"] = json!(local.file_chunk_total);
            obj["file_chunk_succ"] = json!(local.file_chunk_succ);
        }

        if let Some(oss) = &item.attr_oss {
            obj["object_url"] = json!(oss.object_url);
            obj["bucket"] = json!(oss.bucket);
            obj["region"] = json!(oss.region);
        }

        if let Some(tag_attr) = &item.attr_tag {
            let tags: Vec<serde_json::Value> = tag_attr
                .tags
                .iter()
                .map(|t| {
                    json!({
                        "tag_name": t.tag_name,
                        "add_time": t.add_time,
                    })
                })
                .collect();
            obj["tags"] = json!(tags);
        }

        items.push(obj);
    }

    let total = if count_num {
        Some(
            web_dao
                .web_file
                .file_dao
                .data_dao()
                .count_files(filter, &TotalParam::default())
                .await
                .map(PageTotalRowValue::from)?,
        )
    } else {
        None
    };

    let cursor = PageCursorValue::from(&page_data);
    Ok(JsonResponse::data(JsonData::body(JsonPageData::cursor(
        items, cursor, total,
    ))))
}
