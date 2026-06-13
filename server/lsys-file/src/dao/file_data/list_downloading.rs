use std::collections::HashSet;

use lsys_core::db::{CursorPageData, CursorPageParam, QueryBuilderExt, TableMeta, WhereClause};
use sqlx::{MySql, QueryBuilder};

use crate::common::FileResult;
use crate::model::*;

use super::{FileDataDao, FileListAttrParam, FileListItem, FileListItemAttrData};

/// 下载中文件列表过滤参数
///
/// 用于查询用户添加的 URL 中尚未完成下载（下载中 + 未开始下载）的文件列表。
/// 通过 `task_data()` 获取正在下载的 file_id，结合数据库查询
/// `source_url` 非空且 `status = Unfinished` 的文件记录。
#[derive(Debug, Default)]
pub struct DownloadingListParam {
    /// 按用户 ID 过滤
    pub user_id: Option<u64>,
    /// 按应用 ID 过滤
    pub app_id: Option<u64>,
    /// 按是否正在下载过滤：
    /// - Some(true): 仅显示正在下载中的文件（从 task_data 拿 ID 去查表）
    /// - Some(false): 仅显示未开始下载（排队等待）的文件
    /// - None: 显示所有未完成下载的文件（下载中 + 未开始）
    pub is_downloading: Option<bool>,
}

/// 下载中文件列表返回结果
#[derive(Debug, Clone)]
pub struct DownloadingListItemData {
    pub item: FileListItemAttrData,
    /// 是否正在下载中（true=下载中，false=未开始下载/排队中）
    pub is_downloading: bool,
}

impl FileDataDao {
    /// 构建下载文件列表的公共 SQL 前缀和 WHERE 条件
    ///
    /// 包含：排除已删除文件、source_url 非空、status = Unfinished、user_id/app_id 过滤
    fn build_downloading_query<'a>(
        wc: &mut WhereClause<'_, 'a, MySql>,
        filter: &'a DownloadingListParam,
    ) {
        // 排除已删除的文件和 file_ref 记录
        wc.and().field_ne("f.status", FileStatus::Deleted as i8);
        wc.and()
            .field_ne("fu.status", FileUserStatus::Deleted as i8);

        // 核心条件：source_url 非空 且 文件状态为 Unfinished
        wc.and().push(" fu.source_url != ''".to_string());
        wc.and().field_eq("f.status", FileStatus::Unfinished as i8);

        // user_id 过滤
        if let Some(uid) = filter.user_id {
            wc.and().field_eq("fu.user_id", uid);
        }

        // app_id 过滤
        if let Some(aid) = filter.app_id {
            wc.and().field_eq("fu.app_id", aid);
        }
    }

    /// 查询未完成下载的文件列表，支持游标分页
    ///
    /// 逻辑：
    /// - `is_downloading = Some(true)`: 先从 `task_data()` 获取正在下载的 file_id 集合，
    ///   再用这些 ID + 其他过滤参数查询数据库，确保只返回正在下载的记录
    /// - `is_downloading = Some(false)`: 从数据库查询所有未完成的文件，排除正在下载的
    /// - `is_downloading = None`: 从数据库查询所有未完成的文件，标记是否正在下载
    pub async fn list_downloading_files(
        &self,
        filter: &DownloadingListParam,
        page: &CursorPageParam<u64>,
        attr_param: &FileListAttrParam,
    ) -> FileResult<(Vec<DownloadingListItemData>, CursorPageData<u64>)> {
        // 获取正在执行的下载任务 file_id 集合
        let downloading_file_ids: HashSet<u64> =
            match self.download_manager.task_dispatch.task_data().await {
                Ok(task_map) => task_map.keys().map(|id| id.file_id).collect(),
                Err(_) => HashSet::new(),
            };

        if filter.is_downloading == Some(true) {
            // 仅查正在下载的：用 task_data 中的 file_id 去表里查记录
            if downloading_file_ids.is_empty() {
                return Ok((vec![], CursorPageData::default()));
            }

            let downloading_ids: Vec<u64> = downloading_file_ids.iter().copied().collect();
            self.query_downloading_from_ids(
                filter,
                page,
                attr_param,
                &downloading_ids,
                &downloading_file_ids,
            )
            .await
        } else {
            // 从表里查所有未完成的记录，再根据 task_data 判断/过滤
            self.query_downloading_from_db(filter, page, attr_param, &downloading_file_ids)
                .await
        }
    }

    /// is_downloading=true 路径：用 task_data 中的 file_id 去表里查记录
    async fn query_downloading_from_ids(
        &self,
        filter: &DownloadingListParam,
        page: &CursorPageParam<u64>,
        attr_param: &FileListAttrParam,
        downloading_ids: &[u64],
        downloading_set: &HashSet<u64>,
    ) -> FileResult<(Vec<DownloadingListItemData>, CursorPageData<u64>)> {
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

        Self::build_downloading_query(&mut wc, filter);

        // 限定 file_id 为正在下载的
        wc.and().field_in_copied("f.id", downloading_ids);

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
        let attr_list = self.assemble_attr_list(data, attr_param).await?;

        let result = attr_list
            .into_iter()
            .map(|item| DownloadingListItemData {
                is_downloading: downloading_set.contains(&item.item.file_id),
                item,
            })
            .collect();

        Ok((result, next))
    }

    /// is_downloading=false/None 路径：从表里查所有未完成记录，再判断/过滤
    async fn query_downloading_from_db(
        &self,
        filter: &DownloadingListParam,
        page: &CursorPageParam<u64>,
        attr_param: &FileListAttrParam,
        downloading_file_ids: &HashSet<u64>,
    ) -> FileResult<(Vec<DownloadingListItemData>, CursorPageData<u64>)> {
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

        Self::build_downloading_query(&mut wc, filter);

        // is_downloading=false 时排除正在下载的 file_id
        if filter.is_downloading == Some(false) && !downloading_file_ids.is_empty() {
            let exclude_ids: Vec<u64> = downloading_file_ids.iter().copied().collect();
            wc.and().field_not_in_copied("f.id", &exclude_ids);
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
        let attr_list = self.assemble_attr_list(data, attr_param).await?;

        let result = attr_list
            .into_iter()
            .map(|item| {
                let is_downloading = downloading_file_ids.contains(&item.item.file_id);
                DownloadingListItemData {
                    item,
                    is_downloading,
                }
            })
            .collect();

        Ok((result, next))
    }
}
