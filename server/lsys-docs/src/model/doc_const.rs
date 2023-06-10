use serde::{Deserialize, Serialize};
use sqlx_model::SqlxModelStatus;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum DocBuildStatus {
    Succ = 1,    //成功
    Finish = 2,  //完成,部分失败
    Fail = 3,    //完全失败
    Delete = -1, //删除
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum DocCloneStatus {
    Ready = 1,   //待克隆
    Cloned = 2,  //已克隆
    Delete = -1, //删除
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum DocGitStatus {
    Enable = 1,  //启用
    Delete = -1, //删除
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum DocMenuStatus {
    Enable = 1,  //启用
    Delete = -1, //删除
}
