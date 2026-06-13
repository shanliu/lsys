use lsys_core::db::{CursorPageData, CursorPageParam, QueryBuilderExt, TableMeta, WhereClause};
use sqlx::{MySql, QueryBuilder};

use crate::common::FileResult;
use crate::model::*;

use super::{FileDataDao, FileListAttrParam, FileListItem, FileListItemAttrData};

/// 文件关联列表过滤参数
///
/// 用于查询指定文件的关联文件列表（基于 lst_file_lineage 表）。
/// 支持按关系类型、存储类型等维度过滤。
#[derive(Debug, Default)]
pub struct LineageRelatedListParam {
    /// 按关系类型过滤：1=主动拷贝, 2=本地类型转换, 3=OSS↔本地同步
    pub rel_type: Option<i8>,
    /// 按存储类型过滤（目标文件的 storage_type）
    pub storage_type: Option<String>,
}

impl FileDataDao {
    /// 构建关联文件列表的公共 WHERE 条件
    ///
    /// 包含：排除已删除文件、lineage 状态过滤、user_id/app_id/file_id 过滤
    fn build_lineage_related_query<'a>(
        wc: &mut WhereClause<'_, 'a, MySql>,
        file_ref: &'a FileRefModel,
        filter: &'a LineageRelatedListParam,
    ) {
        // 排除已删除的文件和 lineage 记录
        wc.and().field_ne("f.status", FileStatus::Deleted as i8);
        wc.and()
            .field_eq("fl.status", FileLineageStatus::Normal as i8);

        // 核心条件：src_file_id + user_id + app_id
        wc.and().field_eq("fl.src_file_id", file_ref.file_id);
        wc.and().field_eq("fl.user_id", file_ref.user_id);
        wc.and().field_eq("fl.app_id", file_ref.app_id);

        // rel_type 过滤
        if let Some(rt) = filter.rel_type {
            wc.and().field_eq("fl.rel_type", rt);
        }

        // storage_type 过滤
        if let Some(ref st) = filter.storage_type {
            wc.and().field_eq("f.storage_type", st.to_string());
        }
    }

    /// 查询指定文件的关联文件列表（游标分页）
    ///
    /// 逻辑：
    /// - 从 lst_file_lineage 表查询 src_file_id = file_ref.file_id 的记录
    /// - JOIN lst_file 表获取目标文件（dst_file_id）的详细信息
    /// - JOIN lst_file_ref 表获取目标文件在当前用户+应用下的引用信息
    /// - 支持按 rel_type、storage_type 过滤
    ///
    /// # 参数
    /// - `file_ref`: 源文件的引用记录（包含 file_id、user_id、app_id）
    /// - `filter`: 过滤参数（rel_type、storage_type）
    /// - `page`: 游标分页参数
    /// - `attr_param`: 文件属性参数（是否查询 local/oss/tag 等信息）
    pub async fn list_lineage_related_files(
        &self,
        file_ref: &FileRefModel,
        filter: &LineageRelatedListParam,
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
             FROM {} fl \
             INNER JOIN {} f ON fl.dst_file_id = f.id \
             INNER JOIN {} fu ON f.id = fu.file_id AND fu.user_id = fl.user_id AND fu.app_id = fl.app_id",
            FileLineageModel::table_name(),
            FileModel::table_name(),
            FileRefModel::table_name()
        ));

        let mut wc = WhereClause::new(&mut qb);
        Self::build_lineage_related_query(&mut wc, file_ref, filter);

        // 排除已删除的 file_ref 记录
        wc.and()
            .field_ne("fu.status", FileUserStatus::Deleted as i8);

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

    /// 统计指定文件的关联文件数量
    ///
    /// 逻辑：
    /// - 统计 lst_file_lineage 表中 src_file_id = file_ref.file_id 的记录数
    /// - 支持按 rel_type、storage_type 过滤
    ///
    /// # 参数
    /// - `file_ref`: 源文件的引用记录（包含 file_id、user_id、app_id）
    /// - `filter`: 过滤参数（rel_type、storage_type）
    pub async fn count_lineage_related_files(
        &self,
        file_ref: &FileRefModel,
        filter: &LineageRelatedListParam,
    ) -> FileResult<i64> {
        let mut qb: QueryBuilder<MySql> = QueryBuilder::new(format!(
            "SELECT COUNT(*) FROM {} fl \
             INNER JOIN {} f ON fl.dst_file_id = f.id \
             INNER JOIN {} fu ON f.id = fu.file_id AND fu.user_id = fl.user_id AND fu.app_id = fl.app_id",
            FileLineageModel::table_name(),
            FileModel::table_name(),
            FileRefModel::table_name()
        ));

        let mut wc = WhereClause::new(&mut qb);
        Self::build_lineage_related_query(&mut wc, file_ref, filter);

        // 排除已删除的 file_ref 记录
        wc.and()
            .field_ne("fu.status", FileUserStatus::Deleted as i8);

        wc.builder()
            .build_query_scalar()
            .fetch_one(&self.helper.db)
            .await
            .map_err(Into::into)
    }
}
