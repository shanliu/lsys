use std::collections::HashMap;

use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Method,
};
use serde_json::json;
use tracing::debug;

use crate::{
    response_check, response_msg, sms_lib::phone_numbers_check, BranchSendNotifyResult,
    SendNotifyItem, SendNotifyStatus, SendResultItem, SendStatus,
};

use base64::Engine;
use chrono::{DateTime, Utc};
use sha2::{Digest, Sha256};

use super::{rand_str, BranchSendResult, SendError};

/// aliyun sms
pub struct HwSms {}

impl HwSms {
    pub fn send_notify_output(res: &Result<(), String>) -> String {
        match res {
            Ok(_) => {
                json!({
                  "returnCode" : 0,
                  "returnCodeDesc" : "Success"
                })
            }
            Err(err) => {
                json!({
                    "returnCode" : 500,
                    "returnCodeDesc" : err
                })
            }
        }
        .to_string()
    }
    //POST 表单数据,转为HASHMAP提交过来
    pub fn send_notify_parse(notify_data: &HashMap<String, String>) -> BranchSendNotifyResult {
        let receive_time = notify_data.get("updateTime").and_then(|e| {
            DateTime::parse_from_rfc3339(e)
                .map(|t| t.with_timezone(&Utc).timestamp() as u64)
                .ok()
        });
        Ok(vec![SendNotifyItem {
            status: match notify_data.get("status") {
                Some(state) => match state.as_str() {
                    "DELIVRD" | "ACCEPTD" => SendNotifyStatus::Completed,
                    "" => SendNotifyStatus::Progress,
                    _ => SendNotifyStatus::Failed,
                },
                None => SendNotifyStatus::Progress,
            },
            message: notify_data
                .get("status")
                .map(|e| e.to_owned())
                .unwrap_or("".to_string()),
            send_time: None,
            receive_time,
            code: notify_data
                .get("status")
                .map(|e| e.to_owned())
                .unwrap_or("".to_string()),
            send_id: notify_data
                .get("smsMsgId")
                .map(|e| e.to_owned())
                .unwrap_or("".to_string()),
            mobile: notify_data.get("to").map(|e| e.to_owned()),
        }])
    }
    pub fn branch_limit() -> u16 {
        500
    }
    //执行短信发送
    #[allow(clippy::too_many_arguments)]
    pub async fn branch_send(
        client: Client,
        url: &str,
        app_key: &str,
        app_secret: &str,
        sign_name: &str,
        from: &str,
        template_id: &str,
        template_arr: Option<Vec<String>>,
        phone_numbers: &[&str],
        callback_url: &str,
        extend: &str,
    ) -> BranchSendResult {
        let phone_numbers = phone_numbers_check(phone_numbers)?;

        let mut headers = HeaderMap::new();

        if let Ok(value) = HeaderValue::from_str("application/x-www-form-urlencoded") {
            headers.insert("Content-Type", value);
        }

        if let Ok(value) =
            HeaderValue::from_str(r#"WSSE realm="SDP",profile="UsernameToken",type="Appkey""#)
        {
            headers.insert("Authorization", value);
        }

        let dt = Utc::now();
        let formatted = dt.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        if let Ok(value) = HeaderValue::from_str(&formatted) {
            headers.insert("X-Sdk-Date", value);
        }

        let rand_s = rand_str(32);
        let mut hasher = Sha256::new();
        let sign_data = format!("{}{}{}", &rand_s, &formatted, &app_secret);
        hasher.update(sign_data.as_bytes());
        let passdi =  base64::engine::general_purpose::STANDARD.encode(hasher.finalize());

        let sign = format!(
            r#"UsernameToken Username="{}",PasswordDigest="{}",Nonce="{}",Created="{}""#,
            app_key, passdi, rand_s, formatted
        );
        debug!("hw send sign data:{} sign:{}", sign_data, sign);
        if let Ok(value) = HeaderValue::from_str(&sign) {
            headers.insert("x-wsse", value);
        }

        let mut form_data = vec![];
        form_data.push(("from", from));
        let mobile = phone_numbers.join(",");
        form_data.push(("to", &mobile));
        if !extend.is_empty() {
            form_data.push(("extend", extend));
        }
        form_data.push(("templateId", template_id));
        form_data.push(("signature", sign_name));
        form_data.push(("statusCallback", callback_url));
        let ptmp = if let Some(params) = template_arr {
            json!(params).to_string()
        } else {
            "".to_string()
        };
        if !ptmp.is_empty() {
            form_data.push(("templateParas", ptmp.as_str()));
        }

        let request = client
            .request(Method::POST, format!("{}/sms/batchSendSms/v1", url))
            .headers(headers)
            .form(&form_data);
        let result = request
            .send()
            .await
            .map_err(|e| SendError::Next(format!("request send fail:{}", e)))?;
        let (_, res) = response_check(result, true)
            .await
            .map_err(SendError::Next)?;
        if gjson::get(&res, "code").str() == "000000" {
            let items = gjson::get(&res, "result");
            let mut out = Vec::with_capacity(items.array().len());
            for tmp in items.array() {
                out.push(SendResultItem {
                    mobile: tmp.get("originTo").to_string(),
                    status: if tmp.get("status").str() != "000000" {
                        SendStatus::Completed
                    } else {
                        SendStatus::Progress
                    },
                    message: tmp.get("status").to_string(),
                    send_id: tmp.get("smsMsgId").to_string(),
                });
            }
            return Ok(out);
        }
        Err(SendError::Next(response_msg(&res, &["description"])))
    }
}
