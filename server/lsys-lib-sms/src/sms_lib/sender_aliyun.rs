//! # aliyun SMS
//!
//! **阿里云短信sdk**
//!
//! 目前实现了发送短信功能
//!

use crate::{
    now_time, rand_str,
    sms_lib::{phone_numbers_check, response_check, SendStatus},
    BranchSendNotifyResult, SendNotifyItem, SendNotifyStatus,
};
use hmac::{Hmac, Mac};

use chrono::{DateTime, Utc};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    Client, Method,
};
use serde_json::json;
use sha2::{Digest, Sha256};
use std::collections::HashMap;
use tracing::debug;

use super::{
    response_msg, BranchSendDetailResult, BranchSendResult, SendDetailItem, SendError,
    SendResultItem,
};
/// aliyun sms
pub struct AliSms {}

impl AliSms {
    pub fn send_notify_output(res: &Result<(), String>) -> String {
        match res {
            Ok(_) => {
                json!({
                  "code" : 0,
                  "msg" : "接收成功"
                })
            }
            Err(err) => {
                json!({
                  "code" : 400,
                  "msg" : err
                })
            }
        }
        .to_string()
    }
    //post json
    pub fn send_notify_parse(notify_data: &str) -> BranchSendNotifyResult {
        //         [
        //   {
        //     "phone_number" : "1381111****",
        //     "send_time" : "2017-01-01 00:00:00",
        //     "report_time" : "2017-01-01 00:00:00",
        //     "success" : true,
        //     "err_code" : "DELIVERED",
        //     "err_msg" : "用户接收成功",
        //     "sms_size" : "1",
        //     "biz_id" : "12345",
        //     "out_id" : "67890"
        //   }
        // ]
        // println!("{}", notify_data);
        let items = gjson::parse(notify_data);
        let mut out = Vec::with_capacity(items.array().len());
        for tmp in items.array() {
            let send_time = chrono::NaiveDateTime::parse_from_str(
                &tmp.get("send_time").to_string(),
                "%Y-%m-%d %H:%M:%S",
            )
            .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
            .ok();
            let receive_time = chrono::NaiveDateTime::parse_from_str(
                &tmp.get("report_time").to_string(),
                "%Y-%m-%d %H:%M:%S",
            )
            .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
            .ok();

            out.push(SendNotifyItem {
                status: if tmp.get("success").bool() {
                    SendNotifyStatus::Completed
                } else {
                    SendNotifyStatus::Failed
                },
                message: tmp.get("err_msg").to_string(),
                send_time,
                receive_time,
                code: tmp.get("err_code").to_string(),
                send_id: tmp.get("biz_id").to_string(),
                mobile: Some(tmp.get("phone_number").to_string()),
            });
        }
        Ok(out)
    }
    pub async fn send_detail(
        client: Client,
        access_key_id: &str,
        access_secret: &str,
        send_id: &str,
        mobile: &str,
        send_date: &str,
    ) -> BranchSendDetailResult {
        let mut params = HashMap::new();
        params.insert("PhoneNumber", mobile);
        params.insert("BizId", send_id);
        params.insert("SendDate", send_date);
        params.insert("PageSize", "1");
        params.insert("CurrentPage", "1");
        params.insert("Version", "2017-05-25");
        params.insert("Action", "QuerySendDetails");

        let res = Self::build_request(
            client,
            Method::POST,
            "dysmsapi.aliyuncs.com",
            "/",
            "",
            "QuerySendDetails",
            &params,
            access_key_id,
            access_secret,
        )
        .await?;
        if gjson::get(&res, "Code").str() == "OK" {
            let status =
                gjson::get(&res, "SmsSendDetailDTOs.SmsSendDetailDTO.0.SendStatus").to_string();
            let send_time = chrono::NaiveDateTime::parse_from_str(
                &gjson::get(&res, "SmsSendDetailDTOs.SmsSendDetailDTO.0.SendDate").to_string(),
                "%Y-%m-%d %H:%M:%S",
            )
            .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
            .ok();
            let receive_time = chrono::NaiveDateTime::parse_from_str(
                &gjson::get(&res, "SmsSendDetailDTOs.SmsSendDetailDTO.0.ReceiveDate").to_string(),
                "%Y-%m-%d %H:%M:%S",
            )
            .map(|tmp| DateTime::<Utc>::from_naive_utc_and_offset(tmp, Utc).timestamp() as u64)
            .ok();
            // println!("{}", res);
            return Ok(vec![SendDetailItem {
                status: if status.as_str() == "2" {
                    SendNotifyStatus::Failed
                } else if status.as_str() == "3" {
                    SendNotifyStatus::Completed
                } else {
                    SendNotifyStatus::Progress
                },
                message: gjson::get(&res, "SmsSendDetailDTOs.SmsSendDetailDTO.0.Content")
                    .to_string(),
                send_time,
                receive_time,
                code: gjson::get(&res, "SmsSendDetailDTOs.SmsSendDetailDTO.0.ErrCode").to_string(),
                send_id: send_id.to_string(),
                mobile: Some(mobile.to_string()),
            }]);
        }
        Err(response_msg(&res, &["Message"]))
    }
    pub fn branch_limit() -> u16 {
        100
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn branch_send(
        client: Client,
        region: &str, //"cn-hangzhou"
        access_key_id: &str,
        access_secret: &str,
        sign_name: &str,
        template_code: &str,
        template_param: &str,
        phone_numbers: &[&str],
        out_id: &str,
        ip: &str,
    ) -> BranchSendResult {
        let phone_numbers = phone_numbers_check(phone_numbers)?;
        let mut params = HashMap::new();
        let mobile = json!(phone_numbers).to_string();
        if !region.is_empty() {
            params.insert("RegionId", region);
        }
        let sign_name = json!(vec![sign_name; phone_numbers.len()]).to_string();
        let mut param_data = vec![template_param; phone_numbers.len()].join(",");
        param_data = format!("[{}]", param_data);

        // {
        //     "PhoneNumberJson": "[\"13800138000\",\"13800138001\"]",
        //     "SignNameJson": "[\"ddddd\",\"ddddd\"]",
        //     "TemplateCode": "SMS_35115133",
        //     "TemplateParamJson": "[{\"code\":\"111\"},{\"code\":\"111\"}]",
        //     "SourceIp": "218.18.137.26"
        //   }
        params.insert("Version", "2017-05-25");
        params.insert("PhoneNumberJson", mobile.as_str());
        params.insert("SignNameJson", sign_name.as_str());
        params.insert("TemplateCode", template_code);
        params.insert("TemplateParamJson", param_data.as_str());
        params.insert("Action", "SendBatchSms");
        if !out_id.is_empty() {
            params.insert("OutId", out_id);
        }

        if !ip.is_empty() {
            params.insert("SourceIp", ip);
        }

        let res = Self::build_request(
            client,
            Method::POST,
            "dysmsapi.aliyuncs.com",
            "/",
            "",
            "SendBatchSms",
            &params,
            access_key_id,
            access_secret,
        )
        .await
        .map_err(SendError::Next)?;

        // println!("{}", res);

        if gjson::get(&res, "Code").str() == "OK" {
            return Ok(phone_numbers
                .iter()
                .map(|e| SendResultItem {
                    mobile: e.to_string(),
                    status: SendStatus::Progress,
                    message: gjson::get(&res, "Message").to_string(),
                    send_id: gjson::get(&res, "BizId").to_string(),
                })
                .collect());
        }
        Err(SendError::Next(response_msg(&res, &["Message"])))
    }
    #[allow(clippy::too_many_arguments)]
    pub async fn build_request(
        client: Client,
        method: Method,
        host: &str,
        uri: &str,
        query: &str,
        action: &str,
        params: &HashMap<&str, &str>,
        secret_id: &str,
        secret_key: &str,
    ) -> Result<String, String> {
        let now_time = now_time().unwrap_or_default();

        let req_json =
            serde_urlencoded::to_string(params).map_err(|e| format!("urlencoded fail:{}", e))?;

        let mut hasher = Sha256::new();
        hasher.update(&req_json);
        let json_hash = format!("{:x}", hasher.finalize()).to_lowercase();

        let datetime = DateTime::from_timestamp(now_time as i64, 0).unwrap_or_default();

        let datetime_str = datetime.format("%Y-%m-%dT%H:%M:%SZ").to_string();
        let rand_s = rand_str(32);

        let mut headers = HeaderMap::new();
        if let Ok(value) = HeaderValue::from_str(host) {
            headers.insert("Host", value);
        }
        if let Ok(value) = HeaderValue::from_str(action) {
            headers.insert("x-acs-action", value);
        }
        if let Ok(value) = HeaderValue::from_str("2017-05-25") {
            headers.insert("x-acs-version", value);
        }
        if let Ok(value) = HeaderValue::from_str(&datetime_str) {
            headers.insert("x-acs-date", value);
        }
        if let Ok(value) = HeaderValue::from_str(&rand_s) {
            headers.insert("x-acs-signature-nonce", value);
        }
        if let Ok(value) = HeaderValue::from_str(&json_hash) {
            headers.insert("x-acs-content-sha256", value);
        }

        let sign_header_arr = &[
            "host",
            "x-acs-action",
            "x-acs-content-sha256",
            "x-acs-date",
            "x-acs-signature-nonce",
            "x-acs-version",
        ];
        let sign_header = sign_header_arr.join(";");
        let mut my_header = Vec::with_capacity(sign_header_arr.len());
        for tmp in sign_header_arr {
            if *tmp == "host" {
                my_header.push(format!("{}:{}", tmp, host))
            } else if let Some(tval) = headers.get(*tmp) {
                my_header.push(format!("{}:{}", tmp, tval.to_str().unwrap_or_default()))
            }
        }

        let sign = format!(
            "{}\n{}\n{}\n{}\n\n{}\n{}",
            method.as_str(),
            uri,
            query,
            my_header.join("\n"),
            sign_header,
            json_hash
        );

        debug!("post json:{}  header:{}", req_json, sign);

        let mut hasher1 = Sha256::new();
        hasher1.update(&sign);
        let result = hasher1.finalize();

        let string_to_sign = format!("ACS3-HMAC-SHA256\n{:x}", result);

        // println!("SIGN:\n{}", string_to_sign);

        debug!("sign body:{}", string_to_sign);

        let mut mac = Hmac::<sha2::Sha256>::new_from_slice(secret_key.as_bytes())
            .map_err(|e| format!("use data key on sha256 fail:{}", e))?;
        mac.update(string_to_sign.as_bytes());
        let signature = mac.finalize();

        let data_sign = hex::encode(signature.into_bytes());
        // ACS3-HMAC-SHA256 Credential=YourAccessKeyId,SignedHeaders=host;x-acs-action;x-acs-content-sha256;x-acs-date;x-acs-signature-nonce;x-acs-version,Signature=e521358f7776c97df52e6b2891a8bc73026794a071b50c3323388c4e0df64804
        let authdata = format!(
            "ACS3-HMAC-SHA256 Credential={},SignedHeaders={},Signature={}",
            secret_id, sign_header, data_sign
        );

        debug!(" authorization:{}", authdata);

        if let Ok(value) = HeaderValue::from_str(&authdata) {
            headers.insert("Authorization", value);
        }
        let url = format!("https://{}", host);

        // println!(
        //     "JSON-DATA:\n{}\n\n\nCanonicalRequest:\n{}\n\n\nStringToSign :\n{}\n\n\nAuthorization:\n{}\n\n\nheader:\n{:?}\n\n\nurl:\n{}",
        //     req_json, sign, string_to_sign,authdata,headers, url
        // );

        let request = client.request(method, url).headers(headers).form(params);
        let result = request
            .send()
            .await
            .map_err(|e| format!("request send fail:{}", e))?;
        let (_, res) = response_check(result, true).await?;
        Ok(res)
    }
}
