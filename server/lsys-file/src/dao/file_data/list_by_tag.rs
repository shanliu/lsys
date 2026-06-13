use lsys_core::db::{
    CursorPageData, CursorPageParam, QueryBuilderExt, TableMeta, TotalParam, TotalRow, WhereClause,
};
use lsys_core::utils::{STRING_CLEAR_FORMAT, StringClear, string_clear};
use sqlx::{MySql, QueryBuilder};

use super::{
    FileDataDao, FileLineageAttrData, FileListAttrParam, FileListItem, FileListItemAttrData,
    FileTagAttrData,
};
use crate::common::FileResult;
use crate::model::*;

impl FileDataDao {
    /// 构建按标签查询文件的通用 WHERE 条件
    pub(super) fn build_tag_filter_where(
        wc: &mut WhereClause<'_, '_, MySql>,
        tag_name: &str,
        user_id: Option<u64>,
        app_id: Option<u64>,
    ) {
        wc.and().field_ne("f.status", FileStatus::Deleted as i8);
        wc.and()
            .field_ne("fu.status", FileUserStatus::Deleted as i8);
        wc.and().field_eq("ft.status", FileTagStatus::Normal as i8);

        let tag_name = string_clear(
            tag_name,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(200),
        );
        wc.and().field_eq("ft.tag_name", tag_name.to_string());

        if let Some(uid) = user_id {
            wc.and().field_eq("fu.user_id", uid);
            wc.and().field_eq("ft.user_id", uid);
        }
        if let Some(aid) = app_id {
            wc.and().field_eq("fu.app_id", aid);
            wc.and().field_eq("ft.app_id", aid);
        }
    }

    /// 查询拥有指定标签的文件列表（支持分页）
    pub async fn list_files_by_tag(
        &self,
        tag_name: &str,
        user_id: Option<u64>,
        app_id: Option<u64>,
        page: &CursorPageParam<u64>,
        attr_param: &FileListAttrParam,
    ) -> FileResult<(Vec<FileListItemAttrData>, CursorPageData<u64>)> {
        let tag_name = string_clear(
            tag_name,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(200),
        );
        if tag_name.is_empty() {
            return Ok((vec![], CursorPageData::default()));
        }

        let query_limit = page.page_query("fu.id");

        let mut qb: QueryBuilder<MySql> = QueryBuilder::new(format!(
            "SELECT fu.id, f.id AS file_id, \
             f.storage_type, f.status, f.file_md5, f.file_size, \
             f.modify_time, f.content_type, f.from_user_id, f.add_time, f.change_time, \
             fu.user_id, fu.add_user_id, fu.app_id, fu.status AS file_ref_status, \
             fu.source_url, fu.source_md5, fu.add_time AS file_ref_add_time, fu.delete_time, fu.file_name \
             FROM {} f \
             INNER JOIN {} fu ON f.id=fu.file_id \
             INNER JOIN {} ft ON f.id=ft.file_id",
            FileModel::table_name(),
            FileRefModel::table_name(),
            FileTagModel::table_name()
        ));

        let mut wc = WhereClause::new(&mut qb);
        Self::build_tag_filter_where(&mut wc, &tag_name, user_id, app_id);

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

    /// 统计拥有指定标签的文件总数
    pub async fn count_files_by_tag(
        &self,
        tag_name: &str,
        user_id: Option<u64>,
        app_id: Option<u64>,
        total_param: &TotalParam,
    ) -> FileResult<TotalRow> {
        let tag_name = string_clear(
            tag_name,
            StringClear::Option(STRING_CLEAR_FORMAT),
            Some(200),
        );
        if tag_name.is_empty() {
            return Ok(TotalRow::Exact(0));
        }

        let query = total_param.total_count_query();

        let prefix = if query.is_threshold_mode() {
            format!(
                "SELECT COUNT(*) FROM (SELECT 1 FROM {} f \
                 INNER JOIN {} fu ON f.id=fu.file_id \
                 INNER JOIN {} ft ON f.id=ft.file_id",
                FileModel::table_name(),
                FileRefModel::table_name(),
                FileTagModel::table_name()
            )
        } else {
            format!(
                "SELECT COUNT(*) FROM {} f \
                 INNER JOIN {} fu ON f.id=fu.file_id \
                 INNER JOIN {} ft ON f.id=ft.file_id",
                FileModel::table_name(),
                FileRefModel::table_name(),
                FileTagModel::table_name()
            )
        };

        let mut qb: QueryBuilder<MySql> = QueryBuilder::new(prefix);
        let mut wc = WhereClause::new(&mut qb);
        Self::build_tag_filter_where(&mut wc, &tag_name, user_id, app_id);

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

    /// 批量查询多个标签的文件记录
    ///
    /// 为每个标签返回最多 N 个文件记录，以及是否有更多记录的标记。
    pub async fn list_files_by_batch_tags(
        &self,
        tag_names: &[&str],
        user_id: Option<u64>,
        app_id: Option<u64>,
        limit: u32,
        attr_param: &FileListAttrParam,
    ) -> FileResult<std::collections::HashMap<String, (Vec<FileListItemAttrData>, bool)>> {
        let mut result: std::collections::HashMap<String, (Vec<FileListItemAttrData>, bool)> =
            std::collections::HashMap::new();

        let normalized_tags: Vec<String> = tag_names
            .iter()
            .map(|t| string_clear(t, StringClear::Option(STRING_CLEAR_FORMAT), Some(200)))
            .filter(|t| !t.is_empty())
            .collect();

        if normalized_tags.is_empty() {
            return Ok(result);
        }

        let mut qb: QueryBuilder<MySql> = QueryBuilder::new("");

        for (idx, tag_name) in normalized_tags.iter().enumerate() {
            if idx > 0 {
                qb.push(" UNION ALL ");
            }

            qb.push("(SELECT ");
            qb.push_bind(tag_name.clone());
            qb.push(
                " AS tag_name, fu.id, f.id AS file_id, \
                 f.storage_type, f.status, f.file_md5, f.file_size, \
                 f.modify_time, f.content_type, f.from_user_id, f.add_time, f.change_time, \
                 fu.user_id, fu.add_user_id, fu.app_id, fu.status AS file_ref_status, \
                 fu.source_url, fu.source_md5, fu.add_time AS file_ref_add_time, fu.delete_time, fu.file_name \
                 FROM ",
            );
            qb.push(FileModel::table_name());
            qb.push(" f INNER JOIN ");
            qb.push(FileRefModel::table_name());
            qb.push(" fu ON f.id=fu.file_id INNER JOIN ");
            qb.push(FileTagModel::table_name());
            qb.push(" ft ON f.id=ft.file_id WHERE ");

            qb.push("f.status!=");
            qb.push_bind(FileStatus::Deleted as i8);
            qb.push(" AND fu.status!=");
            qb.push_bind(FileUserStatus::Deleted as i8);
            qb.push(" AND ft.status=");
            qb.push_bind(FileTagStatus::Normal as i8);
            qb.push(" AND ft.tag_name=");
            qb.push_bind(tag_name.clone());

            if let Some(uid) = user_id {
                qb.push(" AND fu.user_id=");
                qb.push_bind(uid);
                qb.push(" AND ft.user_id=");
                qb.push_bind(uid);
            }
            if let Some(aid) = app_id {
                qb.push(" AND fu.app_id=");
                qb.push_bind(aid);
                qb.push(" AND ft.app_id=");
                qb.push_bind(aid);
            }

            qb.push(" ORDER BY fu.id DESC LIMIT ");
            qb.push_bind((limit + 1) as i64);
            qb.push(")");
        }

        #[derive(sqlx::FromRow)]
        struct FileListItemWithTag {
            tag_name: String,
            id: u64,
            file_id: u64,
            storage_type: String,
            status: i8,
            file_md5: String,
            file_size: u64,
            modify_time: u64,
            content_type: String,
            from_user_id: u64,
            add_time: u64,
            change_time: u64,
            user_id: u64,
            add_user_id: u64,
            app_id: u64,
            file_ref_status: i8,
            source_url: String,
            source_md5: String,
            file_ref_add_time: u64,
            delete_time: u64,
            file_name: String,
        }

        let all_data: Vec<FileListItemWithTag> =
            qb.build_query_as().fetch_all(&self.helper.db).await?;

        let mut tag_data_map: std::collections::HashMap<String, Vec<FileListItem>> =
            std::collections::HashMap::new();

        for item in all_data {
            let tag_name = item.tag_name.clone();
            let file_item = FileListItem {
                id: item.id,
                file_id: item.file_id,
                storage_type: item.storage_type,
                status: item.status,
                file_md5: item.file_md5,
                file_size: item.file_size,
                modify_time: item.modify_time,
                content_type: item.content_type,
                from_user_id: item.from_user_id,
                add_time: item.add_time,
                change_time: item.change_time,
                user_id: item.user_id,
                add_user_id: item.add_user_id,
                app_id: item.app_id,
                file_ref_status: item.file_ref_status,
                source_url: item.source_url,
                source_md5: item.source_md5,
                file_ref_add_time: item.file_ref_add_time,
                delete_time: item.delete_time,
                file_name: item.file_name,
            };
            tag_data_map.entry(tag_name).or_default().push(file_item);
        }

        let need_local = attr_param.attr_local.unwrap_or(false);
        let need_oss = attr_param.attr_oss.unwrap_or(false);
        let tag_limit = attr_param.attr_tag_list;
        let need_tag_count = attr_param.attr_tag_count.unwrap_or(false);
        let need_tag = tag_limit.map(|l| l > 0).unwrap_or(false) || need_tag_count;
        let need_lineage = attr_param.attr_lineage.unwrap_or(false);

        let all_items: Vec<&FileListItem> = tag_data_map.values().flatten().collect();

        let local_ids: Vec<u64> = if need_local {
            let mut v: Vec<u64> = all_items
                .iter()
                .filter(|d| FileModel::is_local_key(&d.storage_type))
                .map(|d| d.file_id)
                .collect();
            v.sort_unstable();
            v.dedup();
            v
        } else {
            vec![]
        };
        let oss_ids: Vec<u64> = if need_oss {
            let mut v: Vec<u64> = all_items
                .iter()
                .filter(|d| !FileModel::is_local_key(&d.storage_type))
                .map(|d| d.file_id)
                .collect();
            v.sort_unstable();
            v.dedup();
            v
        } else {
            vec![]
        };
        let (shared_fids, shared_uids, shared_aids) = if need_tag || need_lineage {
            let mut fids: Vec<u64> = all_items.iter().map(|d| d.file_id).collect();
            let mut uids: Vec<u64> = all_items.iter().map(|d| d.user_id).collect();
            let mut aids: Vec<u64> = all_items.iter().map(|d| d.app_id).collect();
            fids.sort_unstable();
            fids.dedup();
            uids.sort_unstable();
            uids.dedup();
            aids.sort_unstable();
            aids.dedup();
            (fids, uids, aids)
        } else {
            (vec![], vec![], vec![])
        };

        let local_map = self.batch_load_local(&local_ids).await?;
        let oss_map = self.batch_load_oss(&oss_ids).await?;
        let tag_map = match tag_limit {
            Some(l) if l > 0 => {
                self.batch_load_tags(&shared_fids, &shared_uids, &shared_aids, l)
                    .await?
            }
            _ => Default::default(),
        };
        let tag_count_map = if need_tag_count {
            self.batch_load_tag_counts(&shared_fids, &shared_uids, &shared_aids)
                .await?
        } else {
            Default::default()
        };
        let lineage_map = if need_lineage {
            self.batch_load_lineage_counts(&shared_fids, &shared_uids, &shared_aids)
                .await?
        } else {
            Default::default()
        };

        for (tag_name, mut data) in tag_data_map {
            let has_more = data.len() > limit as usize;
            if has_more {
                data.truncate(limit as usize);
            }
            let items = data
                .into_iter()
                .map(|item| FileListItemAttrData {
                    attr_local: need_local
                        .then(|| local_map.get(&item.file_id).cloned())
                        .flatten(),
                    attr_oss: need_oss
                        .then(|| oss_map.get(&item.file_id).cloned())
                        .flatten(),
                    attr_tag: need_tag.then(|| FileTagAttrData {
                        tags: tag_map
                            .get(&(item.file_id, item.user_id, item.app_id))
                            .cloned()
                            .unwrap_or_default(),
                        count: tag_count_map
                            .get(&(item.file_id, item.user_id, item.app_id))
                            .copied(),
                    }),
                    attr_lineage: need_lineage.then(|| FileLineageAttrData {
                        counts: lineage_map
                            .get(&(item.file_id, item.user_id, item.app_id))
                            .cloned()
                            .unwrap_or_default(),
                    }),
                    attr_url_downloading: None,
                    file_key: self.file_key_encoder.encode(item.id, None),
                    item,
                })
                .collect();
            result.insert(tag_name, (items, has_more));
        }

        Ok(result)
    }
}
