//! # 亿美软通(Emay) SMS
//!
//! **亿美软通短信sdk**
//!
//! 实现了批量发送短信和状态回调解析功能
//!
//! 接口文档: <https://www.b2m.cn/static/doc/sms/>
//!
//! ## 加密说明
//!
//! 请求/响应数据均使用 AES-128-ECB (PKCS7 padding) 对称加密，
//! 密钥为用户 secretKey（超过16字节取前16字节，不足末尾补0）。

use chrono::{DateTime, Utc};
use reqwest::{
    Client, Response,
    header::{HeaderMap, HeaderValue},
};
use serde_json::{Value, json};
use tracing::debug;

use crate::{
    BranchSendNotifyResult, SendNotifyItem, SendNotifyStatus, now_time,
    sms_lib::phone_numbers_check,
};

use super::{BranchSendResult, SendError, SendResultItem, SendStatus};

/// 亿美软通短信
pub struct EmaySms {}

impl EmaySms {
    /// AES-128-ECB 加密，PKCS7 填充
    ///
    /// secretKey 超过 16 字节取前 16 字节，不足 16 字节末尾补 `0x00`
    fn aes_encrypt(secret_key: &str, data: &[u8]) -> Vec<u8> {
        use aes::Aes128;
        use aes::cipher::{BlockEncrypt, KeyInit, generic_array::GenericArray};

        let mut key_bytes = [0u8; 16];
        let key = secret_key.as_bytes();
        let copy_len = key.len().min(16);
        key_bytes[..copy_len].copy_from_slice(&key[..copy_len]);

        // PKCS7 填充：不足 16 字节时补 pad_len 个值为 pad_len 的字节
        let pad_len = 16 - (data.len() % 16);
        let mut padded = data.to_vec();
        padded.extend(std::iter::repeat_n(pad_len as u8, pad_len));

        let cipher = Aes128::new(GenericArray::from_slice(&key_bytes));
        let mut result = Vec::with_capacity(padded.len());
        for chunk in padded.chunks(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.encrypt_block(&mut block);
            result.extend_from_slice(&block);
        }
        result
    }

    /// AES-128-ECB 解密，移除 PKCS7 填充
    fn aes_decrypt(secret_key: &str, data: &[u8]) -> Result<Vec<u8>, String> {
        use aes::Aes128;
        use aes::cipher::{BlockDecrypt, KeyInit, generic_array::GenericArray};

        if data.is_empty() || !data.len().is_multiple_of(16) {
            return Err(format!("响应数据长度无效:{}", data.len()));
        }

        let mut key_bytes = [0u8; 16];
        let key = secret_key.as_bytes();
        let copy_len = key.len().min(16);
        key_bytes[..copy_len].copy_from_slice(&key[..copy_len]);

        let cipher = Aes128::new(GenericArray::from_slice(&key_bytes));
        let mut result = Vec::with_capacity(data.len());
        for chunk in data.chunks(16) {
            let mut block = GenericArray::clone_from_slice(chunk);
            cipher.decrypt_block(&mut block);
            result.extend_from_slice(&block);
        }

        // 移除 PKCS7 填充
        if let Some(&pad_byte) = result.last() {
            let pad_len = pad_byte as usize;
            if pad_len > 0 && pad_len <= 16 && result.len() >= pad_len {
                result.truncate(result.len() - pad_len);
            }
        }
        Ok(result)
    }

    /// 回调响应输出
    ///
    /// 接收到状态报告后必须响应英文字符串 `success`，
    /// 否则推送方将在 10s、1min、10min 后重推。
    pub fn send_notify_output(res: &Result<(), String>) -> String {
        match res {
            Ok(_) => "success".to_string(),
            Err(err) => err.to_string(),
        }
    }

    /// 解析状态报告普通推送回调
    ///
    /// `notify_data` 为 POST 参数 `reports` 的值（UTF-8 JSON 数组字符串）。
    ///
    /// 推送格式示例：
    /// ```json
    /// [{
    ///   "mobile": "15538850000",
    ///   "smsId": "20170392833833891100",
    ///   "customSmsId": "1553885000011111",
    ///   "state": "DELIVRD",
    ///   "desc": "成功",
    ///   "receiveTime": "2017-03-15 12:00:00",
    ///   "submitTime": "2017-03-15 12:00:00",
    ///   "extendedCode": "123"
    /// }]
    /// ```
    pub fn send_notify_parse(notify_data: &str) -> BranchSendNotifyResult {
        let items = gjson::parse(notify_data);
        let mut out = Vec::with_capacity(items.array().len());
        for tmp in items.array() {
            let receive_time = chrono::NaiveDateTime::parse_from_str(
                &tmp.get("receiveTime").to_string(),
                "%Y-%m-%d %H:%M:%S",
            )
            .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc).timestamp() as u64)
            .ok();
            let send_time = chrono::NaiveDateTime::parse_from_str(
                &tmp.get("submitTime").to_string(),
                "%Y-%m-%d %H:%M:%S",
            )
            .map(|t| DateTime::<Utc>::from_naive_utc_and_offset(t, Utc).timestamp() as u64)
            .ok();

            let state = tmp.get("state").to_string();
            out.push(SendNotifyItem {
                status: match state.as_str() {
                    "DELIVRD" => SendNotifyStatus::Completed,
                    "" => SendNotifyStatus::Progress,
                    _ => SendNotifyStatus::Failed,
                },
                message: tmp.get("desc").to_string(),
                send_time,
                receive_time,
                code: state,
                send_id: tmp.get("smsId").to_string(),
                mobile: Some(tmp.get("mobile").to_string()),
            });
        }
        Ok(out)
    }

    /// 单次最多可发送手机号数量
    pub fn branch_limit() -> u16 {
        500
    }

    /// 批量发送短信（非自定义 SMSID）
    ///
    /// 对应接口：`POST /inter/sendBatchOnlySMS`
    ///
    /// # 参数
    /// - `host` — 服务地址，如 `http://ip:port`
    /// - `app_id` — 用户 AppID，通过 HTTP Header `appId` 传输
    /// - `secret_key` — 用户密钥，用于 AES 加解密
    /// - `content` — 短信内容（含签名，如 `【公司名】您的验证码是123456`）
    /// - `phone_numbers` — 手机号列表，最多 500 个
    /// - `extended_code` — 扩展码（选填，可传空串）
    pub async fn branch_send(
        client: Client,
        host: &str,
        app_id: &str,
        secret_key: &str,
        content: &str,
        phone_numbers: &[&str],
        extended_code: &str,
    ) -> BranchSendResult {
        let phone_numbers = phone_numbers_check(phone_numbers)?;
        let now_ms = now_time().unwrap_or_default() * 1000;

        let mut req = json!({
            "mobiles": phone_numbers,
            "content": content,
            "requestTime": now_ms,
            "requestValidPeriod": 60
        });
        if !extended_code.is_empty() {
            req["extendedCode"] = Value::String(extended_code.to_string());
        }

        let json_str = req.to_string();
        debug!("emay batch send body: {}", json_str);

        let encrypted = Self::aes_encrypt(secret_key, json_str.as_bytes());

        let mut headers = HeaderMap::new();
        if let Ok(v) = HeaderValue::from_str(app_id) {
            headers.insert("appId", v);
        }

        let response = client
            .post(format!("{}/inter/sendBatchOnlySMS", host))
            .headers(headers)
            .body(encrypted)
            .send()
            .await
            .map_err(|e| SendError::Next(format!("request send fail:{}", e)))?;

        Self::parse_send_response(response, secret_key).await
    }

    /// 批量发送短信（自定义 SMSID）
    ///
    /// 对应接口：`POST /inter/sendBatchSMS`
    ///
    /// # 参数
    /// - `host` — 服务地址，如 `http://ip:port`
    /// - `app_id` — 用户 AppID，通过 HTTP Header `appId` 传输
    /// - `secret_key` — 用户密钥，用于 AES 加解密
    /// - `content` — 短信内容（含签名）
    /// - `phone_numbers` — 手机号列表，最多 500 个
    /// - `custom_sms_ids` — 自定义消息 ID 列表（与手机号一一对应），可为 `None`
    /// - `extended_code` — 扩展码（选填，可传空串）
    #[allow(clippy::too_many_arguments)]
    pub async fn branch_send_custom(
        client: Client,
        host: &str,
        app_id: &str,
        secret_key: &str,
        content: &str,
        phone_numbers: &[&str],
        custom_sms_ids: Option<Vec<String>>,
        extended_code: &str,
    ) -> BranchSendResult {
        let phone_numbers = phone_numbers_check(phone_numbers)?;
        let now_ms = now_time().unwrap_or_default() * 1000;

        // 每条短信包含手机号与可选的自定义 ID
        let smses: Vec<Value> = phone_numbers
            .iter()
            .enumerate()
            .map(|(i, mobile)| {
                let custom_id: Option<&String> = custom_sms_ids
                    .as_ref()
                    .and_then(|ids| ids.get(i))
                    .filter(|id| !id.is_empty());
                json!({
                    "mobile": mobile,
                    "customSmsId": custom_id,
                })
            })
            .collect();

        let mut req = json!({
            "smses": smses,
            "content": content,
            "requestTime": now_ms,
            "requestValidPeriod": 60
        });
        if !extended_code.is_empty() {
            req["extendedCode"] = Value::String(extended_code.to_string());
        }

        let json_str = req.to_string();
        debug!("emay custom send body: {}", json_str);

        let encrypted = Self::aes_encrypt(secret_key, json_str.as_bytes());

        let mut headers = HeaderMap::new();
        if let Ok(v) = HeaderValue::from_str(app_id) {
            headers.insert("appId", v);
        }

        let response = client
            .post(format!("{}/inter/sendBatchSMS", host))
            .headers(headers)
            .body(encrypted)
            .send()
            .await
            .map_err(|e| SendError::Next(format!("request send fail:{}", e)))?;

        Self::parse_send_response(response, secret_key).await
    }

    /// 解析发送响应（内部公共逻辑）
    ///
    /// 从 HTTP Header `result` 判断是否成功；
    /// 成功则解密响应体并解析 JSON 数组。
    async fn parse_send_response(response: Response, secret_key: &str) -> BranchSendResult {
        // 状态码在 HTTP Header result 中，SUCCESS 表示成功
        let result_code = response
            .headers()
            .get("result")
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();

        if result_code != "SUCCESS" {
            return Err(SendError::Next(format!("api fail, result:{}", result_code)));
        }

        let data = response
            .bytes()
            .await
            .map_err(|e| SendError::Next(format!("read response fail:{}", e)))?;

        let decrypted = Self::aes_decrypt(secret_key, &data)
            .map_err(|e| SendError::Next(format!("decrypt response fail:{}", e)))?;

        let res = String::from_utf8_lossy(&decrypted).to_string();
        debug!("emay send response: {}", res);

        // 响应体解密后为 JSON 数组:
        // [{"mobile":"...", "smsId":"...", "customSmsId":"..."}, ...]
        Ok(gjson::parse(&res)
            .array()
            .iter()
            .map(|tmp| SendResultItem {
                mobile: tmp.get("mobile").to_string(),
                status: SendStatus::Progress,
                message: "".to_string(),
                send_id: tmp.get("smsId").to_string(),
            })
            .collect())
    }
}
