use lsys_core::db::{
    CursorPageData, CursorPageParam, OffsetPageParam, QueryBuilderExt, TableMeta, TotalParam,
    TotalRow, WhereClause,
};
use lsys_core::utils::{STRING_CLEAR_FORMAT, StringClear, string_clear};
use sqlx::{MySql, QueryBuilder};

use super::super::file_helpers::FileHelper;
use super::{
    FileDataDao, FileDataListParam, FileListAttrParam, FileListItem, FileListItemAttrData,
};
use crate::common::FileResult;
use crate::model::*;

impl FileDataDao {
    /// 构建文件列表查询的 WHERE 条件
    ///
    /// 将条件和绑定参数直接推入 QueryBuilder。
    /// 返回 `Ok(false)` 表示 URL 过滤条件匹配不到任何文件，应直接返回空结果。
    /// 返回 `Ok(true)` 表示成功构建 WHERE 条件。
    pub(super) async fn build_file_list_where(
        &self,
        wc: &mut WhereClause<'_, '_, MySql>,
        filter: &FileDataListParam<'_>,
    ) -> FileResult<bool> {
        // 默认排除已删除的文件和 file_ref 记录
        wc.and().field_ne("f.status", FileStatus::Deleted as i8);
        wc.and()
            .field_ne("fu.status", FileUserStatus::Deleted as i8);

        // url 过滤
        if let Some(url) = filter.local_url
            && !url.is_empty()
        {
            let prefix = self.runtime_setting.get_local_public_url_prefix().await?;
            // Support both prefix-only paths and full URLs that contain the prefix,
            // e.g. "/files/..." or "http://host/files/...". Find the prefix anywhere
            // inside the URL and extract the local_path after it.
            if let Some(pos) = url.find(&prefix) {
                let mut local_path = &url[pos + prefix.len()..];
                // 去除前导斜杠
                local_path = local_path.trim_start_matches('/');

                let ids: Vec<u64> = sqlx::query_scalar::<_, u64>(&format!(
                    "SELECT file_id FROM {} WHERE local_path=? LIMIT 100",
                    FileLocalModel::table_name()
                ))
                .bind(local_path)
                .fetch_all(&self.helper.db)
                .await?;

                if ids.is_empty() {
                    return Ok(false);
                }
                wc.and().field_in_copied(" f.id", &ids);
            }
        }

        // source_url 过滤
        if let Some(source_url) = filter.source_url {
            let trimmed = source_url.trim();
            if !trimmed.is_empty() {
                let source_md5 = FileHelper::compute_str_md5(trimmed);
                wc.and().field_eq("fu.source_md5", source_md5);
            }
        }

        // user_id 过滤
        if let Some(uid) = filter.user_id {
            wc.and().field_eq("fu.user_id", uid);
        }

        // 时间范围
        if let Some(start) = filter.add_time_start {
            wc.and().field_gte("fu.add_time", start);
        }
        if let Some(end) = filter.add_time_end {
            wc.and().field_lte("fu.add_time", end);
        }

        // 状态
        if let Some(s) = filter.status {
            wc.and().field_eq("f.status", s);
        }

        // storage_type
        if let Some(ref st) = filter.storage_type {
            wc.and().field_eq("f.storage_type", st.to_string());
        }

        // file_md5
        if let Some(ref md5) = filter.file_md5 {
            wc.and().field_eq("f.file_md5", md5.to_string());
        }

        // app_id
        if let Some(aid) = filter.app_id {
            wc.and().field_eq("fu.app_id", aid);
        }

        // tag_names 过滤 (AND 语义)
        if let Some(tags) = filter.tag_names {
            let tags: Vec<String> = tags
                .iter()
                .map(|t| string_clear(t, StringClear::Option(STRING_CLEAR_FORMAT), Some(200)))
                .filter(|t| !t.is_empty())
                .collect();
            if !tags.is_empty() {
                wc.and().push(format!(
                    "f.id IN (SELECT file_id FROM {}",
                    FileTagModel::table_name()
                ));
                wc.builder().push_where().field_in_string("tag_name", &tags);
                wc.builder()
                    .push_and()
                    .field_eq("status", FileTagStatus::Normal as i8);
                if let Some(uid) = filter.user_id {
                    wc.builder().push_and().field_eq("user_id", uid);
                }
                if let Some(aid) = filter.app_id {
                    wc.builder().push_and().field_eq("app_id", aid);
                }
                wc.builder()
                    .push(" GROUP BY file_id HAVING COUNT(DISTINCT tag_name)=")
                    .push_bind(tags.len() as i64)
                    .push(")");
            }
        }

        Ok(true)
    }

    /// 文件列表查询
    pub async fn list_files(
        &self,
        filter: &FileDataListParam<'_>,
        page: &CursorPageParam<u64>,
        attr_param: &FileListAttrParam,
    ) -> FileResult<(Vec<FileListItemAttrData>, CursorPageData<u64>)> {
        let query_limit = page.page_query("fu.id");

        let mut qb: QueryBuilder<MySql> = QueryBuilder::new(format!(
            "SELECT fu.id, f.id AS file_id, \
             f.storage_type, f.status, f.file_md5, f.file_size, \
             f.modify_time, f.content_type, f.from_user_id, f.add_time, f.change_time, \
             fu.user_id, fu.add_user_id, fu.app_id, fu.status AS file_ref_status, \
             fu.source_url, fu.source_md5, fu.add_time AS file_ref_add_time, fu.delete_time, fu.file_name \
             FROM {} f INNER JOIN {} fu ON f.id=fu.file_id",
            FileModel::table_name(),
            FileRefModel::table_name()
        ));
        let mut wc = WhereClause::new(&mut qb);

        if !self.build_file_list_where(&mut wc, filter).await? {
            return Ok((vec![], CursorPageData::default()));
        }

        if query_limit.has_cursor() {
            wc.and();
            query_limit.push_where(wc.builder());
        }
        query_limit.push_order_by(wc.builder());
        query_limit.push_limit(wc.builder());

        let mut data = wc
            .builder()
            .build_query_as::<FileListItem>()
            .fetch_all(&self.helper.db)
            .await?;

        let next = query_limit.finalize(&mut data, |d, c| d.id == *c, |d| d.id);
        let result = self.assemble_attr_list(data, attr_param).await?;
        Ok((result, next))
    }

    pub async fn count_files(
        &self,
        filter: &FileDataListParam<'_>,
        total_param: &TotalParam,
    ) -> FileResult<TotalRow> {
        let query = total_param.total_count_query();

        let prefix = if query.is_threshold_mode() {
            format!(
                "SELECT COUNT(*) FROM (SELECT 1 FROM {} f INNER JOIN {} fu ON f.id=fu.file_id",
                FileModel::table_name(),
                FileRefModel::table_name()
            )
        } else {
            format!(
                "SELECT COUNT(*) FROM {} f INNER JOIN {} fu ON f.id=fu.file_id",
                FileModel::table_name(),
                FileRefModel::table_name()
            )
        };

        let mut qb: QueryBuilder<MySql> = QueryBuilder::new(prefix);
        let mut wc = WhereClause::new(&mut qb);

        if !self.build_file_list_where(&mut wc, filter).await? {
            return Ok(TotalRow::Exact(0));
        }

        if query.is_threshold_mode() {
            query.push_limit(wc.builder());
            wc.builder().push(") as t");
        }

        let count: i64 = wc
            .builder()
            .build_query_scalar()
            .fetch_one(&self.helper.db)
            .await?;

        Ok(query.finalize(count))
    }

    // ==================== 文件分片查询 ====================

    /// 查询文件分片列表（支持分页）
    pub async fn list_chunks_by_file_id(
        &self,
        file_id: u64,
        page: &OffsetPageParam,
    ) -> FileResult<Vec<FileLocalChunkModel>> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "SELECT * FROM {}",
            FileLocalChunkModel::table_name(),
        ));
        qb.push_where().field_eq("file_id", file_id);
        qb.push(" ORDER BY chunk_index DESC");
        page.push_limit(&mut qb);
        qb.build_query_as::<FileLocalChunkModel>()
            .fetch_all(&self.helper.db)
            .await
            .map_err(Into::into)
    }

    /// 查询文件分片总数
    pub async fn count_chunks_by_file_id(&self, file_id: u64) -> FileResult<i64> {
        let sql = format!(
            "SELECT COUNT(*) as count FROM {} WHERE file_id=?",
            FileLocalChunkModel::table_name(),
        );
        sqlx::query_scalar(&sql)
            .bind(file_id)
            .fetch_one(&self.helper.db)
            .await
            .map_err(Into::into)
    }

    /// 查询指定文件在指定用户+应用下的所有标签（status=Normal），按添加时间升序
    pub async fn list_tags_by_file(
        &self,
        file_id: u64,
        user_id: u64,
        app_id: u64,
    ) -> FileResult<Vec<FileTagModel>> {
        let sql = format!(
            "SELECT * FROM {} WHERE file_id=? AND user_id=? AND app_id=? AND status=? ORDER BY add_time ASC",
            FileTagModel::table_name(),
        );
        let rows = sqlx::query_as::<_, FileTagModel>(&sql)
            .bind(file_id)
            .bind(user_id)
            .bind(app_id)
            .bind(FileTagStatus::Normal as i8)
            .fetch_all(&self.helper.db)
            .await?;
        Ok(rows)
    }

    /// 查询某用户某应用下所有标签名（去重）
    pub async fn list_tag_names_by_user(
        &self,
        user_id: u64,
        app_id: u64,
        tag_name_prefix: Option<&str>,
        limit: u32,
    ) -> FileResult<Vec<String>> {
        let mut qb: QueryBuilder<MySql> = QueryBuilder::new(format!(
            "SELECT DISTINCT tag_name FROM {}",
            FileTagModel::table_name(),
        ));
        qb.push_where().field_eq("user_id", user_id);
        qb.push_and().field_eq("app_id", app_id);
        qb.push_and()
            .field_eq("status", FileTagStatus::Normal as i8);

        if let Some(prefix) = tag_name_prefix {
            let prefix = prefix.trim();
            if !prefix.is_empty() {
                let like_val = string_clear(prefix, StringClear::LikeKeyWord, Some(200));
                qb.push_and()
                    .field_like("tag_name", format!("{}%", like_val));
            }
        }

        qb.push(" ORDER BY tag_name ASC LIMIT ").push_bind(limit);

        let rows: Vec<String> = qb.build_query_scalar().fetch_all(&self.helper.db).await?;
        Ok(rows)
    }

    // ==================== 文件日志查询 ====================

    /// 查询文件日志列表（支持分页）
    pub async fn list_logs_by_file_id(
        &self,
        file_id: u64,
        page: &OffsetPageParam,
    ) -> FileResult<Vec<FileLogModel>> {
        let mut qb =
            QueryBuilder::<MySql>::new(format!("SELECT * FROM {}", FileLogModel::table_name()));
        qb.push_where().field_eq("file_id", file_id);
        qb.push(" ORDER BY id DESC");
        page.push_limit(&mut qb);
        qb.build_query_as::<FileLogModel>()
            .fetch_all(&self.helper.db)
            .await
            .map_err(Into::into)
    }

    /// 查询文件日志总数
    pub async fn count_logs_by_file_id(&self, file_id: u64) -> FileResult<i64> {
        let sql = format!(
            "SELECT COUNT(*) as count FROM {} WHERE file_id=?",
            FileLogModel::table_name(),
        );
        sqlx::query_scalar(&sql)
            .bind(file_id)
            .fetch_one(&self.helper.db)
            .await
            .map_err(Into::into)
    }
}
