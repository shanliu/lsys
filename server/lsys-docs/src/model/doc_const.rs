use lsys_core::db::lsys_model_status;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum DocGitStatus {
    Enable = 1,  //启用
    Delete = -1, //删除
}
#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum DocGitTagStatus {
    Publish = 2, //已发布
    Build = 1,   //已添加
    Delete = -1, //删除
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum DocGitCloneStatus {
    Init = 1,    //待克隆
    Cloned = 2,  //已克隆
    Fail = 3,    //克隆失败
    Delete = -1, //删除
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum DocMenuStatus {
    Enable = 1,  //启用
    Delete = -1, //删除
}
