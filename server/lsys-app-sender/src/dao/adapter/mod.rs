mod mailer;
mod smser;

use std::time::Duration;

#[allow(unused_imports)]
pub use mailer::*;
#[allow(unused_imports)]
pub use smser::*;

use super::SenderExecError;

// 统一创建发送对外请求的客户端
#[allow(dead_code)]
pub(crate) fn create_sender_client() -> Result<reqwest::Client, SenderExecError> {
    let client = reqwest::Client::builder();
    client
        .timeout(Duration::from_secs(60)) //超时60秒
        .build()
        .map_err(|e| SenderExecError::Next(format!("client create fail:{}", e)))
}
