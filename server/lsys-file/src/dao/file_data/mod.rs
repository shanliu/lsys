use std::sync::Arc;

use super::file_helpers::FileHelper;
use super::*;

mod batch_load;
mod is_private;
mod list;
mod list_by_tag;
mod list_downloading;
mod list_lineage_related;
mod read;
mod ref_query;
mod url;

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
}

/// 文件列表返回结果 (file join file_ref)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, sqlx::FromRow)]
pub struct FileListItem {
    // file_ref primary key
    pub id: u64,
    // file primary key
    pub file_id: u64,
    // file fields
    pub storage_type: String,
    pub status: i8,
    pub file_md5: String,
    pub file_size: u64,
    pub modify_time: u64,
    pub content_type: String,
    pub from_user_id: u64,
    pub add_time: u64,
    pub change_time: u64,
    // file_ref fields
    pub user_id: u64,
    pub add_user_id: u64,
    pub app_id: u64,
    pub file_ref_status: i8,
    pub source_url: String,
    pub source_md5: String,
    pub file_ref_add_time: u64,
    pub delete_time: u64,
    /// 用户自定义文件名（来自 lst_file_ref）
    pub file_name: String,
}

/// 文件列表 attr 参数
///
/// 用于指定在列表查询中是否需要查询关联表的详细信息。
/// - attr_local: 为 true 时，对于 storage_type 为 "local" 的文件，查询并返回 file_local 表的关键信息
/// - attr_oss: 为 true 时，对于 storage_type 非 "local" 的文件，查询并返回 file_oss 表的关键信息
/// - attr_tag_list: Some(n) 时，查询并返回最多 n 个标签列表（None 或 Some(0) 表示不返回标签列表）
/// - attr_tag_count: true 时，查询并返回该文件的标签总数（可独立于 attr_tag_list 使用）
#[derive(Debug, Default)]
pub struct FileListAttrParam {
    pub attr_local: Option<bool>,
    pub attr_oss: Option<bool>,
    pub attr_tag_list: Option<u32>,
    pub attr_tag_count: Option<bool>,
    pub attr_lineage: Option<bool>,
    /// 为 true 时，检查 source_url 非空且 status 为未完成的文件是否正在下载中
    pub attr_url_downloading: Option<bool>,
}

/// 本地文件属性（摊平后的关键数据）
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileLocalAttrData {
    pub id: u64,
    pub source_type: i8,
    pub source_name: String,
    pub local_path: String,
    pub file_chunk_total: u32,
    pub file_chunk_succ: u32,
    pub file_chunk_size: u64,
    pub last_error: String,
}

/// OSS 文件属性（摊平后的关键数据）
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileOssAttrData {
    pub id: u64,
    pub object_key: String,
    pub object_url: String,
    pub bucket: String,
    pub region: String,
    pub size: u64,
    pub last_error: String,
}

/// 文件标签数据
#[derive(Debug, Clone)]
pub struct FileTagAttrData {
    /// 标签列表（attr_tag_list=true 时填充，否则为空）
    pub tags: Vec<FileTagItem>,
    /// 标签数量（attr_tag_count=true 时填充）
    pub count: Option<i64>,
}

/// 单个标签信息
#[derive(Debug, Clone)]
pub struct FileTagItem {
    pub tag_name: String,
    pub add_time: u64,
}

/// 单个关联类型+存储类型的关联文件数量
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileLineageCountItem {
    pub rel_type: i8,
    pub storage_type: String,
    pub count: i64,
}

/// 文件关联（lineage）统计数据
#[derive(Debug, Clone, serde::Serialize)]
pub struct FileLineageAttrData {
    pub counts: Vec<FileLineageCountItem>,
}

/// 文件列表返回结果（包含 attr 数据）
#[derive(Debug, Clone)]
pub struct FileListItemAttrData {
    pub item: FileListItem,
    pub attr_local: Option<FileLocalAttrData>,
    pub attr_oss: Option<FileOssAttrData>,
    pub attr_tag: Option<FileTagAttrData>,
    pub attr_lineage: Option<FileLineageAttrData>,
    /// 文件是否正在下载中（仅当 attr_url_downloading=true 且文件为 URL 类型且未完成时有值）
    pub attr_url_downloading: Option<bool>,
    /// 文件对外唯一短链标识（根据 ref_id 混淆生成）
    pub file_key: String,
}

/// 文件数据查询 DAO（列表、统计等只读查询）
pub struct FileDataDao {
    pub(crate) helper: Arc<FileHelper>,
    pub(crate) oss_config: Arc<super::FileOssConfigDao>,
    pub(crate) runtime_setting: Arc<super::FileRuntimeSettingDao>,
    pub(crate) download_manager: Arc<super::FileDownloadDispatchManager>,
    pub(crate) file_key_encoder: Arc<super::file_helpers::FileKeyEncoder>,
}

impl FileDataDao {
    pub fn new(
        helper: Arc<FileHelper>,
        oss_config: Arc<super::FileOssConfigDao>,
        runtime_setting: Arc<super::FileRuntimeSettingDao>,
        download_manager: Arc<super::FileDownloadDispatchManager>,
        file_key_encoder: Arc<super::file_helpers::FileKeyEncoder>,
    ) -> Self {
        Self {
            helper,
            oss_config,
            runtime_setting,
            download_manager,
            file_key_encoder,
        }
    }
}

pub use list_downloading::{DownloadingListItemData, DownloadingListParam};
pub use list_lineage_related::LineageRelatedListParam;
pub use read::{FileReadChunk, FileReadIterator, PlainFileIterator, UnifiedFileStream};
