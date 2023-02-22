use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_model::SqlxModel;

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "sender_aliyun_config")]
pub struct SenderAliyunConfigModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    /// 显示给用户配置名
    #[sqlx(default)]
    pub name: String,

    /// Id
    #[sqlx(default)]
    pub access_id: String,

    ///secret
    #[sqlx(default)]
    pub access_secret: String,

    /// 0 禁用 1 启用
    #[sqlx(default)]
    pub status: i8,

    /// 添加用户ID
    #[sqlx(default)]
    pub add_user_id: u64,

    /// 添加时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 删除时间
    #[sqlx(default)]
    pub delete_time: u64,
}

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "sender_sms_aliyun")]
pub struct SenderSmsAliyunModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    /// 应用ID
    #[sqlx(default)]
    pub app_id: u64,
    /// 名称
    #[sqlx(default)]
    pub name: String,
    /// 内部模板名
    #[sqlx(default)]
    pub sms_tpl: String,

    /// 配置ID
    #[sqlx(default)]
    pub aliyun_config_id: u64,

    /// 阿里云签名
    #[sqlx(default)]
    pub aliyun_sign_name: String,

    /// 阿里云模板名
    #[sqlx(default)]
    pub aliyun_sms_tpl: String,

    /// 最大发送次数
    #[sqlx(default)]
    pub max_try_num: u16,

    /// 0 禁用 1 启用
    #[sqlx(default)]
    pub status: i8,

    /// 属于用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 添加用户ID
    #[sqlx(default)]
    pub add_user_id: u64,

    /// 删除用户ID
    #[sqlx(default)]
    pub delete_user_id: u64,

    /// 添加时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 更新时间
    #[sqlx(default)]
    pub delete_time: u64,
}

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "sender_sms_config")]
pub struct SenderSmsConfigModel {
    /// ID
    #[sqlx(default)]
    pub id: u64,

    /// 应用ID 0 为全局限制
    #[sqlx(default)]
    pub app_id: u64,

    /// 优先级
    #[sqlx(default)]
    pub priority: i8,

    /// 配置类型
    #[sqlx(default)]
    pub config_type: i8,

    /// 配置数据 JSON
    #[sqlx(default)]
    pub config_data: String,

    /// 0 禁用 1 启用
    #[sqlx(default)]
    pub status: i8,

    /// 属于用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 添加用户ID
    #[sqlx(default)]
    pub add_user_id: u64,

    /// 删除用户ID
    #[sqlx(default)]
    pub delete_user_id: u64,

    /// 添加时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 发送时间
    #[sqlx(default)]
    pub delete_time: u64,
}

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "sender_sms_message")]
pub struct SenderSmsMessageModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    /// 应用ID
    #[sqlx(default)]
    pub app_id: u64,

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

    /// 预期发送时间
    #[sqlx(default)]
    pub expected_time: u64,

    /// 实际发送时间
    #[sqlx(default)]
    pub send_time: u64,

    /// 发送用户ID
    #[sqlx(default)]
    pub user_id: u64,
}

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "sender_sms_cancel")]
pub struct SenderSmsCancelModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    /// 冗余appid
    #[sqlx(default)]
    pub app_id: u64,
    /// 取消句柄
    #[sqlx(default)]
    pub cancel_hand: String,

    /// 消息id
    #[sqlx(default)]
    pub sms_message_id: u64,

    /// 日志状态 1 待发送 4 取消
    #[sqlx(default)]
    pub status: i8,

    /// 执行取消用户ID
    #[sqlx(default)]
    pub cancel_user_id: u64,

    /// 取消时间
    #[sqlx(default)]
    pub cancel_time: u64,
}

#[derive(FromRow, SqlxModel, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(table_name = "sender_sms_log")]
pub struct SenderSmsLogModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    /// 消息id
    #[sqlx(default)]
    pub sms_message_id: u64,

    /// 应用ID
    #[sqlx(default)]
    pub app_id: u64,

    /// 日志类型
    #[sqlx(default)]
    pub log_type: i8,

    /// 触发来源
    #[sqlx(default)]
    pub send_type: String,

    /// 日志状态 2 成功 3 失败
    #[sqlx(default)]
    pub status: i8,

    /// 日志消息
    #[sqlx(default)]
    pub message: String,

    /// 发送时间
    #[sqlx(default)]
    pub create_time: u64,

    /// 操作用户
    #[sqlx(default)]
    pub user_id: u64,
}
