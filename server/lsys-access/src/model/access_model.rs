use lsys_core::db::lsys_model;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "user")]
pub struct UserModel {
    #[sqlx(default)]
    pub id: u64,

    /// 应用ID,内置账号登录为0
    #[sqlx(default)]
    pub app_id: u64,

    ///用户数据
    #[sqlx(default)]
    pub user_data: String,

    ///尝试登录账号
    #[sqlx(default)]
    pub user_account: String,

    ///tmp_user_nickname
    #[sqlx(default)]
    pub user_nickname: String,

    /// 最后更新时间
    #[sqlx(default)]
    pub(crate) change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "session")]
pub struct SessionModel {
    #[sqlx(default)]
    pub(crate) id: u64,

    /// 用户ID
    #[sqlx(default)]
    pub(crate) user_id: u64,

    /// 冗余yaf_user的app_id
    #[sqlx(default)]
    pub user_app_id: u64,

    /// OAUTH登录时的app_id
    #[sqlx(default)]
    pub oauth_app_id: u64,

    ///授权token
    #[sqlx(default)]
    pub token_data: String,

    ///原授权token
    #[sqlx(default)]
    pub source_token_data: String,

    ///登录类型
    #[sqlx(default)]
    pub login_type: String,

    ///登陆者IP
    #[sqlx(default)]
    pub login_ip: String,

    ///设备ID
    #[sqlx(default)]
    pub device_id: String,

    ///设备名
    #[sqlx(default)]
    pub device_name: String,

    ///状态
    #[sqlx(default)]
    pub status: i8,

    /// 登录时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 超时时间
    #[sqlx(default)]
    pub expire_time: u64,

    /// 超时时间
    #[sqlx(default)]
    pub logout_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[lsys_model(table_name = "session_data")]
pub struct SessionDataModel {
    #[sqlx(default)]
    pub(crate) id: u64,

    /// 冗余SESSION id
    #[sqlx(default)]
    pub(crate) session_id: u64,

    ///尝试登录账号
    #[sqlx(default)]
    pub data_key: String,

    ///用户名称
    #[sqlx(default)]
    pub data_val: String,

    /// 登录时间
    #[sqlx(default)]
    pub(crate) change_time: u64,
}
