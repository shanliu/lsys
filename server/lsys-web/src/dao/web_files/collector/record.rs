// 执行记录查询 + 计数 + 记录关联文件/日志独立查询

use lsys_core::db::{
    CursorPageData, CursorPageParam, OffsetPageParam, SqlQuote, TableMeta,
};
use lsys_core::sql_format;
use lsys_files::dao::{FileDataListParam, FileListAttrParam};

use crate::dao::result::WebResult;
use crate::model::*;

use super::script::{ScriptFileItem, ScriptFileTag};
use super::WebFileCollector;

/// 记录关联的文件信息（含 URL + tag）
pub type RecordFileItem = ScriptFileItem;

/// 记录关联的文件 tag
pub type RecordFileTag = ScriptFileTag;

impl WebFileCollector {
    /// 构建记录查询的 WHERE 子句
    fn build_record_where(script_id: u64, request_id: Option<&str>, status: Option<i8>) -> String {
        let mut clauses: Vec<String> = vec![sql_format!("script_id={}", script_id)];
        if let Some(rid) = request_id {
            let rid = rid.trim();
            if !rid.is_empty() {
                clauses.push(sql_format!("request_id={}", rid));
            }
        }
        if let Some(s) = status {
            clauses.push(sql_format!("status={}", s));
        }
        clauses.join(" AND ")
    }

    /// 按 request_id 查询记录
    pub async fn find_record_by_request_id(
        &self,
        request_id: &str,
    ) -> WebResult<Option<CollectorRecordModel>> {
        let sql = sql_format!(
            "SELECT * FROM {} WHERE request_id={}",
            CollectorRecordModel::table_name(),
            request_id
        );
        let row = sqlx::query_as::<_, CollectorRecordModel>(&sql)
            .fetch_optional(&self.db)
            .await?;
        Ok(row)
    }

    /// 记录列表（分页），可按 script_id / request_id / status 过滤
    pub async fn list_records(
        &self,
        script_id: u64,
        request_id: Option<&str>,
        status: Option<i8>,
        page: &CursorPageParam<u64>,
    ) -> WebResult<(Vec<CollectorRecordModel>, CursorPageData<u64>)> {
        let where_str = Self::build_record_where(script_id, request_id, status);
        let query_limit = page.page_query("id");
        let suff_sql = query_limit.build_query_sql(Some(&where_str));

        let sql = format!(
            "SELECT * FROM {} {}",
            CollectorRecordModel::table_name().sql_quote(),
            suff_sql
        );

        let mut data = sqlx::query_as::<_, CollectorRecordModel>(&sql)
            .fetch_all(&self.db)
            .await?;

        let next = query_limit.finalize(&mut data, |d, c| d.id == *c, |d| d.id);

        Ok((data, next))
    }

    /// 记录总数
    pub async fn count_records(
        &self,
        script_id: u64,
        request_id: Option<&str>,
        status: Option<i8>,
    ) -> WebResult<u64> {
        let where_str = Self::build_record_where(script_id, request_id, status);
        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE {}",
            CollectorRecordModel::table_name().sql_quote(),
            where_str
        );

        let count = sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.db)
            .await?;
        Ok(count as u64)
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
    ) -> WebResult<(Vec<RecordFileItem>, CursorPageData<u64>)> {
        let tag_name = format!("request_{}", record.request_id);
        let tag_refs: Vec<&str> = vec![&tag_name];

        let file_filter = FileDataListParam {
            app_id,
            tag_any_names: Some(&tag_refs),
            ..Default::default()
        };
        let file_attr = FileListAttrParam {
            attr_local: Some(true),
            attr_oss: Some(false),
            attr_tag: Some(true),
        };

        let (files, page_data) = self
            .file_dao
            .data_dao()
            .list_files(&file_filter, page, &file_attr)
            .await?;

        // 批量获取 URL
        let file_models: Vec<lsys_files::model::FileModel> = files
            .iter()
            .map(|item| lsys_files::model::FileModel {
                id: item.item.id,
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
            .unwrap_or_default();

        let items: Vec<RecordFileItem> = files
            .iter()
            .map(|item| {
                let file_url = url_map.get(&item.item.id).cloned();
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
                    file_id: item.item.id,
                    file_name: item.item.file_name.clone(),
                    file_md5: item.item.file_md5.clone(),
                    file_size: item.item.file_size,
                    storage_type: item.item.storage_type.clone(),
                    content_type: item.item.content_type.clone(),
                    file_url,
                    add_time: item.item.file_user_add_time,
                    tags,
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
    ) -> WebResult<u64> {
        let tag_name = format!("request_{}", record.request_id);
        let tag_refs: Vec<&str> = vec![&tag_name];

        let file_filter = FileDataListParam {
            app_id,
            tag_any_names: Some(&tag_refs),
            ..Default::default()
        };

        let count = self
            .file_dao
            .data_dao()
            .count_files(&file_filter)
            .await?;

        Ok(count as u64)
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
    ) -> WebResult<Vec<CollectorLogModel>> {
        let mut clauses: Vec<String> =
            vec![sql_format!("request_id={}", &record.request_id)];
        if let Some(lv) = level {
            clauses.push(sql_format!("level={}", lv));
        }
        let where_str = clauses.join(" AND ");
        let limit_sql = page.page_query().limit_sql().unwrap_or_default();

        let sql = format!(
            "SELECT * FROM {} WHERE {} ORDER BY id ASC{}",
            CollectorLogModel::table_name().sql_quote(),
            where_str,
            limit_sql
        );

        let data = sqlx::query_as::<_, CollectorLogModel>(&sql)
            .fetch_all(&self.db)
            .await?;

        Ok(data)
    }

    /// 查询指定记录关联的日志总数
    pub async fn count_record_logs(
        &self,
        record: &CollectorRecordModel,
        level: Option<u8>,
    ) -> WebResult<u64> {
        let mut clauses: Vec<String> =
            vec![sql_format!("request_id={}", &record.request_id)];
        if let Some(lv) = level {
            clauses.push(sql_format!("level={}", lv));
        }
        let where_str = clauses.join(" AND ");

        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE {}",
            CollectorLogModel::table_name().sql_quote(),
            where_str
        );

        let count = sqlx::query_scalar::<_, i64>(&sql)
            .fetch_one(&self.db)
            .await?;
        Ok(count as u64)
    }
}
