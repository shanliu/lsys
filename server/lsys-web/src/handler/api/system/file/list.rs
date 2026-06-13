use crate::common::{JsonData, JsonResponse, JsonResult, RequestDao, UserAuthQueryDao};
use crate::dao::WebDao;
use crate::dao::access::RbacAccessCheckEnv;
use crate::dao::access::api::system::admin::CheckAdminFileManage;
use lsys_access::dao::AccessSession;
use lsys_core::api_utils::{JsonPageData, PageCursorValue, PageTotalRowValue};
use lsys_core::db::{CursorPageSort, TotalParam};
use lsys_file::dao::{FileDataListParam, FileListAttrParam};
use serde::Deserialize;
use serde_json::json;

#[derive(Debug, Deserialize)]
pub struct AdminFileListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub user_id: Option<u64>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub source_url: Option<String>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub add_time_start: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub add_time_end: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_i8")]
    pub status: Option<i8>,
    #[serde(default)]
    pub storage_type: Option<String>,
    #[serde(default)]
    pub file_md5: Option<String>,
    pub limit: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
    /// 按标签名过滤
    #[serde(default)]
    pub tag_names: Option<Vec<String>>,
}

/// 管理员文件列表
pub async fn admin_file_list(
    param: &AdminFileListParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse>
{
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

    use crate::common::ToCursorPageParam;
    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let tag_refs: Option<Vec<&str>> = param
        .tag_names
        .as_ref()
        .map(|v| v.iter().map(String::as_str).collect());
    let filter = FileDataListParam {
        local_url: param.url.as_deref(),
        source_url: param.source_url.as_deref(),
        user_id: param.user_id,
        app_id: Some(0),
        add_time_start: param.add_time_start,
        add_time_end: param.add_time_end,
        status: param.status,
        storage_type: param.storage_type.as_deref(),
        file_md5: param.file_md5.as_deref(),
        tag_names: tag_refs.as_deref(),
    };

    let attr_param = FileListAttrParam {
        attr_local: Some(true),
        attr_oss: Some(true),
        attr_tag_list: Some(3),
        attr_tag_count: Some(true),
        ..Default::default()
    };

    let (data, page_data) = web_dao
        .web_file.file_dao
        .data_dao()
        .list_files(&filter, &page, &attr_param)
        .await?;

    let mut items: Vec<serde_json::Value> = Vec::with_capacity(data.len());

    for item in &data {
        // 使用回调函数生成文件访问 URL（传入已有的 file_key）
        let mut obj = json!({
            "id": item.item.id,
            "file_id": item.item.file_id,
            "file_key": item.file_key,
            "file_name": item.item.file_name,
            "file_md5": item.item.file_md5,
            "file_size": item.item.file_size,
            "storage_type": item.item.storage_type,
            "status": item.item.status,
            "content_type": item.item.content_type,
            "source_url": item.item.source_url,
            "is_downloading": item.attr_url_downloading,
            "add_time": item.item.file_ref_add_time,
            "user_id": item.item.user_id,
            "from_user_id": item.item.from_user_id,
            "copy_file_id": 0,
        });

        if let Some(local) = &item.attr_local {
            obj["local_id"] = json!(local.id);
            obj["source_type"] = json!(local.source_type);
            obj["source_name"] = json!(local.source_name);
            obj["local_path"] = json!(local.local_path);
            obj["file_chunk_total"] = json!(local.file_chunk_total);
            obj["file_chunk_succ"] = json!(local.file_chunk_succ);
            obj["file_chunk_size"] = json!(local.file_chunk_size);
        }

        if let Some(oss) = &item.attr_oss {
            obj["oss_id"] = json!(oss.id);
            obj["object_key"] = json!(oss.object_key);
            obj["object_url"] = json!(oss.object_url);
            obj["bucket"] = json!(oss.bucket);
            obj["region"] = json!(oss.region);
            obj["oss_size"] = json!(oss.size);
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
            if let Some(count) = tag_attr.count {
                obj["tag_count"] = json!(count);
            }
        }

        items.push(obj);
    }

    let total = if param.count_num.unwrap_or(false) {
        Some(
            web_dao
                .web_file.file_dao
                .data_dao()
                .count_files(&filter, &TotalParam::default())
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

/// 管理员查询文件关联详细列表参数
#[derive(Debug, Deserialize)]
pub struct AdminFileLineageRelatedListParam {
    /// file_ref_id
    #[serde(deserialize_with = "crate::common::deserialize_u64")]
    pub id: u64,
    /// 关系类型（可选）：1=拷贝, 2=转换, 3=OSS同步
    #[serde(default)]
    pub rel_type: Option<i8>,
    /// 按存储类型过滤（可选）
    #[serde(default)]
    pub storage_type: Option<String>,
    pub limit: Option<crate::common::LimitParam>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub count_num: Option<bool>,
}

/// 管理员查询下载中文件列表参数
#[derive(Debug, Deserialize)]
pub struct AdminFileDownloadingListParam {
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub user_id: Option<u64>,
    #[serde(default, deserialize_with = "crate::common::deserialize_option_u64")]
    pub app_id: Option<u64>,
    /// 是否正在下载：true=仅下载中, false=仅排队中, 空=全部
    #[serde(default, deserialize_with = "crate::common::deserialize_option_bool")]
    pub is_downloading: Option<bool>,
    pub limit: Option<crate::common::LimitParam>,
}

/// 管理员查询文件关联详细列表
pub async fn admin_file_lineage_related_list(
    param: &AdminFileLineageRelatedListParam,
    req_dao: &RequestDao,
    auth_dao: &UserAuthQueryDao,
    web_dao: &WebDao,
) -> JsonResult<JsonResponse>
{
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

    use crate::common::ToCursorPageParam;
    use lsys_file::dao::LineageRelatedListParam;
    use lsys_file::model::FileRefModel;

    // 查询 file_ref
    let file_ref: FileRefModel = web_dao
        .web_file.file_dao
        .helper()
        .find_file_ref_by_id(param.id)
        .await?
        .ok_or_else(|| {
            lsys_file::common::FileError::Param(lsys_core::fluent_message!("param-error"))
        })?;

    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let filter = LineageRelatedListParam {
        rel_type: param.rel_type,
        storage_type: param.storage_type.clone(),
    };

    let attr_param = FileListAttrParam {
        attr_local: Some(true),
        attr_oss: Some(true),
        attr_tag_list: None,
        attr_tag_count: Some(true),
        attr_lineage: None,
        attr_url_downloading: None,
    };

    let (data, page_data) = web_dao
        .web_file.file_dao
        .data_dao()
        .list_lineage_related_files(&file_ref, &filter, &page, &attr_param)
        .await?;

    let mut items: Vec<serde_json::Value> = Vec::with_capacity(data.len());
    for item in &data {
        // 使用回调函数生成文件访问 URL（传入已有的 file_key）
        let mut obj = json!({
            "id": item.item.id,
            "file_id": item.item.file_id,
            "file_key": item.file_key,
            "file_name": item.item.file_name,
            "file_md5": item.item.file_md5,
            "file_size": item.item.file_size,
            "storage_type": item.item.storage_type,
            "status": item.item.status,
            "content_type": item.item.content_type,
            "source_url": item.item.source_url,
            "add_time": item.item.file_ref_add_time,
            "user_id": item.item.user_id,
            "from_user_id": item.item.from_user_id,
        });

        // 摊平 attr_local 数据
        if let Some(local) = &item.attr_local {
            obj["local_id"] = json!(local.id);
            obj["source_type"] = json!(local.source_type);
            obj["source_name"] = json!(local.source_name);
            obj["local_path"] = json!(local.local_path);
            obj["file_chunk_total"] = json!(local.file_chunk_total);
            obj["file_chunk_succ"] = json!(local.file_chunk_succ);
            obj["file_chunk_size"] = json!(local.file_chunk_size);
        }

        // 摊平 attr_oss 数据
        if let Some(oss) = &item.attr_oss {
            obj["oss_id"] = json!(oss.id);
            obj["object_key"] = json!(oss.object_key);
            obj["object_url"] = json!(oss.object_url);
            obj["bucket"] = json!(oss.bucket);
            obj["region"] = json!(oss.region);
            obj["oss_size"] = json!(oss.size);
        }

        // 标签数量
        if let Some(tag_attr) = &item.attr_tag
            && let Some(count) = tag_attr.count {
                obj["tag_count"] = json!(count);
            }

        items.push(obj);
    }

    let total = if param.count_num.unwrap_or(false) {
        let count = web_dao
            .web_file.file_dao
            .data_dao()
            .count_lineage_related_files(&file_ref, &filter)
            .await?;
        Some(PageTotalRowValue {
            exact: Some(count as u64),
            over: None,
        })
    } else {
        None
    };

    let cursor = PageCursorValue::from(&page_data);
    Ok(JsonResponse::data(JsonData::body(JsonPageData::cursor(
        items, cursor, total,
    ))))
}

/// 管理员查询下载中文件列表
pub async fn admin_file_downloading_list(
    param: &AdminFileDownloadingListParam,
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

    use crate::common::ToCursorPageParam;
    use lsys_file::dao::DownloadingListParam;

    let page = param.limit.to_u64_cursor_page_param(CursorPageSort::Desc);

    let filter = DownloadingListParam {
        user_id: param.user_id,
        app_id: param.app_id,
        is_downloading: param.is_downloading,
    };

    let attr_param = FileListAttrParam {
        attr_local: Some(true),
        attr_oss: Some(true),
        attr_tag_list: None,
        attr_tag_count: Some(true),
        attr_lineage: None,
        attr_url_downloading: None,
    };

    let (data, page_data) = web_dao
        .web_file.file_dao
        .data_dao()
        .list_downloading_files(&filter, &page, &attr_param)
        .await?;

    // 批量获取文件 URL
    let file_models: Vec<lsys_file::model::FileModel> = data
        .iter()
        .map(|item| lsys_file::model::FileModel {
            id: item.item.item.file_id,
            storage_type: item.item.item.storage_type.clone(),
            status: item.item.item.status,
            origin_name: item.item.item.file_name.clone(),
            file_md5: item.item.item.file_md5.clone(),
            file_size: item.item.item.file_size,
            content_type: item.item.item.content_type.clone(),
            ..Default::default()
        })
        .collect();
    let url_map = web_dao
        .web_file.file_dao
        .data_dao()
        .get_file_urls(&file_models)
        .await
        .unwrap_or_default();

    let mut items: Vec<serde_json::Value> = Vec::with_capacity(data.len());
    for item in &data {
        let url = url_map.get(&item.item.item.file_id).cloned().flatten();
        let mut obj = json!({
            "id": item.item.item.id,
            "file_id": item.item.item.file_id,
            "file_name": item.item.item.file_name,
            "file_md5": item.item.item.file_md5,
            "file_size": item.item.item.file_size,
            "storage_type": item.item.item.storage_type,
            "status": item.item.item.status,
            "content_type": item.item.item.content_type,
            "source_url": item.item.item.source_url,
            "file_url": url,
            "add_time": item.item.item.file_ref_add_time,
            "user_id": item.item.item.user_id,
            "from_user_id": item.item.item.from_user_id,
            "is_downloading": item.is_downloading,
        });

        // 摊平 attr_local 数据
        if let Some(local) = &item.item.attr_local {
            obj["local_id"] = json!(local.id);
            obj["source_type"] = json!(local.source_type);
            obj["source_name"] = json!(local.source_name);
            obj["local_path"] = json!(local.local_path);
            obj["file_chunk_total"] = json!(local.file_chunk_total);
            obj["file_chunk_succ"] = json!(local.file_chunk_succ);
            obj["file_chunk_size"] = json!(local.file_chunk_size);
        }

        // 摊平 attr_oss 数据
        if let Some(oss) = &item.item.attr_oss {
            obj["oss_id"] = json!(oss.id);
            obj["object_key"] = json!(oss.object_key);
            obj["object_url"] = json!(oss.object_url);
            obj["bucket"] = json!(oss.bucket);
            obj["region"] = json!(oss.region);
            obj["oss_size"] = json!(oss.size);
        }

        // 标签数量
        if let Some(tag_attr) = &item.item.attr_tag
            && let Some(count) = tag_attr.count {
                obj["tag_count"] = json!(count);
            }

        items.push(obj);
    }

    // 下载中列表暂不支持 count（因为需要实时查询任务状态）
    let total: Option<PageTotalRowValue> = None;

    let cursor = PageCursorValue::from(&page_data);
    Ok(JsonResponse::data(JsonData::body(JsonPageData::cursor(
        items, cursor, total,
    ))))
}
