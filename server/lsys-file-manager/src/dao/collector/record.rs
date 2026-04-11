// 执行记录查询 + 计数 + 记录关联文件/日志独立查询

use lsys_core::db::{
    CursorPageData, CursorPageParam, OffsetPageParam, QueryBuilderExt, TableMeta, TotalParam,
    TotalRow, WhereClause,
};
use lsys_core::utils::{StringClear, string_clear};
use lsys_file::dao::FileListAttrParam;
use sqlx::{MySql, QueryBuilder};

use crate::dao::result::FileManagerResult;
use crate::model::*;

use super::FileCollector;
use super::script::{ScriptFileItem, ScriptFileTag};

/// 记录关联的文件信息（含 URL + tag）
pub type RecordFileItem = ScriptFileItem;

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
                attr_oss: attr.attr_file_oss,
                attr_tag: attr.attr_file_tag,
            };

            // 使用批量查询，每个标签获取最多 1 个文件（实际查询 2 个用于判断是否有更多）
            // 使用脚本的 add_user_id 和 app_id 来过滤，提高查询效率
            let batch_result = self
                .file_dao
                .data_dao()
                .batch_list_files_by_tags(
                    &tag_refs,
                    Some(script.add_user_id),
                    Some(script.app_id),
                    1,
                    &file_attr,
                )
                .await?;

            // 批量获取 URL
            let all_file_models: Vec<lsys_file::model::FileModel> = batch_result
                .values()
                .filter_map(|(files, _)| files.first())
                .map(|item| lsys_file::model::FileModel {
                    id: item.item.file_id,
                    storage_type: item.item.storage_type.clone(),
                    status: item.item.status,
                    file_name: item.item.file_name.clone(),
                    file_md5: item.item.file_md5.clone(),
                    file_size: item.item.file_size,
                    modify_time: item.item.modify_time,
                    content_type: item.item.content_type.clone(),
                    copy_file_id: item.item.copy_file_id,
                    from_user_id: item.item.from_user_id,
                    add_time: item.item.add_time,
                    change_time: item.item.change_time,
                })
                .collect();

            let url_map = self
                .file_dao
                .get_file_urls(&all_file_models)
                .await
                .unwrap_or_else(|_| std::collections::HashMap::new());

            // 构建 tag_name → (RecordFileItem, has_more) 的映射
            for (tag_name, (files, has_more)) in batch_result {
                if let Some(item) = files.first() {
                    let file_url = url_map.get(&item.item.file_id).cloned();
                    let tags: Vec<RecordFileTag> = item
                        .attr_tag
                        .as_ref()
                        .map(|t| {
                            t.tags
                                .iter()
                                .map(|tag| RecordFileTag {
                                    tag_name: tag.tag_name.clone(),
                                    add_time: tag.add_time,
                                })
                                .collect()
                        })
                        .unwrap_or_default();

                    let file_item = RecordFileItem {
                        file_id: item.item.file_id,
                        file_name: item.item.file_name.clone(),
                        file_md5: item.item.file_md5.clone(),
                        file_size: item.item.file_size,
                        storage_type: item.item.storage_type.clone(),
                        content_type: item.item.content_type.clone(),
                        file_url,
                        add_time: item.item.file_user_add_time,
                        tags,
                        user_id: item.item.user_id,
                        add_user_id: item.item.add_user_id,
                        app_id: item.item.app_id,
                        status: item.item.status,
                        file_user_status: item.item.file_user_status,
                        source_url: item.item.source_url.clone(),
                        source_md5: item.item.source_md5.clone(),
                        modify_time: item.item.modify_time,
                        attr_local: item.attr_local.clone(),
                        attr_oss: item.attr_oss.clone(),
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
            .unwrap_or(0i64) as u64;
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
            attr_tag: Some(true),
        };

        let (files, page_data): (Vec<lsys_file::dao::FileListItemAttr>, _) = self
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

        // 批量获取 URL
        let file_models: Vec<lsys_file::model::FileModel> = files
            .iter()
            .map(|item| lsys_file::model::FileModel {
                id: item.item.file_id,
                storage_type: item.item.storage_type.clone(),
                status: item.item.status,
                file_name: item.item.file_name.clone(),
                file_md5: item.item.file_md5.clone(),
                file_size: item.item.file_size,
                modify_time: item.item.modify_time,
                content_type: item.item.content_type.clone(),
                copy_file_id: item.item.copy_file_id,
                from_user_id: item.item.from_user_id,
                add_time: item.item.add_time,
                change_time: item.item.change_time,
            })
            .collect();
        let url_map = self
            .file_dao
            .get_file_urls(&file_models)
            .await
            .unwrap_or_else(|_| std::collections::HashMap::new());

        let items: Vec<RecordFileItem> = files
            .iter()
            .map(|item| {
                let file_url = url_map.get(&item.item.file_id).cloned();
                let tags: Vec<RecordFileTag> = item
                    .attr_tag
                    .as_ref()
                    .map(|t| {
                        t.tags
                            .iter()
                            .map(|tag| RecordFileTag {
                                tag_name: tag.tag_name.clone(),
                                add_time: tag.add_time,
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                RecordFileItem {
                    file_id: item.item.file_id,
                    file_name: item.item.file_name.clone(),
                    file_md5: item.item.file_md5.clone(),
                    file_size: item.item.file_size,
                    storage_type: item.item.storage_type.clone(),
                    content_type: item.item.content_type.clone(),
                    file_url,
                    add_time: item.item.file_user_add_time,
                    tags,
                    user_id: item.item.user_id,
                    add_user_id: item.item.add_user_id,
                    app_id: item.item.app_id,
                    status: item.item.status,
                    file_user_status: item.item.file_user_status,
                    source_url: item.item.source_url.clone(),
                    source_md5: item.item.source_md5.clone(),
                    modify_time: item.item.modify_time,
                    attr_local: item.attr_local.clone(),
                    attr_oss: item.attr_oss.clone(),
                }
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

    // ==================== 记录关联日志（按 request_id 查日志） ====================

    /// 查询指定记录关联的日志列表（OffsetPageParam 分页）
    ///
    /// - `record`: 记录实体
    /// - `level`: 可选日志级别过滤
    /// - `page`: OffsetPageParam 分页
    pub async fn list_record_logs(
        &self,
        record: &CollectorRecordModel,
        level: Option<u8>,
        page: &OffsetPageParam,
    ) -> FileManagerResult<Vec<CollectorLogModel>> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "SELECT * FROM {}",
            CollectorLogModel::table_name()
        ));
        qb.push_where()
            .field_eq("request_id", record.request_id.to_owned());
        if let Some(lv) = level {
            qb.push_and().field_eq("level", lv);
        }
        qb.push(" ORDER BY id ASC");
        page.push_limit(&mut qb);

        let data = qb
            .build_query_as::<CollectorLogModel>()
            .fetch_all(&self.db)
            .await?;

        Ok(data)
    }

    /// 查询指定记录关联的日志总数
    pub async fn count_record_logs(
        &self,
        record: &CollectorRecordModel,
        level: Option<u8>,
    ) -> FileManagerResult<u64> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "SELECT COUNT(*) FROM {}",
            CollectorLogModel::table_name()
        ));
        qb.push_where()
            .field_eq("request_id", record.request_id.to_owned());
        if let Some(lv) = level {
            qb.push_and().field_eq("level", lv);
        }

        let count = qb
            .build_query_scalar()
            .fetch_one(&self.db)
            .await
            .unwrap_or(0i64);
        Ok(count as u64)
    }
}
