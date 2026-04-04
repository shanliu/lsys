use std::sync::Arc;

use lsys_core::db::{
    CursorPageData, CursorPageParam, OffsetPageParam, QueryBuilderExt, TableMeta, TotalParam,
    TotalRow, WhereClause,
};
use sqlx::{MySql, QueryBuilder};

use super::file_helpers::FileHelper;
use super::*;
use crate::model::*;

/// 文件列表过滤参数
#[derive(Debug, Default)]
pub struct FileDataListParam<'a> {
    pub local_url: Option<&'a str>,
    pub source_url: Option<&'a str>,
    pub user_id: Option<u64>,
    pub app_id: Option<u64>,
    pub add_time_start: Option<u64>,
    pub add_time_end: Option<u64>,
    pub status: Option<i8>,
    pub storage_type: Option<&'a str>,
    pub file_md5: Option<&'a str>,
    /// 按标签名过滤（AND 语义：文件必须拥有所有指定标签）
    pub tag_names: Option<&'a [&'a str]>,
    /// 按标签名过滤（OR 语义：文件只需拥有任意一个指定标签）
    pub tag_any_names: Option<&'a [&'a str]>,
}

/// 文件列表返回结果 (file join file_user)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct FileListItem {
    // file_user primary key
    pub id: u64,
    // file primary key
    pub file_id: u64,
    // file fields
    pub storage_type: String,
    pub status: i8,
    pub file_name: String,
    pub file_md5: String,
    pub file_size: u64,
    pub modify_time: u64,
    pub content_type: String,
    pub copy_file_id: u64,
    pub from_user_id: u64,
    pub add_time: u64,
    pub change_time: u64,
    // file_user fields
    pub user_id: u64,
    pub add_user_id: u64,
    pub app_id: u64,
    pub file_user_status: i8,
    pub source_url: String,
    pub source_md5: String,
    pub file_user_add_time: u64,
    pub delete_time: u64,
}

/// 文件列表 attr 参数
///
/// 用于指定在列表查询中是否需要查询关联表的详细信息。
/// - attr_local: 为 true 时，对于 storage_type 为 "local" 的文件，查询并返回 file_local 表的关键信息
/// - attr_oss: 为 true 时，对于 storage_type 非 "local" 的文件，查询并返回 file_oss 表的关键信息
/// - attr_tag: 为 true 时，查询并返回该文件关联的所有标签
#[derive(Debug, Default)]
pub struct FileListAttrParam {
    pub attr_local: Option<bool>,
    pub attr_oss: Option<bool>,
    pub attr_tag: Option<bool>,
}

/// 本地文件属性（摊平后的关键数据）
#[derive(Debug, Clone)]
pub struct FileLocalAttr {
    pub id: u64,
    pub source_type: i8,
    pub source_name: String,
    pub oss_file_id: u64,
    pub local_path: String,
    pub file_chunk_total: u32,
    pub file_chunk_succ: u32,
    pub file_chunk_size: u64,
    pub last_error: String,
}

/// OSS 文件属性（摊平后的关键数据）
#[derive(Debug, Clone)]
pub struct FileOssAttr {
    pub id: u64,
    pub object_key: String,
    pub local_file_id: u64,
    pub object_url: String,
    pub bucket: String,
    pub region: String,
    pub size: u64,
    pub last_error: String,
}

/// 文件标签属性
#[derive(Debug, Clone)]
pub struct FileTagAttr {
    pub tags: Vec<FileTagItem>,
}

/// 单个标签信息
#[derive(Debug, Clone)]
pub struct FileTagItem {
    pub tag_name: String,
    pub add_time: u64,
}

/// 文件列表返回结果（包含 attr 属性）
#[derive(Debug, Clone)]
pub struct FileListItemAttr {
    pub item: FileListItem,
    pub attr_local: Option<FileLocalAttr>,
    pub attr_oss: Option<FileOssAttr>,
    pub attr_tag: Option<FileTagAttr>,
}

/// 文件数据查询 DAO（列表、统计等只读查询）
pub struct FileDataDao {
    pub(crate) helper: Arc<FileHelper>,
}

impl FileDataDao {
    pub fn new(helper: Arc<FileHelper>) -> Self {
        Self { helper }
    }

    /// 构建文件列表查询的 WHERE 条件
    ///
    /// 将条件和绑定参数直接推入 QueryBuilder。
    /// 返回 `Ok(false)` 表示 URL 过滤条件匹配不到任何文件，应直接返回空结果。
    /// 返回 `Ok(true)` 表示成功构建 WHERE 条件。
    async fn build_file_list_where(
        &self,
        wc: &mut WhereClause<'_, '_, MySql>,
        filter: &FileDataListParam<'_>,
    ) -> FileResult<bool> {
        // 默认排除已删除的文件和 file_user 记录
        wc.and().field_ne("f.status",FileStatus::Deleted as i8);
        wc.and().field_ne("fu.status",FileUserStatus::Deleted as i8);

        // url 过滤
        if let Some(url) = filter.local_url
            && !url.is_empty() {
                let prefix = &self.helper.config.local_file_url_prefix;
                // Support both prefix-only paths and full URLs that contain the prefix,
                // e.g. "/file/..." or "http://host/file/...". Find the prefix anywhere
                // inside the URL and extract the local_path after it.
                if let Some(pos) = url.find(prefix) {
                    let local_path = &url[pos + prefix.len()..];
                    let rows = sqlx::query_as::<_, FileLocalModel>(
                        &format!(
                            "SELECT * FROM {} WHERE local_path=? LIMIT 100",
                            FileLocalModel::table_name()
                        )
                    )
                    .bind(local_path)
                    .fetch_all(&self.helper.db)
                    .await?;

                    let ids: Vec<u64> = rows.iter().map(|r| r.file_id).collect();
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
                .map(|t| t.trim().to_lowercase())
                .filter(|t| !t.is_empty())
                .collect();
            if !tags.is_empty() {
                wc.and().push(format!(
                    "f.id IN (SELECT file_id FROM {}",
                    FileTagModel::table_name()
                ));
                wc.builder().push_where().field_in_string("tag_name", &tags);
                wc.builder().push_and().field_eq("status", FileTagStatus::Normal as i8);
                if let Some(aid) = filter.app_id {
                    wc.builder().push_and().field_eq("app_id", aid);
                }
                wc.builder().push(" GROUP BY file_id HAVING COUNT(DISTINCT tag_name)=").push_bind(tags.len() as i64).push(")");
            }
        }

        // tag_any_names 过滤 (OR 语义：任意一个标签匹配即可)
        if let Some(tags) = filter.tag_any_names {
            let tags: Vec<String> = tags
                .iter()
                .map(|t| t.trim().to_lowercase())
                .filter(|t| !t.is_empty())
                .collect();
            if !tags.is_empty() {
                wc.and().push(format!(
                    "f.id IN (SELECT DISTINCT file_id FROM {}",
                    FileTagModel::table_name()
                ));
                wc.builder().push_where().field_in_string("tag_name", &tags);
                wc.builder().push_and().field_eq("status", FileTagStatus::Normal as i8);
                if let Some(aid) = filter.app_id {
                    wc.builder().push_and().field_eq("app_id", aid);
                }
                wc.builder().push(")");
            }
        }

        Ok(true)
    }

    /// 文件列表查询
    ///
    /// 此方法基于 FileListAttrParam 参数决定是否查询关联表的详细信息。
    /// 返回数据已将关联表信息摊平到 FileListItemAttr 中。
    pub async fn list_files(
        &self,
        filter: &FileDataListParam<'_>,
        page: &CursorPageParam<u64>,
        attr_param: &FileListAttrParam,
    ) -> FileResult<(Vec<FileListItemAttr>, CursorPageData<u64>)> {
        let query_limit = page.page_query("fu.id");

        let mut qb: QueryBuilder<MySql> = QueryBuilder::new(format!(
            "SELECT fu.id, f.id AS file_id, \
             f.storage_type, f.status, f.file_name, f.file_md5, f.file_size, \
             f.modify_time, f.content_type, f.copy_file_id, f.from_user_id, f.add_time, f.change_time, \
             fu.user_id, fu.add_user_id, fu.app_id, fu.status AS file_user_status, \
             fu.source_url, fu.source_md5, fu.add_time AS file_user_add_time, fu.delete_time \
             FROM {} f INNER JOIN {} fu ON f.id=fu.file_id",
            FileModel::table_name(),
            FileUserModel::table_name()
        ));
        let mut wc = WhereClause::new(&mut qb);

        if !self.build_file_list_where(&mut wc, filter).await? {
            return Ok((vec![], CursorPageData::default()));
        }

        // Cursor pagination conditions
        if query_limit.has_cursor() {
            wc.and();
            query_limit.push_where(wc.builder());
        }
        query_limit.push_order_by(wc.builder());
        query_limit.push_limit(wc.builder());

        let mut data = wc.builder().build_query_as::<FileListItem>()
            .fetch_all(&self.helper.db)
            .await?;

        let next = query_limit.finalize(&mut data, |d, c| d.id == *c, |d| d.id);

        // 收集需要查询的 file_id
        let need_attr_local = attr_param.attr_local.unwrap_or(false);
        let need_attr_oss = attr_param.attr_oss.unwrap_or(false);
        let need_attr_tag = attr_param.attr_tag.unwrap_or(false);

        let mut local_file_ids: Vec<u64> = Vec::new();
        let mut oss_file_ids: Vec<u64> = Vec::new();
        let mut all_file_ids: Vec<u64> = Vec::new();

        for item in &data {
            if need_attr_local && item.storage_type == FileModel::STORAGE_TYPE_LOCAL_PUBLIC {
                local_file_ids.push(item.file_id);
            }
            if need_attr_oss && item.storage_type != FileModel::STORAGE_TYPE_LOCAL_PUBLIC {
                oss_file_ids.push(item.file_id);
            }
            if need_attr_tag {
                all_file_ids.push(item.file_id);
            }
        }

        // 批量查询 file_local 记录
        let mut local_map: std::collections::HashMap<u64, FileLocalAttr> =
            std::collections::HashMap::new();
        if !local_file_ids.is_empty() {
            let mut local_qb: QueryBuilder<MySql> = QueryBuilder::new(format!(
                "SELECT * FROM {}",
                FileLocalModel::table_name()
            ));
            local_qb.push_where().field_in_copied("file_id", &local_file_ids);
            let locals: Vec<FileLocalModel> = local_qb.build_query_as()
                .fetch_all(&self.helper.db)
                .await?;
            for local in locals {
                local_map.insert(
                    local.file_id,
                    FileLocalAttr {
                        id: local.id,
                        source_type: local.source_type,
                        source_name: local.source_name,
                        oss_file_id: local.oss_file_id,
                        local_path: local.local_path,
                        file_chunk_total: local.file_chunk_total,
                        file_chunk_succ: local.file_chunk_succ,
                        file_chunk_size: local.file_chunk_size,
                        last_error: local.last_error,
                    },
                );
            }
        }

        // 批量查询 file_oss 记录
        let mut oss_map: std::collections::HashMap<u64, FileOssAttr> =
            std::collections::HashMap::new();
        if !oss_file_ids.is_empty() {
            let mut oss_qb: QueryBuilder<MySql> = QueryBuilder::new(format!(
                "SELECT * FROM {}",
                FileOssModel::table_name()
            ));
            oss_qb.push_where().field_in_copied("file_id", &oss_file_ids);
            let osses: Vec<FileOssModel> = oss_qb.build_query_as()
                .fetch_all(&self.helper.db)
                .await?;
            for oss in osses {
                oss_map.insert(
                    oss.file_id,
                    FileOssAttr {
                        id: oss.id,
                        object_key: oss.object_key,
                        local_file_id: oss.local_file_id,
                        object_url: oss.object_url,
                        bucket: oss.bucket,
                        region: oss.region,
                        size: oss.size,
                        last_error: oss.last_error,
                    },
                );
            }
        }

        // 批量查询 file_tag 记录
        let mut tag_map: std::collections::HashMap<u64, Vec<FileTagItem>> =
            std::collections::HashMap::new();
        if !all_file_ids.is_empty() {
            let mut tag_qb: QueryBuilder<MySql> = QueryBuilder::new(format!(
                "SELECT * FROM {}",
                FileTagModel::table_name()
            ));
            tag_qb.push_where().field_in_copied("file_id", &all_file_ids);
            tag_qb.push_and().field_eq("status", FileTagStatus::Normal as i8);
            let tags: Vec<FileTagModel> = tag_qb.build_query_as()
                .fetch_all(&self.helper.db)
                .await?;
            for tag in tags {
                tag_map.entry(tag.file_id).or_default().push(FileTagItem {
                    tag_name: tag.tag_name,
                    add_time: tag.add_time,
                });
            }
        }

        // 组合返回结果
        let mut result = Vec::with_capacity(data.len());
        for item in data {
            let attr_local = if need_attr_local {
                local_map.remove(&item.file_id)
            } else {
                None
            };
            let attr_oss = if need_attr_oss {
                oss_map.remove(&item.file_id)
            } else {
                None
            };
            let attr_tag = if need_attr_tag {
                Some(FileTagAttr {
                    tags: tag_map.remove(&item.file_id).unwrap_or_default(),
                })
            } else {
                None
            };

            result.push(FileListItemAttr {
                item,
                attr_local,
                attr_oss,
                attr_tag,
            });
        }

        Ok((result, next))
    }

    /// 文件总数统计
    pub async fn count_files(
        &self,
        filter: &FileDataListParam<'_>,
        total_param: &TotalParam,
    ) -> FileResult<TotalRow> {
        let query = total_param.total_count_query();

        let prefix = if query.is_threshold_mode() {
            // Threshold 模式：用子查询 + LIMIT
            format!(
                "SELECT COUNT(*) FROM (SELECT 1 FROM {} f INNER JOIN {} fu ON f.id=fu.file_id",
                FileModel::table_name(),
                FileUserModel::table_name()
            )
        } else {
            // Full 模式：直接 COUNT(*)，不用子查询
            format!(
                "SELECT COUNT(*) FROM {} f INNER JOIN {} fu ON f.id=fu.file_id",
                FileModel::table_name(),
                FileUserModel::table_name()
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

        let count: (i64,) = wc.builder().build_query_as()
            .fetch_one(&self.helper.db)
            .await?;

        Ok(query.finalize(count.0 as u64))
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
    ///
    /// 用于前端标签抽屉展示。
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

    /// 统计指定文件在指定用户+应用下的标签数量（status=Normal）
    pub async fn count_tags_by_file(
        &self,
        file_id: u64,
        user_id: u64,
        app_id: u64,
    ) -> FileResult<i64> {
        let sql = format!(
            "SELECT COUNT(*) FROM {} WHERE file_id=? AND user_id=? AND app_id=? AND status=?",
            FileTagModel::table_name(),
        );
        let count = sqlx::query_scalar::<_, i64>(&sql)
            .bind(file_id)
            .bind(user_id)
            .bind(app_id)
            .bind(FileTagStatus::Normal as i8)
            .fetch_one(&self.helper.db)
            .await?;
        Ok(count)
    }

    /// 查询某用户某应用下所有标签名（去重）
    ///
    /// - `tag_name_prefix`: 可选的标签名前缀过滤（LIKE 'prefix%'）
    /// - `limit`: 最大返回条数
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
        qb.push_and().field_eq("status", FileTagStatus::Normal as i8);

        if let Some(prefix) = tag_name_prefix {
            let prefix = prefix.trim();
            if !prefix.is_empty() {
                let like_val = format!("{}%", prefix.replace('%', "\\%").replace('_', "\\_"));
                qb.push_and().field_like("tag_name", like_val);
            }
        }

        qb.push(" ORDER BY tag_name ASC LIMIT ").push_bind(limit);

        let rows: Vec<(String,)> = qb.build_query_as().fetch_all(&self.helper.db).await?;

        Ok(rows.into_iter().map(|r| r.0).collect())
    }

    // ==================== 文件日志查询 ====================

    /// 查询文件日志列表（支持分页）
    pub async fn list_logs_by_file_id(
        &self,
        file_id: u64,
        page: &OffsetPageParam,
    ) -> FileResult<Vec<FileLogModel>> {
        let mut qb = QueryBuilder::<MySql>::new(format!(
            "SELECT * FROM {}",
            FileLogModel::table_name(),
        ));
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


    /// 查询文件的所有标签名（status=Normal，去重）
    ///
    /// 用于文件拷贝/同步时获取源文件的标签，以便复制到新文件。
    pub(crate) async fn get_file_tag_names(&self, file_id: u64) -> FileResult<Vec<String>> {
        let sql = format!(
            "SELECT DISTINCT tag_name FROM {} WHERE file_id=? AND status=? ORDER BY tag_name ASC",
            FileTagModel::table_name(),
        );
        let rows: Vec<(String,)> = sqlx::query_as(&sql)
            .bind(file_id)
            .bind(FileTagStatus::Normal as i8)
            .fetch_all(&self.helper.db).await?;
        Ok(rows.into_iter().map(|r| r.0).collect())
    }
}
