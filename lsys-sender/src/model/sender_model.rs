use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_model::SqlxModel;

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "sender_sms_message")]
pub struct SenderSmsMessageModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    /// 区号
    #[sqlx(default)]
    pub area: String,

    /// 手机号
    #[sqlx(default)]
    pub mobile: String,

    /// 模板ID
    #[sqlx(default)]
    pub tpl_id: String,

    /// 模板变量
    #[sqlx(default)]
    pub tpl_var: String,

    /// 尝试次数
    #[sqlx(default)]
    pub try_num: u16,

    /// 1 未发送 2 已发送 3 发送失败
    #[sqlx(default)]
    pub status: i8,

    /// 添加时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 发送时间
    #[sqlx(default)]
    pub send_time: u64,
}

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "sender_sms_history")]
pub struct SenderSmsHistoryModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    /// 消息id
    #[sqlx(default)]
    pub sms_message_id: u64,

    /// 发送类型
    #[sqlx(default)]
    pub send_type: String,

    /// 2 已发送 3 发送失败
    #[sqlx(default)]
    pub status: i8,

    /// 发送消息
    #[sqlx(default)]
    pub send_message: String,

    /// 发送时间
    #[sqlx(default)]
    pub send_time: u64,
}
