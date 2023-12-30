use serde::{Deserialize, Serialize};
use sqlx_model::sqlx_model_status;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[sqlx_model_status(field_type = "i8")]
pub enum AppStatus {
    Init = 1,
    Ok = 2,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[sqlx_model_status(field_type = "i8")]
pub enum AppsTokenStatus {
    Enable = 1,
    Delete = -1,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[sqlx_model_status(field_type = "i8")]
pub enum AppSubUserStatus {
    Enable = 1,  //正常
    Disable = 0, //禁用
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[sqlx_model_status(field_type = "i8")]
pub enum AppSubAppsStatus {
    Enable = 1,  //正常
    Disable = 2, //app_sub_user 被禁用
    Delete = -1, //删除
}
