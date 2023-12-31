use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use sqlx_model::sqlx_model;

// 公共表 -start

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "sender_config")]
pub struct SenderConfigModel {
    /// ID
    #[sqlx(default)]
    pub id: u64,

    /// 应用ID 0 为全局限制
    #[sqlx(default)]
    pub app_id: u64,

    /// 优先级
    #[sqlx(default)]
    pub priority: i8,

    //发送来源
    #[sqlx(default)]
    pub sender_type: i8,

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

    /// 最后更新用户id
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 最后更新时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "sender_log")]
pub struct SenderLogModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    //发送来源
    #[sqlx(default)]
    pub sender_type: i8,

    /// 消息id
    #[sqlx(default)]
    pub sender_message_id: u64,

    /// 应用ID
    #[sqlx(default)]
    pub app_id: u64,

    /// 日志类型
    #[sqlx(default)]
    pub log_type: i8,

    /// 日志状态 2 成功 3 失败
    #[sqlx(default)]
    pub status: i8,

    /// 执行发送类型
    #[sqlx(default)]
    pub executor_type: String,

    /// 日志消息
    #[sqlx(default)]
    pub message: String,

    /// 发送时间
    #[sqlx(default)]
    pub create_time: u64,
}

// 模板配置
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "sender_tpl_config")]
pub struct SenderTplConfigModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    //发送来源
    #[sqlx(default)]
    pub sender_type: i8,

    /// 应用ID
    #[sqlx(default)]
    pub app_id: u64,

    /// 名称
    #[sqlx(default)]
    pub name: String,

    /// 内部模板名
    #[sqlx(default)]
    pub tpl_id: String,

    /// 配置ID
    #[sqlx(default)]
    pub setting_id: u64,

    /// 配置JSON数据
    #[sqlx(default)]
    pub config_data: String,

    /// 0 禁用 1 启用
    #[sqlx(default)]
    pub status: i8,

    /// 属于用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 修改时间
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 修改时间
    #[sqlx(default)]
    pub change_time: u64,
}
// 模板内容
#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "sender_tpl_body")]
pub struct SenderTplBodyModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    //来源
    #[sqlx(default)]
    pub sender_type: i8,

    /// 模板ID
    #[sqlx(default)]
    pub tpl_id: String,

    /// 模板
    #[sqlx(default)]
    pub tpl_data: String,

    /// 状态
    #[sqlx(default)]
    pub status: i8,

    /// 发送用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 修改时间
    #[sqlx(default)]
    pub change_user_id: u64,

    /// 修改时间
    #[sqlx(default)]
    pub change_time: u64,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "sender_message_cancel")]
pub struct SenderMessageCancelModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    /// 冗余appid
    #[sqlx(default)]
    pub app_id: u64,

    //发送来源
    #[sqlx(default)]
    pub sender_type: i8,

    /// 消息内容id
    #[sqlx(default)]
    pub sender_body_id: u64,

    /// 消息id
    #[sqlx(default)]
    pub sender_message_id: u64,

    /// 执行取消用户ID
    #[sqlx(default)]
    pub cancel_user_id: u64,

    /// 取消时间
    #[sqlx(default)]
    pub cancel_time: u64,
}

// 短信数据

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "sender_sms_body")]
pub struct SenderSmsBodyModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    /// 应用ID
    #[sqlx(default)]
    pub app_id: u64,

    /// 模板ID
    #[sqlx(default)]
    pub tpl_id: String,

    /// 模板变量
    #[sqlx(default)]
    pub tpl_var: String,

    /// 最大发送次数
    #[sqlx(default)]
    pub max_try_num: u16,

    /// 1 未发送 2 已发送 3 发送失败
    #[sqlx(default)]
    pub status: i8,

    /// 添加时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 预期发送时间
    #[sqlx(default)]
    pub expected_time: u64,

    /// 完成发送时间
    #[sqlx(default)]
    pub finish_time: u64,

    /// 发送用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 发送用户IP
    #[sqlx(default)]
    pub user_ip: String,

    /// 请求ID
    #[sqlx(default)]
    pub request_id: String,
}

// 短信数据

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "sender_sms_message")]
pub struct SenderSmsMessageModel {
    ///自增ID
    #[sqlx(default)]
    pub id: u64,

    /// 消息ID
    #[sqlx(default)]
    pub snid: u64,

    /// 应用ID
    #[sqlx(default)]
    pub sender_body_id: u64,

    /// 区号
    #[sqlx(default)]
    pub area: String,

    /// 手机号
    #[sqlx(default)]
    pub mobile: String,

    /// 尝试次数
    #[sqlx(default)]
    pub try_num: u16,

    /// 1 未发送 2 已发送 3 发送失败
    #[sqlx(default)]
    pub status: i8,

    /// 添加时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 实际发送时间
    #[sqlx(default)]
    pub send_time: u64,

    /// 完成发送时间
    #[sqlx(default)]
    pub receive_time: u64,

    /// 成功发送配置ID
    #[sqlx(default)]
    pub setting_id: u64,

    /// 发送返回
    #[sqlx(default)]
    pub res_data: String,
}

// 邮件数据

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "sender_mail_body")]
pub struct SenderMailBodyModel {
    /// 消息ID
    #[sqlx(default)]
    pub id: u64,

    /// 应用ID
    #[sqlx(default)]
    pub app_id: u64,

    #[sqlx(default)]
    pub tpl_id: String,

    /// 模板变量
    #[sqlx(default)]
    pub tpl_var: String,

    /// 最大发送次数
    #[sqlx(default)]
    pub max_try_num: u16,

    #[sqlx(default)]
    pub reply_mail: String,

    /// 1 未发送 2 已发送 3 发送失败
    #[sqlx(default)]
    pub status: i8,

    /// 添加时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 预期发送时间
    #[sqlx(default)]
    pub expected_time: u64,

    /// 完成发送时间
    #[sqlx(default)]
    pub finish_time: u64,

    /// 发送用户ID
    #[sqlx(default)]
    pub user_id: u64,

    /// 发送用户IP
    #[sqlx(default)]
    pub user_ip: String,
    /// 请求ID
    #[sqlx(default)]
    pub request_id: String,
}

#[derive(FromRow, Clone, Debug, Serialize, Deserialize)]
#[sqlx_model(db_type = "MySql", table_name = "sender_mail_message")]
pub struct SenderMailMessageModel {
    /// 自增ID
    #[sqlx(default)]
    pub id: u64,

    /// 消息ID
    #[sqlx(default)]
    pub snid: u64,

    /// 应用ID
    #[sqlx(default)]
    pub sender_body_id: u64,

    #[sqlx(default)]
    pub to_mail: String,

    /// 尝试次数
    #[sqlx(default)]
    pub try_num: u16,

    /// 1 未发送 2 已发送 3 发送失败
    #[sqlx(default)]
    pub status: i8,

    /// 添加时间
    #[sqlx(default)]
    pub add_time: u64,

    /// 实际发送时间
    #[sqlx(default)]
    pub send_time: u64,

    /// 完成发送时间
    #[sqlx(default)]
    pub receive_time: u64,

    /// 成功发送配置ID
    #[sqlx(default)]
    pub setting_id: u64,

    /// 发送返回
    #[sqlx(default)]
    pub res_data: String,
}
