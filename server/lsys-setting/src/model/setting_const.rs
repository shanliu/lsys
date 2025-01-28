use lsys_core::db::lsys_model_status;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SettingStatus {
    Enable = 1, //正常
    Delete = 2, //已删除
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SettingType {
    Single = 1,   //正常
    Multiple = 2, //已删除
}
