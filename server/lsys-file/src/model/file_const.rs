use lsys_core::db::lsys_model_status;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum FileSourceType {
    Upload = 1,
    Url = 2,
    LocalPath = 3,
    OssSync = 4,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum FileStatus {
    Normal = 1,
    Deleted = 2,
    Unfinished = 3,
    Failed = 4,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum FileChunkStatus {
    Normal = 1,
    Deleted = 2,
    Unfinished = 3,
    Failed = 4,
    Merged = 5,
    Cleaned = 6,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum FileUserStatus {
    Normal = 1,
    Deleted = 2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum FileTagStatus {
    Normal = 1,
    Deleted = 2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum FileLineageStatus {
    Normal = 1,
    Deleted = 2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum FileLineageRelType {
    /// 主动拷贝（产生独立物理文件）
    Copy = 1,
    /// 本地存储类型转换（public/private/crypto 互转，方向由 src/dst 的 storage_type 决定）
    Convert = 2,
    /// OSS ↔ 本地同步（双向：local→OSS 和 OSS→local，方向由 src/dst 的 storage_type 决定）
    OssSync = 3,
}
