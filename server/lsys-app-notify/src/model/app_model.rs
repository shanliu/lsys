use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_model::sqlx_model;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "notify_config")]
pub struct NotifyConfigModel {
    /// 用户ID
    #[sqlx(default)]
    pub id: u64,

    /// 应用ID
    #[sqlx(default)]
    pub app_id: u64,

    /// 请求方法名
    #[sqlx(default)]
    pub method: String,

    /// 请求方法名
    #[sqlx(default)]
    pub call_url: String,

    /// 用户ID 0 为系统角色
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 应用ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 下次推送时间
    #[sqlx(default)]
    pub change_time: u64,

    /// 创建时间
    #[sqlx(default)]
    pub create_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "notify_data")]
pub struct NotifyDataModel {
    /// 用户ID
    #[sqlx(default)]
    pub id: u64,

    /// 应用ID
    #[sqlx(default)]
    pub app_id: u64,

    /// 请求方法名
    #[sqlx(default)]
    pub method: String,

    /// 请求JSON数据
    #[sqlx(default)]
    pub payload: String,

    /// 请求状态
    #[sqlx(default)]
    pub status: i8,

    /// 请求结果
    #[sqlx(default)]
    pub result: String,

    /// 请求次数
    #[sqlx(default)]
    pub try_num: i8,

    /// 最后推送时间
    #[sqlx(default)]
    pub publish_time: u64,

    /// 下次推送时间
    #[sqlx(default)]
    pub next_time: u64,

    /// 创建时间
    #[sqlx(default)]
    pub create_time: u64,
}
