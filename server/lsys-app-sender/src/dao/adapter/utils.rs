// 统一创建发送对外请求的客户端
#[cfg(any(
    feature = "sms-jdcloud",
    feature = "sms-huawei",
    feature = "sms-aliyun",
    feature = "sms-netease",
    feature = "sms-cloopen",
    feature = "sms-tencent"
))]
pub(crate) fn create_sender_client() -> Result<reqwest::Client, crate::dao::SenderExecError> {
    let client = reqwest::Client::builder();
    client
        .timeout(std::time::Duration::from_secs(60)) //超时60秒
        .build()
        .map_err(|e| crate::dao::SenderExecError::Next(format!("client create fail:{}", e)))
}
