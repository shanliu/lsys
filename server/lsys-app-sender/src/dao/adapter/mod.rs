mod mailer;
mod smser;

#[allow(unused_imports)]
pub use mailer::*;
#[allow(unused_imports)]
pub use smser::*;

// 统一创建发送对外请求的客户端
#[cfg(any(
    feature = "sms-jdcloud",
    feature = "sms-huawei",
    feature = "sms-aliyun",
    feature = "sms-netease",
    feature = "sms-cloopen",
    feature = "sms-tencent"
))]
pub(crate) fn create_sender_client() -> Result<reqwest::Client, super::SenderExecError> {
    use super::SenderExecError;
    let client = reqwest::Client::builder();
    client
        .timeout(std::time::Duration::from_secs(60)) //超时60秒
        .build()
        .map_err(|e| SenderExecError::Next(format!("client create fail:{}", e)))
}
