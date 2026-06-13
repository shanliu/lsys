// 执行记录查询 + 计数 + 记录关联文件/日志独立查询

use lsys_core::db::{
    CursorPageData, CursorPageParam, QueryBuilderExt, TableMeta, TotalParam,
    TotalRow, WhereClause,
};
use lsys_core::utils::{StringClear, string_clear};
use lsys_file::dao::{FileListAttrParam, FileLocalAttrData};
use sqlx::{MySql, QueryBuilder};

use crate::dao::result::FileManagerResult;
use crate::model::*;

use super::FileCollector;
use super::script::ScriptFileTag;

/// 记录关联的文件信息（使用私有存储，通过 file_key 访问）
#[derive(Debug, Clone, serde::Serialize)]
pub struct RecordFileItem {
    pub file_id: u64,
    pub file_name: String,
    pub file_md5: String,
    pub file_size: u64,
    pub storage_type: String,
    pub content_type: String,
    pub file_key: String,
    pub add_time: u64,
    pub user_id: u64,
    pub add_user_id: u64,
    pub app_id: u64,
    pub status: i8,
    pub file_ref_status: i8,
    pub source_url: String,
    pub source_md5: String,
    pub modify_time: u64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attr_local: Option<FileLocalAttrData>,
}

/// 记录关联的文件 tag
pub type RecordFileTag = ScriptFileTag;

/// 记录列表项 = 记录本身 + 可选的首个文件信息 + 是否有更多文件标记
#[derive(Debug, Clone, serde::Serialize)]
pub struct CollectorRecordItem {
    #[serde(flatten)]
    pub record: CollectorRecordModel,
    /// 首个关联文件（如果存在）
    pub file: Option<RecordFileItem>,
    /// 是否有更多文件
    pub has_more_files: bool,
}

/// 记录列表查询的 ATTR 参数
#[derive(Debug, Default)]
pub struct CollectorRecordListAttr {
    /// 是否附加文件信息（首个文件 + 是否有更多）
    pub attr_file: Option<bool>,
    /// 是否附加文件的 local 属性
    pub attr_file_local: Option<bool>,
    /// 是否附加文件的 oss 属性
    pub attr_file_oss: Option<bool>,
    /// 是否附加文件的 tag 属性
    pub attr_file_tag: Option<bool>,
}

impl FileCollector {
    /// 构建记录查询的 WHERE 子句
    fn build_record_where<'a, 'args>(
        wb: &mut WhereClause<'a, 'args, MySql>,
        script: &CollectorScriptModel,
        request_id: Option<&str>,
        status: Option<i8>,
    ) {
        wb.and().field_eq("script_id", script.id);
        if let Some(rid) = request_id {
            let rid = string_clear(rid, StringClear::Ident, Some(512));
            if !rid.is_empty() {
                wb.and().field_eq("request_id", rid);
            }
        }
        if let Some(s) = status {
            wb.and().field_eq("status", s);
        }
    }

    /// 按 request_id 查询记录
    pub async fn find_record_by_request_id(
        &self,
        request_id: &str,
    ) -> FileManagerResult<Option<CollectorRecordModel>> {
        let row = sqlx::query_as::<_, CollectorRecordModel>(&format!(
            "SELECT * FROM {} WHERE request_id=?",
            CollectorRecordModel::table_name()
        ))
        .bind(request_id)
        .fetch_optional(&self.db)
        .await?;
        Ok(row)
    }

    /// 记录列表（分页），可按 script_id / request_id / status 过滤
    ///
    /// 支持通过 attr 参数附加每个记录的首个文件信息和是否有更多文件的标记。
    /// 使用脚本的 add_user_id 和 app_id 来过滤文件查询，提高查询效率。
    pub async fn list_records(
        &self,
        script: &CollectorScriptModel,
        request_id: Option<&str>,
        status: Option<i8>,
        page: &CursorPageParam<u64>,
        attr: &CollectorRecordListAttr,
    ) -> FileManagerResult<(Vec<CollectorRecordItem>, CursorPageData<u64>)> {
        let query_limit = page.page_query("id");
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "SELECT * FROM {}",
            CollectorRecordModel::table_name()
        ));
        {
            let mut wb = WhereClause::new(&mut qb);
            Self::build_record_where(&mut wb, script, request_id, status);
            if query_limit.has_cursor() {
                query_limit.push_where(wb.and());
            }
        }
        query_limit.push_order_by(&mut qb);
        query_limit.push_limit(&mut qb);

        let mut data = qb
            .build_query_as::<CollectorRecordModel>()
            .fetch_all(&self.db)
            .await?;

        let next = query_limit.finalize(&mut data, |d, c| d.id == *c, |d| d.id);

        let mut file_map: std::collections::HashMap<u64, (Option<RecordFileItem>, bool)> =
            std::collections::HashMap::new();

        // ── ATTR：按需批量加载记录的关联文件 ────────────────────────────
        if attr.attr_file == Some(true) && !data.is_empty() {
            // 建立 tag_name → record_id 的映射，避免后续从字符串解析 ID
            let tag_to_record: std::collections::HashMap<String, u64> = data
                .iter()
                .map(|r| (format!("script_record_{}", r.id), r.id))
                .collect();
            let tag_names: Vec<String> = tag_to_record.keys().cloned().collect();
            let tag_refs: Vec<&str> = tag_names.iter().map(String::as_str).collect();

            let file_attr = FileListAttrParam {
                attr_local: attr.attr_file_local,
                ..Default::default()
            };

            // 使用批量查询，每个标签获取最多 1 个文件（实际查询 2 个用于判断是否有更多）
            // 使用脚本的 add_user_id 和 app_id 来过滤，提高查询效率
            let batch_result = self
                .file_dao
                .data_dao()
                .list_files_by_batch_tags(
                    &tag_refs,
                    Some(script.add_user_id),
                    Some(script.app_id),
                    1,
                    &file_attr,
                )
                .await?;

            // 构建 tag_name → (RecordFileItem, has_more) 的映射（不获取 URL）
            for (tag_name, (files, has_more)) in batch_result {
                if let Some(item) = files.first() {
                    let file_item = RecordFileItem {
                        file_id: item.item.file_id,
                        file_name: item.item.file_name.clone(),
                        file_md5: item.item.file_md5.clone(),
                        file_size: item.item.file_size,
                        storage_type: item.item.storage_type.clone(),
                        content_type: item.item.content_type.clone(),
                        file_key: item.file_key.clone(),
                        add_time: item.item.file_ref_add_time,
                        user_id: item.item.user_id,
                        add_user_id: item.item.add_user_id,
                        app_id: item.item.app_id,
                        status: item.item.status,
                        file_ref_status: item.item.file_ref_status,
                        source_url: item.item.source_url.clone(),
                        source_md5: item.item.source_md5.clone(),
                        modify_time: item.item.modify_time,
                        attr_local: item.attr_local.clone(),
                    };

                    // 通过预建的映射获取 record_id
                    if let Some(&record_id) = tag_to_record.get(&tag_name) {
                        file_map.insert(record_id, (Some(file_item), has_more));
                    }
                } else {
                    // 没有文件的情况
                    if let Some(&record_id) = tag_to_record.get(&tag_name) {
                        file_map.insert(record_id, (None, false));
                    }
                }
            }
        }

        // ── 组装结果 ──────────────────────────────────────────────────────────
        let result = data
            .into_iter()
            .map(|record| {
                let (file, has_more_files) = file_map.remove(&record.id).unwrap_or((None, false));
                CollectorRecordItem {
                    record,
                    file,
                    has_more_files,
                }
            })
            .collect();

        Ok((result, next))
    }

    /// 记录总数
    pub async fn count_records(
        &self,
        script: &CollectorScriptModel,
        request_id: Option<&str>,
        status: Option<i8>,
        total_param: &TotalParam,
    ) -> FileManagerResult<TotalRow> {
        let query = total_param.total_count_query();
        let mut qb = if query.is_threshold_mode() {
            QueryBuilder::<MySql>::new(format!(
                "SELECT COUNT(*) FROM (SELECT 1 FROM {}",
                CollectorRecordModel::table_name()
            ))
        } else {
            QueryBuilder::<MySql>::new(format!(
                "SELECT COUNT(*) FROM {}",
                CollectorRecordModel::table_name()
            ))
        };
        {
            let mut wb = WhereClause::new(&mut qb);
            Self::build_record_where(&mut wb, script, request_id, status);
        }
        if query.is_threshold_mode() {
            query.push_limit(&mut qb);
            qb.push(") as t");
        }

        let count = qb
            .build_query_scalar()
            .fetch_one(&self.db)
            .await
            .unwrap_or(0i64);
        Ok(query.finalize(count))
    }

    // ==================== 记录关联文件（按 request_id 查文件） ====================

    /// 查询指定记录关联的文件列表（通过 tag "request_{request_id}" 匹配）
    ///
    /// - `record`: 记录实体
    /// - `page`: CursorPageParam 分页（与 file_dao 一致）
    /// - `app_id`: 用于文件查询的 app_id 过滤（用户级接口传入，系统级可传 None）
    pub async fn list_record_files(
        &self,
        record: &CollectorRecordModel,
        page: &CursorPageParam<u64>,
        app_id: Option<u64>,
    ) -> FileManagerResult<(Vec<RecordFileItem>, CursorPageData<u64>)> {
        let tag_name = format!("script_record_{}", record.id);

        let file_attr = FileListAttrParam {
            attr_local: Some(true),
            attr_oss: Some(false),
            attr_tag_list: None,
            ..Default::default()
        };

        let (files, page_data): (Vec<lsys_file::dao::FileListItemAttrData>, _) = self
            .file_dao
            .data_dao()
            .list_files_by_tag(
                &tag_name,
                Some(record.add_user_id),
                app_id,
                page,
                &file_attr,
            )
            .await?;

        // 不获取 URL，直接构建结果
        let items: Vec<RecordFileItem> = files
            .iter()
            .map(|item| RecordFileItem {
                file_id: item.item.file_id,
                file_name: item.item.file_name.clone(),
                file_md5: item.item.file_md5.clone(),
                file_size: item.item.file_size,
                storage_type: item.item.storage_type.clone(),
                content_type: item.item.content_type.clone(),
                file_key: item.file_key.clone(),
                add_time: item.item.file_ref_add_time,
                user_id: item.item.user_id,
                add_user_id: item.item.add_user_id,
                app_id: item.item.app_id,
                status: item.item.status,
                file_ref_status: item.item.file_ref_status,
                source_url: item.item.source_url.clone(),
                source_md5: item.item.source_md5.clone(),
                modify_time: item.item.modify_time,
                attr_local: item.attr_local.clone(),
            })
            .collect();

        Ok((items, page_data))
    }

    /// 查询指定记录关联的文件总数
    pub async fn count_record_files(
        &self,
        record: &CollectorRecordModel,
        app_id: Option<u64>,
        total_param: &TotalParam,
    ) -> FileManagerResult<TotalRow> {
        let tag_name = format!("script_record_{}", record.id);

        Ok(self
            .file_dao
            .data_dao()
            .count_files_by_tag(&tag_name, Some(record.add_user_id), app_id, total_param)
            .await?)
    }

}
