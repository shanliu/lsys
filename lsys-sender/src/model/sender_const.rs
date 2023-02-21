use serde::{Deserialize, Serialize};
use sqlx_model::SqlxModelStatus;

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum SenderSmsMessageStatus {
    Init = 1,     //待发送
    IsSend = 2,   //已发送
    SendFail = 3, //发送失败
    IsCancel = 4, //已取消
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum SenderSmsLogType {
    Send = 1,   //发送日志
    Cancel = 2, //取消发送
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum SenderSmsCancelStatus {
    Init = 1,     //待发送
    IsCancel = 4, //取消
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum SenderSmsLogStatus {
    Succ = 2,          //成功
    Fail = 3,          //失败
    KeyCancel = 4,     //取消
    MessageCancel = 5, //取消
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum SenderSmsConfigStatus {
    Enable = 1,
    Delete = 2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum SenderSmsConfigType {
    Limit = 1,     //频率限制
    Block = 2,     //指定号码屏蔽
    Close = 3,     // 关闭功能
    PassTpl = 4,   //指定模板不检测限制
    MaxOfSend = 5, //每次最大发送数量
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct SenderSmsConfigLimit {
    //时间范围
    pub range_time: u64,
    //最大可发送量
    pub max_send: u32,
}
//配置数据

pub enum SenderSmsConfigData {
    Limit(SenderSmsConfigLimit), //限制频率
    MaxOfSend(u32),              //每次最大发送数量
    Block(String, String),       //屏蔽的手机号
    PassTpl(String),             //不检测限制指定模板
    Close,                       //关闭发送功能
    None,                        //类型或数据异常
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum SenderSmsAliyunStatus {
    Enable = 1,
    Delete = 2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, SqlxModelStatus, PartialEq, Eq)]
#[sqlx_model_status(type = "i8")]
pub enum SenderAliyunConfigStatus {
    Enable = 1,
    Delete = 2,
}
