use lsys_core::db::lsys_model;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(table_name = "lst_file")]
pub struct FileModel {
    #[sqlx(default)]
    pub id: u64,

    #[sqlx(default)]
    pub storage_type: String,

    #[sqlx(default)]
    pub status: i8,

    #[sqlx(default)]
    pub file_name: String,

    #[sqlx(default)]
    pub file_md5: String,

    #[sqlx(default)]
    pub file_size: u64,

    #[sqlx(default)]
    pub modify_time: u64,

    #[sqlx(default)]
    pub content_type: String,

    #[sqlx(default)]
    pub copy_file_id: u64,

    #[sqlx(default)]
    pub from_user_id: u64,

    #[sqlx(default)]
    pub add_time: u64,

    #[sqlx(default)]
    pub change_time: u64,
}

impl FileModel {
    /// 存储类型: 本地
    pub const STORAGE_TYPE_LOCAL_PUBLIC: &'static str = "local_public";
    pub const STORAGE_TYPE_LOCAL_PRIVATE: &'static str = "local_private";
    pub const STORAGE_TYPE_LOCAL_CRYPTO: &'static str = "local_crypto";

    /// 静态方法：判断指定的 key 是否为本地存储类型
    pub fn is_local_key(key: &str) -> bool {
        key == Self::STORAGE_TYPE_LOCAL_PUBLIC
        || key == Self::STORAGE_TYPE_LOCAL_PRIVATE
        || key == Self::STORAGE_TYPE_LOCAL_CRYPTO
    }

    /// 是否为本地存储
    pub fn is_local(&self) -> bool {
        Self::is_local_key(&self.storage_type)
    }

    /// 是否为 OSS 存储（非 local 即 OSS）
    pub fn is_oss(&self) -> bool {
        !self.is_local()
    }

    /// 获取 OSS config_key（仅 OSS 类型有效）
    ///
    /// storage_type 直接就是 config_key（如 "aliyun-prod"、"aws-backup"）
    pub fn oss_config_key(&self) -> Option<&str> {
        if self.is_oss() {
            Some(&self.storage_type)
        } else {
            None
        }
    }
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(table_name = "lst_file_local")]
pub struct FileLocalModel {
    #[sqlx(default)]
    pub id: u64,

    #[sqlx(default)]
    pub file_id: u64,

    #[sqlx(default)]
    pub source_type: i8,

    #[sqlx(default)]
    pub source_name: String,

    #[sqlx(default)]
    pub oss_file_id: u64,

    #[sqlx(default)]
    pub local_path: String,

    #[sqlx(default)]
    pub file_chunk_total: u32,

    #[sqlx(default)]
    pub file_chunk_succ: u32,

    #[sqlx(default)]
    pub file_chunk_size: u64,

    #[sqlx(default)]
    pub last_error: String,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(table_name = "lst_file_local_chunk")]
pub struct FileLocalChunkModel {
    #[sqlx(default)]
    pub id: u64,

    #[sqlx(default)]
    pub file_id: u64,

    #[sqlx(default)]
    pub chunk_index: u32,

    #[sqlx(default)]
    pub start_offset: u64,

    #[sqlx(default)]
    pub chunk_md5: String,

    #[sqlx(default)]
    pub upload_md5: String,

    #[sqlx(default)]
    pub chunk_path: String,

    #[sqlx(default)]
    pub file_size: u64,

    #[sqlx(default)]
    pub complete_size: u64,

    #[sqlx(default)]
    pub status: i8,

    #[sqlx(default)]
    pub add_time: u64,

    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(table_name = "lst_file_oss")]
pub struct FileOssModel {
    #[sqlx(default)]
    pub id: u64,

    #[sqlx(default)]
    pub file_id: u64,

    #[sqlx(default)]
    pub object_key: String,

    #[sqlx(default)]
    pub local_file_id: u64,

    #[sqlx(default)]
    pub object_url: String,

    #[sqlx(default)]
    pub object_url_md5: String,

    #[sqlx(default)]
    pub bucket: String,

    #[sqlx(default)]
    pub region: String,

    #[sqlx(default)]
    pub size: u64,

    #[sqlx(default)]
    pub last_error: String,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(table_name = "lst_file_log")]
pub struct FileLogModel {
    #[sqlx(default)]
    pub id: u64,

    #[sqlx(default)]
    pub file_id: u64,

    #[sqlx(default)]
    pub file_chunk_id: u64,

    #[sqlx(default)]
    pub message: String,

    #[sqlx(default)]
    pub user_id: u64,

    #[sqlx(default)]
    pub add_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(table_name = "lst_file_user")]
pub struct FileUserModel {
    #[sqlx(default)]
    pub id: u64,

    /// 文件属于的用户ID,0=系统
    #[sqlx(default)]
    pub user_id: u64,

    /// 文件添加(上传)用户ID
    #[sqlx(default)]
    pub add_user_id: u64,

    /// 应用ID,0=系统,>0=具体应用
    #[sqlx(default)]
    pub app_id: u64,

    #[sqlx(default)]
    pub file_id: u64,

    #[sqlx(default)]
    pub status: i8,

    #[sqlx(default)]
    pub source_url: String,

    #[sqlx(default)]
    pub source_md5: String,

    #[sqlx(default)]
    pub add_time: u64,

    #[sqlx(default)]
    pub delete_time: u64,

    /// 过期时间，0表示永不过期
    #[sqlx(default)]
    pub expire_time: u64,

    /// 上传策略标识
    #[sqlx(default)]
    pub policy: String,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[lsys_model(table_name = "lst_file_tag")]
pub struct FileTagModel {
    #[sqlx(default)]
    pub id: u64,

    #[sqlx(default)]
    pub file_id: u64,

    #[sqlx(default)]
    pub user_id: u64,

    /// 应用ID,0=系统,>0=具体应用
    #[sqlx(default)]
    pub app_id: u64,

    #[sqlx(default)]
    pub tag_name: String,

    #[sqlx(default)]
    pub status: i8,

    #[sqlx(default)]
    pub add_time: u64,

    #[sqlx(default)]
    pub change_time: u64,
}
