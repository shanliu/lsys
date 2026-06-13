use lsys_core::db::{QueryBuilderExt, TableMeta};
use sqlx::{MySql, QueryBuilder};
use std::collections::HashSet;

use super::{
    FileDataDao, FileLineageAttrData, FileLineageCountItem, FileListAttrParam, FileListItem,
    FileListItemAttrData, FileLocalAttrData, FileOssAttrData, FileTagAttrData, FileTagItem,
};
use crate::common::FileResult;
use crate::model::*;

impl FileDataDao {
    /// 批量查询文件关联（lineage）统计：按 (src_file_id, user_id, app_id, rel_type, storage_type) 分组计数
    pub(super) async fn batch_load_lineage_counts(
        &self,
        file_ids: &[u64],
        user_ids: &[u64],
        app_ids: &[u64],
    ) -> FileResult<std::collections::HashMap<(u64, u64, u64), Vec<FileLineageCountItem>>> {
        let mut map: std::collections::HashMap<(u64, u64, u64), Vec<FileLineageCountItem>> =
            std::collections::HashMap::new();
        if file_ids.is_empty() {
            return Ok(map);
        }

        #[derive(sqlx::FromRow)]
        struct LineageCountRow {
            src_file_id: u64,
            user_id: u64,
            app_id: u64,
            rel_type: i8,
            storage_type: String,
            cnt: i64,
        }

        let mut qb: QueryBuilder<MySql> = QueryBuilder::new(format!(
            "SELECT fl.src_file_id, fl.user_id, fl.app_id, fl.rel_type, f.storage_type, COUNT(*) as cnt \
             FROM {} fl INNER JOIN {} f ON fl.dst_file_id = f.id",
            FileLineageModel::table_name(),
            FileModel::table_name()
        ));
        qb.push_where().field_in_copied("fl.src_file_id", file_ids);
        qb.push_and()
            .field_eq("fl.status", FileLineageStatus::Normal as i8);
        if !user_ids.is_empty() {
            qb.push_and().field_in_copied("fl.user_id", user_ids);
        }
        if !app_ids.is_empty() {
            qb.push_and().field_in_copied("fl.app_id", app_ids);
        }
        qb.push(" GROUP BY fl.src_file_id, fl.user_id, fl.app_id, fl.rel_type, f.storage_type");

        let rows: Vec<LineageCountRow> = qb.build_query_as().fetch_all(&self.helper.db).await?;

        for row in rows {
            map.entry((row.src_file_id, row.user_id, row.app_id))
                .or_default()
                .push(FileLineageCountItem {
                    rel_type: row.rel_type,
                    storage_type: row.storage_type,
                    count: row.cnt,
                });
        }

        Ok(map)
    }

    /// 批量加载 local 文件属性
    pub(super) async fn batch_load_local(
        &self,
        file_ids: &[u64],
    ) -> FileResult<std::collections::HashMap<u64, FileLocalAttrData>> {
        let mut map = std::collections::HashMap::new();
        if file_ids.is_empty() {
            return Ok(map);
        }
        let mut qb: QueryBuilder<MySql> =
            QueryBuilder::new(format!("SELECT * FROM {}", FileLocalModel::table_name()));
        qb.push_where().field_in_copied("file_id", file_ids);
        let locals: Vec<FileLocalModel> = qb.build_query_as().fetch_all(&self.helper.db).await?;
        for local in locals {
            map.insert(
                local.file_id,
                FileLocalAttrData {
                    id: local.id,
                    source_type: local.source_type,
                    source_name: local.source_name,
                    local_path: local.local_path,
                    file_chunk_total: local.file_chunk_total,
                    file_chunk_succ: local.file_chunk_succ,
                    file_chunk_size: local.file_chunk_size,
                    last_error: local.last_error,
                },
            );
        }
        Ok(map)
    }

    /// 批量加载 OSS 文件属性
    pub(super) async fn batch_load_oss(
        &self,
        file_ids: &[u64],
    ) -> FileResult<std::collections::HashMap<u64, FileOssAttrData>> {
        let mut map = std::collections::HashMap::new();
        if file_ids.is_empty() {
            return Ok(map);
        }
        let mut qb: QueryBuilder<MySql> =
            QueryBuilder::new(format!("SELECT * FROM {}", FileOssModel::table_name()));
        qb.push_where().field_in_copied("file_id", file_ids);
        let osses: Vec<FileOssModel> = qb.build_query_as().fetch_all(&self.helper.db).await?;
        for oss in osses {
            map.insert(
                oss.file_id,
                FileOssAttrData {
                    id: oss.id,
                    object_key: oss.object_key,
                    object_url: oss.object_url,
                    bucket: oss.bucket,
                    region: oss.region,
                    size: oss.size,
                    last_error: oss.last_error,
                },
            );
        }
        Ok(map)
    }

    /// 批量加载文件标签列表，按 (file_id, user_id, app_id) 分组，最多返回 limit 条
    pub(super) async fn batch_load_tags(
        &self,
        file_ids: &[u64],
        user_ids: &[u64],
        app_ids: &[u64],
        limit: u32,
    ) -> FileResult<std::collections::HashMap<(u64, u64, u64), Vec<FileTagItem>>> {
        let mut map = std::collections::HashMap::new();
        if file_ids.is_empty() || limit == 0 {
            return Ok(map);
        }
        let mut qb: QueryBuilder<MySql> =
            QueryBuilder::new(format!("SELECT * FROM {}", FileTagModel::table_name()));
        qb.push_where().field_in_copied("file_id", file_ids);
        qb.push_and()
            .field_eq("status", FileTagStatus::Normal as i8);
        if !user_ids.is_empty() {
            qb.push_and().field_in_copied("user_id", user_ids);
        }
        if !app_ids.is_empty() {
            qb.push_and().field_in_copied("app_id", app_ids);
        }
        qb.push(" ORDER BY add_time DESC");
        let tags: Vec<FileTagModel> = qb.build_query_as().fetch_all(&self.helper.db).await?;
        for tag in tags {
            let entry = map
                .entry((tag.file_id, tag.user_id, tag.app_id))
                .or_insert_with(Vec::new);
            if (entry.len() as u32) < limit {
                entry.push(FileTagItem {
                    tag_name: tag.tag_name,
                    add_time: tag.add_time,
                });
            }
        }
        Ok(map)
    }

    /// 批量加载文件标签总数，按 (file_id, user_id, app_id) 分组统计
    pub(super) async fn batch_load_tag_counts(
        &self,
        file_ids: &[u64],
        user_ids: &[u64],
        app_ids: &[u64],
    ) -> FileResult<std::collections::HashMap<(u64, u64, u64), i64>> {
        let mut map = std::collections::HashMap::new();
        if file_ids.is_empty() {
            return Ok(map);
        }
        #[derive(sqlx::FromRow)]
        struct TagCountRow {
            file_id: u64,
            user_id: u64,
            app_id: u64,
            cnt: i64,
        }
        let mut qb: QueryBuilder<MySql> = QueryBuilder::new(format!(
            "SELECT file_id, user_id, app_id, COUNT(*) as cnt FROM {}",
            FileTagModel::table_name()
        ));
        qb.push_where().field_in_copied("file_id", file_ids);
        qb.push_and()
            .field_eq("status", FileTagStatus::Normal as i8);
        if !user_ids.is_empty() {
            qb.push_and().field_in_copied("user_id", user_ids);
        }
        if !app_ids.is_empty() {
            qb.push_and().field_in_copied("app_id", app_ids);
        }
        qb.push(" GROUP BY file_id, user_id, app_id");
        let counts: Vec<TagCountRow> = qb.build_query_as().fetch_all(&self.helper.db).await?;
        for row in counts {
            map.insert((row.file_id, row.user_id, row.app_id), row.cnt);
        }
        Ok(map)
    }

    /// 收集 ID、调用批量加载函数并组装结果（每个 file_id 在列表中仅出现一次）
    pub(super) async fn assemble_attr_list(
        &self,
        data: Vec<FileListItem>,
        attr_param: &FileListAttrParam,
    ) -> FileResult<Vec<FileListItemAttrData>> {
        let need_local = attr_param.attr_local.unwrap_or(false);
        let need_oss = attr_param.attr_oss.unwrap_or(false);
        let tag_limit = attr_param.attr_tag_list;
        let need_tag_count = attr_param.attr_tag_count.unwrap_or(false);
        let need_tag = tag_limit.map(|l| l > 0).unwrap_or(false) || need_tag_count;
        let need_lineage = attr_param.attr_lineage.unwrap_or(false);
        let need_url_downloading = attr_param.attr_url_downloading.unwrap_or(false);

        let local_ids: Vec<u64> = if need_local {
            data.iter()
                .filter(|d| FileModel::is_local_key(&d.storage_type))
                .map(|d| d.file_id)
                .collect()
        } else {
            vec![]
        };
        let oss_ids: Vec<u64> = if need_oss {
            data.iter()
                .filter(|d| !FileModel::is_local_key(&d.storage_type))
                .map(|d| d.file_id)
                .collect()
        } else {
            vec![]
        };

        let (tag_fids, tag_uids, tag_aids) = if need_tag || need_lineage {
            let mut fids: Vec<u64> = data.iter().map(|d| d.file_id).collect();
            let mut uids: Vec<u64> = data.iter().map(|d| d.user_id).collect();
            let mut aids: Vec<u64> = data.iter().map(|d| d.app_id).collect();
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

        let mut local_map = self.batch_load_local(&local_ids).await?;
        let mut oss_map = self.batch_load_oss(&oss_ids).await?;
        let mut tag_map = match tag_limit {
            Some(l) if l > 0 => {
                self.batch_load_tags(&tag_fids, &tag_uids, &tag_aids, l)
                    .await?
            }
            _ => Default::default(),
        };
        let mut tag_count_map = if need_tag_count {
            self.batch_load_tag_counts(&tag_fids, &tag_uids, &tag_aids)
                .await?
        } else {
            Default::default()
        };
        let mut lineage_map = if need_lineage {
            self.batch_load_lineage_counts(&tag_fids, &tag_uids, &tag_aids)
                .await?
        } else {
            Default::default()
        };

        // 检查 URL 类型文件是否正在下载中
        // 条件：source_url 非空 且 status 为 Unfinished（下载未完成）
        // 通过 task_dispatch.task_data() 获取当前 Redis 中正在执行的下载任务，按 file_id 索引
        let downloading_file_ids: Option<HashSet<u64>> = if need_url_downloading {
            let url_unfinished_ids: Vec<u64> = data
                .iter()
                .filter(|d| !d.source_url.is_empty() && d.status == FileStatus::Unfinished as i8)
                .map(|d| d.file_id)
                .collect();
            if url_unfinished_ids.is_empty() {
                Some(HashSet::new())
            } else {
                match self.download_manager.task_dispatch.task_data().await {
                    Ok(task_map) => {
                        let downloading: HashSet<u64> =
                            task_map.keys().map(|id| id.file_id).collect();
                        Some(downloading)
                    }
                    Err(_) => Some(HashSet::new()),
                }
            }
        } else {
            None
        };

        Ok(data
            .into_iter()
            .map(|item| {
                let attr_url_downloading = downloading_file_ids.as_ref().map(|downloading| {
                    // 只有 URL 类型且未完成的文件才可能在下载中
                    if !item.source_url.is_empty() && item.status == FileStatus::Unfinished as i8 {
                        downloading.contains(&item.file_id)
                    } else {
                        false
                    }
                });
                FileListItemAttrData {
                    attr_local: need_local
                        .then(|| local_map.remove(&item.file_id))
                        .flatten(),
                    attr_oss: need_oss.then(|| oss_map.remove(&item.file_id)).flatten(),
                    attr_tag: need_tag.then(|| FileTagAttrData {
                        tags: tag_map
                            .remove(&(item.file_id, item.user_id, item.app_id))
                            .unwrap_or_default(),
                        count: tag_count_map.remove(&(item.file_id, item.user_id, item.app_id)),
                    }),
                    attr_lineage: need_lineage.then(|| FileLineageAttrData {
                        counts: lineage_map
                            .remove(&(item.file_id, item.user_id, item.app_id))
                            .unwrap_or_default(),
                    }),
                    attr_url_downloading,
                    file_key: self.file_key_encoder.encode(item.id, None),
                    item,
                }
            })
            .collect())
    }
}
