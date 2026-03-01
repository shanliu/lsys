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
