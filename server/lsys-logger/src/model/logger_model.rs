use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_model::sqlx_model;

#[derive(FromRow, Clone, Debug, Serialize, Deserialize, Default)]
#[sqlx_model(db_type = "MySql", table_name = "change_logs")]
pub struct ChangeLogModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    /// 日志类型
    #[sqlx(default)]
    pub log_type: String,

    /// 数据
    #[sqlx(default)]
    pub log_data: String,

    /// 简化消息
    #[sqlx(default)]
    pub message: String,

    /// 用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 用户ID
    #[sqlx(default)]
    pub source_id: u64,

    /// 用户ID
    #[sqlx(default)]
    pub add_user_id: u64,

    /// 操作者IP
    #[sqlx(default)]
    pub user_ip: String,

    /// 操作者IP
    #[sqlx(default)]
    pub request_id: String,

    /// 浏览器头
    #[sqlx(default)]
    pub request_user_agent: String,

    /// 时间
    #[sqlx(default)]
    pub add_time: u64,
}
