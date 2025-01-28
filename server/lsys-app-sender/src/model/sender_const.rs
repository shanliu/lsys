use serde::{Deserialize, Serialize};
use lsys_core::db::lsys_model_status;

//发送模块公共常量 -start

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderType {
    Smser = 1,  //短信
    Mailer = 2, //邮件
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderLogType {
    Init = 1,   //新增完成
    Send = 2,   //发送日志
    Cancel = 3, //取消发送
}

// #[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
// #[lsys_model_status(field_type = "i8")]
// pub enum SenderCancelStatus {
//     Init = 1,     //待发送
//     IsCancel = 4, //取消
// }

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderLogStatus {
    Succ = 2, //成功
    Fail = 3, //失败

    MessageCancel = 5, //取消
    NotifySucc = 6,    //回调成功
    NotifyFail = 7,    //回调失败
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderConfigStatus {
    Enable = 1,
    Delete = 2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderTplBodyStatus {
    Enable = 1,
    Delete = 2,
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderTplConfigStatus {
    Enable = 1,
    Delete = 2,
}

//发送模块公共常量 -end

//短信公共常量-start

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderSmsConfigType {
    Close = 1,     // 关闭功能
    Limit = 2,     //频率限制
    MaxOfSend = 3, //每次最大发送数量
    PassTpl = 4,   //指定模板不检测限制
    Block = 10,    //指定号码屏蔽
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct SenderSmsConfigLimit {
    //时间范围
    pub range_time: u64,
    //最大可发送量
    pub max_send: u32,
}

//短信配置数据
pub enum SenderSmsConfigData {
    Limit(SenderSmsConfigLimit),            //限制频率
    MaxOfSend(u32),                         //每次最大发送数量
    Block { area: String, mobile: String }, //屏蔽的手机号
    PassTpl(String),                        //不检测限制指定模板
    Close,                                  //关闭发送功能
    None,                                   //类型或数据异常
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderSmsBodyStatus {
    Init = 1,   //待发送
    Finish = 2, //已发送
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderSmsMessageStatus {
    Init = 1,       //待发送
    IsSend = 2,     //已发送
    IsReceived = 5, //已接收
    SendFail = 3,   //发送失败
    IsCancel = 4,   //已取消
}

//短信公共常量-end

//aliyun 短信相关常量 -start

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderSmsAliyunStatus {
    Enable = 1,
    Delete = 2,
}
//aliyun 短信相关常量 -end

//邮件公共常量-start

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderMailConfigType {
    Close = 1,        //关闭功能
    Limit = 2,        //频率限制
    MaxOfSend = 3,    //每次最大发送数量
    PassTpl = 4,      //指定模板不检测限制
    Block = 20,       //指定邮箱屏蔽
    BlockDomain = 21, //指定邮箱屏蔽
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug)]
pub struct SenderMailConfigLimit {
    //时间范围
    pub range_time: u64,
    //最大可发送量
    pub max_send: u32,
}

//短信配置数据
pub enum SenderMailConfigData {
    Limit(SenderMailConfigLimit),   //限制频率
    MaxOfSend(u32),                 //每次最大发送数量
    Block { to: String },           //屏蔽的邮箱
    PassTpl(String),                //不检测限制指定模板
    Close,                          //关闭发送功能
    None,                           //类型或数据异常
    BlockDomain { domain: String }, //屏蔽的指定域名
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderMailBodyStatus {
    Init = 1,   //待发送
    Finish = 2, //已发送
}

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderMailMessageStatus {
    Init = 1,       //待发送
    IsSend = 2,     //已发送
    IsReceived = 5, //已接收
    SendFail = 3,   //发送失败
    IsCancel = 4,   //已取消
}

//邮件公共常量-end

//smtp 邮件相关常量

#[derive(Serialize, Deserialize, Clone, Copy, Debug, PartialEq, Eq)]
#[lsys_model_status(field_type = "i8")]
pub enum SenderMailSmtpStatus {
    Enable = 1,
    Delete = 2,
}
